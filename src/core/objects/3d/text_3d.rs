use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use kinematic_macros::{Object, Trackable};
use lyon_tessellation::{
    FillOptions, FillTessellator, FillVertex,
    geometry_builder::{BuffersBuilder, VertexBuffers},
    math::{Point, point},
    path::Path,
};

use crate::core::{
    Tween,
    components::{Draw3D, GeometryKey, Material, RenderContext3D, Transform3D},
    objects::{
        Font, ObjectHandler, TextShape, global_matrix3d,
        string_morph::{ContentMorph, fade_string},
    },
    types::Vector3,
};

use super::super::two_d::text_2d::{text_box, text_path};

const TEXT_GEOMETRY_SCALE: f32 = 64.0;
const TEXT_TESSELLATION_TOLERANCE: f32 = 0.005;

/// Content, typography and extrusion depth of a 3D text object.
#[derive(Clone, Trackable)]
pub struct Text3DShape {
    /// Text displayed by the object.
    #[track]
    pub text: String,
    /// Font size in world units.
    #[track(min = 0.0)]
    pub size: f32,
    /// Horizontal line alignment from `-1.0` left to `1.0` right.
    #[track(min = -1.0, max = 1.0)]
    pub align: f32,
    /// Extra glyph thickness in world units.
    #[track(min = 0.0)]
    pub thickness: f32,
    /// Extrusion depth in world units.
    #[track(min = 0.0)]
    pub depth: f32,

    /// Font used to build the text geometry.
    pub font: Font,
}

impl Default for Text3DShape {
    fn default() -> Self {
        let shape = TextShape::default();
        Self {
            text: shape.text,
            size: 1.0,
            align: shape.align,
            thickness: shape.thickness,
            depth: 0.1,
            font: shape.font,
        }
    }
}

impl Text3DShape {
    fn text_shape(&self, text: &str) -> TextShape {
        TextShape {
            text: text.to_owned(),
            size: self.size * TEXT_GEOMETRY_SCALE,
            align: self.align,
            thickness: self.thickness * TEXT_GEOMETRY_SCALE,
            font: self.font.clone(),
        }
    }

    fn geometry_key(&self, text: &str) -> GeometryKey {
        let mut hash = DefaultHasher::new();
        text.hash(&mut hash);
        self.font.path().hash(&mut hash);
        self.size.to_bits().hash(&mut hash);
        self.align.to_bits().hash(&mut hash);
        self.thickness.to_bits().hash(&mut hash);
        GeometryKey::new::<Text3DShape>(hash.finish())
    }
}

/// Built-in extruded text scene object.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "text_3d")]
pub struct Text3D {
    #[trackable]
    pub shape: Text3DShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Text3D {
    fn default() -> Self {
        Self {
            shape: Text3DShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_text_3d,
                get_box: text_3d_box,
                ..Default::default()
            },
        }
    }
}

impl Text3DBuilder {
    /// Sets a bundled font or the path to a user-provided TTF or OTF file.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.object.shape.font = font.into();
        self
    }
}

impl Text3DHandler {
    /// Cross-fades the current string into `text` on the same object.
    pub fn fade(&self, text: impl Into<String>) -> Tween<Text3D> {
        let from = self.get(Text3DShape::text_property());
        let to = text.into();
        let tween = self.text(to.clone());
        fade_string(tween, self.get_id(), from, to)
    }
}

fn text_3d_box(world: &hecs::World, entity: hecs::Entity) -> Vector3 {
    let shape = world.get::<&Text3DShape>(entity).unwrap();
    (text_box(&shape.text_shape(&shape.text)) / TEXT_GEOMETRY_SCALE).extend(shape.depth.abs())
}

fn draw_text_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&Text3DShape>(entity).unwrap();
    if !shape.size.is_finite()
        || !shape.align.is_finite()
        || !shape.thickness.is_finite()
        || !shape.depth.is_finite()
        || shape.size < 0.0
        || shape.thickness < 0.0
        || shape.depth < 0.0
    {
        return Err("Text dimensions must be finite and nonnegative.".into());
    }
    let material = world.get::<&Material>(entity).unwrap();
    let transform = global_matrix3d(world, entity)
        * glam::Mat4::from_scale(glam::Vec3::new(
            1.0 / TEXT_GEOMETRY_SCALE,
            1.0 / TEXT_GEOMETRY_SCALE,
            shape.depth,
        ));
    let progress = world
        .get::<&ContentMorph>(entity)
        .ok()
        .filter(|morph| morph.active && morph.progress > 0.0)
        .and_then(|morph| {
            morph.transitions[morph.transition as usize].fade_layers(morph.progress.clamp(0.0, 1.0))
        });

    if let Some(layers) = progress {
        for (text, opacity) in layers {
            draw_text_geometry(&shape, &text, opacity, &material, transform, context)?;
        }
    } else {
        draw_text_geometry(&shape, &shape.text, 1.0, &material, transform, context)?;
    }
    Ok(())
}

