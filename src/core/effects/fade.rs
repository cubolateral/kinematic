use crate::core::effects::Effect;
use crate::core::{
    Easing,
    components::{Draw, Transform as TransformComponent},
    objects::ObjectHandler,
    types::Vector2,
};

/// Position or direction used by a fade effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FadeFrom {
    /// Starts or ends to the left.
    Left,
    /// Starts or ends to the right.
    Right,
    /// Starts or ends above the object.
    Up,
    /// Starts or ends below the object.
    Down,
    /// Starts or ends at a custom position.
    Position(Vector2),
}

impl FadeFrom {
    fn vector(self, offset: f32) -> Vector2 {
        match self {
            Self::Left => Vector2::new(-offset, 0.0),
            Self::Right => Vector2::new(offset, 0.0),
            Self::Up => Vector2::new(0.0, -offset),
            Self::Down => Vector2::new(0.0, offset),
            Self::Position(position) => position,
        }
    }
}

fn play_fade<T>(
    handler: &T,
    duration: f32,
    easing: Easing,
    opacity: (f32, f32),
    scale: (Vector2, Vector2),
    position: (Vector2, Vector2),
    rotation: (f32, f32),
) where
    T: ObjectHandler,
{
    handler
        .animate_from(Draw::opacity_property(), opacity.0, opacity.1)
        .animate_from(TransformComponent::scale_property(), scale.0, scale.1)
        .animate_from(
            TransformComponent::position_property(),
            position.0,
            position.1,
        )
        .animate_from(
            TransformComponent::rotation_property(),
            rotation.0,
            rotation.1,
        )
        .duration(duration)
        .easing(easing)
        .play();
}

/// Fades an object in from an optional scale and position.
pub struct FadeIn {
    duration: f32,
    easing: Easing,
    scale: f32,
    spin: f32,
    from: Option<FadeFrom>,
    offset: f32,
}

impl FadeIn {
    /// Creates a fade-in with a one-second duration and default easing.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
            scale: 1.0,
            spin: 0.0,
            from: None,
            offset: 100.0,
        }
    }

    /// Sets the effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing curve used by every animated property.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the starting scale factor relative to the object's current scale.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the starting rotation offset in radians.
    pub fn spin(mut self, spin: f32) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the distance used by the configured direction.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the position or direction from which the object enters.
    pub fn from(mut self, from: FadeFrom) -> Self {
        self.from = Some(from);
        self
    }
}

impl<T: ObjectHandler> Effect<T> for FadeIn {
    fn play(self, handler: &T) {
        let position = handler.get(TransformComponent::position_property());
        let scale = handler.get(TransformComponent::scale_property());
        let rotation = handler.get(TransformComponent::rotation_property());
        let start_position = self
            .from
            .map(|from| match from {
                FadeFrom::Position(position) => position,
                direction => position + direction.vector(self.offset),
            })
            .unwrap_or(position);

        play_fade(
            handler,
            self.duration,
            self.easing,
            (0.0, 1.0),
            (scale * self.scale, scale),
            (start_position, position),
            (rotation + self.spin, rotation),
        );
    }
}

/// Fades an object out toward an optional scale and position.
pub struct FadeOut {
    duration: f32,
    easing: Easing,
    scale: f32,
    spin: f32,
    from: Option<FadeFrom>,
    offset: f32,
}

impl FadeOut {
    /// Creates a fade-out with a one-second duration and default easing.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
            scale: 1.0,
            spin: 0.0,
            from: None,
            offset: 100.0,
        }
    }

    /// Sets the effect duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing curve used by every animated property.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the ending scale factor relative to the object's current scale.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the ending rotation offset in radians.
    pub fn spin(mut self, spin: f32) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the distance used by the configured direction.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the position or direction toward which the object exits.
    pub fn from(mut self, from: FadeFrom) -> Self {
        self.from = Some(from);
        self
    }
}

impl<T: ObjectHandler> Effect<T> for FadeOut {
    fn play(self, handler: &T) {
        let position = handler.get(TransformComponent::position_property());
        let scale = handler.get(TransformComponent::scale_property());
        let rotation = handler.get(TransformComponent::rotation_property());
        let start_position = position;
        let end_position = self
            .from
            .map(|from| match from {
                FadeFrom::Position(position) => position,
                direction => position + direction.vector(self.offset),
            })
            .unwrap_or(position);

        play_fade(
            handler,
            self.duration,
            self.easing,
            (1.0, 0.0),
            (scale, scale * self.scale),
            (start_position, end_position),
            (rotation, rotation + self.spin),
        );
    }
}

/// Plays a default fade-in effect on an object handler.
pub fn fade_in<T: ObjectHandler>(handler: &T) {
    FadeIn::new().play(handler);
}

/// Plays a default fade-out effect on an object handler.
pub fn fade_out<T: ObjectHandler>(handler: &T) {
    FadeOut::new().play(handler);
}
