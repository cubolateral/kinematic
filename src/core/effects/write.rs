use crate::core::effects::Effect;
use crate::core::{
    Easing,
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
    handler: &T,
    duration: f32,
    by: WriteBy,
    scale: f32,
    outline_width: f32,
    progress: (f32, f32),
    reverse: bool,
) where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    let by_word = matches!(by, WriteBy::Word);

    handler
        .animate_from(TextShape::write_progress_property(), progress.0, progress.1)
        .animate_from(TextShape::write_scale_property(), scale, scale)
        .animate_from(TextShape::write_by_word_property(), by_word, by_word)
        .animate_from(TextShape::write_reverse_property(), reverse, reverse)
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
    reverse: bool,
}

impl Write {
    /// Creates a write effect that sequences letters.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            by: WriteBy::Letter,
            scale: 2.5,
            outline_width: 1.0,
            reverse: false,
        }
    }

    /// Sets the unit used to sequence the effect.
    pub fn by(mut self, by: WriteBy) -> Self {
        self.by = by;
        self
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

    /// Sequences the writing units from right to left.
    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }
}

impl<T> Effect<T> for Write
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    fn play(self, handler: &T) {
        play_text_effect(
            handler,
            self.duration,
            self.by,
            self.scale,
            self.outline_width,
            (0.0, 1.0),
            self.reverse,
        );
    }
}

/// Removes text units one character or word at a time.
pub struct Unwrite {
    duration: f32,
    by: WriteBy,
    scale: f32,
    outline_width: f32,
    reverse: bool,
}

impl Unwrite {
    /// Creates an unwrite effect that sequences letters.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            by: WriteBy::Letter,
            scale: 0.0,
            outline_width: 1.0,
            reverse: false,
        }
    }

    /// Sets the unit used to sequence the effect.
    pub fn by(mut self, by: WriteBy) -> Self {
        self.by = by;
        self
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

    /// Reverses the default right-to-left removal sequence.
    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }
}

impl<T> Effect<T> for Unwrite
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    fn play(self, handler: &T) {
        play_text_effect(
            handler,
            self.duration,
            self.by,
            self.scale,
            self.outline_width,
            (1.0, 0.0),
            self.reverse,
        );
    }
}

/// Plays a default write effect on a text object handler.
pub fn write<T>(handler: &T)
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    Write::new().play(handler);
}

/// Plays a default unwrite effect on a text object handler.
pub fn unwrite<T>(handler: &T)
where
    T: ObjectHandler,
    T::Object: ObjectTrackable<TextShape>,
{
    Unwrite::new().play(handler);
}
