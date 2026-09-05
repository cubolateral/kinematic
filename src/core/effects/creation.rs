use crate::core::effects::Effect;
use crate::core::{
    Easing,
    components::{ParticleStyle, Style},
    objects::{ObjectHandler, ObjectTrackable},
};

fn play_creation<T>(handler: &T, duration: f32, easing: Easing, progress: (f32, f32))
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<ParticleStyle> + ObjectTrackable<Style>,
{
    handler
        .animate_from(Style::progress_property(), progress.0, progress.1)
        .animate_from(ParticleStyle::particles_enabled_property(), true, false)
        .duration(duration)
        .easing(easing)
        .play();
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

    /// Sets the easing curve used by the shared style progress.
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
    T::Object: ObjectTrackable<ParticleStyle> + ObjectTrackable<Style>,
{
    fn play(self, handler: &T) {
        play_creation(handler, self.duration, self.easing, (0.0, 1.0));
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

    /// Sets the easing curve used by the shared style progress.
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
    T::Object: ObjectTrackable<ParticleStyle> + ObjectTrackable<Style>,
{
    fn play(self, handler: &T) {
        play_creation(handler, self.duration, self.easing, (1.0, 0.0));
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
