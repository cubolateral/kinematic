use std::any::TypeId;

use crate::core::{
    TrackInfo, TrackValue,
    components::{Animation, Inspection, Morph},
};

pub(crate) struct AppearanceEdit {
    pub entity: hecs::Entity,
    pub component: TypeId,
    pub track: &'static TrackInfo,
    pub before: TrackValue,
    pub value: TrackValue,
}

impl AppearanceEdit {
    pub(super) fn apply(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        let current = (self.track.get)(world, entity);
        // Preserve distinct text endpoints when editing a content transition.
        if matches!(self.value, TrackValue::String(_)) && current != self.before {
            return false;
        }
        if current == self.value {
            return false;
        }
        (self.track.set)(world, entity, self.value.clone());
        true
    }
}

struct AppearanceValue {
    entity: hecs::Entity,
    component: TypeId,
    track: &'static TrackInfo,
    value: TrackValue,
}

pub(crate) struct AppearanceSnapshot(Vec<AppearanceValue>);

impl AppearanceSnapshot {
    pub(crate) fn capture(world: &hecs::World, entity: hecs::Entity) -> Self {
        let mut values = Vec::new();
        Self::collect(world, entity, &mut values);
        Self(values)
    }

    fn collect(world: &hecs::World, entity: hecs::Entity, values: &mut Vec<AppearanceValue>) {
        if let Ok(inspection) = world.get::<&Inspection>(entity) {
            let mut components = (inspection.get)(world, entity).to_vec();
            if world.get::<&Morph>(entity).is_ok() {
                components.push(Morph::INFO);
            }
            if world
                .get::<&super::string_morph::ContentMorph>(entity)
                .is_ok()
            {
                components.push(super::string_morph::ContentMorph::INFO);
            }
            for component in components {
                for track in (component.get)() {
                    values.push(AppearanceValue {
                        entity,
                        component: (component.type_id)(),
                        track,
                        value: (track.get)(world, entity),
                    });
                }
            }
        }
        for child in crate::core::objects::child_iter(world, entity) {
            Self::collect(world, child, values);
        }
    }

    pub(crate) fn edit(&mut self, edits: &[AppearanceEdit]) -> bool {
        let mut changed = false;
        for edit in edits {
            for saved in &mut self.0 {
                if saved.entity == edit.entity
                    && saved.component == edit.component
                    && saved.track.id == edit.track.id
                    && saved.value != edit.value
                    && (!matches!(edit.value, TrackValue::String(_)) || saved.value == edit.before)
                {
                    saved.value = edit.value.clone();
                    changed = true;
                }
            }
        }
        changed
    }

    pub(crate) fn with_values<R>(&self, world: &hecs::World, f: impl FnOnce() -> R) -> R {
        let previous: Vec<_> = self
            .0
            .iter()
            .map(|saved| {
                let value = (saved.track.get)(world, saved.entity);
                (saved.track.set)(world, saved.entity, saved.value.clone());
                value
            })
            .collect();
        struct Restore<'a> {
            snapshot: &'a AppearanceSnapshot,
            world: &'a hecs::World,
            previous: Vec<TrackValue>,
        }
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                for (saved, value) in self.snapshot.0.iter().zip(self.previous.drain(..)) {
                    (saved.track.set)(self.world, saved.entity, value);
                }
            }
        }
        let _restore = Restore {
            snapshot: self,
            world,
            previous,
        };
        f()
    }
}

