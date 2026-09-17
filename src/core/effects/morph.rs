use crate::core::{
    Easing, Task,
    components::{Draw2D, Morph as MorphState, PARTICLE_FADE_START, TreeNode},
    objects::{
        Object, ObjectHandler, ObjectTrackable, Rect,
        appearance::{AppearanceEdit, AppearanceSnapshot},
        attach_child, capture_appearance, deactivate_subtree,
        particle::ParticleTransform,
    },
};

/// Replaces an attached object with an unattached destination through particle silhouettes.
///
/// Both objects must belong to the same scene. The destination inherits the source's
/// parent and uses its own local position, scale, and rotation. Appearances are captured
/// when scheduled, including descendants and their colors. The destination becomes
/// active at completion and can then receive further effects.
pub struct MorphEffect {
    duration: f32,
    easing: Easing,
    fade_from: bool,
}

impl MorphEffect {
    /// Creates a two-second transformation into the destination object.
    pub fn new() -> Self {
        Self {
            duration: 2.0,
            easing: Easing::default(),
            fade_from: true,
        }
    }

    /// Sets a finite, positive duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        assert!(
            duration.is_finite() && duration > 0.0,
            "Morph duration must be finite and positive."
        );
        self.duration = duration;
        self
    }

    /// Sets the easing curve for particle travel.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets whether the source fades out when the morph begins.
    pub fn fade_from(mut self, fade_from: bool) -> Self {
        self.fade_from = fade_from;
        self
    }
}

/// Builds a particle morph into an unattached destination object.
pub fn morph() -> MorphEffect {
    MorphEffect::new()
}

