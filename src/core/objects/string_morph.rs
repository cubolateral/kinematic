use kinematic_macros::Trackable;

use crate::core::{
    Tween,
    components::{Style, Transform2D},
    objects::{
        Object,
        appearance::AppearanceEdit,
        particle::{ParticleTransform, morph_opacities},
    },
    types::{Color, Vector2},
};

type RefreshContentMorph = dyn Fn(&[AppearanceEdit]) -> Option<PreparedContentMorph> + Send + Sync;

pub(super) struct ContentMorphTransition {
    prepared: Option<PreparedContentMorph>,
    refresh: Option<Box<RefreshContentMorph>>,
    pub(super) from_text: String,
    pub(super) to_text: String,
}

pub(super) enum PreparedContentMorph {
    Text(TextMorphPlan),
    Paths(PathMorphPlan),
    Fade,
}

#[derive(Clone)]
pub(super) struct MorphPath {
    pub(super) path: std::sync::Arc<std::sync::Mutex<skia_safe::Path>>,
    pub(super) color: Option<Color>,
}

pub(super) struct MovingMorphPath {
    pub(super) from: MorphPath,
    pub(super) to: MorphPath,
}

pub(super) struct PathMorphPlan {
    pub(super) stable: Vec<MovingMorphPath>,
    pub(super) source: Vec<MorphPath>,
    pub(super) target: Vec<MorphPath>,
    pub(super) particles: ParticleTransform,
}

pub(super) struct GlyphLayer {
    pub(super) glyphs: Vec<skia_safe::GlyphId>,
    pub(super) positions: Vec<skia_safe::Point>,
}

pub(super) struct MovingGlyphLayer {
    pub(super) glyphs: Vec<skia_safe::GlyphId>,
    pub(super) from: Vec<skia_safe::Point>,
    pub(super) to: Vec<skia_safe::Point>,
}

pub(super) struct TextMorphPlan {
    pub(super) stable: MovingGlyphLayer,
    pub(super) source: GlyphLayer,
    pub(super) target: GlyphLayer,
    pub(super) particles: ParticleTransform,
}

impl ContentMorphTransition {
    pub(super) fn draw(&self, progress: f32, opacity: f32, draw: impl Fn(&str, f32)) {
        let (source_opacity, target_opacity) =
            if matches!(self.prepared, Some(PreparedContentMorph::Fade)) {
                (1.0 - progress, progress)
            } else {
                morph_opacities(progress)
            };
        for (text, fade) in [
            (&self.from_text, source_opacity),
            (&self.to_text, target_opacity),
        ] {
            if fade > 0.0 {
                draw(text, opacity * fade);
            }
        }
    }

    pub(super) fn is_fade(&self) -> bool {
        matches!(self.prepared, Some(PreparedContentMorph::Fade))
    }

    pub(super) fn fade_layers(&self, progress: f32) -> Option<[(String, f32); 2]> {
        self.is_fade().then(|| {
            [
                (self.from_text.clone(), 1.0 - progress),
                (self.to_text.clone(), progress),
            ]
        })
    }

    pub(super) fn text_plan(&self) -> &TextMorphPlan {
        let PreparedContentMorph::Text(plan) = self
            .prepared
            .as_ref()
            .expect("Morph must be prepared before drawing.")
        else {
            panic!("String morph must use its silhouette renderer.");
        };

        plan
    }

    pub(super) fn path_plan(&self) -> &PathMorphPlan {
        let PreparedContentMorph::Paths(plan) = self
            .prepared
            .as_ref()
            .expect("Morph must be prepared before drawing.")
        else {
            panic!("Path morph must use its path renderer.");
        };

        plan
    }
}

