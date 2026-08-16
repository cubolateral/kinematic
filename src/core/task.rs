use crate::core::{Easing, TrackId, TrackSetter, TrackValue};

/// A unit of work in an [`Animator`](crate::core::Animator) timeline.
///
/// Tasks are sequenced by the containing animator. [`Self::All`] is the one
/// exception: its children start together and it lasts as long as its longest
/// child.
pub enum Task {
    /// Interpolates one tracked component field over a duration.
    Tween {
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_id: TrackId,
        track_setter: TrackSetter,
        from: TrackValue,
        to: TrackValue,
        duration: f32,
        easing: Easing,
    },
    /// Advances the timeline without changing scene state.
    Wait(f32),
    /// Runs all child tasks from the same timeline position.
    All(Vec<Task>),
}
