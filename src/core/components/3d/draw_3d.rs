use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::core::{components::Material, objects::CanvasTexture};
use three_d::{Geometry, InnerSpace, SquareMatrix};

pub(crate) struct CachedGeometry {
    mesh: three_d::Mesh,
    outline: Option<MeshOutline>,
}

struct MeshOutline {
    positions: three_d::VertexBuffer<three_d::Vec3>,
    barycentric: three_d::VertexBuffer<three_d::Vec3>,
    context: three_d::Context,
    bounds: three_d::AxisAlignedBoundingBox,
    transformation: three_d::Mat4,
}

impl MeshOutline {
    fn new(context: &three_d::Context, mesh: &three_d::CpuMesh) -> Self {
        let (positions, barycentric) = outline_vertices(mesh);
        Self {
            positions: three_d::VertexBuffer::new_with_data(context, &positions),
            barycentric: three_d::VertexBuffer::new_with_data(context, &barycentric),
            context: context.clone(),
            bounds: three_d::AxisAlignedBoundingBox::new_with_positions(&positions),
            transformation: three_d::Mat4::identity(),
        }
    }

    fn set_transformation(&mut self, transformation: three_d::Mat4) {
        self.transformation = transformation;
    }
}

impl three_d::Geometry for MeshOutline {
    fn draw(
        &self,
        viewer: &dyn three_d::Viewer,
        program: &three_d::Program,
        render_states: three_d::RenderStates,
    ) {
        program.use_uniform("viewProjection", viewer.projection() * viewer.view());
        program.use_uniform("modelMatrix", self.transformation);
        program.use_vertex_attribute("position", &self.positions);
        program.use_vertex_attribute("barycentric", &self.barycentric);
        program.draw_arrays(
            render_states,
            viewer.viewport(),
            self.positions.vertex_count(),
        );
    }

    fn vertex_shader_source(&self) -> String {
        r#"
            uniform mat4 viewProjection;
            uniform mat4 modelMatrix;
            in vec3 position;
            in vec3 barycentric;
            out vec3 bary;

            void main() {
                bary = barycentric;
                gl_Position = viewProjection * modelMatrix * vec4(position, 1.0);
            }
        "#
        .to_owned()
    }

    fn id(&self) -> three_d::GeometryId {
        three_d::GeometryId(0x7ffe)
    }

    fn render_with_material(
        &self,
        material: &dyn three_d::Material,
        viewer: &dyn three_d::Viewer,
        lights: &[&dyn three_d::Light],
    ) {
        three_d::render_with_material(&self.context, viewer, self, material, lights)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    fn render_with_effect(
        &self,
        effect: &dyn three_d::Effect,
        viewer: &dyn three_d::Viewer,
        lights: &[&dyn three_d::Light],
        color_texture: Option<three_d::ColorTexture>,
        depth_texture: Option<three_d::DepthTexture>,
    ) {
        three_d::render_with_effect(
            &self.context,
            viewer,
            self,
            effect,
            lights,
            color_texture,
            depth_texture,
        )
        .unwrap_or_else(|error| panic!("{error}"));
    }

    fn aabb(&self) -> three_d::AxisAlignedBoundingBox {
        self.bounds.transformed(self.transformation)
    }
}

struct OutlineMaterial {
    width: f32,
    color: three_d::Srgba,
}

impl three_d::Material for OutlineMaterial {
    fn fragment_shader_source(&self, _lights: &[&dyn three_d::Light]) -> String {
        r#"
            layout (location = 0) out vec4 outColor;
            uniform float lineWidth;
            uniform vec4 lineColor;
            in vec3 bary;

            void main() {
                vec3 derivatives = fwidth(bary);
                vec3 interior = step(derivatives * lineWidth, bary);
                float fill = min(min(interior.x, interior.y), interior.z);
                outColor = vec4(lineColor.rgb, lineColor.a * (1.0 - fill));
                gl_FragDepth = gl_FragCoord.z - 0.0001;
            }
        "#
        .to_owned()
    }

    fn id(&self) -> three_d::EffectMaterialId {
        three_d::EffectMaterialId(0x4ffe)
    }

    fn use_uniforms(
        &self,
        program: &three_d::Program,
        _viewer: &dyn three_d::Viewer,
        _lights: &[&dyn three_d::Light],
    ) {
        program.use_uniform("lineWidth", self.width);
        program.use_uniform("lineColor", three_d::Vec4::from(self.color));
    }

    fn render_states(&self) -> three_d::RenderStates {
        three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            blend: three_d::Blend::TRANSPARENCY,
            depth_test: three_d::DepthTest::LessOrEqual,
            ..Default::default()
        }
    }

    fn material_type(&self) -> three_d::MaterialType {
        three_d::MaterialType::Transparent
    }
}