pub(super) fn match_items<K: PartialEq>(
    from: &[K],
    from_origins: &[Vector2],
    to: &[K],
    to_origins: &[Vector2],
) -> Vec<(usize, usize)> {
    let columns = to.len() + 1;
    let mut lengths = vec![0usize; (from.len() + 1) * columns];

    for from_index in (0..from.len()).rev() {
        for to_index in (0..to.len()).rev() {
            let index = from_index * columns + to_index;
            lengths[index] = if from[from_index] == to[to_index] {
                1 + lengths[(from_index + 1) * columns + to_index + 1]
            } else {
                lengths[(from_index + 1) * columns + to_index]
                    .max(lengths[from_index * columns + to_index + 1])
            };
        }
    }

    let mut matches = Vec::new();
    let (mut from_index, mut to_index) = (0, 0);
    while from_index < from.len() && to_index < to.len() {
        if from[from_index] == to[to_index] {
            matches.push((from_index, to_index));
            from_index += 1;
            to_index += 1;
        } else if lengths[(from_index + 1) * columns + to_index]
            >= lengths[from_index * columns + to_index + 1]
        {
            from_index += 1;
        } else {
            to_index += 1;
        }
    }

    let mut matched_from = vec![false; from.len()];
    let mut matched_to = vec![false; to.len()];
    for &(from_index, to_index) in &matches {
        matched_from[from_index] = true;
        matched_to[to_index] = true;
    }

    for source in 0..from.len() {
        if matched_from[source] {
            continue;
        }
        let target = (0..to.len())
            .filter(|&target| !matched_to[target] && from[source] == to[target])
            .min_by(|&left, &right| {
                from_origins[source]
                    .distance_squared(to_origins[left])
                    .total_cmp(&from_origins[source].distance_squared(to_origins[right]))
                    .then_with(|| left.cmp(&right))
            });
        if let Some(target) = target {
            matched_from[source] = true;
            matched_to[target] = true;
            matches.push((source, target));
        }
    }

    matches.sort_unstable();
    matches
}

#[derive(Default, Trackable)]
pub(super) struct ContentMorph {
    #[track(min = 0.0, max = 1.0)]
    pub(super) progress: f32,
    #[track]
    pub(super) transition: u32,
    // Ends at the actual keyframe, even when easing rounds progress to one early.
    #[track]
    pub(super) active: bool,

    pub(super) transitions: Vec<ContentMorphTransition>,
}

pub(super) fn morph_text<T: Object, S: hecs::Component + Clone>(
    tween: Tween<T>,
    entity: hecs::Entity,
    from_text: String,
    text: String,
    prepare: fn(&S, &Style, &Transform2D, &S, &Style, &Transform2D) -> TextMorphPlan,
) -> Tween<T> {
    morph_string_with(
        tween,
        entity,
        from_text,
        text,
        move |from_shape, from_style, from_transform, to_shape, to_style, to_transform| {
            PreparedContentMorph::Text(prepare(
                &from_shape,
                &from_style,
                &from_transform,
                &to_shape,
                &to_style,
                &to_transform,
            ))
        },
    )
}

pub(super) fn fade_string<T: Object>(
    tween: Tween<T>,
    entity: hecs::Entity,
    from_text: String,
    text: String,
) -> Tween<T> {
    let (world, _) = tween.context();
    let transition_index = {
        let mut world = world.borrow_mut();
        if world.get::<&ContentMorph>(entity).is_err() {
            world.insert_one(entity, ContentMorph::default()).unwrap();
        }
        let mut morph = world.get::<&mut ContentMorph>(entity).unwrap();
        let index = morph.transitions.len();
        morph.transitions.push(ContentMorphTransition {
            prepared: Some(PreparedContentMorph::Fade),
            refresh: None,
            from_text,
            to_text: text,
        });
        index
    };
    tween
        .animate_from(
            ContentMorph::transition_property(),
            transition_index as u32,
            transition_index as u32,
        )
        .animate_from(ContentMorph::active_property(), true, false)
        .animate_from(ContentMorph::progress_property(), 0.0, 1.0)
}

