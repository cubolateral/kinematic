use crate::core::{Easing, effects::Effect, objects::Text2DHandler};

/// Writes text one character at a time through overlapping particle morphs.
pub struct Write {
    duration: f32,
    easing: Easing,
}

impl Write {
    /// Creates a two-and-a-half-second write effect.
    pub fn new() -> Self {
        Self {
            duration: 2.5,
            easing: Easing::default(),
        }
    }

    /// Sets the total effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        assert!(
            duration.is_finite() && duration > 0.0,
            "Write duration must be finite and positive."
        );
        self.duration = duration;
        self
    }

    /// Sets the easing curve used by particle travel.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

impl Default for Write {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect<Text2DHandler> for Write {
    fn play(self, handler: &Text2DHandler) {
        handler.play_write(self.duration, self.easing, false);
    }
}

/// Builds a default overlapping text write effect.
pub fn write() -> Write {
    Write::new()
}

/// Removes text one character at a time through overlapping particle uncreations.
pub struct Unwrite {
    duration: f32,
    easing: Easing,
}

impl Unwrite {
    /// Creates a two-and-a-half-second reverse write effect.
    pub fn new() -> Self {
        Self {
            duration: 2.5,
            easing: Easing::default(),
        }
    }

    /// Sets the total effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        assert!(
            duration.is_finite() && duration > 0.0,
            "Unwrite duration must be finite and positive."
        );
        self.duration = duration;
        self
    }

    /// Sets the easing curve used by particle travel.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

impl Default for Unwrite {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect<Text2DHandler> for Unwrite {
    fn play(self, handler: &Text2DHandler) {
        handler.play_write(self.duration, self.easing, true);
    }
}

/// Builds a default overlapping reverse text write effect.
pub fn unwrite() -> Unwrite {
    Unwrite::new()
}