/// Stable identity for a reusable GPU geometry.
///
/// Use the object's shape component as `T` and encode geometry-changing settings in `variant`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryKey {
    kind: TypeId,
    variant: u64,
}

impl GeometryKey {
    pub fn new<T: 'static>(variant: u64) -> Self {
        Self {
            kind: TypeId::of::<T>(),
            variant,
        }
    }
}

/// Resources available to a [`Draw3D`] callback during one canvas pass.
///
/// Geometries are cached across frames. Callbacks can either use [`Self::render_material`]
/// or access three-d directly through the public camera, target, and context fields.
pub struct RenderContext3D<'a> {
    pub camera: &'a three_d::Camera,
    pub target: &'a three_d::RenderTarget<'static>,
    pub three_d: &'a three_d::Context,
    geometries: &'a mut HashMap<GeometryKey, CachedGeometry>,
    used_geometries: &'a mut HashSet<GeometryKey>,
    physical: &'a mut three_d::PhysicalMaterial,
    ambient: &'a three_d::AmbientLight,
    sun: &'a three_d::DirectionalLight,
    texture: &'a dyn Fn(CanvasTexture) -> Option<glow::NativeTexture>,
    current_transform: glam::Mat4,
}

impl<'a> RenderContext3D<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        camera: &'a three_d::Camera,
        target: &'a three_d::RenderTarget<'static>,
        three_d: &'a three_d::Context,
        geometries: &'a mut HashMap<GeometryKey, CachedGeometry>,
        used_geometries: &'a mut HashSet<GeometryKey>,
        physical: &'a mut three_d::PhysicalMaterial,
        ambient: &'a three_d::AmbientLight,
        sun: &'a three_d::DirectionalLight,
        texture: &'a dyn Fn(CanvasTexture) -> Option<glow::NativeTexture>,
    ) -> Self {
        Self {
            camera,
            target,
            three_d,
            geometries,
            used_geometries,
            physical,
            ambient,
            sun,
            texture,
            current_transform: glam::Mat4::IDENTITY,
        }
    }

    /// Returns the transform applied before local callback geometry transforms.
    pub fn current_transform(&self) -> glam::Mat4 {
        self.current_transform
    }

    pub(crate) fn set_current_transform(&mut self, transform: glam::Mat4) -> glam::Mat4 {
        std::mem::replace(&mut self.current_transform, transform)
    }

    /// Returns a shared mesh, creating it the first time its key is used.
    pub fn geometry(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
    ) -> &mut three_d::Mesh {
        self.used_geometries.insert(key);
        &mut self
            .geometries
            .entry(key)
            .or_insert_with(|| CachedGeometry {
                mesh: three_d::Mesh::new(self.three_d, &create()),
                outline: None,
            })
            .mesh
    }

    /// Resolves a canvas texture for custom materials.
    pub fn canvas_texture(&self, texture: CanvasTexture) -> Option<glow::NativeTexture> {
        (self.texture)(texture)
    }

    /// Draws cached geometry with the standard Kinematic material and lights.
    ///
    /// `transformation` is relative to [`Self::current_transform`]. Regular
    /// object callbacks use an identity current transform; simulation callbacks
    /// use their object's inherited global transform.
    pub fn render_material(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
        transformation: glam::Mat4,
        data: &Material,
    ) -> Result<(), String> {
        let transformation = combine_transforms(self.current_transform, transformation);
        validate_transformation(transformation)?;
        if transformation.determinant().abs() <= f32::EPSILON {
            return Ok(());
        }
        let [r, g, b, a] = data.albedo.rgba();
        let transparent = data.opacity * a < 1.0;
        let color = three_d::Srgba::new(
            channel(r),
            channel(g),
            channel(b),
            channel(a * data.opacity),
        );
        let states = render_states(transparent, false);
        let camera = self.camera;
        self.prepare_geometry(key, create, data.outline_width > 0.0);
        if data.unlit {
            let material = three_d::ColorMaterial {
                color,
                texture: None,
                render_states: states,
                is_transparent: transparent,
            };
            let mesh = &mut self.geometries.get_mut(&key).unwrap().mesh;
            mesh.set_transformation(transformation.to_cols_array_2d().into());
            mesh.render_with_material(&material, camera, &[]);
        } else {
            self.physical.albedo = color;
            self.physical.metallic = data.metallic.clamp(0.0, 1.0);
            self.physical.roughness = data.roughness.clamp(0.04, 1.0);
            self.physical.is_transparent = transparent;
            self.physical.render_states = states;
            let physical = &*self.physical;
            let lights: [&dyn three_d::Light; 2] = [self.ambient, self.sun];
            let mesh = &mut self.geometries.get_mut(&key).unwrap().mesh;
            mesh.set_transformation(transformation.to_cols_array_2d().into());
            mesh.render_with_material(physical, camera, &lights);
        }
        self.render_outline(key, transformation, data);
        Ok(())
    }

    /// Draws cached geometry with an albedo texture and the configured material response.
    pub(crate) fn render_textured_material(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
        transformation: glam::Mat4,
        texture: three_d::Texture2DRef,
        data: &Material,
    ) -> Result<(), String> {
        let transformation = combine_transforms(self.current_transform, transformation);
        validate_transformation(transformation)?;
        if transformation.determinant().abs() <= f32::EPSILON {
            return Ok(());
        }

        let [r, g, b, a] = data.albedo.rgba();
        let color = three_d::Srgba::new(
            channel(r),
            channel(g),
            channel(b),
            channel(a * data.opacity),
        );
        let states = render_states(true, false);
        let camera = self.camera;
        self.prepare_geometry(key, create, data.outline_width > 0.0);

        if data.unlit {
            let mesh = &mut self.geometries.get_mut(&key).unwrap().mesh;
            mesh.set_transformation(transformation.to_cols_array_2d().into());
            mesh.render_with_material(
                &three_d::ColorMaterial {
                    color,
                    texture: Some(texture),
                    render_states: states,
                    is_transparent: true,
                },
                camera,
                &[],
            );
        } else {
            let mut material = self.physical.clone();
            material.albedo = color;
            material.albedo_texture = Some(texture);
            material.metallic = data.metallic.clamp(0.0, 1.0);
            material.roughness = data.roughness.clamp(0.04, 1.0);
            material.is_transparent = true;
            material.render_states = states;
            let lights: [&dyn three_d::Light; 2] = [self.ambient, self.sun];
            let mesh = &mut self.geometries.get_mut(&key).unwrap().mesh;
            mesh.set_transformation(transformation.to_cols_array_2d().into());
            mesh.render_with_material(&material, camera, &lights);
        }
        self.render_outline(key, transformation, data);
        Ok(())
    }

    fn prepare_geometry(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
        outline: bool,
    ) {
        self.used_geometries.insert(key);
        let needs_mesh = !self.geometries.contains_key(&key);
        let needs_outline = outline
            && self
                .geometries
                .get(&key)
                .is_none_or(|geometry| geometry.outline.is_none());
        if !needs_mesh && !needs_outline {
            return;
        }

        let cpu = create();
        if needs_mesh {
            self.geometries.insert(
                key,
                CachedGeometry {
                    mesh: three_d::Mesh::new(self.three_d, &cpu),
                    outline: outline.then(|| MeshOutline::new(self.three_d, &cpu)),
                },
            );
        } else {
            self.geometries.get_mut(&key).unwrap().outline =
                Some(MeshOutline::new(self.three_d, &cpu));
        }
    }

    fn render_outline(&mut self, key: GeometryKey, transformation: glam::Mat4, data: &Material) {
        if data.outline_width <= 0.0 {
            return;
        }
        let [r, g, b, a] = data.outline_color.rgba();
        let outline = self
            .geometries
            .get_mut(&key)
            .unwrap()
            .outline
            .as_mut()
            .unwrap();
        outline.set_transformation(transformation.to_cols_array_2d().into());
        outline.render_with_material(
            &OutlineMaterial {
                width: data.outline_width,
                color: three_d::Srgba::new(
                    channel(r),
                    channel(g),
                    channel(b),
                    channel(a * data.opacity),
                ),
            },
            self.camera,
            &[],
        );
    }
}

