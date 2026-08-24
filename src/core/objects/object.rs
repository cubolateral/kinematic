use crate::core::{
    AnimatorHandle, SceneWorld, TrackProperty, TrackValueType, Trackable, Tween,
    components::{Animation, Inspection, Name, Node},
    types::Vector2,
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
    fn builder(world: SceneWorld, animator: AnimatorHandle) -> Self::Builder;

    /// Builds the handler from the spawned entity.
    fn handler(world: SceneWorld, entity: hecs::Entity, animator: AnimatorHandle) -> Self::Handler;

    /// Returns the inspection metadata component attached to spawned entities.
    fn inspection() -> Inspection;

    /// Spawns an inactive object and returns its typed handler.
    #[doc(hidden)]
    fn spawn(
        world: SceneWorld,
        animator: AnimatorHandle,
        object: Self,
        name: Name,
    ) -> Self::Handler {
        let entity = world.borrow_mut().spawn(
            hecs::EntityBuilder::new()
                .add_bundle(object)
                .add(Animation::default())
                .add(Node::default())
                .add(name)
                .add(Self::inspection())
                .build(),
        );

        Self::handler(world, entity, animator)
    }
}

/// Common access to the entity represented by a typed object handler.
pub trait ObjectHandler {
    /// Object type represented by this handler.
    type Object: Object;

    /// Returns the ECS entity represented by this handler.
    fn get_id(&self) -> hecs::Entity;

    /// Returns the object's user-facing name.
    fn get_name(&self) -> String;

    /// Replaces the object's user-facing name.
    fn set_name(&self, name: impl Into<String>);

    /// Returns the object's local bounding-box size.
    fn get_box(&self) -> Vector2;

    /// Reads a typed trackable property from the object.
    fn get<T: TrackValueType>(&self, property: TrackProperty<T>) -> T;

    /// Creates a tween from the current property value to a target value.
    fn animate<T: TrackValueType>(&self, property: TrackProperty<T>, to: T) -> Tween<Self::Object>;

    /// Creates a tween from an explicit starting value to a target value.
    fn animate_from<T: TrackValueType>(
        &self,
        property: TrackProperty<T>,
        from: T,
        to: T,
    ) -> Tween<Self::Object>;
}

/// Marks an object as containing a specific trackable component.
#[doc(hidden)]
pub trait ObjectTrackable<T: Trackable>: Object {}

/// Carries the object type through the generated handler-field layers.
#[doc(hidden)]
pub trait HandlerContext {
    type Object: Object;
}

/// Innermost marker used by generated object handlers.
#[doc(hidden)]
pub struct HandlerRoot<T: Object>(std::marker::PhantomData<T>);

impl<T: Object> HandlerRoot<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: Object> HandlerContext for HandlerRoot<T> {
    type Object = T;
}

/// Internal bridge used by generated component setters on object builders.
#[doc(hidden)]
pub trait ObjectBuilderComponent<T> {
    fn component_mut(&mut self) -> &mut T;
}
