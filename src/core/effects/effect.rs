use crate::core::{Scene, objects::ObjectHandler};

/// A reusable animation that can be applied to any compatible object handler.
pub trait Effect<T: ObjectHandler> {
    /// Schedules this effect for an object handler.
    fn play(self, s: &mut Scene, handler: &T);
}