type PositionKey = [u32; 3];

fn outline_vertices(mesh: &three_d::CpuMesh) -> (Vec<three_d::Vec3>, Vec<three_d::Vec3>) {
    let source = mesh.positions.to_f32();
    let indices = mesh
        .indices
        .to_u32()
        .unwrap_or_else(|| (0..source.len() as u32).collect());
    let positions: Vec<_> = indices
        .iter()
        .map(|index| source[*index as usize])
        .collect();
    let triangle_count = positions.len() / 3;
    let mut edges: HashMap<(PositionKey, PositionKey), Vec<(usize, usize, three_d::Vec3)>> =
        HashMap::new();

    for (triangle, vertices) in positions.chunks_exact(3).enumerate() {
        let normal = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]);
        let normal = if normal.magnitude2() > f32::EPSILON {
            normal.normalize()
        } else {
            three_d::vec3(0.0, 0.0, 0.0)
        };
        for (edge, [from, to]) in [[1, 2], [2, 0], [0, 1]].into_iter().enumerate() {
            let from = position_key(vertices[from]);
            let to = position_key(vertices[to]);
            let key = if from <= to { (from, to) } else { (to, from) };
            edges.entry(key).or_default().push((triangle, edge, normal));
        }
    }

    let mut hidden = vec![[false; 3]; triangle_count];
    for adjacent in edges.values() {
        if let [
            (first_triangle, first_edge, first_normal),
            (second_triangle, second_edge, second_normal),
        ] = adjacent.as_slice()
            && first_normal.magnitude2() > 0.0
            && second_normal.magnitude2() > 0.0
            && first_normal.dot(*second_normal).abs() >= 1.0 - 1e-5
        {
            hidden[*first_triangle][*first_edge] = true;
            hidden[*second_triangle][*second_edge] = true;
        }
    }

    let mut barycentric = Vec::with_capacity(positions.len());
    for hidden_edges in hidden {
        let mut triangle = [
            three_d::vec3(1.0, 0.0, 0.0),
            three_d::vec3(0.0, 1.0, 0.0),
            three_d::vec3(0.0, 0.0, 1.0),
        ];
        for (edge, hidden) in hidden_edges.into_iter().enumerate() {
            if hidden {
                for barycentric in &mut triangle {
                    barycentric[edge] = 1.0;
                }
            }
        }
        barycentric.extend(triangle);
    }

    (positions, barycentric)
}

