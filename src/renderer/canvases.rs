use super::{
    plan::{active_subtree, canvas_order, visible_subtree_3d},
    target::{Target, reset_gl},
};
use crate::core::{
    Scene, SceneIdentity,
    components::{Draw3D, GeometryKey, RenderContext3D},
    objects::{
        CanvasDimension, CanvasSettings, CanvasTexture, Perspective, ProjectionSource,
        draw_canvas2d_with_images, global_matrix3d, global_rotation3d,
    },
};
use glow::HasContext;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub(crate) struct Canvases {
    targets: HashMap<CanvasTexture, Target>,
    geometries: HashMap<GeometryKey, three_d::Mesh>,
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
            geometries: HashMap::new(),
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
        let mut used_geometries = HashSet::new();

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
                    skia.reset(None);
                    let sources: HashSet<_> = active_subtree(&world, *entity)
                        .into_iter()
                        .filter_map(|entity| {
                            world
                                .get::<&ProjectionSource>(entity)
                                .ok()
                                .and_then(|source| source.0)
                        })
                        .collect();
                    let images: HashMap<_, _> = sources
                        .into_iter()
                        .map(|source| {
                            self.targets
                                .get(&source)
                                .ok_or("Projection source texture is unavailable.")?
                                .image(skia)
                                .map(|image| (source, image))
                        })
                        .collect::<Result<_, _>>()?;
                    self.targets
                        .get_mut(&key)
                        .unwrap()
                        .draw_skia(skia, |canvas| {
                            draw_canvas2d_with_images(&world, *entity, canvas, &images)
                        });
                }
                CanvasDimension::Three => {
                    reset_gl(&self.gl, settings.resolution);
                    let camera_entity = settings.camera.ok_or("Canvas3D requires a camera.")?;
                    let camera = camera(&world, camera_entity, settings.resolution)?;
                    let target = &self.targets[&key];
                    unsafe {
                        self.gl
                            .bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(target.framebuffer()));
                    }
                    let [r, g, b, a] = settings.clear.rgba();
                    let pass = FramebufferPass::new(&self.context, target);
                    pass.target().clear(three_d::ClearState::color_and_depth(
                        r * a,
                        g * a,
                        b * a,
                        a,
                        1.0,
                    ));
                    let textures = &self.targets;
                    let resolve_texture =
                        |texture: CanvasTexture| textures.get(&texture).map(Target::texture);
                    let context = &self.context;
                    let geometries = &mut self.geometries;
                    let physical = &mut self.physical;
                    let ambient = &self.ambient;
                    let sun = &self.sun;
                    pass.target()
                        .write(|| {
                            let mut render = RenderContext3D::new(
                                &camera,
                                pass.target(),
                                context,
                                geometries,
                                &mut used_geometries,
                                physical,
                                ambient,
                                sun,
                                &resolve_texture,
                            );
                            for child in visible_subtree_3d(&world, *entity) {
                                if let Ok(draw) = world.get::<&Draw3D>(child) {
                                    (draw.on_draw)(&world, child, &mut render)
                                        .map_err(std::io::Error::other)?;
                                }
                            }
                            Ok::<_, std::io::Error>(())
                        })
                        .map_err(|error| error.to_string())?;
                }
            }
        }

        self.geometries
            .retain(|key, _| used_geometries.contains(key));
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
        self.geometries.clear();
        // Program objects retain Context clones; clear this cache to break that cycle.
        if let Ok(mut programs) = self.context.programs.write() {
            programs.clear();
        }
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