pub(super) fn morph_string_with<T: Object, S: hecs::Component + Clone>(
    tween: Tween<T>,
    entity: hecs::Entity,
    from_text: String,
    text: String,
    prepare: impl Fn(S, Style, Transform2D, S, Style, Transform2D) -> PreparedContentMorph
    + Send
    + Sync
    + 'static,
) -> Tween<T> {
    let (world, _) = tween.context();
    let (shape, style, transform) = {
        let world = world.borrow();
        (
            (*world.get::<&S>(entity).unwrap()).clone(),
            (*world.get::<&Style>(entity).unwrap()).clone(),
            (*world.get::<&Transform2D>(entity).unwrap()).clone(),
        )
    };
    let transition_index = {
        let mut world = world.borrow_mut();
        let missing = world.get::<&ContentMorph>(entity).is_err();

        if missing {
            world.insert_one(entity, ContentMorph::default()).unwrap();
        }

        let mut morph = world.get::<&mut ContentMorph>(entity).unwrap();
        let transition_index = morph.transitions.len();
        morph.transitions.push(ContentMorphTransition {
            prepared: None,
            refresh: None,
            from_text,
            to_text: text,
        });
        transition_index
    };
    tween
        .animate_from(
            ContentMorph::transition_property(),
            transition_index as u32,
            transition_index as u32,
        )
        .animate_from(ContentMorph::active_property(), true, false)
        .animate_from(ContentMorph::progress_property(), 0.0, 1.0)
        .prepare(move |tween| {
            let mut endpoints = hecs::World::new();
            let from = endpoints.spawn((
                tween.endpoint(&shape, false),
                tween.endpoint(&style, false),
                tween.endpoint(&transform, false),
            ));
            let to = endpoints.spawn((
                tween.endpoint(&shape, true),
                tween.endpoint(&style, true),
                tween.endpoint(&transform, true),
            ));
            let build = move |endpoints: &hecs::World| {
                let read = |id| {
                    (
                        (*endpoints.get::<&S>(id).unwrap()).clone(),
                        (*endpoints.get::<&Style>(id).unwrap()).clone(),
                        (*endpoints.get::<&Transform2D>(id).unwrap()).clone(),
                    )
                };
                let (a, b, c) = read(from);
                let (d, e, f) = read(to);
                prepare(a, b, c, d, e, f)
            };
            let prepared = build(&endpoints);
            let refresh = move |edits: &[AppearanceEdit]| {
                let mut changed = false;
                for edit in edits.iter().filter(|edit| edit.entity == entity) {
                    if [
                        std::any::TypeId::of::<S>(),
                        std::any::TypeId::of::<Style>(),
                        std::any::TypeId::of::<Transform2D>(),
                    ]
                    .contains(&edit.component)
                    {
                        if edit.component == std::any::TypeId::of::<Transform2D>()
                            && edit.track.name != "scale"
                        {
                            continue;
                        }
                        changed |= edit.apply(&endpoints, from);
                        changed |= edit.apply(&endpoints, to);
                    }
                }
                changed.then(|| build(&endpoints))
            };
            let world = world.borrow();
            let mut morph = world.get::<&mut ContentMorph>(entity).unwrap();
            let transition = &mut morph.transitions[transition_index];
            transition.prepared = Some(prepared);
            transition.refresh = Some(Box::new(refresh));
        })
}

