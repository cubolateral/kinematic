use super::{
    plan::{PlanCache, visible_subtree_3d},
    target::{Target, reset_gl},
};
use crate::core::{
    Scene, SceneIdentity,
    components::{Camera3D, Draw3D, GeometryKey, RenderContext3D},
    objects::{CanvasDimension, CanvasSettings, CanvasTexture, draw_canvas2d_with_images},
};
use glow::HasContext;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub(crate) struct Canvases {
    targets: HashMap<CanvasTexture, Target>,
    geometries: HashMap<GeometryKey, three_d::Mesh>,
    plans: PlanCache,
    visible: Vec<hecs::Entity>,
    images: HashMap<CanvasTexture, skia_safe::Image>,
    used_geometries: HashSet<GeometryKey>,
    frame: u64,
    target_usage: HashMap<CanvasTexture, u64>,
    geometry_usage: HashMap<GeometryKey, u64>,
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
            plans: PlanCache::default(),
            visible: Vec::new(),
            images: HashMap::new(),
            used_geometries: HashSet::new(),
            frame: 0,
            target_usage: HashMap::new(),
            geometry_usage: HashMap::new(),
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
        self.frame += 1;
        let plan = self.plans.get(scene, self.frame)?;
        let order = &plan.order;
        let world = scene.get_world();
        let scene_id = world
            .get::<&SceneIdentity>(scene.get_root().get_id())
            .unwrap()
            .0;
        self.used_geometries.clear();

        for entity in order {
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

            self.target_usage.insert(key, self.frame);

            match settings.dimension {
                CanvasDimension::Two => {
                    skia.reset(None);
                    self.images.clear();
                    for source in &plan.sources[entity] {
                        let image = self
                            .targets
                            .get(source)
                            .ok_or("Projection source texture is unavailable.")?
                            .image(skia)?;
                        self.images.insert(*source, image);
                    }
                    self.targets
                        .get_mut(&key)
                        .unwrap()
                        .draw_skia(skia, |canvas| {
                            draw_canvas2d_with_images(&world, *entity, canvas, &self.images)
                        });
                }
                CanvasDimension::Three => {
                    reset_gl(&self.gl, settings.resolution);
                    let camera = camera(&world, *entity, settings.resolution)?;
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
                    visible_subtree_3d(&world, *entity, &mut self.visible);
                    pass.target()
                        .write(|| {
                            let mut render = RenderContext3D::new(
                                &camera,
                                pass.target(),
                                context,
                                geometries,
                                &mut self.used_geometries,
                                physical,
                                ambient,
                                sun,
                                &resolve_texture,
                            );
                            for child in &self.visible {
                                if let Ok(draw) = world.get::<&Draw3D>(*child) {
                                    (draw.on_draw)(&world, *child, &mut render)
                                        .map_err(std::io::Error::other)?;
                                }
                            }
                            Ok::<_, std::io::Error>(())
                        })
                        .map_err(|error| error.to_string())?;
                }
            }
        }

        self.images.clear();
        for key in &self.used_geometries {
            self.geometry_usage.insert(*key, self.frame);
        }
        // Retain recently used resources across scene switches; reclaim at most one of each per render.
        let target_bytes: u64 = self
            .targets
            .values()
            .map(|target| u64::from(target.size.0) * u64::from(target.size.1) * 8)
            .sum();
        evict_oldest(
            &mut self.targets,
            &mut self.target_usage,
            self.frame,
            target_bytes > 256 * 1024 * 1024,
        );
        let over_budget = self.geometries.len() > 256;
        evict_oldest(
            &mut self.geometries,
            &mut self.geometry_usage,
            self.frame,
            over_budget,
        );
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
        self.images.clear();
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
    let camera_component = world
        .get::<&Camera3D>(entity)
        .map_err(|_| "Camera3D lens is missing.")?;
    camera_component.validate()?;
    let matrix = camera_component.matrix();
    let rotation = crate::core::normalized_quaternion(camera_component.camera_rotation);
    let position = matrix.transform_point3(glam::Vec3::ZERO);
    let target = position + rotation * -glam::Vec3::Z;
    let up = rotation * glam::Vec3::Y;
    let convert = |v: glam::Vec3| three_d::vec3(v.x, v.y, v.z);
    let mut camera = three_d::Camera::new_perspective(
        three_d::Viewport::new_at_origo(size.0, size.1),
        convert(position),
        convert(target),
        convert(up),
        three_d::radians(camera_component.camera_fov),
        camera_component.camera_near,
        camera_component.camera_far,
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
    fn eviction_retains_recent_resources_and_reclaims_one_old_resource() {
        use super::evict_oldest;
        use std::collections::HashMap;
        let mut cache = HashMap::from([(1, "First"), (2, "Second"), (3, "Current")]);
        let mut usage = HashMap::from([(1, 1), (2, 2), (3, 3)]);
        evict_oldest(&mut cache, &mut usage, 3, false);
        assert_eq!(cache.len(), 3);
        evict_oldest(&mut cache, &mut usage, 3, true);
        assert!(!cache.contains_key(&1));
        assert!(cache.contains_key(&3));
        evict_oldest(&mut cache, &mut usage, 604, false);
        assert_eq!(cache.len(), 1);
        assert!(cache.contains_key(&3));
    }

    #[test]
    fn perspective_aspect_follows_canvas_resolution() {
        let scene = Scene::new();
        let handler = scene.get_world_3d();
        for resolution in [(1920, 1080), (512, 1024)] {
            let view = camera(&scene.get_world(), handler.get_id(), resolution).unwrap();
            let projection = view.projection();
            let aspect = projection.y.y / projection.x.x;
            assert!((aspect - resolution.0 as f32 / resolution.1 as f32).abs() < 1e-5);
        }
    }
}

fn evict_oldest<K: Copy + Eq + std::hash::Hash, V>(
    cache: &mut HashMap<K, V>,
    usage: &mut HashMap<K, u64>,
    frame: u64,
    over_budget: bool,
) {
    if let Some(key) = usage
        .iter()
        .filter(|(_, used)| **used < frame && (over_budget || frame.saturating_sub(**used) > 600))
        .min_by_key(|(_, used)| **used)
        .map(|(key, _)| *key)
    {
        cache.remove(&key);
        usage.remove(&key);
    }
}
