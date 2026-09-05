use crate::core::{
    Easing, Task,
    components::{Draw, Node, PARTICLE_COUNT, PARTICLE_FADE_START, Style, stroke_width_for_scale},
    objects::{
        GlobalTransform, Object, ObjectHandler, ObjectTrackable, Rect, attach_child, children,
        deactivate_subtree, global_transform, local_transform,
        particle::{ParticleTransform, Silhouette},
    },
    types::Vector2,
};

/// Replaces an attached object with an unattached destination through particle silhouettes.
///
/// Both objects must belong to the same scene. The destination inherits the source's
/// parent and uses its own local position, scale, and rotation. Appearances are captured
/// when scheduled, including descendants and their colors. The destination becomes
/// active at completion and can then receive further effects.
pub struct Morph {
    duration: f32,
    easing: Easing,
    fade_from: bool,
}

impl Morph {
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
pub fn morph() -> Morph {
    Morph::new()
}

impl Morph {
    pub fn play<F, T>(self, from: &F, to: &T)
    where
        F: ObjectHandler,
        T: ObjectHandler,
        F::Object: ObjectTrackable<Draw>,
        T::Object: ObjectTrackable<Draw>,
    {
        let (world, animator) = from
            .animate(Draw::opacity_property(), from.get(Draw::opacity_property()))
            .context();
        let source_opacity = stored_opacity(&world, from.get_id());
        let target_opacity = stored_opacity(&world, to.get_id());
        let (target_world, _) = to
            .animate(Draw::opacity_property(), target_opacity)
            .context();
        assert!(
            std::rc::Rc::ptr_eq(&world, &target_world),
            "Morph objects must belong to the same scene."
        );
        assert_ne!(
            from.get_id(),
            to.get_id(),
            "Morph requires distinct objects."
        );
        let start = animator.time();
        let end = start + self.duration;
        let (parent, from_silhouette, to_silhouette) = {
            let world = world.borrow();
            let node = world.get::<&Node>(from.get_id()).unwrap();
            let parent = node.parent.expect("Morph source must be attached.");
            assert!(
                node.lifetime[0] <= start && start < node.lifetime[1],
                "Morph source must be alive at the scheduled time."
            );
            let target_parent = world.get::<&Node>(to.get_id()).unwrap().parent;
            assert!(
                target_parent.is_none() || target_parent == Some(parent),
                "Morph objects must share the same parent."
            );
            let count = PARTICLE_COUNT as usize;
            let from_silhouette =
                capture(&world, from.get_id(), parent, count, start, source_opacity);
            let to_silhouette = capture(&world, to.get_id(), parent, count, start, target_opacity);
            (parent, from_silhouette, to_silhouette)
        };
        let data = ParticleTransform {
            from: from_silhouette,
            to: to_silhouette,
            easing: self.easing,
        };
        let object = Rect {
            draw: Draw {
                on_draw: draw_transform,
                get_box: |world, entity| {
                    let data = world.get::<&ParticleTransform>(entity).unwrap();
                    let bounds = union(data.from.bounds, data.to.bounds);
                    Vector2::new(
                        bounds.left.abs().max(bounds.right.abs()) * 2.0,
                        bounds.top.abs().max(bounds.bottom.abs()) * 2.0,
                    )
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
        world
            .borrow_mut()
            .insert_one(carrier.get_id(), data)
            .unwrap();
        attach_child(&world, parent, carrier.get_id(), start);
        if world
            .borrow()
            .get::<&Node>(to.get_id())
            .unwrap()
            .parent
            .is_none()
        {
            attach_child(&world, parent, to.get_id(), start);
        }
        {
            let world = world.borrow();
            let mut node = world.get::<&mut Node>(parent).unwrap();
            let children = node.children.as_mut().unwrap();
            children.retain(|entity| *entity != carrier.get_id() && *entity != to.get_id());
            let index = children
                .iter()
                .position(|entity| *entity == from.get_id())
                .unwrap()
                + 1;
            children.splice(index..index, [carrier.get_id(), to.get_id()]);
            drop(node);
            deactivate_subtree(&world, carrier.get_id(), end);
        }
        let progress = carrier
            .animate_from(Style::progress_property(), 0.0, 1.0)
            .duration(self.duration)
            .easing(Easing::Linear)
            .task();
        let fade_duration = self.duration * (1.0 - PARTICLE_FADE_START);
        let target_fade = Task::Chain(vec![
            Task::Wait(self.duration - fade_duration),
            to.animate_from(Draw::opacity_property(), 0.0, target_opacity)
                .duration(fade_duration)
                .easing(Easing::Linear)
                .task(),
        ]);
        let mut tasks = vec![progress, target_fade];
        if self.fade_from {
            tasks.push(
                from.animate_from(Draw::opacity_property(), source_opacity, 0.0)
                    .duration(self.duration * (1.0 - PARTICLE_FADE_START))
                    .easing(Easing::Linear)
                    .task(),
            );
        }
        animator.play(Task::All(tasks));
    }
}

#[derive(Clone, Copy)]
struct MorphOpacity(f32);

fn stored_opacity(world: &crate::core::SceneWorld, entity: hecs::Entity) -> f32 {
    if let Ok(opacity) = world.borrow().get::<&MorphOpacity>(entity) {
        return opacity.0;
    }

    let opacity = world.borrow().get::<&Draw>(entity).unwrap().opacity;
    world
        .borrow_mut()
        .insert_one(entity, MorphOpacity(opacity))
        .unwrap();
    opacity
}

fn matrix(transform: GlobalTransform) -> skia_safe::Matrix {
    let (sin, cos) = transform.rotation.sin_cos();
    skia_safe::Matrix::new_all(
        cos * transform.scale.x,
        -sin * transform.scale.y,
        transform.position.x,
        sin * transform.scale.x,
        cos * transform.scale.y,
        transform.position.y,
        0.0,
        0.0,
        1.0,
    )
}

fn union(a: skia_safe::Rect, b: skia_safe::Rect) -> skia_safe::Rect {
    skia_safe::Rect::new(
        a.left.min(b.left),
        a.top.min(b.top),
        a.right.max(b.right),
        a.bottom.max(b.bottom),
    )
}

// Captures scheduled appearances without changing node activation or animation data.
fn record(
    world: &hecs::World,
    entity: hecs::Entity,
    root: hecs::Entity,
    root_opacity: f32,
    parent: GlobalTransform,
    canvas: &skia_safe::Canvas,
    basis: &skia_safe::Matrix,
    time: f32,
) -> Option<skia_safe::Rect> {
    let local = local_transform(world, entity);
    let global = parent.append(local);
    let relative = skia_safe::Matrix::concat(basis, &matrix(global));
    let draw = world.get::<&Draw>(entity).unwrap();
    let size = (draw.get_box)(world, entity);
    let padding = world
        .get::<&Style>(entity)
        .map(|style| stroke_width_for_scale(style.stroke_width.max(0.0), local.scale))
        .unwrap_or(0.0)
        + 2.0;
    let mut bounds = (size.x > 0.0 && size.y > 0.0).then(|| {
        relative
            .map_rect(skia_safe::Rect::from_xywh(
                -size.x * 0.5 - padding,
                -size.y * 0.5 - padding,
                size.x + padding * 2.0,
                size.y + padding * 2.0,
            ))
            .0
    });
    let draw_opacity = if entity == root {
        root_opacity
    } else {
        draw.opacity
    };
    let layer = canvas.save_layer_alpha_f(None, draw_opacity.clamp(0.0, 1.0));
    let saved = canvas.save();
    canvas.concat(&relative);
    (draw.on_draw)(world, entity, canvas, 1.0);
    canvas.restore_to_count(saved);
    for child in children(world, entity) {
        let node = world.get::<&Node>(child).unwrap();
        if node.lifetime[1] <= time || (node.lifetime[0].is_finite() && node.lifetime[0] > time) {
            continue;
        }
        if let Some(child_bounds) = record(
            world,
            child,
            root,
            root_opacity,
            global,
            canvas,
            basis,
            time,
        ) {
            bounds = Some(
                bounds
                    .map(|bounds| union(bounds, child_bounds))
                    .unwrap_or(child_bounds),
            );
        }
    }
    canvas.restore_to_count(layer);
    bounds
}

fn capture(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: hecs::Entity,
    count: usize,
    time: f32,
    opacity: f32,
) -> Silhouette {
    let parent = global_transform(world, parent);
    let basis = matrix(parent)
        .invert()
        .expect("Morph parent must have an invertible transform.");
    let mut recorder = skia_safe::PictureRecorder::new();
    let canvas = recorder.begin_recording(skia_safe::Rect::from_xywh(-1e9, -1e9, 2e9, 2e9), false);
    let bounds = record(world, entity, entity, opacity, parent, canvas, &basis, time)
        .expect("Morph object must have drawable bounds.");
    let picture = recorder.finish_recording_as_picture(None).unwrap();
    let silhouette = Silhouette::capture(bounds, count, |canvas| {
        canvas.draw_picture(&picture, None, None);
    });
    assert!(
        !silhouette.is_empty(),
        "Morph object must have a visible silhouette."
    );
    silhouette
}

fn draw_transform(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) {
    let data = world.get::<&ParticleTransform>(entity).unwrap();
    let progress = world
        .get::<&Style>(entity)
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
        objects::{Circle, Group, Text},
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
            let source = Rect::builder()
                .size(vec2(20.0, 20.0))
                .position(vec2(-30.0, 0.0))
                .fill(Color::RED)
                .build(scene);
            let target = Circle::builder()
                .radius(10.0)
                .position(vec2(30.0, 0.0))
                .fill(Color::BLUE)
                .build(scene);
            scene.get_root().add(&source);
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
        let world = scene.get_world();
        let mut nodes = world.query::<&Node>();
        assert_eq!(nodes.iter().filter(|node| node.is_activated).count(), 2);
    }

    #[test]
    fn morph_captures_text_and_groups_and_allows_chaining() {
        struct Groups;
        impl SceneBuilder for Groups {
            fn build(&mut self, scene: &mut Scene) {
                let parent = Group::builder()
                    .position(vec2(12.0, 3.0))
                    .opacity(0.5)
                    .build(scene);
                let source = Group::builder().build(scene);
                let child = Rect::builder()
                    .size(vec2(12.0, 12.0))
                    .fill(Color::RED)
                    .build(scene);
                source.add(&child);
                parent.add(&source);
                scene.get_root().add(&parent);
                let target = Text::builder().text("A".to_owned()).size(20.0).build(scene);
                morph().duration(1.0).play(&source, &target);
                let next = Circle::builder().radius(8.0).build(scene);
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
                let circle = Circle::builder().radius(10.0).fill(Color::RED).build(scene);
                let rect = Rect::builder()
                    .size(vec2(20.0, 20.0))
                    .fill(Color::BLUE)
                    .build(scene);
                scene.get_root().add(&circle);

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
                let source = Rect::builder()
                    .size(vec2(20.0, 20.0))
                    .position(vec2(-30.0, 0.0))
                    .fill(Color::RED)
                    .build(scene);
                let target = Circle::builder()
                    .radius(10.0)
                    .position(vec2(30.0, 0.0))
                    .fill(Color::BLUE)
                    .build(scene);
                scene.get_root().add(&source);

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
                let source = Rect::builder()
                    .size(vec2(30.0, 30.0))
                    .fill(Color::RED)
                    .build(scene);
                let overlay = Rect::builder()
                    .size(vec2(10.0, 10.0))
                    .fill(Color::GREEN)
                    .build(scene);
                let target = Circle::builder()
                    .radius(15.0)
                    .fill(Color::BLUE)
                    .build(scene);
                scene.get_root().add(&source);
                scene.get_root().add(&overlay);
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
                let parent = Group::builder()
                    .scale(vec2(2.0, 0.7))
                    .rotation(0.4)
                    .build(scene);
                let source = Rect::builder()
                    .size(vec2(30.0, 12.0))
                    .rotation(0.6)
                    .fill(Color::RED)
                    .build(scene);
                let target = Group::builder().build(scene);
                let child = Circle::builder().radius(10.0).build(scene);
                target.add(&child);
                parent.add(&source);
                scene.get_root().add(&parent);
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
        let source = Rect::builder().build(&mut scene);
        let target = Circle::builder().build(&mut other);
        scene.get_root().add(&source);
        morph().play(&source, &target);
    }

    #[test]
    fn morph_drawing_does_not_change_evaluated_state() {
        let mut scene = Scene::new();
        scene.build(&mut MorphScene);
        let first = pixels(&scene, 2.0);
        let before = {
            let world = scene.get_world();
            world
                .query::<(&Node, &Style)>()
                .iter()
                .map(|(node, style)| (node.is_activated, style.progress))
                .collect::<Vec<_>>()
        };
        let mut surface = skia_safe::surfaces::raster_n32_premul((160, 80)).unwrap();
        scene.draw(surface.canvas());
        let after = {
            let world = scene.get_world();
            world
                .query::<(&Node, &Style)>()
                .iter()
                .map(|(node, style)| (node.is_activated, style.progress))
                .collect::<Vec<_>>()
        };
        assert_eq!(before, after);
        assert_eq!(first, pixels(&scene, 2.0));
    }
}
