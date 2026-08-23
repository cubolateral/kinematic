use crate::core::{Scene, objects::ObjectHandler};

/// A reusable animation that can be applied to any compatible object handler.
pub trait Effect {
    /// Schedules this effect for an object handler.
    fn play<T: ObjectHandler>(self, s: &mut Scene, handler: &T);
}