pub(super) fn refresh_appearance(world: &hecs::World, edits: &[AppearanceEdit]) {
    for (index, edit) in edits.iter().enumerate() {
        let entity = edit.entity;
        if edits[..index]
            .iter()
            .any(|previous| previous.entity == entity)
        {
            continue;
        }
        let Ok(mut morph) = world.get::<&mut ContentMorph>(entity) else {
            continue;
        };
        for transition in &mut morph.transitions {
            if let Some(prepared) = transition
                .refresh
                .as_ref()
                .and_then(|refresh| refresh(edits))
            {
                transition.prepared = Some(prepared);
                for edit in edits
                    .iter()
                    .filter(|edit| edit.entity == entity && edit.track.name == "text")
                {
                    if let (
                        crate::core::TrackValue::String(before),
                        crate::core::TrackValue::String(value),
                    ) = (&edit.before, &edit.value)
                    {
                        if transition.from_text == *before {
                            transition.from_text = value.clone();
                        }
                        if transition.to_text == *before {
                            transition.to_text = value.clone();
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::components::Style;
    use crate::core::objects::particle::CAPTURE_COUNT;
    use crate::prelude::*;

    struct StyledMorph {
        latex: bool,
        size: f32,
        style: Style,
        animate: bool,
    }

    impl SceneBuilder for StyledMorph {
        fn build(&mut self, scene: &mut Scene) {
            let task = if self.latex {
                let object = latex_2d()
                    .text("x")
                    .size(self.size)
                    .fill(self.style.fill)
                    .stroke(self.style.stroke)
                    .stroke_width(self.style.stroke_width)
                    .build(scene);
                scene.world_2d().add(&object);
                let tween = object.morph(r"\frac{1}{2}");
                if self.animate {
                    tween
                        .fill(Color::YELLOW)
                        .stroke(Color::BLUE)
                        .stroke_width(4.0)
                        .size(96.0)
                } else {
                    tween
                }
                .easing(Easing::Linear)
                .task()
            } else {
                let object = text_2d()
                    .text("A\nBC")
                    .size(self.size)
                    .fill(self.style.fill)
                    .stroke(self.style.stroke)
                    .stroke_width(self.style.stroke_width)
                    .build(scene);
                scene.world_2d().add(&object);
                let tween = object.morph("DE\nF");
                if self.animate {
                    tween
                        .fill(Color::YELLOW)
                        .stroke(Color::BLUE)
                        .stroke_width(4.0)
                        .size(96.0)
                } else {
                    tween
                }
                .easing(Easing::Linear)
                .task()
            };
            scene.play(task);
        }
    }

    fn pixels(scene: &Scene) -> Vec<skia_safe::Color> {
        let mut surface = skia_safe::surfaces::raster_n32_premul((480, 320)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((240.0, 160.0));
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();
        (0..320)
            .flat_map(|y| (0..480).map(move |x| (x, y)))
            .map(|point| pixels.get_color(point))
            .collect()
    }

    #[test]
    #[ignore = "Manual frame timing."]
    fn morph_frame_timing() {
        let mut scene = Scene::new();
        scene.build(&mut StyledMorph {
            latex: true,
            size: 48.0,
            style: Style::default(),
            animate: true,
        });
        let mut surface = skia_safe::surfaces::raster_n32_premul((480, 320)).unwrap();
        surface.canvas().translate((240.0, 160.0));
        let start = std::time::Instant::now();
        for frame in 1..=120 {
            scene.update(frame as f32 / 121.0);
            surface.canvas().clear(skia_safe::colors::TRANSPARENT);
            scene.draw(surface.canvas());
        }
        eprintln!("120 morph frames: {:?}.", start.elapsed());
    }

    #[test]
    fn chained_appearance_tracks_reach_particles_and_remain_seekable() {
        for latex in [false, true] {
            CAPTURE_COUNT.set(0);
            let mut scene = Scene::new();
            scene.build(&mut StyledMorph {
                latex,
                size: 48.0,
                style: Style::default(),
                animate: true,
            });
            assert_eq!(CAPTURE_COUNT.get(), 2);
            scene.update(0.5);
            let middle = pixels(&scene);
            assert_eq!(CAPTURE_COUNT.get(), 2, "Drawing must reuse silhouettes.");
            assert!(middle.iter().any(|color| color.a() > 0));
            let mut frozen = Scene::new();
            frozen.build(&mut StyledMorph {
                latex,
                size: 48.0,
                style: Style::default(),
                animate: false,
            });
            frozen.update(0.5);
            assert!(
                middle != pixels(&frozen),
                "Chained properties must reach particles."
            );
            assert!(middle.iter().any(|color| color.a() > 0 && color.b() < 200));
            let captures = CAPTURE_COUNT.get();
            scene.update(1.0);
            scene.update(0.5);
            assert_eq!(middle, pixels(&scene));
            assert_eq!(
                CAPTURE_COUNT.get(),
                captures,
                "Seeking must reuse silhouettes."
            );
        }
    }
}
