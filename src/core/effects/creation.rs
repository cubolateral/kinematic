use crate::core::{
    Easing, Task,
    components::{Draw2D, Morph},
    effects::Effect,
    objects::{Morphable, ObjectHandler},
};

fn play_creation<T>(
    handler: &T,
    duration: f32,
    easing: Easing,
    progress: (f32, f32),
    hide_at_end: bool,
) where
    T: ObjectHandler,
    T::Object: Morphable,
{
    let anchor = handler.animate(
        Draw2D::opacity_property(),
        handler.get(Draw2D::opacity_property()),
    );
    let (world, animator) = anchor.context();
    let transition = Morph::progress_property()
        .handle(world.clone(), handler.get_id(), animator.clone())
        .animate_from::<T::Object>(progress.0, progress.1)
        .duration(duration)
        .easing(easing)
        .task();
    let particles = Morph::particles_enabled_property()
        .handle(world, handler.get_id(), animator.clone())
        .animate_from::<T::Object>(true, false)
        .duration(duration)
        .easing(easing)
        .task();
    let mut tasks = vec![transition, particles];
    if hide_at_end {
        tasks.push(Task::Chain(vec![
            Task::Wait(duration),
            handler
                .animate_from(
                    Draw2D::opacity_property(),
                    handler.get(Draw2D::opacity_property()),
                    0.0,
                )
                .duration(0.0)
                .task(),
        ]));
    }
    animator.play(Task::All(tasks));
}

/// Forms an object from its signature particle cloud.
pub struct Creation {
    duration: f32,
    easing: Easing,
}

impl Creation {
    /// Creates a one-second organized particle creation effect.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
        }
    }

    /// Sets the effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing curve used by the transition.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

impl Default for Creation {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Effect<T> for Creation
where
    T: ObjectHandler,
    T::Object: Morphable,
{
    fn play(self, handler: &T) {
        play_creation(handler, self.duration, self.easing, (0.0, 1.0), false);
    }
}

/// Reverses the signature creation effect.
pub struct Uncreation {
    duration: f32,
    easing: Easing,
}

impl Uncreation {
    /// Creates a one-second reverse creation effect.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
        }
    }

    /// Sets the effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing curve used by the transition.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

impl Default for Uncreation {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Effect<T> for Uncreation
where
    T: ObjectHandler,
    T::Object: Morphable,
{
    fn play(self, handler: &T) {
        play_creation(handler, self.duration, self.easing, (1.0, 0.0), true);
    }
}

/// Builds the default signature creation effect.
pub fn creation() -> Creation {
    Creation::new().duration(2.5)
}

/// Builds the default reverse creation effect.
pub fn uncreation() -> Uncreation {
    Uncreation::new().duration(2.5)
}
