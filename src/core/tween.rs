use crate::core::{AnimatorHandle, Easing, Task, TrackInfo, TrackValue};

/// Describes the interpolation of a single tracked field.
///
/// A tween becomes part of its scene timeline when [`Self::play`] is called.
/// It can also be converted into a [`Task`] and passed to an
/// [`Animator`](crate::core::Animator) manually.
pub struct Tween {
    entity: hecs::Entity,
    type_id: std::any::TypeId,
    track_info: &'static TrackInfo,
    from: TrackValue,
    to: TrackValue,
    duration: f32,
    easing: Easing,
    animator: AnimatorHandle,
}

impl Tween {
    /// Creates a tween with zero duration and [`Easing::default()`] easing.
    pub fn new(
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: TrackValue,
        to: TrackValue,
        animator: AnimatorHandle,
    ) -> Self {
        Self {
            entity,
            type_id,
            track_info,
            from,
            to,
            duration: 0.0,
            easing: Easing::default(),
            animator,
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

    /// Registers this tween in the animator associated with its object handler.
    pub fn play(self) {
        let animator = self.animator.clone();
        animator.play(self.task());
    }

    /// Converts this description into a task that can be scheduled by an animator.
    pub fn task(self) -> Task {
        Task::Tween {
            entity: self.entity,
            type_id: self.type_id,
            track_info: self.track_info,
            from: self.from,
            to: self.to,
            duration: self.duration,
            easing: self.easing,
        }
    }
}