impl MorphEffect {
    pub fn play<F, T>(self, from: &F, to: &T)
    where
        F: ObjectHandler,
        T: ObjectHandler,
        F::Object: ObjectTrackable<Draw2D>,
        T::Object: ObjectTrackable<Draw2D>,
    {
        let (world, animator) = from
            .animate(
                Draw2D::opacity_property(),
                from.get(Draw2D::opacity_property()),
            )
            .context();
        let source_opacity = stored_opacity(&world, from.entity());
        let target_opacity = stored_opacity(&world, to.entity());
        let (target_world, _) = to
            .animate(Draw2D::opacity_property(), target_opacity)
            .context();
        assert!(
            std::rc::Rc::ptr_eq(&world, &target_world),
            "Morph objects must belong to the same scene."
        );
        assert_ne!(
            from.entity(),
            to.entity(),
            "Morph requires distinct objects."
        );
        let start = animator.time();
        let end = start + self.duration;
        let (parent, from_silhouette, to_silhouette) = {
            let world = world.borrow();
            let node = world.get::<&TreeNode>(from.entity()).unwrap();
            let parent = node.parent.expect("Morph source must be attached.");
            assert!(
                node.lifetime[0] <= start && start < node.lifetime[1],
                "Morph source must be alive at the scheduled time."
            );
            let target_parent = world.get::<&TreeNode>(to.entity()).unwrap().parent;
            assert!(
                target_parent.is_none() || target_parent == Some(parent),
                "Morph objects must share the same parent."
            );
            let from_silhouette =
                capture_appearance(&world, from.entity(), parent, start, source_opacity);
            let to_silhouette =
                capture_appearance(&world, to.entity(), parent, start, target_opacity);
            (parent, from_silhouette, to_silhouette)
        };
        let data = ParticleTransform::new(from_silhouette, to_silhouette, self.easing);
        let object = Rect {
            draw: Draw2D {
                on_draw: draw_transform,
                box_size: |world, entity| {
                    let data = world.get::<&ParticleTransform>(entity).unwrap();
                    let bounds = union(data.from.bounds, data.to.bounds);
                    glam::vec2(
                        bounds.left.abs().max(bounds.right.abs()) * 2.0,
                        bounds.top.abs().max(bounds.bottom.abs()) * 2.0,
                    )
                },
                visual_bounds: |world, entity| {
                    let data = world.get::<&ParticleTransform>(entity).unwrap();
                    union(data.from.bounds, data.to.bounds)
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let carrier = Rect::spawn(
            world.clone(),
            animator.clone(),
            object,
            crate::core::components::Name::new("Morph"),
        );
        let endpoints = {
            let world = world.borrow();
            MorphEndpoints {
                from: from.entity(),
                to: to.entity(),
                parent,
                time: start,
                from_opacity: source_opacity,
                to_opacity: target_opacity,
                from_values: AppearanceSnapshot::capture(&world, from.entity()),
                to_values: AppearanceSnapshot::capture(&world, to.entity()),
            }
        };
        world
            .borrow_mut()
            .insert(carrier.entity(), (data, endpoints))
            .unwrap();
        attach_child(&world, parent, carrier.entity(), start);
        if world
            .borrow()
            .get::<&TreeNode>(to.entity())
            .unwrap()
            .parent
            .is_none()
        {
            attach_child(&world, parent, to.entity(), start);
        }
        {
            let world = world.borrow();
            let mut node = world.get::<&mut TreeNode>(parent).unwrap();
            let children = node.children.as_mut().unwrap();
            children.retain(|entity| *entity != carrier.entity() && *entity != to.entity());
            let index = children
                .iter()
                .position(|entity| *entity == from.entity())
                .unwrap()
                + 1;
            children.splice(index..index, [carrier.entity(), to.entity()]);
            drop(node);
            deactivate_subtree(&world, carrier.entity(), end);
        }
        let progress = MorphState::progress_property()
            .handle(world.clone(), carrier.entity(), animator.clone())
            .animate_from::<Rect>(0.0, 1.0)
            .duration(self.duration)
            .easing(Easing::Linear)
            .task();
        let fade_duration = self.duration * (1.0 - PARTICLE_FADE_START);
        let target_fade = Task::Chain(vec![
            Task::Wait(self.duration - fade_duration),
            to.animate_from(Draw2D::opacity_property(), 0.0, target_opacity)
                .duration(fade_duration)
                .easing(Easing::Linear)
                .task(),
        ]);
        let mut tasks = vec![progress, target_fade];
        if self.fade_from {
            tasks.push(
                from.animate_from(Draw2D::opacity_property(), source_opacity, 0.0)
                    .duration(fade_duration)
                    .easing(Easing::Linear)
                    .task(),
            );
        }
        animator.play(Task::All(tasks));
    }
}

struct MorphEndpoints {
    from: hecs::Entity,
    to: hecs::Entity,
    parent: hecs::Entity,
    time: f32,
    from_opacity: f32,
    to_opacity: f32,
    from_values: AppearanceSnapshot,
    to_values: AppearanceSnapshot,
}

pub(crate) fn refresh_morphs(world: &hecs::World, edits: &[AppearanceEdit]) {
    for (entity, endpoints) in world.query::<(hecs::Entity, &mut MorphEndpoints)>().iter() {
        let from_changed = endpoints.from_values.edit(edits);
        let to_changed = endpoints.to_values.edit(edits);
        if !from_changed && !to_changed {
            continue;
        }
        for edit in edits {
            if edit.component == std::any::TypeId::of::<Draw2D>() && edit.track.name == "opacity" {
                if let crate::core::TrackValue::F32(opacity) = edit.value {
                    if edit.entity == endpoints.from {
                        endpoints.from_opacity = opacity;
                    }
                    if edit.entity == endpoints.to {
                        endpoints.to_opacity = opacity;
                    }
                }
            }
        }
        let from = from_changed.then(|| {
            endpoints.from_values.with_values(world, || {
                capture_appearance(
                    world,
                    endpoints.from,
                    endpoints.parent,
                    endpoints.time,
                    endpoints.from_opacity,
                )
            })
        });
        let to = to_changed.then(|| {
            endpoints.to_values.with_values(world, || {
                capture_appearance(
                    world,
                    endpoints.to,
                    endpoints.parent,
                    endpoints.time,
                    endpoints.to_opacity,
                )
            })
        });
        let mut data = world.get::<&mut ParticleTransform>(entity).unwrap();
        if let Some(from) = from {
            data.from = from;
        }
        if let Some(to) = to {
            data.to = to;
        }
        data.rebuild_routes();
    }
}

#[derive(Clone, Copy)]
struct MorphOpacity(f32);

fn stored_opacity(world: &crate::core::SceneWorld, entity: hecs::Entity) -> f32 {
    if let Ok(opacity) = world.borrow().get::<&MorphOpacity>(entity) {
        return opacity.0;
    }

    let opacity = world.borrow().get::<&Draw2D>(entity).unwrap().opacity;
    world
        .borrow_mut()
        .insert_one(entity, MorphOpacity(opacity))
        .unwrap();
    opacity
}

fn union(a: skia_safe::Rect, b: skia_safe::Rect) -> skia_safe::Rect {
    skia_safe::Rect::new(
        a.left.min(b.left),
        a.top.min(b.top),
        a.right.max(b.right),
        a.bottom.max(b.bottom),
    )
}

fn draw_transform(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) {
    let data = world.get::<&ParticleTransform>(entity).unwrap();
    let progress = world
        .get::<&MorphState>(entity)
        .unwrap()
        .progress
        .clamp(0.0, 1.0);
    data.draw(canvas, progress, opacity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene, SceneBuilder,
        types::{Color, vec2},
    };
    use crate::prelude::*;

    fn pixels(scene: &Scene, time: f32) -> Vec<skia_safe::Color> {
        scene.update(time);
        let mut surface = skia_safe::surfaces::raster_n32_premul((160, 80)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((80.0, 40.0));
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();
        (0..80)
            .flat_map(|y| (0..160).map(move |x| (x, y)))
            .map(|point| pixels.get_color(point))
            .collect()
    }

    struct MorphScene;
    impl SceneBuilder for MorphScene {
        fn build(&mut self, scene: &mut Scene) {
            let source = rect()
                .size(vec2(20.0, 20.0))
                .position(vec2(-30.0, 0.0))
                .fill(Color::RED)
                .build(scene);
            let target = circle()
                .radius(10.0)
                .position(vec2(30.0, 0.0))
                .fill(Color::BLUE)
                .build(scene);
            scene.world_2d().add(&source);
            scene.wait(1.0);
            morph()
                .duration(2.0)
                .easing(Easing::Linear)
                .play(&source, &target);
            target.position_y(10.0).duration(1.0).play();
        }
    }

    #[test]
    fn morph_moves_colored_particles_and_restores_lifetimes_when_seeking() {
        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut MorphScene), 4.0);
        let before = pixels(&scene, 0.5);
        let _start = pixels(&scene, 1.0);
        let middle = pixels(&scene, 2.0);
        assert!(
            middle
                .iter()
                .any(|color| color.r() > 80 && color.b() > 80 && color.a() > 0)
        );
        let end = pixels(&scene, 3.0);
        assert_eq!(end[40 * 160 + 110], skia_safe::Color::BLUE);
        assert_eq!(end[40 * 160 + 50].a(), 0);
        assert_eq!(pixels(&scene, 2.0), middle);
        assert_eq!(pixels(&scene, 0.5), before);
        let world = scene.world();
        let mut nodes = world.query::<(hecs::Entity, &TreeNode)>();
        assert_eq!(
            nodes
                .iter()
                .filter(|(entity, node)| {
                    node.is_activated && world.get::<&CanvasSettings>(*entity).is_err()
                })
                .count(),
            2
        );
    }

    #[test]
    fn morph_captures_text_and_groups_and_allows_chaining() {
        struct Groups;
        impl SceneBuilder for Groups {
            fn build(&mut self, scene: &mut Scene) {
                let parent = group_2d()
                    .position(vec2(12.0, 3.0))
                    .opacity(0.5)
                    .build(scene);
                let source = group_2d().build(scene);
                let child = rect().size(vec2(12.0, 12.0)).fill(Color::RED).build(scene);
                source.add(&child);
                parent.add(&source);
                scene.world_2d().add(&parent);
                let target = text_2d().text("A").size(20.0).build(scene);
                morph().duration(1.0).play(&source, &target);
                let next = circle().radius(8.0).build(scene);
                morph().duration(1.0).play(&target, &next);
            }
        }
        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut Groups), 2.0);
        for time in [0.0, 0.5, 1.0, 1.5, 2.0, 0.0] {
            let image = pixels(&scene, time);
            let _ = image;
            assert!(image.iter().all(|color| color.a() <= 128));
        }
    }

