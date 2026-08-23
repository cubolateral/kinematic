use crate::core::effects::Effect;
use crate::core::{
    Easing, Scene,
    components::{Draw, Transform as TransformComponent},
    objects::ObjectHandler,
    types::Vector2,
};

/// Direction used by the optional positional shift of a fade effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    /// Shifts horizontally to the left.
    Left,
    /// Shifts horizontally to the right.
    Right,
    /// Shifts vertically upward.
    Up,
    /// Shifts vertically downward.
    Down,
}

impl Shift {
    /// Direction aliases matching the conventional animation notation.
    pub const LEFT: Self = Self::Left;
    pub const RIGHT: Self = Self::Right;
    pub const UP: Self = Self::Up;
    pub const DOWN: Self = Self::Down;

    fn vector(self, offset: f32) -> Vector2 {
        match self {
            Self::Left => Vector2::new(-offset, 0.0),
            Self::Right => Vector2::new(offset, 0.0),
            Self::Up => Vector2::new(0.0, -offset),
            Self::Down => Vector2::new(0.0, offset),
        }
    }
}

/// Fades an object in from an optional scale and positional offset.
pub struct FadeIn {
    duration: f32,
    easing: Easing,
    scale: f32,
    spin: f32,
    shift: Option<Shift>,
    offset: f32,
    from: Option<Vector2>,
}

impl FadeIn {
    /// Creates a fade-in with a one-second duration and default easing.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
            scale: 1.0,
            spin: 0.0,
            shift: None,
            offset: 100.0,
            from: None,
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

    /// Sets the direction from which the object enters.
    pub fn shift(mut self, shift: Shift) -> Self {
        self.shift = Some(shift);
        self
    }

    /// Sets the distance used by the configured shift direction.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the explicit starting position of the object.
    pub fn from(mut self, position: Vector2) -> Self {
        self.from = Some(position);
        self
    }
}

impl Effect for FadeIn {
    fn play<T: ObjectHandler>(self, s: &mut Scene, handler: &T) {
        let position = handler.get(TransformComponent::position_property());
        let scale = handler.get(TransformComponent::scale_property());
        let rotation = handler.get(TransformComponent::rotation_property());
        let start_position = self
            .from
            .or_else(|| self.shift.map(|shift| position + shift.vector(self.offset)))
            .unwrap_or(position);

        s.all(|_| {
            handler
                .animate_from(Draw::opacity_property(), 0.0, 1.0)
                .duration(self.duration)
                .easing(self.easing)
                .play();

            handler
                .animate_from(
                    TransformComponent::scale_property(),
                    scale * self.scale,
                    scale,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();

            handler
                .animate_from(
                    TransformComponent::position_property(),
                    start_position,
                    position,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();
            handler
                .animate_from(
                    TransformComponent::rotation_property(),
                    rotation + self.spin,
                    rotation,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();
        });
    }
}

/// Fades an object out toward an optional scale and positional offset.
pub struct FadeOut {
    duration: f32,
    easing: Easing,
    scale: f32,
    spin: f32,
    shift: Option<Shift>,
    offset: f32,
    from: Option<Vector2>,
}

impl FadeOut {
    /// Creates a fade-out with a one-second duration and default easing.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
            scale: 1.0,
            spin: 0.0,
            shift: None,
            offset: 100.0,
            from: None,
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

    /// Sets the direction toward which the object exits.
    pub fn shift(mut self, shift: Shift) -> Self {
        self.shift = Some(shift);
        self
    }

    /// Sets the distance used by the configured shift direction.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the explicit starting position of the object.
    pub fn from(mut self, position: Vector2) -> Self {
        self.from = Some(position);
        self
    }
}

impl Effect for FadeOut {
    fn play<T: ObjectHandler>(self, s: &mut Scene, handler: &T) {
        let position = handler.get(TransformComponent::position_property());
        let scale = handler.get(TransformComponent::scale_property());
        let rotation = handler.get(TransformComponent::rotation_property());
        let start_position = self.from.unwrap_or(position);
        let end_position = self
            .shift
            .map_or(position, |shift| position + shift.vector(self.offset));

        s.all(|_| {
            handler
                .animate_from(Draw::opacity_property(), 1.0, 0.0)
                .duration(self.duration)
                .easing(self.easing)
                .play();

            handler
                .animate_from(
                    TransformComponent::scale_property(),
                    scale,
                    scale * self.scale,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();

            handler
                .animate_from(
                    TransformComponent::position_property(),
                    start_position,
                    end_position,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();
            handler
                .animate_from(
                    TransformComponent::rotation_property(),
                    rotation,
                    rotation + self.spin,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();
        });
    }
}
