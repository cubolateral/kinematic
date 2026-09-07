use super::{
    plan::{active_subtree, canvas_order},
    target::{Target, reset_gl},
};
use crate::core::{
    Scene, SceneIdentity,
    components::Material,
    objects::{
        CanvasDimension, CanvasSettings, CanvasTexture, CuboidShape, Perspective, PlaneShape,
        ProjectionSource, SphereShape, draw_canvas2d, global_matrix3d, global_rotation3d,
    },
};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
use three_d::Geometry;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum MeshKey {
    Cuboid,
    Sphere(u32),
    Plane,
}

pub(crate) struct Canvases {
    targets: HashMap<CanvasTexture, Target>,
    meshes: HashMap<MeshKey, three_d::Mesh>,
    physical: three_d::PhysicalMaterial,
    ambient: three_d::AmbientLight,
    sun: three_d::DirectionalLight,
    context: three_d::Context,
    gl: Rc<glow::Context>,
}

impl Canvases {
    pub fn new(context: three_d::Context, gl: &Rc<glow::Context>) -> Self {
        Self {
            targets: HashMap::new(),
            meshes: HashMap::new(),
            physical: three_d::PhysicalMaterial::default(),
            ambient: three_d::AmbientLight::new(&context, 0.4, three_d::Srgba::WHITE),
            sun: three_d::DirectionalLight::new(
                &context,
                2.0,
                three_d::Srgba::WHITE,
                three_d::vec3(-1.0, -2.0, -3.0),
            ),
            context,
            gl: Rc::clone(gl),
        }
    }

