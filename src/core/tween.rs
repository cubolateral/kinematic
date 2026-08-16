use crate::core::{Easing, Task, TrackId, TrackSetter, TrackValue};

/// Describes the interpolation of a single tracked field.
///
/// A tween becomes part of a timeline only after being converted with
/// [`Self::task`] and played by an [`Animator`](crate::core::Animator).
pub struct Tween {
    entity: hecs::Entity,
    type_id: std::any::TypeId,
    track_id: TrackId,
    track_setter: TrackSetter,
    from: TrackValue,
    to: TrackValue,
    duration: f32,
    easing: Easing,
}

impl Tween {
    /// Creates a tween with zero duration and [`Easing::InOutQuad`] easing.
    pub fn new(
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_id: TrackId,
        track_setter: TrackSetter,
        from: TrackValue,
        to: TrackValue,
    ) -> Self {
        Self {
            entity,
            type_id,
            track_id,
            track_setter,
            from,
            to,
            duration: 0.0,
            easing: Easing::InOutQuad,
        }
    }

    /// Sets the duration in timeline seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing function used between the start and target values.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Converts this description into a task that can be scheduled by an animator.
    pub fn task(self) -> Task {
        Task::Tween {
            entity: self.entity,
            type_id: self.type_id,
            track_id: self.track_id,
            track_setter: self.track_setter,
            from: self.from,
            to: self.to,
            duration: self.duration,
            easing: self.easing,
        }
    }
}