fn draw_text_geometry(
    shape: &Text3DShape,
    text: &str,
    opacity: f32,
    material: &Material,
    transform: glam::Mat4,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    if opacity <= 0.0 || text.is_empty() {
        return Ok(());
    }
    let mut material = material.clone();
    material.opacity *= opacity;
    context.render_material(
        shape.geometry_key(text),
        || {
            let text_shape = shape.text_shape(text);
            extruded_path_mesh(
                &text_path(&text_shape),
                (text_shape.size * TEXT_TESSELLATION_TOLERANCE).max(0.05),
            )
        },
        transform,
        &material,
    )
}

fn lyon_path(path: &skia_safe::Path) -> Path {
    let mut builder = Path::builder();
    let mut contour_open = false;
    let mut current = skia_safe::Point::default();
    for record in path.iter() {
        let points = record.points();
        match record.verb() {
            skia_safe::PathVerb::Move => {
                if contour_open {
                    builder.end(false);
                }
                current = points[0];
                builder.begin(point(current.x, -current.y));
                contour_open = true;
            }
            skia_safe::PathVerb::Line => {
                current = points[0];
                builder.line_to(point(current.x, -current.y));
            }
            skia_safe::PathVerb::Quad => {
                current = points[1];
                builder.quadratic_bezier_to(
                    point(points[0].x, -points[0].y),
                    point(current.x, -current.y),
                );
            }
            skia_safe::PathVerb::Conic => {
                let mut quads = [skia_safe::Point::default(); 9];
                let count = skia_safe::Path::convert_conic_to_quads(
                    current,
                    points[0],
                    points[1],
                    record.conic_weight(),
                    &mut quads,
                    2,
                )
                .expect("Font conic must be finite.");
                for index in 0..count {
                    let control = quads[index * 2 + 1];
                    let end = quads[index * 2 + 2];
                    builder.quadratic_bezier_to(point(control.x, -control.y), point(end.x, -end.y));
                }
                current = points[1];
            }
            skia_safe::PathVerb::Cubic => {
                current = points[2];
                builder.cubic_bezier_to(
                    point(points[0].x, -points[0].y),
                    point(points[1].x, -points[1].y),
                    point(current.x, -current.y),
                );
            }
            skia_safe::PathVerb::Close => {
                builder.end(true);
                contour_open = false;
            }
        }
    }
    if contour_open {
        builder.end(false);
    }
    builder.build()
}

