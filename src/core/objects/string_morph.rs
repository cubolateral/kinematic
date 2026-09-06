use kinematic_macros::Trackable;

use crate::core::{
    Easing, Tween,
    components::{Style, Transform},
    objects::{
        Object,
        particle::{ParticleTransform, Silhouette, morph_opacities},
    },
};

pub(super) struct ContentMorphTransition {
    particles: Option<ParticleTransform>,
    pub(super) from_text: String,
    pub(super) to_text: String,
}

impl ContentMorphTransition {
    pub(super) fn draw(
        &self,
        canvas: &skia_safe::Canvas,
        progress: f32,
        opacity: f32,
        draw: impl Fn(&str, f32),
    ) {
        let (source_opacity, target_opacity) = morph_opacities(progress);
        for (text, fade) in [
            (&self.from_text, source_opacity),
            (&self.to_text, target_opacity),
        ] {
            if fade > 0.0 {
                draw(text, opacity * fade);
            }
        }
        self.particles
            .as_ref()
            .expect("Morph must be prepared before drawing.")
            .draw(canvas, progress, opacity);
    }
}

#[derive(Default, Trackable)]
pub(super) struct ContentMorph {
    #[track]
    pub(super) progress: f32,
    #[track]
    pub(super) transition: u32,
    // Ends at the actual keyframe, even when easing rounds progress to one early.
    #[track]
    pub(super) active: bool,

    pub(super) transitions: Vec<ContentMorphTransition>,
}

pub(super) fn morph_string<T: Object, S: hecs::Component + Clone>(
    tween: Tween<T>,
    entity: hecs::Entity,
    from_text: String,
    text: String,
    capture: fn(&S, &Style, &Transform) -> Silhouette,
) -> Tween<T> {
    let (world, _) = tween.context();
    let (shape, style, transform) = {
        let world = world.borrow();
        (
            (*world.get::<&S>(entity).unwrap()).clone(),
            (*world.get::<&Style>(entity).unwrap()).clone(),
            (*world.get::<&Transform>(entity).unwrap()).clone(),
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
            particles: None,
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
            let capture_endpoint = |end| {
                capture(
                    &tween.endpoint(&shape, end),
                    &tween.endpoint(&style, end),
                    &tween.endpoint(&transform, end),
                )
            };
            let particles = ParticleTransform::new(
                capture_endpoint(false),
                capture_endpoint(true),
                Easing::Linear,
            );
            world
                .borrow()
                .get::<&mut ContentMorph>(entity)
                .unwrap()
                .transitions[transition_index]
                .particles = Some(particles);
        })
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
                let object = latex()
                    .text("x".to_owned())
                    .size(self.size)
                    .fill(self.style.fill)
                    .stroke(self.style.stroke)
                    .stroke_width(self.style.stroke_width)
                    .build(scene);
                scene.get_root().add(&object);
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
                let object = text()
                    .text("A\nBC".to_owned())
                    .size(self.size)
                    .fill(self.style.fill)
                    .stroke(self.style.stroke)
                    .stroke_width(self.style.stroke_width)
                    .build(scene);
                scene.get_root().add(&object);
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