pub(crate) fn refresh_appearance(world: &hecs::World, edits: &[AppearanceEdit]) {
    if edits.is_empty() {
        return;
    }
    // Keep text keyframes in sync with the silhouettes used by content effects.
    for edit in edits {
        if edit.track.name == "text"
            && matches!(edit.value, TrackValue::String(_))
            && let Ok(mut animation) = world.get::<&mut Animation>(edit.entity)
        {
            animation.replace_values(edit.component, edit.track, &edit.before, &edit.value);
        }
    }
    super::refresh_write_plans(world, edits);
    super::string_morph::refresh_appearance(world, edits);
    crate::core::effects::refresh_morphs(world, edits);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::particle::CAPTURE_COUNT;
    use crate::prelude::*;

    struct Build<F: FnMut(&mut Scene)>(F);

    impl<F: FnMut(&mut Scene)> SceneBuilder for Build<F> {
        fn build(&mut self, scene: &mut Scene) {
            (self.0)(scene);
        }
    }

    fn edit(scene: &Scene, entity: hecs::Entity, name: &str, value: TrackValue) {
        let world = scene.get_world();
        let inspection = world.get::<&Inspection>(entity).unwrap();
        let (component, track) = (inspection.get)(&world, entity)
            .iter()
            .find_map(|component| {
                (component.get)()
                    .iter()
                    .find(|track| track.name == name)
                    .map(|track| ((component.type_id)(), track))
            })
            .unwrap();
        let before = (track.get)(&world, entity);
        (track.set)(&world, entity, value.clone());
        refresh_appearance(
            &world,
            &[AppearanceEdit {
                entity,
                component,
                track,
                before,
                value,
            }],
        );
    }

    fn pixels(scene: &Scene) -> Vec<skia_safe::Color> {
        let mut surface = skia_safe::surfaces::raster_n32_premul((240, 160)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((120.0, 80.0));
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();
        (0..160)
            .flat_map(|y| (0..240).map(move |x| (x, y)))
            .map(|point| pixels.get_color(point))
            .collect()
    }

    #[test]
    fn content_edits_refresh_particles_and_preserve_distinct_text_endpoints() {
        for use_latex in [false, true] {
            let mut scene = Scene::new();
            let mut entity = None;
            scene.build(&mut Build(|scene: &mut Scene| {
                entity = Some(if use_latex {
                    let object = latex().text("A").size(32.0).build(scene);
                    scene.get_world_2d().add(&object);
                    object.morph("B").easing(Easing::Linear).play();
                    object.morph("C").easing(Easing::Linear).play();
                    object.get_id()
                } else {
                    let object = text().text("A").size(32.0).build(scene);
                    scene.get_world_2d().add(&object);
                    object.morph("B").easing(Easing::Linear).play();
                    object.morph("C").easing(Easing::Linear).play();
                    object.get_id()
                });
            }));
            let entity = entity.unwrap();
            scene.update(0.5);
            for (name, value) in [
                ("size", TrackValue::F32(48.0)),
                ("fill", TrackValue::Color(Color::RED)),
                ("text", TrackValue::String("XY".into())),
            ] {
                let before = pixels(&scene);
                let captures_before = CAPTURE_COUNT.get();
                edit(&scene, entity, name, value);
                if name == "text" {
                    assert_eq!(
                        CAPTURE_COUNT.get(),
                        captures_before + 2,
                        "Unchanged transitions must keep their cached silhouettes."
                    );
                }
                let after = pixels(&scene);
                assert!(
                    before != after,
                    "Editing {name} must update the transition."
                );
                let captures = CAPTURE_COUNT.get();
                assert!(after == pixels(&scene));
                assert_eq!(captures, CAPTURE_COUNT.get());
            }
            let captures = CAPTURE_COUNT.get();
            edit(
                &scene,
                entity,
                "position",
                TrackValue::Vector2(vec2(10.0, 0.0)),
            );
            assert_eq!(
                CAPTURE_COUNT.get(),
                captures,
                "Translation must reuse local silhouettes."
            );
            let world = scene.get_world();
            let morph = world
                .get::<&super::super::string_morph::ContentMorph>(entity)
                .unwrap();
            assert_eq!(morph.transitions[0].from_text, "XY");
            assert_eq!(morph.transitions[0].to_text, "B");
            assert_eq!(morph.transitions[1].from_text, "B");
            assert_eq!(morph.transitions[1].to_text, "C");
            drop(morph);
            drop(world);
            scene.update(0.0);
            scene.update(0.5);
            let before = pixels(&scene);
            edit(&scene, entity, "text", TrackValue::String("Z".into()));
            assert!(
                before != pixels(&scene),
                "Editing after seeking must refresh the same endpoint."
            );
            let world = scene.get_world();
            let morph = world
                .get::<&super::super::string_morph::ContentMorph>(entity)
                .unwrap();
            assert_eq!(morph.transitions[0].from_text, "Z");
        }
    }

    #[test]
    fn object_morph_refreshes_only_the_edited_subtree_and_restores_live_state() {
        let mut scene = Scene::new();
        let mut ids = None;
        scene.build(&mut Build(|scene: &mut Scene| {
            let group = group_2d().build(scene);
            let source = text().text("A").size(32.0).build(scene);
            group.add(&source);
            scene.get_world_2d().add(&group);
            let target = circle().radius(20.0).build(scene);
            morph()
                .duration(2.0)
                .easing(Easing::Linear)
                .play(&group, &target);
            ids = Some((group.get_id(), source.get_id()));
        }));
        let (group, source) = ids.unwrap();
        scene.update(1.0);
        for (name, value) in [
            ("fill", TrackValue::Color(Color::RED)),
            ("size", TrackValue::F32(48.0)),
            ("text", TrackValue::String("XY".into())),
        ] {
            let before = pixels(&scene);
            let captures = CAPTURE_COUNT.get();
            edit(&scene, source, name, value);
            assert_eq!(
                CAPTURE_COUNT.get(),
                captures + 1,
                "Only the changed endpoint needs capture."
            );
            let after = pixels(&scene);
            assert!(before != after, "Editing {name} must update particles.");
            let opacity = scene
                .get_world()
                .get::<&crate::core::components::Draw2D>(group)
                .unwrap()
                .opacity;
            assert_eq!(opacity, 0.0, "Capture must restore the evaluated fade.");
            scene.update(0.5);
            scene.update(1.0);
            assert!(after == pixels(&scene));
            assert_eq!(
                CAPTURE_COUNT.get(),
                captures + 1,
                "Seeking must reuse prepared data."
            );
        }
        edit(&scene, source, "text", TrackValue::String(String::new()));
        let _ = pixels(&scene);
    }

    #[test]
    fn text_edits_remain_consistent_across_content_effects() {
        fn sequence(
            use_latex: bool,
            from: &str,
            to: &str,
            use_write: bool,
        ) -> (Scene, hecs::Entity) {
            let mut scene = Scene::new();
            let mut entity = None;
            scene.build(&mut Build(|scene: &mut Scene| {
                if use_latex {
                    let object = latex().text(from).size(32.0).build(scene);
                    entity = Some(object.get_id());
                    scene.get_world_2d().add(&object);
                    creation().duration(1.0).play(&object);
                    object.morph(to).easing(Easing::Linear).play();
                    scene.wait(1.0);
                    uncreation().duration(1.0).play(&object);
                } else {
                    let object = text().text(from).size(32.0).build(scene);
                    entity = Some(object.get_id());
                    scene.get_world_2d().add(&object);
                    if use_write {
                        write().duration(1.0).play(&object);
                    } else {
                        creation().duration(1.0).play(&object);
                    }
                    object.morph(to).easing(Easing::Linear).play();
                    scene.wait(1.0);
                    if use_write {
                        unwrite().duration(1.0).play(&object);
                    } else {
                        uncreation().duration(1.0).play(&object);
                    }
                }
            }));
            (scene, entity.unwrap())
        }

        for (use_latex, use_write) in [(false, false), (true, false), (false, true)] {
            let (scene, entity) = sequence(use_latex, "A", "B", use_write);
            scene.update(0.5);
            edit(&scene, entity, "text", TrackValue::String("XY".into()));
            let (expected, _) = sequence(use_latex, "XY", "B", use_write);
            let captures = CAPTURE_COUNT.get();
            for time in [0.5, 0.999, 1.0, 1.5, 1.999, 2.0, 2.5, 3.5, 0.5] {
                scene.update(time);
                expected.update(time);
                assert!(
                    pixels(&scene) == pixels(&expected),
                    "Edited source changed at {time}."
                );
            }
            assert_eq!(
                CAPTURE_COUNT.get(),
                captures,
                "Playback must reuse morph silhouettes."
            );

            scene.update(3.5);
            edit(&scene, entity, "text", TrackValue::String("Z".into()));
            let (expected, _) = sequence(use_latex, "XY", "Z", use_write);
            let captures = CAPTURE_COUNT.get();
            for time in [3.5, 3.0, 2.0, 1.999, 1.5, 1.0, 0.5, 3.999] {
                scene.update(time);
                expected.update(time);
                assert!(
                    pixels(&scene) == pixels(&expected),
                    "Edited destination changed at {time}."
                );
            }
            assert_eq!(
                CAPTURE_COUNT.get(),
                captures,
                "Seeking must reuse morph silhouettes."
            );
        }
    }

    #[test]
    fn object_morph_captures_complete_text_during_write_and_unwrite_edits() {
        fn sequence(from: &str, to: &str) -> (Scene, hecs::Entity, hecs::Entity) {
            let mut scene = Scene::new();
            let mut ids = None;
            scene.build(&mut Build(|scene: &mut Scene| {
                let source = text().text(from).size(32.0).build(scene);
                let target = text().text(to).size(32.0).build(scene);
                scene.get_world_2d().add(&source);
                write().duration(1.0).play(&source);
                morph()
                    .duration(2.0)
                    .easing(Easing::Linear)
                    .play(&source, &target);
                // This adds WriteState after the morph has captured its destination.
                unwrite().duration(1.0).play(&target);
                ids = Some((source.get_id(), target.get_id()));
            }));
            let (source, target) = ids.unwrap();
            (scene, source, target)
        }

        let (scene, source, target) = sequence("A", "B");
        scene.update(0.5);
        edit(&scene, source, "text", TrackValue::String("XY".into()));
        let (expected, _, _) = sequence("XY", "B");
        for time in [0.5, 1.0, 1.5, 2.0, 2.999, 3.5, 0.5] {
            scene.update(time);
            expected.update(time);
            assert!(
                pixels(&scene) == pixels(&expected),
                "Source capture contains write particles at {time}."
            );
        }

        scene.update(3.5);
        edit(&scene, target, "text", TrackValue::String("Z".into()));
        let (expected, _, _) = sequence("XY", "Z");
        let captures = CAPTURE_COUNT.get();
        for time in [3.5, 2.0, 1.5, 3.0, 3.999, 0.5] {
            scene.update(time);
            expected.update(time);
            assert!(
                pixels(&scene) == pixels(&expected),
                "Destination capture contains unwrite particles at {time}."
            );
        }
        assert_eq!(
            CAPTURE_COUNT.get(),
            captures,
            "Frames must reuse morph silhouettes."
        );
    }

    #[test]
    fn particle_effects_follow_live_appearance_edits() {
        for (reverse, use_write) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut scene = Scene::new();
            let mut entity = None;
            scene.build(&mut Build(|scene: &mut Scene| {
                let object = text().text("A").size(32.0).build(scene);
                entity = Some(object.get_id());
                scene.get_world_2d().add(&object);
                if use_write && reverse {
                    unwrite().duration(2.0).play(&object);
                } else if use_write {
                    write().duration(2.0).play(&object);
                } else if reverse {
                    uncreation().duration(2.0).play(&object);
                } else {
                    creation().duration(2.0).play(&object);
                }
            }));
            let entity = entity.unwrap();
            scene.update(1.0);
            for (name, value) in [
                ("text", TrackValue::String("XY".into())),
                ("size", TrackValue::F32(48.0)),
                ("fill", TrackValue::Color(Color::RED)),
            ] {
                let before = pixels(&scene);
                edit(&scene, entity, name, value);
                let after = pixels(&scene);
                assert!(before != after, "Editing {name} must update particles.");
                assert!(after == pixels(&scene));
            }
        }
    }
}