fn position_key(position: three_d::Vec3) -> PositionKey {
    [
        normalized_float_bits(position.x),
        normalized_float_bits(position.y),
        normalized_float_bits(position.z),
    ]
}

fn normalized_float_bits(value: f32) -> u32 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

/// Local three-dimensional rendering callback and bounds for an entity.
#[derive(Clone, kinematic_macros::Trackable)]
pub struct Draw3D {
    /// Whether this entity and, for containers, its subtree are drawn.
    #[track]
    pub visibility: bool,

    /// Draws this entity using its current ECS state.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut RenderContext3D<'_>) -> Result<(), String>,

    /// Returns the object's local bounding-box size.
    pub get_box: fn(&hecs::World, hecs::Entity) -> glam::Vec3,
}

impl Default for Draw3D {
    fn default() -> Self {
        Self {
            visibility: true,
            on_draw: |_, _, _| Ok(()),
            get_box: |_, _| glam::Vec3::ZERO,
        }
    }
}

pub(crate) fn validate_dimensions(size: glam::Vec3) -> Result<(), String> {
    if !size.is_finite() || size.min_element() < 0.0 {
        return Err("Primitive dimensions must be finite and nonnegative.".into());
    }
    Ok(())
}

pub(crate) fn validate_transformation(transformation: glam::Mat4) -> Result<(), String> {
    if !transformation.is_finite() {
        return Err("Object transform must be finite.".into());
    }
    Ok(())
}

pub(crate) fn render_states(transparent: bool, premultiplied: bool) -> three_d::RenderStates {
    use three_d::{Blend, BlendEquationType as E, BlendMultiplierType as M};
    three_d::RenderStates {
        cull: three_d::Cull::None,
        write_mask: if transparent {
            three_d::WriteMask::COLOR
        } else {
            three_d::WriteMask::COLOR_AND_DEPTH
        },
        blend: if transparent {
            Blend::Enabled {
                source_rgb_multiplier: if premultiplied { M::One } else { M::SrcAlpha },
                source_alpha_multiplier: M::One,
                destination_rgb_multiplier: M::OneMinusSrcAlpha,
                destination_alpha_multiplier: M::OneMinusSrcAlpha,
                rgb_equation: E::Add,
                alpha_equation: E::Add,
            }
        } else {
            Blend::Disabled
        },
        ..Default::default()
    }
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn combine_transforms(base: glam::Mat4, local: glam::Mat4) -> glam::Mat4 {
    base * local
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Shape;
    struct OtherShape;

    #[test]
    fn geometry_keys_include_type_and_variant() {
        assert_eq!(GeometryKey::new::<Shape>(3), GeometryKey::new::<Shape>(3));
        assert_ne!(GeometryKey::new::<Shape>(3), GeometryKey::new::<Shape>(4));
        assert_ne!(
            GeometryKey::new::<Shape>(3),
            GeometryKey::new::<OtherShape>(3)
        );
    }

    #[test]
    fn current_transform_is_applied_before_callback_local_transform() {
        let base = glam::Mat4::from_translation(glam::vec3(4.0, 0.0, 0.0));
        let local = glam::Mat4::from_translation(glam::vec3(0.0, 3.0, 0.0));

        assert_eq!(
            combine_transforms(base, local).transform_point3(glam::Vec3::ZERO),
            glam::vec3(4.0, 3.0, 0.0)
        );
    }

    #[test]
    fn outline_hides_only_the_coplanar_diagonal_of_each_cube_face() {
        let (_, barycentric) = outline_vertices(&three_d::CpuMesh::cube());

        for triangle in barycentric.chunks_exact(3) {
            let hidden_edges = (0..3)
                .filter(|edge| triangle.iter().all(|vertex| vertex[*edge] == 1.0))
                .count();
            assert_eq!(hidden_edges, 1);
        }
    }
}
