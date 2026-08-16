use crate::core::TrackableInfo;

/// Entity-level metadata used by the inspector UI.
#[derive(Clone, Copy)]
pub struct Inspection {
    /// Returns the static set of trackable components for this entity type.
    pub get: fn(&hecs::World, hecs::Entity) -> &'static [TrackableInfo],
}

impl Default for Inspection {
    fn default() -> Self {
        Self {
            get: |_world, _entity| &[],
        }
    }
}
