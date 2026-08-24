use crate::core::effects::Effect;
use crate::core::{
    Easing, Scene,
    objects::{ObjectHandler, ObjectTrackable, TextShape},
};

/// Unit used to sequence a text effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteBy {
    /// Applies the effect one character at a time.
    Letter,
    /// Applies the effect one word at a time.
    Word,
}

fn play_text_effect<T>(
    _s: &mut Scene,
    handler: &T,
    duration: f32,
    by: WriteBy,
    scale: f32,
    outline_width: f32,
    progress: (f32, f32),
) where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    let by_word = matches!(by, WriteBy::Word);

    handler
        .animate_from(TextShape::write_progress_property(), progress.0, progress.1)
        .animate_from(TextShape::write_scale_property(), scale, scale)
        .animate_from(TextShape::write_by_word_property(), by_word, by_word)
        .animate_from(
            TextShape::write_outline_width_property(),
            outline_width,
            outline_width,
        )
        .duration(duration)
        .easing(Easing::Linear)
        .play();
}

/// Writes text units with independent scale and outline transitions.
pub struct Write {
    duration: f32,
    by: WriteBy,
    scale: f32,
    outline_width: f32,
}

impl Write {
    /// Creates a write effect with a fixed sequencing unit.
    pub fn new(by: WriteBy) -> Self {
        Self {
            duration: 1.0,
            by,
            scale: 0.0,
            outline_width: 1.0,
        }
    }

    /// Sets the effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the initial scale applied to each unit.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the temporary inner outline width.
    pub fn outline_width(mut self, outline_width: f32) -> Self {
        self.outline_width = outline_width;
        self
    }
}

impl<T> Effect<T> for Write
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    fn play(self, s: &mut Scene, handler: &T) {
        play_text_effect(
            s,
            handler,
            self.duration,
            self.by,
            self.scale,
            self.outline_width,
            (0.0, 1.0),
        );
    }
}

/// Removes text units one character or word at a time.
pub struct Unwrite {
    duration: f32,
    by: WriteBy,
    scale: f32,
    outline_width: f32,
}

impl Unwrite {
    /// Creates an unwrite effect with a fixed sequencing unit.
    pub fn new(by: WriteBy) -> Self {
        Self {
            duration: 1.0,
            by,
            scale: 0.0,
            outline_width: 1.0,
        }
    }

    /// Sets the effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the final scale applied to each unit.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the temporary inner outline width.
    pub fn outline_width(mut self, outline_width: f32) -> Self {
        self.outline_width = outline_width;
        self
    }
}

impl<T> Effect<T> for Unwrite
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    fn play(self, s: &mut Scene, handler: &T) {
        play_text_effect(
            s,
            handler,
            self.duration,
            self.by,
            self.scale,
            self.outline_width,
            (1.0, 0.0),
        );
    }
}
