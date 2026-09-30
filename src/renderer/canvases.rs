use super::{
    image_shaders::ImageShaders,
    plan::{PlanCache, active_subtree, canvas_plan_for, visible_subtree_3d},
    target::{Target, reset_gl},
};
use crate::core::{
    Scene, SceneIdentity,
    components::{
        CachedGeometry, Camera2D, Camera3D, Camera3DMode, Draw2D, Draw3D, GeometryKey,
        MeshProgramCache, RenderContext3D,
    },
    objects::{
        CanvasDimension, CanvasSettings, CanvasTexture, DirectionalLightKind, ImageShaderData,
        ImageShaderImage, Light3D, LightAttenuation, PointLightKind, SpotLightCone, SpotLightKind,
        draw_canvas2d_with_shader_images, draw_image_shader_source, effective_mesh_shader,
        global_matrix3d, global_rotation3d, global_transform, image_shader_bounds,
        object_follows_camera,
    },
};
use crate::renderer::editor_guides::{EditorGuideRenderer, EditorGuides3D};
use glow::HasContext;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub(crate) struct Canvases {
    targets: HashMap<CanvasTexture, Target>,
    geometries: HashMap<GeometryKey, CachedGeometry>,
    mesh_programs: MeshProgramCache,
    plans: PlanCache,
    visible: Vec<hecs::Entity>,
    images: HashMap<CanvasTexture, skia_safe::Image>,
    shader_images: HashMap<hecs::Entity, ImageShaderImage>,
    shader_targets: HashMap<CanvasTexture, Target>,
    shader_outputs: HashSet<CanvasTexture>,
    object_targets: HashMap<(u64, hecs::Entity), (Target, Target)>,
    image_shaders: ImageShaders,
    used_geometries: HashSet<GeometryKey>,
    frame: u64,
    target_usage: HashMap<CanvasTexture, u64>,
    geometry_usage: HashMap<GeometryKey, u64>,
    physical: three_d::PhysicalMaterial,
    ambient: three_d::AmbientLight,
    sun: three_d::DirectionalLight,
    editor_guides: EditorGuideRenderer,
    context: three_d::Context,
    gl: Rc<glow::Context>,
}

