use crate::core::components::Inspection;

/// Marker trait for bundle types that can be spawned into the scene.
///
/// The derive macro generates the entity handle type used by `Scene::create`.
pub trait Object {
    /// Handle type returned after spawning the bundle into the ECS world.
    type Handle;

    /// Builds the handle from the spawned entity.
    fn handle(entity: hecs::Entity) -> Self::Handle;

    /// Returns the inspection metadata component attached to spawned entities.
    fn inspection() -> Inspection;
}
