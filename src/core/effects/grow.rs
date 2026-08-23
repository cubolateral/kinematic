use crate::core::effects::Effect;
use crate::core::{Easing, Scene, components::Transform, objects::ObjectHandler, types::Vector2};

/// Anchor used as the origin of a grow effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GrowFrom {
    /// Grows from the top-left anchor.
    TopLeft,
    /// Grows from the top anchor.
    Top,
    /// Grows from the top-right anchor.
    TopRight,
    /// Grows from the left anchor.
    Left,
    /// Grows from the center anchor.
    Center,
    /// Grows from the right anchor.
    Right,
    /// Grows from the bottom-left anchor.
    BottomLeft,
    /// Grows from the bottom anchor.
    Bottom,
    /// Grows from the bottom-right anchor.
    BottomRight,
    /// Grows from a custom position.
    Position(Vector2),
}

impl GrowFrom {
    fn resolve(self, center: Vector2, size: Vector2, scale: Vector2, rotation: f32) -> Vector2 {
        let half_size = size * 0.5;
        let local_anchor = match self {
            Self::TopLeft => Vector2::new(-half_size.x, -half_size.y),
            Self::Top => Vector2::new(0.0, -half_size.y),
            Self::TopRight => Vector2::new(half_size.x, -half_size.y),
            Self::Left => Vector2::new(-half_size.x, 0.0),
            Self::Center => Vector2::ZERO,
            Self::Right => Vector2::new(half_size.x, 0.0),
            Self::BottomLeft => Vector2::new(-half_size.x, half_size.y),
            Self::Bottom => Vector2::new(0.0, half_size.y),
            Self::BottomRight => Vector2::new(half_size.x, half_size.y),
            Self::Position(position) => position,
        };
        let scaled_anchor = local_anchor * scale;
        let (sin, cos) = rotation.sin_cos();
        let rotated_anchor = Vector2::new(
            scaled_anchor.x * cos - scaled_anchor.y * sin,
            scaled_anchor.x * sin + scaled_anchor.y * cos,
        );

        center + rotated_anchor
    }
}

/// Grows an object from zero scale at a configurable anchor.
pub struct GrowIn {
    duration: f32,
    easing: Easing,
    spin: f32,
    from: GrowFrom,
}

impl GrowIn {
    /// Creates a grow-in with a one-second duration and a centered origin.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
            spin: 0.0,
            from: GrowFrom::Center,
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

    /// Sets the starting rotation offset in radians.
    pub fn spin(mut self, spin: f32) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the position from which the object grows.
    pub fn from(mut self, from: GrowFrom) -> Self {
        self.from = from;
        self
    }
}

impl Default for GrowIn {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for GrowIn {
    fn play<T: ObjectHandler>(self, s: &mut Scene, handler: &T) {
        let position = handler.get(Transform::position_property());
        let scale = handler.get(Transform::scale_property());
        let rotation = handler.get(Transform::rotation_property());
        let start_position = self
            .from
            .resolve(position, handler.get_box(), scale, rotation);

        s.all(|_| {
            handler
                .animate_from(Transform::scale_property(), Vector2::ZERO, scale)
                .duration(self.duration)
                .easing(self.easing)
                .play();
            handler
                .animate_from(Transform::position_property(), start_position, position)
                .duration(self.duration)
                .easing(self.easing)
                .play();
            handler
                .animate_from(
                    Transform::rotation_property(),
                    rotation + self.spin,
                    rotation,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();
        });
    }
}

/// Shrinks an object to zero scale toward a configurable anchor.
pub struct GrowOut {
    duration: f32,
    easing: Easing,
    spin: f32,
    from: GrowFrom,
}

impl GrowOut {
    /// Creates a grow-out with a one-second duration and a centered origin.
    pub fn new() -> Self {
        Self {
            duration: 1.0,
            easing: Easing::default(),
            spin: 0.0,
            from: GrowFrom::Center,
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

    /// Sets the ending rotation offset in radians.
    pub fn spin(mut self, spin: f32) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the position toward which the object shrinks.
    pub fn from(mut self, from: GrowFrom) -> Self {
        self.from = from;
        self
    }
}

impl Default for GrowOut {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for GrowOut {
    fn play<T: ObjectHandler>(self, s: &mut Scene, handler: &T) {
        let position = handler.get(Transform::position_property());
        let scale = handler.get(Transform::scale_property());
        let rotation = handler.get(Transform::rotation_property());
        let end_position = self
            .from
            .resolve(position, handler.get_box(), scale, rotation);

        s.all(|_| {
            handler
                .animate_from(Transform::scale_property(), scale, Vector2::ZERO)
                .duration(self.duration)
                .easing(self.easing)
                .play();
            handler
                .animate_from(Transform::position_property(), position, end_position)
                .duration(self.duration)
                .easing(self.easing)
                .play();
            handler
                .animate_from(
                    Transform::rotation_property(),
                    rotation,
                    rotation + self.spin,
                )
                .duration(self.duration)
                .easing(self.easing)
                .play();
        });
    }
}
