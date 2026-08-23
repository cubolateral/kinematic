use crate::core::{Easing, TrackInfo, TrackValue};

/// A unit of work in a [`Scene`](crate::core::Scene) timeline.
///
/// Tasks are sequenced by the containing scene. [`Self::Chain`] runs its
/// children sequentially, [`Self::All`] starts its children together, and
/// [`Self::Repeat`] repeats its children sequentially.
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
    /// Advances the timeline without changing scene state.
    Wait(f32),
    /// Runs child tasks sequentially from the same group.
    Chain(Vec<Task>),
    /// Runs all child tasks from the same timeline position.
    All(Vec<Task>),
    /// Repeats a sequence of child tasks a fixed number of times.
    Repeat(usize, Vec<Task>),
}
