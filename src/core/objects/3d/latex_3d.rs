use std::hash::{DefaultHasher, Hash, Hasher};

use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw3D, GeometryKey, Material, RenderContext3D, Transform3D},
    objects::{TextFade, global_matrix3d, latex_geometry::geometry},
    types::Vector3,
};

use super::{super::two_d::text_2d::weighted_path, text_3d::extruded_path_mesh};

// A visible polygonal curve keeps formula meshes compact.
const LATEX_TESSELLATION_TOLERANCE: f32 = 0.005;

/// Mathematical source, size and extrusion depth of a 3D LaTeX object.
#[derive(Clone, Trackable)]
pub struct Latex3DShape {
    /// LaTeX math source, without dollar delimiters.
    #[track]
    pub text: String,
    /// Formula size in world units.
    #[track(min = 0.0)]
    pub size: f32,
    /// Extra glyph thickness in world units.
    #[track(min = 0.0)]
    pub thickness: f32,
    /// Extrusion depth in world units.
    #[track(min = 0.0)]
    pub depth: f32,
}

impl Default for Latex3DShape {
    fn default() -> Self {
        Self {
            text: r"e^{i\pi}+1=0".to_owned(),
            size: 1.0,
            thickness: 0.0,
            depth: 0.1,
        }
    }
}

impl Latex3DShape {
    fn geometry_key(&self, text: &str, part: usize) -> GeometryKey {
        let mut hash = DefaultHasher::new();

        text.hash(&mut hash);
        part.hash(&mut hash);
        self.thickness.to_bits().hash(&mut hash);

        GeometryKey::new::<Latex3DShape>(hash.finish())
    }
}

/// Built-in extruded LaTeX math scene object.
///
/// Supports mathematical LaTeX accepted by RaTeX in display style.
/// Invalid or unsupported source panics when its geometry is first requested.
#[derive(Object)]
#[object(spatial = "3d", builder = "latex_3d")]
pub struct Latex3D {
    #[trackable]
    pub shape: Latex3DShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,

    pub(crate) fade: TextFade,
}

impl Default for Latex3D {
    fn default() -> Self {
        Self {
            shape: Latex3DShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_latex_object,
                box_size: latex_3d_box,
                ..Default::default()
            },
            fade: Default::default(),
        }
    }
}

fn latex_3d_box(world: &hecs::World, entity: hecs::Entity) -> Vector3 {
    let shape = world.get::<&Latex3DShape>(entity).unwrap();
    let fade = world.get::<&TextFade>(entity).unwrap();
    let mut size = geometry(&shape.text).size;

    if let Some(text) = fade.incoming_text(&shape.text) {
        let incoming = geometry(text).size;

        size = size.max(incoming);
    }

    (size * shape.size.max(0.0) + glam::Vec2::splat(shape.thickness.max(0.0)))
        .extend(shape.depth.abs())
}

fn validate_latex_shape(shape: &Latex3DShape) -> Result<(), String> {
    if !shape.size.is_finite()
        || !shape.thickness.is_finite()
        || !shape.depth.is_finite()
        || shape.size < 0.0
        || shape.thickness < 0.0
        || shape.depth < 0.0
    {
        return Err("LaTeX dimensions must be finite and nonnegative.".into());
    }

    Ok(())
}

fn draw_latex_object(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&Latex3DShape>(entity).unwrap();

    validate_latex_shape(&shape)?;

    let material = world.get::<&Material>(entity).unwrap();
    let transform = global_matrix3d(world, entity);

    let fade = world.get::<&TextFade>(entity).unwrap();

    if let Some(text) = fade.incoming_text(&shape.text) {
        let incoming = Latex3DShape {
            text: text.to_owned(),
            ..(*shape).clone()
        };
        let mut outgoing_material = (*material).clone();
        let mut incoming_material = (*material).clone();

        outgoing_material.opacity *= 1.0 - fade.progress;
        incoming_material.opacity *= fade.progress;

        draw_latex_3d(&shape, &outgoing_material, transform, context)?;
        draw_latex_3d(&incoming, &incoming_material, transform, context)?;
    } else {
        draw_latex_3d(&shape, &material, transform, context)?;
    }

    Ok(())
}

fn latex_transform(shape: &Latex3DShape, transform: glam::Mat4) -> glam::Mat4 {
    transform * glam::Mat4::from_scale(glam::Vec3::new(shape.size, shape.size, shape.depth))
}

/// Draws an extruded LaTeX formula from reusable rendering data.
pub fn draw_latex_3d(
    shape: &Latex3DShape,
    material: &Material,
    transform: glam::Mat4,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    validate_latex_shape(shape)?;

    let text = &shape.text;
    let transform = latex_transform(shape, transform);

    if material.opacity <= 0.0 || text.is_empty() {
        return Ok(());
    }

    let formula = geometry(text);

    for (index, part) in formula.parts.iter().enumerate() {
        let mut material = material.clone();

        material.albedo = part.color.unwrap_or(material.albedo);

        context.render_material(
            shape.geometry_key(text, index),
            || {
                let path = if shape.size > f32::EPSILON {
                    weighted_path(&part.path, shape.thickness / shape.size)
                } else {
                    part.path.clone()
                };

                extruded_path_mesh(&path, LATEX_TESSELLATION_TOLERANCE)
            },
            transform,
            &material,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{Object3DHandler, Scene};

    #[test]
    fn depth_is_trackable_and_part_of_the_local_box() {
        let mut scene = Scene::new();
        let formula = latex_3d().text("x").depth(0.25).build(&mut scene);

        assert_eq!(formula.depth.get(), 0.25);
        assert_eq!(formula.box_size().z, 0.25);
    }

    #[test]
    fn formula_builds_extruded_geometry() {
        let formula = geometry("x");
        let mesh = extruded_path_mesh(&formula.parts[0].path, 0.001);
        let normals = mesh.normals.unwrap();

        assert!(normals.iter().any(|normal| normal.z > 0.9));
        assert!(normals.iter().any(|normal| normal.z < -0.9));
        assert!(normals.iter().any(|normal| normal.z.abs() < 0.1));
    }
}