impl Canvases {
    pub fn new(context: three_d::Context, gl: &Rc<glow::Context>) -> Self {
        Self {
            targets: HashMap::new(),
            geometries: HashMap::new(),
            mesh_programs: HashMap::new(),
            plans: PlanCache::default(),
            visible: Vec::new(),
            images: HashMap::new(),
            shader_images: HashMap::new(),
            shader_targets: HashMap::new(),
            shader_outputs: HashSet::new(),
            object_targets: HashMap::new(),
            image_shaders: ImageShaders::new(gl),
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
            editor_guides: EditorGuideRenderer::new(),
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
        let (order, sources) = {
            let plan = self.plans.get(scene, self.frame)?;
            (plan.order.clone(), plan.sources.clone())
        };
        self.render_targets(scene, &order, &sources, skia)?;

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
        let output = scene.view_texture();
        if order.contains(&output.entity) {
            self.rendered_target(output)
                .ok_or("Output canvas is unavailable.")?
                .present_to(final_target);
        } else {
            final_target.draw_skia(skia, |canvas| {
                canvas.clear(skia_safe::colors::BLACK);
            });
        }
        Ok(())
    }

    pub fn render_editor_2d(
        &mut self,
        scene: &Scene,
        entity: hecs::Entity,
        target: &mut Target,
        skia: &mut skia_safe::gpu::DirectContext,
        pan: [f32; 2],
        zoom: f32,
        correction: [f32; 2],
        camera_view: bool,
    ) -> Result<(), String> {
        let plan = canvas_plan_for(scene, entity)?;
        let dependencies = plan
            .order
            .iter()
            .copied()
            .filter(|candidate| *candidate != entity)
            .collect::<Vec<_>>();
        self.render_targets(scene, &dependencies, &plan.sources, skia)?;
        for key in &self.used_geometries {
            self.geometry_usage.insert(*key, self.frame);
        }

        let world = scene.world();
        let settings = world
            .get::<&CanvasSettings>(entity)
            .map_err(|_| "Selected canvas is unavailable.")?;
        if settings.dimension != CanvasDimension::Two {
            return Err("Selected canvas is not two-dimensional.".into());
        }

        self.images.clear();
        for source in &plan.sources[&entity] {
            let image = self
                .rendered_target(*source)
                .ok_or("Projection source texture is unavailable.")?
                .image(skia)?;
            self.images.insert(*source, image);
        }
        let scene_id = world
            .get::<&SceneIdentity>(scene.root().entity())
            .unwrap()
            .0;
        self.prepare_object_shaders(scene_id, scene.time(), &world, entity, skia)?;
        let target_size = target.size;
        target.draw_skia(skia, |canvas| {
            crate::core::objects::draw_canvas2d_editor_with_images(
                &world,
                entity,
                canvas,
                &self.images,
                &self.shader_images,
                target_size,
                pan,
                zoom,
                correction,
                camera_view,
            )
        });
        Ok(())
    }

    pub fn render_editor_3d(
        &mut self,
        scene: &Scene,
        entity: hecs::Entity,
        target: &mut Target,
        skia: &mut skia_safe::gpu::DirectContext,
        camera_component: &Camera3D,
        guides: &EditorGuides3D,
    ) -> Result<(), String> {
        let plan = canvas_plan_for(scene, entity)?;
        let dependencies = plan
            .order
            .iter()
            .copied()
            .filter(|candidate| *candidate != entity)
            .collect::<Vec<_>>();
        self.render_targets(scene, &dependencies, &plan.sources, skia)?;

        let world = scene.world();
        let settings = world
            .get::<&CanvasSettings>(entity)
            .map_err(|_| "Selected canvas is unavailable.")?;
        if settings.dimension != CanvasDimension::Three {
            return Err("Selected canvas is not three-dimensional.".into());
        }
        reset_gl(&self.gl, target.size);
        let camera = camera_from_component(camera_component, target.size)?;
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
        let resolve_texture = |texture: CanvasTexture| textures.get(&texture).map(Target::texture);
        visible_subtree_3d(&world, entity, &mut self.visible);
        let lights = scene_lights(&world, &self.visible, &self.context)?;
        let mut light_refs: Vec<&dyn three_d::Light> = vec![&self.ambient];
        if lights.is_empty() {
            light_refs.push(&self.sun);
        }
        light_refs.extend(lights.iter().map(|light| light.as_ref()));
        pass.target()
            .write(|| {
                let mut render = RenderContext3D::new(
                    &camera,
                    pass.target(),
                    &self.context,
                    &mut self.geometries,
                    &mut self.used_geometries,
                    &mut self.physical,
                    &light_refs,
                    &resolve_texture,
                    &mut self.mesh_programs,
                    scene.time(),
                );
                for child in &self.visible {
                    if let Ok(draw) = world.get::<&Draw3D>(*child) {
                        render.set_mesh_shader(
                            effective_mesh_shader(&world, *child).map_err(std::io::Error::other)?,
                        );
                        (draw.on_draw)(&world, *child, &mut render)
                            .map_err(std::io::Error::other)?;
                        render
                            .finish_mesh_shader((draw.box_size)(&world, *child) != glam::Vec3::ZERO)
                            .map_err(std::io::Error::other)?;
                    }
                }
                Ok::<_, std::io::Error>(())
            })
            .map_err(|error| error.to_string())?;
        pass.target()
            .write(|| {
                self.editor_guides
                    .render(&self.context, &camera, camera_component, guides)
                    .map_err(std::io::Error::other)
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn render_targets(
        &mut self,
        scene: &Scene,
        order: &[hecs::Entity],
        sources: &HashMap<hecs::Entity, Vec<CanvasTexture>>,
        skia: &mut skia_safe::gpu::DirectContext,
    ) -> Result<(), String> {
        let world = scene.world();
        let scene_id = world
            .get::<&SceneIdentity>(scene.root().entity())
            .unwrap()
            .0;
        self.images.clear();
        self.shader_images.clear();
        self.used_geometries.clear();
        self.shader_outputs.clear();

        for entity in order {
            let settings = world.get::<&CanvasSettings>(*entity).unwrap().clone();
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
                    self.images.clear();
                    for source in &sources[entity] {
                        let image = self
                            .rendered_target(*source)
                            .ok_or("Projection source texture is unavailable.")?
                            .image(skia)?;
                        self.images.insert(*source, image);
                    }
                    self.prepare_object_shaders(scene_id, scene.time(), &world, *entity, skia)?;
                    self.targets
                        .get_mut(&key)
                        .unwrap()
                        .draw_skia(skia, |canvas| {
                            draw_canvas2d_with_shader_images(
                                &world,
                                *entity,
                                canvas,
                                &self.images,
                                &self.shader_images,
                            )
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
                    let shader_targets = &self.shader_targets;
                    let shader_outputs = &self.shader_outputs;
                    let resolve_texture = |texture: CanvasTexture| {
                        if shader_outputs.contains(&texture) {
                            shader_targets.get(&texture).map(Target::texture)
                        } else {
                            textures.get(&texture).map(Target::texture)
                        }
                    };
                    let context = &self.context;
                    let geometries = &mut self.geometries;
                    let physical = &mut self.physical;
                    visible_subtree_3d(&world, *entity, &mut self.visible);
                    let lights = scene_lights(&world, &self.visible, context)?;
                    let mut light_refs: Vec<&dyn three_d::Light> = vec![&self.ambient];
                    if lights.is_empty() {
                        light_refs.push(&self.sun);
                    }
                    light_refs.extend(lights.iter().map(|light| light.as_ref()));
                    pass.target()
                        .write(|| {
                            let mut render = RenderContext3D::new(
                                &camera,
                                pass.target(),
                                context,
                                geometries,
                                &mut self.used_geometries,
                                physical,
                                &light_refs,
                                &resolve_texture,
                                &mut self.mesh_programs,
                                scene.time(),
                            );
                            for child in &self.visible {
                                if let Ok(draw) = world.get::<&Draw3D>(*child) {
                                    render.set_mesh_shader(
                                        effective_mesh_shader(&world, *child)
                                            .map_err(std::io::Error::other)?,
                                    );
                                    (draw.on_draw)(&world, *child, &mut render)
                                        .map_err(std::io::Error::other)?;
                                    render
                                        .finish_mesh_shader(
                                            (draw.box_size)(&world, *child) != glam::Vec3::ZERO,
                                        )
                                        .map_err(std::io::Error::other)?;
                                }
                            }
                            Ok::<_, std::io::Error>(())
                        })
                        .map_err(|error| error.to_string())?;
                }
            }
            self.apply_canvas_shader(scene.time(), &world, key, skia)?;
        }
        Ok(())
    }

    fn rendered_target(&self, texture: CanvasTexture) -> Option<&Target> {
        if self.shader_outputs.contains(&texture) {
            self.shader_targets.get(&texture)
        } else {
            self.targets.get(&texture)
        }
    }

    fn prepare_object_shaders(
        &mut self,
        scene_id: u64,
        time: f32,
        world: &hecs::World,
        canvas: hecs::Entity,
        skia: &mut skia_safe::gpu::DirectContext,
    ) -> Result<(), String> {
        self.shader_images.clear();
        let camera_zoom = world
            .get::<&Camera2D>(canvas)
            .map_or(1.0, |camera| camera.camera_zoom);
        let entities = active_subtree(world, canvas);
        for entity in entities.into_iter().rev() {
            let Ok(data) = world.get::<&ImageShaderData>(entity) else {
                continue;
            };
            let Ok(draw) = world.get::<&Draw2D>(entity) else {
                continue;
            };
            let Some(bounds) = image_shader_bounds(world, entity, data.padding) else {
                continue;
            };
            let scale = global_transform(world, entity).scale.abs();
            let camera = if object_follows_camera(world, canvas, entity) {
                camera_zoom
            } else {
                1.0
            };
            let requested = (
                (bounds.width() * scale.x * camera).ceil().max(1.0) as u32,
                (bounds.height() * scale.y * camera).ceil().max(1.0) as u32,
            );
            let size = (bucket(requested.0), bucket(requested.1));
            let key = (scene_id, entity);
            if self
                .object_targets
                .get(&key)
                .is_none_or(|targets| targets.0.size != size)
            {
                self.object_targets.insert(
                    key,
                    (
                        Target::new(size, false, skia, &self.gl)?,
                        Target::new(size, false, skia, &self.gl)?,
                    ),
                );
            }
            let textures = data
                .textures
                .values()
                .map(|texture| {
                    self.rendered_target(*texture)
                        .map(|target| (*texture, target.texture()))
                        .ok_or("Image shader texture is unavailable.")
                })
                .collect::<Result<HashMap<_, _>, _>>()?;
            let data = (*data).clone();
            let alpha = draw.opacity.clamp(0.0, 1.0);
            let targets = self.object_targets.get_mut(&key).unwrap();
            targets.0.draw_skia(skia, |surface| {
                let saved = surface.save();
                surface.clear(skia_safe::colors::TRANSPARENT);
                surface.scale((
                    size.0 as f32 / bounds.width(),
                    size.1 as f32 / bounds.height(),
                ));
                surface.translate((-bounds.left, -bounds.top));
                draw_image_shader_source(world, entity, surface, &self.images, &self.shader_images);
                surface.restore_to_count(saved);
            });
            self.image_shaders
                .apply(&targets.0, &targets.1, &data, time, alpha, |texture| {
                    textures.get(&texture).copied()
                })?;
            let image = targets.1.image(skia)?;
            self.shader_images
                .insert(entity, ImageShaderImage { image, bounds });
        }
        Ok(())
    }

    fn apply_canvas_shader(
        &mut self,
        time: f32,
        world: &hecs::World,
        key: CanvasTexture,
        skia: &mut skia_safe::gpu::DirectContext,
    ) -> Result<(), String> {
        let Ok(data) = world.get::<&ImageShaderData>(key.entity) else {
            return Ok(());
        };
        if data.padding != 0.0 {
            return Err(
                "Canvas image shaders have fixed output bounds and cannot use padding.".into(),
            );
        }
        let data = (*data).clone();
        let size = self.targets[&key].size;
        if self
            .shader_targets
            .get(&key)
            .is_none_or(|target| target.size != size)
        {
            self.shader_targets
                .insert(key, Target::new(size, false, skia, &self.gl)?);
        }
        let textures = data
            .textures
            .values()
            .map(|texture| {
                self.rendered_target(*texture)
                    .map(|target| (*texture, target.texture()))
                    .ok_or("Image shader texture is unavailable.")
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        self.image_shaders.apply(
            &self.targets[&key],
            &self.shader_targets[&key],
            &data,
            time,
            1.0,
            |texture| textures.get(&texture).copied(),
        )?;
        self.shader_outputs.insert(key);
        Ok(())
    }
}

fn bucket(size: u32) -> u32 {
    size.saturating_add(63) / 64 * 64
}

impl Drop for Canvases {
    fn drop(&mut self) {
        self.images.clear();
        self.shader_images.clear();
        self.targets.clear();
        self.shader_targets.clear();
        self.object_targets.clear();
        self.geometries.clear();
        self.mesh_programs.clear();
        // Program objects retain Context clones; clear this cache to break that cycle.
        if let Ok(mut programs) = self.context.programs.write() {
            programs.clear();
        }
    }
}

fn scene_lights(
    world: &hecs::World,
    visible: &[hecs::Entity],
    context: &three_d::Context,
) -> Result<Vec<Box<dyn three_d::Light>>, String> {
    let mut lights: Vec<Box<dyn three_d::Light>> = Vec::new();
    for &entity in visible {
        let Ok(light) = world.get::<&Light3D>(entity) else {
            continue;
        };
        if !light.intensity.is_finite() || light.intensity < 0.0 {
            return Err("Light intensity must be finite and nonnegative.".into());
        }
        let color = three_d::Srgba::new(
            light_channel(light.color.r),
            light_channel(light.color.g),
            light_channel(light.color.b),
            255,
        );
        let position = global_matrix3d(world, entity).transform_point3(glam::Vec3::ZERO);
        let direction = global_rotation3d(world, entity) * glam::Vec3::NEG_Z;
        let position = three_d::vec3(position.x, position.y, position.z);
        let direction = three_d::vec3(direction.x, direction.y, direction.z);
        if world.get::<&DirectionalLightKind>(entity).is_ok() {
            lights.push(Box::new(three_d::DirectionalLight::new(
                context,
                light.intensity,
                color,
                direction,
            )));
        } else {
            let attenuation = world.get::<&LightAttenuation>(entity).unwrap();
            let coefficients = [
                attenuation.constant,
                attenuation.linear,
                attenuation.quadratic,
            ];
            if coefficients
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0)
                || coefficients.iter().all(|value| *value == 0.0)
            {
                return Err("Light attenuation must have finite, nonnegative coefficients and at least one positive coefficient.".into());
            }
            let attenuation = three_d::Attenuation {
                constant: attenuation.constant,
                linear: attenuation.linear,
                quadratic: attenuation.quadratic,
            };
            if world.get::<&PointLightKind>(entity).is_ok() {
                lights.push(Box::new(three_d::PointLight::new(
                    context,
                    light.intensity,
                    color,
                    position,
                    attenuation,
                )));
            } else if world.get::<&SpotLightKind>(entity).is_ok() {
                let cutoff = world.get::<&SpotLightCone>(entity).unwrap().cutoff;
                if !cutoff.is_finite() || !(0.0..std::f32::consts::PI).contains(&cutoff) {
                    return Err("Spot light cutoff must be between zero and pi radians.".into());
                }
                lights.push(Box::new(three_d::SpotLight::new(
                    context,
                    light.intensity,
                    color,
                    position,
                    direction,
                    three_d::radians(cutoff),
                    attenuation,
                )));
            }
        }
    }
    Ok(lights)
}

fn light_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn camera(
    world: &hecs::World,
    entity: hecs::Entity,
    size: (u32, u32),
) -> Result<three_d::Camera, String> {
    let camera_component = world
        .get::<&Camera3D>(entity)
        .map_err(|_| "Camera3D lens is missing.")?;
    camera_from_component(&camera_component, size)
}

fn camera_from_component(
    camera_component: &Camera3D,
    size: (u32, u32),
) -> Result<three_d::Camera, String> {
    camera_component.validate()?;
    let matrix = camera_component.matrix();
    let rotation = crate::core::normalized_quaternion(camera_component.camera_rotation);
    let position = matrix.transform_point3(glam::Vec3::ZERO);
    let target = position + rotation * -glam::Vec3::Z;
    let up = rotation * glam::Vec3::Y;
    let convert = |v: glam::Vec3| three_d::vec3(v.x, v.y, v.z);
    let viewport = three_d::Viewport::new_at_origo(size.0, size.1);
    let mut camera = match camera_component.camera_mode {
        Camera3DMode::Perspective => three_d::Camera::new_perspective(
            viewport,
            convert(position),
            convert(target),
            convert(up),
            three_d::radians(camera_component.camera_fov),
            camera_component.camera_near,
            camera_component.camera_far,
        ),
        Camera3DMode::Orthogonal => three_d::Camera::new_orthographic(
            viewport,
            convert(position),
            convert(target),
            convert(up),
            camera_component.camera_fov,
            camera_component.camera_near,
            camera_component.camera_far,
        ),
    };
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
        let handler = scene.world_3d();
        for resolution in [(1920, 1080), (512, 1024)] {
            let view = camera(&scene.world(), handler.entity(), resolution).unwrap();
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
