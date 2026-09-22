use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::core::{
    TrackValue,
    components::Material,
    objects::{CanvasTexture, EffectiveMeshShader},
};
use three_d::{Geometry, InnerSpace, SquareMatrix, Viewer};

pub(crate) struct CachedGeometry {
    mesh: three_d::Mesh,
    outline: Option<MeshOutline>,
    attributes: MeshAttributes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshAttributes {
    normal: bool,
    tangent: bool,
    uv: bool,
    color: bool,
}

impl MeshAttributes {
    fn new(mesh: &three_d::CpuMesh) -> Self {
        Self {
            normal: mesh.normals.is_some(),
            tangent: mesh.tangents.is_some(),
            uv: mesh.uvs.is_some(),
            color: mesh.colors.is_some(),
        }
    }
}

pub(crate) type MeshProgramCache = HashMap<(u64, MeshAttributes), three_d::Program>;

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
/// or access three-d directly through the public camera, target, and context fields. Mesh
/// shaders require `render_material`; direct three-d drawing has no compatible interception
/// point and is rejected when a shader is configured or inherited.
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
    mesh_programs: &'a mut MeshProgramCache,
    current_transform: glam::Mat4,
    mesh_shader: Option<EffectiveMeshShader>,
    mesh_shader_used: bool,
    direct_geometry_access: bool,
    scene_time: f32,
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
        mesh_programs: &'a mut MeshProgramCache,
        scene_time: f32,
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
            mesh_programs,
            current_transform: glam::Mat4::IDENTITY,
            mesh_shader: None,
            mesh_shader_used: false,
            direct_geometry_access: false,
            scene_time,
        }
    }

    pub(crate) fn set_mesh_shader(&mut self, shader: Option<EffectiveMeshShader>) {
        self.mesh_shader = shader;
        self.mesh_shader_used = false;
        self.direct_geometry_access = false;
    }

    pub(crate) fn finish_mesh_shader(&self, expects_mesh: bool) -> Result<(), String> {
        if expects_mesh
            && self.mesh_shader.is_some()
            && (!self.mesh_shader_used || self.direct_geometry_access)
        {
            return Err(
                "Custom 3D objects with mesh shaders must draw through RenderContext3D::render_material."
                    .into(),
            );
        }
        Ok(())
    }

    /// Returns the transform applied before local callback geometry transforms.
    pub fn current_transform(&self) -> glam::Mat4 {
        self.current_transform
    }

    /// Replaces the transform applied before local geometry and returns the previous one.
    ///
    /// Custom object wrappers should restore the returned transform after delegating drawing.
    pub fn set_current_transform(&mut self, transform: glam::Mat4) -> glam::Mat4 {
        std::mem::replace(&mut self.current_transform, transform)
    }

    /// Returns a shared mesh, creating it the first time its key is used.
    pub fn geometry(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
    ) -> &mut three_d::Mesh {
        self.direct_geometry_access = true;
        self.used_geometries.insert(key);
        &mut self
            .geometries
            .entry(key)
            .or_insert_with(|| {
                let cpu = create();
                CachedGeometry {
                    mesh: three_d::Mesh::new(self.three_d, &cpu),
                    outline: None,
                    attributes: MeshAttributes::new(&cpu),
                }
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
        self.mesh_shader_used |= self.mesh_shader.is_some();
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
        if self.render_mesh_shader(key, transformation, data, states)? {
            self.render_outline(key, transformation, data);
            return Ok(());
        }
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
        self.mesh_shader_used |= self.mesh_shader.is_some();
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

        if self.render_mesh_shader(key, transformation, data, states)? {
            self.render_outline(key, transformation, data);
            return Ok(());
        }

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

    pub(crate) fn render_mesh_shader_only(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
        transformation: glam::Mat4,
        data: &Material,
    ) -> Result<bool, String> {
        if self.mesh_shader.is_none() {
            return Ok(false);
        }
        self.mesh_shader_used = true;
        let transformation = combine_transforms(self.current_transform, transformation);
        validate_transformation(transformation)?;
        if transformation.determinant().abs() <= f32::EPSILON {
            return Ok(true);
        }
        let a = data.albedo.a;
        let transparent = data.opacity * a < 1.0;
        let states = render_states(transparent, false);
        self.prepare_geometry(key, create, data.outline_width > 0.0);
        self.render_mesh_shader(key, transformation, data, states)?;
        self.render_outline(key, transformation, data);
        Ok(true)
    }

    fn render_mesh_shader(
        &mut self,
        key: GeometryKey,
        transformation: glam::Mat4,
        data: &Material,
        states: three_d::RenderStates,
    ) -> Result<bool, String> {
        let Some(shader) = self.mesh_shader.clone() else {
            return Ok(false);
        };
        let attributes = self.geometries[&key].attributes;
        let cache_key = (shader.shader.id, attributes);
        if !self.mesh_programs.contains_key(&cache_key) {
            let vertex = shader
                .shader
                .vertex
                .as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| default_mesh_vertex_shader(attributes));
            let program =
                three_d::Program::from_source(self.three_d, &vertex, &shader.shader.fragment)
                    .map_err(|error| format!("Mesh shader compilation failed: {error}"))?;
            validate_mesh_program(&program, attributes)?;
            self.mesh_programs.insert(cache_key, program);
        }
        let program = &self.mesh_programs[&cache_key];
        use_uniforms(
            program,
            &shader,
            attributes,
            self.camera,
            self.scene_time,
            data,
        )?;
        let mesh = &mut self.geometries.get_mut(&key).unwrap().mesh;
        mesh.set_transformation(transformation.to_cols_array_2d().into());
        mesh.draw(self.camera, program, states);
        self.mesh_shader_used = true;
        Ok(true)
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
                    attributes: MeshAttributes::new(&cpu),
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

fn default_mesh_vertex_shader(attributes: MeshAttributes) -> String {
    format!(
        r#"
        in vec3 position;
        {normal_attribute}
        {uv_attribute}
        uniform mat4 modelMatrix;
        uniform mat4 viewProjection;
        {normal_uniform}
        out vec3 k_world_position;
        out vec3 k_normal;
        out vec2 k_uv;

        void main() {{
            vec4 world = modelMatrix * vec4(position, 1.0);
            k_world_position = world.xyz;
            {normal_value}
            {uv_value}
            gl_Position = viewProjection * world;
        }}
        "#,
        normal_attribute = attributes.normal.then_some("in vec3 normal;").unwrap_or(""),
        uv_attribute = attributes
            .uv
            .then_some("in vec2 uv_coordinates;")
            .unwrap_or(""),
        normal_uniform = attributes
            .normal
            .then_some("uniform mat4 normalMatrix;")
            .unwrap_or(""),
        normal_value = if attributes.normal {
            "k_normal = normalize((normalMatrix * vec4(normal, 0.0)).xyz);"
        } else {
            "k_normal = vec3(0.0);"
        },
        uv_value = if attributes.uv {
            "k_uv = uv_coordinates;"
        } else {
            "k_uv = vec2(0.0);"
        },
    )
}

fn validate_mesh_program(
    program: &three_d::Program,
    attributes: MeshAttributes,
) -> Result<(), String> {
    for required in ["position"] {
        if !program.requires_attribute(required) {
            return Err(format!(
                "Mesh shader vertex stage must actively use `{required}`."
            ));
        }
    }
    for required in ["modelMatrix", "viewProjection"] {
        if !program.requires_uniform(required) {
            return Err(format!(
                "Mesh shader vertex stage must actively use `{required}`."
            ));
        }
    }
    for (name, available) in [
        ("normal", attributes.normal),
        ("tangent", attributes.tangent),
        ("uv_coordinates", attributes.uv),
        ("color", attributes.color),
    ] {
        if program.requires_attribute(name) && !available {
            return Err(format!(
                "Mesh shader requires attribute `{name}`, but this geometry does not provide it."
            ));
        }
    }
    Ok(())
}

fn use_uniforms(
    program: &three_d::Program,
    shader: &EffectiveMeshShader,
    attributes: MeshAttributes,
    camera: &three_d::Camera,
    time: f32,
    material: &Material,
) -> Result<(), String> {
    program.use_uniform_if_required("viewMatrix", camera.view());
    program.use_uniform_if_required("projectionMatrix", camera.projection());
    program.use_uniform_if_required("cameraPosition", camera.position());
    program.use_uniform_if_required("sceneTime", time);
    let [r, g, b, a] = material.albedo.rgba();
    let color = three_d::Srgba::new(
        channel(r),
        channel(g),
        channel(b),
        channel(a * material.opacity),
    )
    .to_linear_srgb();
    program.use_uniform_if_required("materialColor", color);
    program.use_uniform_if_required("hasNormal", i32::from(attributes.normal));
    program.use_uniform_if_required("hasUv", i32::from(attributes.uv));
    for (name, value) in &shader.uniforms {
        if !program.requires_uniform(name) {
            return Err(format!(
                "Mesh shader uniform `{name}` is missing or inactive."
            ));
        }
        match value {
            TrackValue::F32(value) => program.use_uniform(name, *value),
            TrackValue::Vector2(value) => {
                program.use_uniform(name, three_d::vec2(value.x, value.y))
            }
            TrackValue::Vector3(value) => {
                program.use_uniform(name, three_d::vec3(value.x, value.y, value.z))
            }
            TrackValue::Quad(value) => {
                let [x, y, z, w] = value.to_array();
                program.use_uniform(name, three_d::vec4(x, y, z, w))
            }
            TrackValue::Quaternion(value) => {
                program.use_uniform(name, three_d::vec4(value.x, value.y, value.z, value.w))
            }
            TrackValue::Color(value) => {
                let [r, g, b, a] = value.rgba();
                program.use_uniform(name, three_d::vec4(r, g, b, a))
            }
            TrackValue::I32(value) => program.use_uniform(name, *value),
            TrackValue::U32(value) => program.use_uniform(name, *value),
            TrackValue::Bool(value) => program.use_uniform(name, i32::from(*value)),
            TrackValue::Enum(_) | TrackValue::String(_) => unreachable!(),
        }
    }
    Ok(())
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
    pub box_size: fn(&hecs::World, hecs::Entity) -> glam::Vec3,
}

impl Default for Draw3D {
    fn default() -> Self {
        Self {
            visibility: true,
            on_draw: |_, _, _| Ok(()),
            box_size: |_, _| glam::Vec3::ZERO,
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
