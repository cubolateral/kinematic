use crate::core::{
    SceneWorld,
    components::{Animation, Inspection, Node},
};

/// Marker trait for object types that can be spawned into the scene.
///
/// The derive macro generates the builder returned by `Scene::create` and the
/// typed handler returned by that builder.
pub trait Object: hecs::DynamicBundle + Sized {
    /// Builder type returned by [`Scene::create`](crate::core::Scene::create).
    type Builder;
    /// Handler type returned after spawning the object into the ECS world.
    type Handler;

    /// Builds an object builder connected to a scene world.
    fn builder(world: SceneWorld) -> Self::Builder;

    /// Builds the handler from the spawned entity.
    fn handler(world: SceneWorld, entity: hecs::Entity) -> Self::Handler;

    /// Returns the inspection metadata component attached to spawned entities.
    fn inspection() -> Inspection;

    /// Spawns an inactive object and returns its typed handler.
    #[doc(hidden)]
    fn spawn(world: SceneWorld, object: Self) -> Self::Handler {
        let entity = world.borrow_mut().spawn(
            hecs::EntityBuilder::new()
                .add_bundle(object)
                .add(Animation::default())
                .add(Node::default())
                .add(Self::inspection())
                .build(),
        );

        Self::handler(world, entity)
    }
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