pub(crate) fn extruded_path_mesh(path: &skia_safe::Path, tolerance: f32) -> three_d::CpuMesh {
    let path = lyon_path(path);
    let mut tessellator = FillTessellator::new();
    let mut geometry: VertexBuffers<Point, u32> = VertexBuffers::new();
    tessellator
        .tessellate_path(
            &path,
            &FillOptions::default().with_tolerance(tolerance),
            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex<'_>| vertex.position()),
        )
        .expect("Text outline must be tessellated.");

    let mut triangles = Vec::with_capacity(geometry.indices.len() / 3);
    let mut edges = HashMap::<(u32, u32), (u32, u32, u32)>::new();
    for triangle in geometry.indices.chunks_exact(3) {
        let mut triangle = [triangle[0], triangle[1], triangle[2]];
        let [a, b, c] = triangle.map(|index| geometry.vertices[index as usize]);
        if (b - a).cross(c - a) < 0.0 {
            triangle.swap(1, 2);
        }
        triangles.push(triangle);
        for [from, to] in [
            [triangle[0], triangle[1]],
            [triangle[1], triangle[2]],
            [triangle[2], triangle[0]],
        ] {
            let key = (from.min(to), from.max(to));
            edges
                .entry(key)
                .and_modify(|edge| edge.0 += 1)
                .or_insert((1, from, to));
        }
    }

    let vertex = |index: u32, z: f32| {
        let point = geometry.vertices[index as usize];
        three_d::vec3(point.x, point.y, z)
    };
    let boundary_edges: Vec<_> = edges
        .into_values()
        .filter(|edge| edge.0 == 1)
        .map(|(_, from, to)| (from, to))
        .collect();
    let mut side_normals = HashMap::<u32, glam::Vec2>::new();
    for &(from, to) in &boundary_edges {
        let from_point = geometry.vertices[from as usize];
        let to_point = geometry.vertices[to as usize];
        let direction = glam::Vec2::new(to_point.x - from_point.x, to_point.y - from_point.y);
        let outward = glam::Vec2::new(direction.y, -direction.x).normalize_or_zero();
        side_normals
            .entry(from)
            .and_modify(|normal| *normal += outward)
            .or_insert(outward);
        side_normals
            .entry(to)
            .and_modify(|normal| *normal += outward)
            .or_insert(outward);
    }
    for normal in side_normals.values_mut() {
        *normal = normal.normalize_or_zero();
    }
    let vertex_count = geometry.vertices.len() as u32;
    let mut positions = Vec::with_capacity(geometry.vertices.len() * 2 + side_normals.len() * 2);
    let mut normals = Vec::with_capacity(positions.capacity());
    for index in 0..vertex_count {
        positions.push(vertex(index, 0.5));
        normals.push(three_d::vec3(0.0, 0.0, 1.0));
    }
    for index in 0..vertex_count {
        positions.push(vertex(index, -0.5));
        normals.push(three_d::vec3(0.0, 0.0, -1.0));
    }
    let mut indices = Vec::with_capacity(triangles.len() * 6 + boundary_edges.len() * 6);
    for [a, b, c] in triangles {
        indices.extend_from_slice(&[a, b, c]);
        indices.extend_from_slice(&[vertex_count + a, vertex_count + c, vertex_count + b]);
    }
    let mut side_vertices = HashMap::with_capacity(side_normals.len());
    for (&index, normal) in &side_normals {
        let front = positions.len() as u32;
        let normal = three_d::vec3(normal.x, normal.y, 0.0);
        positions.extend_from_slice(&[vertex(index, 0.5), vertex(index, -0.5)]);
        normals.extend_from_slice(&[normal, normal]);
        side_vertices.insert(index, front);
    }
    for (from, to) in boundary_edges {
        let front_from = side_vertices[&from];
        let back_from = front_from + 1;
        let front_to = side_vertices[&to];
        let back_to = front_to + 1;
        indices.extend_from_slice(&[front_from, back_from, back_to]);
        indices.extend_from_slice(&[front_from, back_to, front_to]);
    }

    three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        indices: three_d::Indices::U32(indices),
        normals: Some(normals),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{Object3DHandler, Scene, vec3};

    #[test]
    fn depth_is_trackable_and_part_of_the_local_box() {
        let mut scene = Scene::new();
        let text = text_3d().text("A").depth(2.5).build(&mut scene);

        assert_eq!(text.depth.get(), 2.5);
        assert_eq!(text.get_box().z, 2.5);
        assert_eq!(text.get_global_position(), vec3(0.0, 0.0, 0.0));
    }

    #[test]
    fn default_size_matches_other_3d_objects() {
        let mut scene = Scene::new();
        let text = text_3d().text("Text").build(&mut scene);
        let bounds = text.get_box();

        assert!(bounds.x <= 4.0, "Unexpected text width: {}.", bounds.x);
        assert!(bounds.y <= 2.0, "Unexpected text height: {}.", bounds.y);
        assert_eq!(bounds.z, 0.1);
    }

    #[test]
    fn mesh_has_front_back_and_side_normals() {
        let shape = Text3DShape::default().text_shape("A");
        let mesh = extruded_path_mesh(&text_path(&shape), 0.01);
        let vertex_count = mesh.vertex_count();
        let index_count = mesh.indices.len().expect("Text mesh must be indexed.");
        let normals = mesh.normals.as_ref().unwrap();

        assert!(vertex_count < index_count);
        assert!(normals.iter().any(|normal| normal.z > 0.9));
        assert!(normals.iter().any(|normal| normal.z < -0.9));
        assert!(normals.iter().any(|normal| normal.z.abs() < 0.1));
    }

    #[test]
    fn polygonal_tolerance_reduces_curved_glyph_vertices() {
        let shape = Text3DShape::default().text_shape("O");
        let path = text_path(&shape);
        let detailed = extruded_path_mesh(&path, 0.01);
        let polygonal =
            extruded_path_mesh(&path, (shape.size * TEXT_TESSELLATION_TOLERANCE).max(0.05));

        assert!(polygonal.vertex_count() < detailed.vertex_count());
    }

    #[test]
    fn fade_keeps_the_same_text_object() {
        let mut scene = Scene::new();
        let text = text_3d().text("From").build(&mut scene);
        let entity = text.get_id();

        text.fade("To").play();

        assert_eq!(text.get_id(), entity);
        assert_eq!(scene.get_world().query::<&Text3DShape>().iter().count(), 1);
        scene.update(1.0);
        assert_eq!(text.text.get(), "To");
    }

    #[test]
    fn text_3d_is_not_morphable() {
        assert!(!<Text3D as crate::core::objects::Object>::MORPHABLE);
    }
}
