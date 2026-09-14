use crate::core::TrackableInfo;

/// Concrete Rust type of a spawned scene object.
pub(crate) struct ObjectType(pub(crate) std::any::TypeId);

/// Entity-level metadata used by the inspector UI.
#[derive(Clone)]
pub struct Inspection {
    /// Object name for the entity.
    pub object_name: &'static str,

    /// Returns the static set of trackable components for this entity type.
    pub get: fn(&hecs::World, hecs::Entity) -> &'static [TrackableInfo],

    /// Trackable components added dynamically while building the entity.
    pub(crate) additional: Vec<TrackableInfo>,
}

impl Inspection {
    #[doc(hidden)]
    pub fn new(
        object_name: &'static str,
        get: fn(&hecs::World, hecs::Entity) -> &'static [TrackableInfo],
    ) -> Self {
        Self {
            object_name,
            get,
            additional: Vec::new(),
        }
    }

    pub(crate) fn add_trackable(&mut self, trackable: TrackableInfo) {
        self.additional.push(trackable);
    }

    /// Iterates over every fixed and dynamically added trackable component.
    pub(crate) fn trackables<'a>(
        &'a self,
        world: &hecs::World,
        entity: hecs::Entity,
    ) -> impl Iterator<Item = &'a TrackableInfo> {
        (self.get)(world, entity)
            .iter()
            .chain(self.additional.iter())
    }
}

impl Default for Inspection {
    fn default() -> Self {
        Self::new("Object", |_world, _entity| &[])
    }
}
