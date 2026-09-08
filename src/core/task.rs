use crate::core::{
    Easing, TrackInfo, TrackValue,
    types::{Quaternion, Vector3},
};

/// A unit of work in a [`Scene`](crate::core::Scene) timeline.
///
/// Tasks are sequenced by the containing scene. [`Self::Chain`] runs its
/// children sequentially, [`Self::All`] starts its children together, and
/// [`Self::Repeat`] loops a finite cycle without advancing the containing timeline.
#[derive(Clone)]
pub enum Task {
    /// Interpolates one tracked component field over a duration.
    Tween {
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: TrackValue,
        to: TrackValue,
        duration: f32,
        easing: Easing,
    },
    /// Rotates a quaternion through an axis-angle path without losing full turns.
    #[doc(hidden)]
    RotationTween {
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: Quaternion,
        axis: Vector3,
        angle: f32,
        duration: f32,
        easing: Easing,
    },
    /// Advances the timeline without changing scene state.
    Wait(f32),
    /// Runs child tasks sequentially from the same group.
    Chain(Vec<Task>),
    /// Runs all child tasks from the same timeline position.
    All(Vec<Task>),
    /// Loops a finite, positive-duration sequence without advancing the timeline.
    /// Nested repeats and concurrent writes to the same property are rejected.
    Repeat(Vec<Task>),
}

impl Task {
    /// Delays this task by the specified number of timeline seconds.
    pub fn delay(self, duration: f32) -> Self {
        Self::Chain(vec![Self::Wait(duration), self])
    }
}