    pub fn render(
        &mut self,
        scene: &Scene,
        final_target: &mut Target,
        skia: &mut skia_safe::gpu::DirectContext,
    ) -> Result<(), String> {
        let order = canvas_order(scene)?;
        let world = scene.get_world();
        let scene_id = world
            .get::<&SceneIdentity>(scene.get_root().get_id())
            .unwrap()
            .0;
        self.targets
            .retain(|key, _| key.scene == scene_id && world.contains(key.entity));
        let mut used_meshes = HashSet::new();
        for entity in &order {
            let settings = world.get::<&CanvasSettings>(*entity).unwrap();
            let key = CanvasTexture {
                scene: scene_id,
                entity: *entity,
            };
            if self
                .targets
                .get(&key)
                .is_none_or(|target| target.size != settings.resolution)
            {
                let target = Target::new(
                    settings.resolution,
                    settings.dimension == CanvasDimension::Three,
                    skia,
                    &self.gl,
                )?;
                self.targets.insert(key, target);
            }
            match settings.dimension {
                CanvasDimension::Two => {
                    self.targets
                        .get_mut(&key)
                        .unwrap()
                        .draw_skia(skia, |canvas| draw_canvas2d(&world, *entity, canvas));
                }
                CanvasDimension::Three => {
                    reset_gl(&self.gl, settings.resolution);
                    let camera_entity = settings.camera.ok_or("Canvas3D requires a camera.")?;
                    let camera = camera(&world, camera_entity, settings.resolution)?;
                    let camera_position =
                        global_matrix3d(&world, camera_entity).transform_point3(glam::Vec3::ZERO);
                    let forward = global_rotation3d(&world, camera_entity) * -glam::Vec3::Z;
                    let mut objects = Vec::new();
                    for child in active_subtree(&world, *entity) {
                        if let Some((mesh_key, scale)) = geometry(&world, child)? {
                            let matrix =
                                global_matrix3d(&world, child) * glam::Mat4::from_scale(scale);
                            if !matrix.is_finite() {
                                return Err("Object transform must be finite.".into());
                            }
                            if matrix.determinant().abs() <= f32::EPSILON {
                                continue;
                            }
                            used_meshes.insert(mesh_key);
                            let transparent = world.get::<&ProjectionSource>(child).is_ok()
                                || world
                                    .get::<&Material>(child)
                                    .is_ok_and(|m| m.opacity * m.albedo.rgba()[3] < 1.0);
                            let distance = (matrix.transform_point3(glam::Vec3::ZERO)
                                - camera_position)
                                .dot(forward);
                            objects.push((child, mesh_key, matrix, transparent, distance));
                            self.meshes.entry(mesh_key).or_insert_with(|| {
                                three_d::Mesh::new(&self.context, &cpu_mesh(mesh_key))
                            });
                        }
                    }
                    objects.sort_by(|a, b| {
                        a.3.cmp(&b.3).then_with(|| {
                            if a.3 {
                                b.4.total_cmp(&a.4)
                            } else {
                                a.4.total_cmp(&b.4)
                            }
                        })
                    });
                    let target = &self.targets[&key];
                    let [r, g, b, a] = settings.clear.rgba();
                    let pass = FramebufferPass::new(&self.context, target);
                    pass.target().clear(three_d::ClearState::color_and_depth(
                        r * a,
                        g * a,
                        b * a,
                        a,
                        1.0,
                    ));
                    pass.target()
                        .write(|| -> Result<(), std::io::Error> {
                            for (child, mesh_key, matrix, transparent, _) in objects {
                                let mesh = self.meshes.get_mut(&mesh_key).unwrap();
                                mesh.set_transformation(matrix.to_cols_array_2d().into());
                                if let Ok(source) = world.get::<&ProjectionSource>(child) {
                                    let texture = self.targets[&source.0.unwrap()].texture();
                                    mesh.render_with_material(
                                        &ProjectionMaterial { texture },
                                        &camera,
                                        &[],
                                    );
                                } else {
                                    let data = world.get::<&Material>(child).unwrap();
                                    let [r, g, b, a] = data.albedo.rgba();
                                    let color = three_d::Srgba::new(
                                        channel(r),
                                        channel(g),
                                        channel(b),
                                        channel(a * data.opacity),
                                    );
                                    let states = render_states(transparent, false);
                                    if data.unlit {
                                        let material = three_d::ColorMaterial {
                                            color,
                                            texture: None,
                                            render_states: states,
                                            is_transparent: transparent,
                                        };
                                        mesh.render_with_material(&material, &camera, &[]);
                                    } else {
                                        self.physical.albedo = color;
                                        self.physical.metallic = data.metallic.clamp(0.0, 1.0);
                                        self.physical.roughness = data.roughness.clamp(0.04, 1.0);
                                        self.physical.is_transparent = transparent;
                                        self.physical.render_states = states;
                                        mesh.render_with_material(
                                            &self.physical,
                                            &camera,
                                            &[&self.ambient, &self.sun],
                                        );
                                    }
                                }
                            }
                            Ok(())
                        })
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        self.meshes.retain(|key, _| used_meshes.contains(key));
        let output = scene.get_view();
        if order.contains(&output.entity) {
            self.targets
                .get(&output)
                .ok_or("Output canvas is unavailable.")?
                .present_to(final_target);
        } else {
            final_target.draw_skia(skia, |canvas| {
                canvas.clear(skia_safe::colors::BLACK);
            });
        }
        Ok(())
    }
}

impl Drop for Canvases {
    fn drop(&mut self) {
        self.targets.clear();
        self.meshes.clear();
        // Program objects retain Context clones; clear this cache to break that cycle.
        if let Ok(mut programs) = self.context.programs.write() {
            programs.clear();
        }
    }
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn geometry(
    world: &hecs::World,
    entity: hecs::Entity,
) -> Result<Option<(MeshKey, glam::Vec3)>, String> {
    let result = if let Ok(shape) = world.get::<&CuboidShape>(entity) {
        Some((MeshKey::Cuboid, shape.size * 0.5))
    } else if let Ok(shape) = world.get::<&SphereShape>(entity) {
        if !(3..=256).contains(&shape.segments) {
            return Err("Sphere segments must be between 3 and 256.".into());
        }
        Some((
            MeshKey::Sphere(shape.segments),
            glam::Vec3::splat(shape.radius),
        ))
    } else if let Ok(shape) = world.get::<&PlaneShape>(entity) {
        Some((MeshKey::Plane, (shape.size * 0.5).extend(1.0)))
    } else {
        None
    };
    if result.is_some_and(|(_, size)| !size.is_finite() || size.min_element() < 0.0) {
        return Err("Primitive dimensions must be finite and nonnegative.".into());
    }
    Ok(result)
}

fn cpu_mesh(key: MeshKey) -> three_d::CpuMesh {
    match key {
        MeshKey::Cuboid => three_d::CpuMesh::cube(),
        MeshKey::Sphere(segments) => three_d::CpuMesh::sphere(segments),
        MeshKey::Plane => three_d::CpuMesh::square(),
    }
}

fn camera(
    world: &hecs::World,
    entity: hecs::Entity,
    size: (u32, u32),
) -> Result<three_d::Camera, String> {
    let lens = world
        .get::<&Perspective>(entity)
        .map_err(|_| "Camera3D lens is missing.")?;
    lens.validate()?;
    let matrix = global_matrix3d(world, entity);
    if !matrix.is_finite() {
        return Err("Camera transform must be finite.".into());
    }
    let rotation = global_rotation3d(world, entity);
    let position = matrix.transform_point3(glam::Vec3::ZERO);
    let target = position + rotation * -glam::Vec3::Z;
    let up = rotation * glam::Vec3::Y;
    let convert = |v: glam::Vec3| three_d::vec3(v.x, v.y, v.z);
    let mut camera = three_d::Camera::new_perspective(
        three_d::Viewport::new_at_origo(size.0, size.1),
        convert(position),
        convert(target),
        convert(up),
        three_d::radians(lens.fov),
        lens.near,
        lens.far,
    );
    camera.tone_mapping = three_d::ToneMapping::None;
    Ok(camera)
}

fn render_states(transparent: bool, premultiplied: bool) -> three_d::RenderStates {
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

struct ProjectionMaterial {
    texture: glow::NativeTexture,
}
impl three_d::Material for ProjectionMaterial {
    fn id(&self) -> three_d::EffectMaterialId {
        three_d::EffectMaterialId(0)
    }
    fn fragment_shader_source(&self, _: &[&dyn three_d::Light]) -> String {
        // Skia and the output use display-encoded premultiplied RGBA. No gamma conversion here.
        // Mesh upload already flips the asset UVs into OpenGL orientation.
        "in vec2 uvs; uniform sampler2D source; out vec4 outColor;
        void main() { outColor = texture(source, uvs); }"
            .into()
    }
    fn use_uniforms(
        &self,
        program: &three_d::Program,
        _: &dyn three_d::Viewer,
        _: &[&dyn three_d::Light],
    ) {
        // Three-d 0.19 cannot borrow an externally owned Texture2D through its typed API.
        #[allow(deprecated)]
        program.use_raw_texture("source", glow::TEXTURE_2D, self.texture);
    }
    fn render_states(&self) -> three_d::RenderStates {
        render_states(true, true)
    }
    fn material_type(&self) -> three_d::MaterialType {
        three_d::MaterialType::Transparent
    }
}

/// Borrows a framebuffer owned by Target without transferring its destruction to three-d.
struct FramebufferPass(Option<three_d::RenderTarget<'static>>);
impl FramebufferPass {
    fn new(context: &three_d::Context, target: &Target) -> Self {
        Self(Some(three_d::RenderTarget::from_framebuffer(
            context,
            target.size.0,
            target.size.1,
            target.framebuffer(),
        )))
    }
    fn target(&self) -> &three_d::RenderTarget<'static> {
        self.0.as_ref().unwrap()
    }
}
impl Drop for FramebufferPass {
    fn drop(&mut self) {
        if let Some(target) = self.0.take() {
            target.into_framebuffer();
        }
    }
}

#[cfg(test)]
#[path = "graphics_tests.rs"]
mod graphics_tests;

#[cfg(test)]
mod tests {
    use super::camera;
    use crate::prelude::*;
    use three_d::Viewer;

    #[test]
    fn perspective_aspect_follows_canvas_resolution() {
        let mut scene = Scene::new();
        let handler = camera_3d().build(&mut scene);
        for resolution in [(1920, 1080), (512, 1024)] {
            let view = camera(&scene.get_world(), handler.get_id(), resolution).unwrap();
            let projection = view.projection();
            let aspect = projection.y.y / projection.x.x;
            assert!((aspect - resolution.0 as f32 / resolution.1 as f32).abs() < 1e-5);
        }
    }
}
