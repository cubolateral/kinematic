use crate::core::{SceneWorld, components::Inspection};

/// Marker trait for object types that can be spawned into the scene.
///
/// The derive macro generates the typed handler returned by `Scene::create`.
pub trait Object {
    /// Handler type returned after spawning the object into the ECS world.
    type Handler;

    /// Builds the handler from the spawned entity.
    fn handler(world: SceneWorld, entity: hecs::Entity) -> Self::Handler;

    /// Returns the inspection metadata component attached to spawned entities.
    fn inspection() -> Inspection;
}

/// Common access to the entity represented by a typed object handler.
pub trait ObjectHandler {
    /// Returns the ECS entity represented by this handler.
    fn entity(&self) -> hecs::Entity;
}

/// Internal bridge used by generated component setters on object builders.
#[doc(hidden)]
pub trait ObjectBuilderComponent<T> {
    fn component_mut(&mut self) -> &mut T;
}