    #[test]
    fn morph_can_return_to_a_previous_object_without_rebuilding_it() {
        struct RoundTrip;

        impl SceneBuilder for RoundTrip {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().radius(10.0).fill(Color::RED).build(scene);
                let rect = rect().size(vec2(20.0, 20.0)).fill(Color::BLUE).build(scene);
                scene.world_2d().add(&circle);

                morph().duration(1.0).play(&circle, &rect);
                morph().duration(1.0).play(&rect, &circle);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut RoundTrip), 2.0);
        let image = pixels(&scene, 2.0);

        assert_eq!(image[40 * 160 + 80], skia_safe::Color::RED);
    }

    #[test]
    fn morph_can_keep_source_visible() {
        struct KeepSource;

        impl SceneBuilder for KeepSource {
            fn build(&mut self, scene: &mut Scene) {
                let source = rect()
                    .size(vec2(20.0, 20.0))
                    .position(vec2(-30.0, 0.0))
                    .fill(Color::RED)
                    .build(scene);
                let target = circle()
                    .radius(10.0)
                    .position(vec2(30.0, 0.0))
                    .fill(Color::BLUE)
                    .build(scene);
                scene.world_2d().add(&source);

                morph()
                    .duration(1.0)
                    .fade_from(false)
                    .play(&source, &target);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut KeepSource), 1.0);
        let image = pixels(&scene, 1.0);

        assert_eq!(image[40 * 160 + 50], skia_safe::Color::RED);
        assert_eq!(image[40 * 160 + 110], skia_safe::Color::BLUE);
    }

    #[test]
    fn morph_preserves_sibling_order() {
        struct Overlap;
        impl SceneBuilder for Overlap {
            fn build(&mut self, scene: &mut Scene) {
                let source = rect().size(vec2(30.0, 30.0)).fill(Color::RED).build(scene);
                let overlay = rect()
                    .size(vec2(10.0, 10.0))
                    .fill(Color::GREEN)
                    .build(scene);
                let target = circle().radius(15.0).fill(Color::BLUE).build(scene);
                scene.world_2d().add(&source);
                scene.world_2d().add(&overlay);
                morph().duration(1.0).play(&source, &target);
            }
        }
        let mut scene = Scene::new();
        scene.build(&mut Overlap);
        for time in [0.0, 0.5, 1.0] {
            assert_eq!(pixels(&scene, time)[40 * 160 + 80], skia_safe::Color::GREEN);
        }
    }

    #[test]
    fn morph_capture_preserves_rotations_under_nonuniform_parent_scale() {
        struct Nested;
        impl SceneBuilder for Nested {
            fn build(&mut self, scene: &mut Scene) {
                let parent = group_2d().scale(vec2(2.0, 0.7)).rotation(0.4).build(scene);
                let source = rect()
                    .size(vec2(30.0, 12.0))
                    .rotation(0.6)
                    .fill(Color::RED)
                    .build(scene);
                let target = group_2d().build(scene);
                let child = circle().radius(10.0).build(scene);
                target.add(&child);
                parent.add(&source);
                scene.world_2d().add(&parent);
                scene.wait(1.0);
                morph().duration(1.0).play(&source, &target);
            }
        }
        let mut scene = Scene::new();
        scene.build(&mut Nested);
        let before = pixels(&scene, 0.5);
        let start = pixels(&scene, 1.0);
        let difference = before
            .iter()
            .zip(start.iter())
            .filter(|(a, b)| (a.a() > 127) != (b.a() > 127))
            .count();
        assert!(difference < 600, "Silhouette moved by {difference} pixels.");
        assert!(pixels(&scene, 2.0).iter().any(|color| color.a() > 0));
    }

    #[test]
    #[should_panic(expected = "Morph objects must belong to the same scene.")]
    fn morph_rejects_foreign_handlers_even_when_entity_ids_match() {
        let mut scene = Scene::new();
        let mut other = Scene::new();
        let source = rect().build(&mut scene);
        let target = circle().build(&mut other);
        scene.world_2d().add(&source);
        morph().play(&source, &target);
    }

    #[test]
    fn morph_drawing_does_not_change_evaluated_state() {
        let mut scene = Scene::new();
        scene.build(&mut MorphScene);
        let first = pixels(&scene, 2.0);
        let before = {
            let world = scene.world();
            world
                .query::<(&TreeNode, &MorphState)>()
                .iter()
                .map(|(node, morph)| (node.is_activated, morph.progress))
                .collect::<Vec<_>>()
        };
        let mut surface = skia_safe::surfaces::raster_n32_premul((160, 80)).unwrap();
        scene.draw(surface.canvas());
        let after = {
            let world = scene.world();
            world
                .query::<(&TreeNode, &MorphState)>()
                .iter()
                .map(|(node, morph)| (node.is_activated, morph.progress))
                .collect::<Vec<_>>()
        };
        assert_eq!(before, after);
        assert_eq!(first, pixels(&scene, 2.0));
    }
}
