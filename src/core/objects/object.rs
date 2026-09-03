use crate::core::{
    AnimatorHandle, SceneWorld, TrackInfo, TrackProperty, TrackValue, TrackValueType, Trackable,
    Tween,
    components::{Animation, Inspection, Name, Node},
    objects::{deactivate_subtree, is_attached},
    types::Vector2,
};

struct SnapshotValue {
    type_id: std::any::TypeId,
    track_info: &'static TrackInfo,
    value: TrackValue,
}

#[derive(Default)]
struct SnapshotStack(Vec<Vec<SnapshotValue>>);

/// Marker trait for object types that can be spawned into the scene.
///
/// The derive macro generates the builder returned by `Object::builder` and the
/// typed handler returned by that builder.
pub trait Object: hecs::DynamicBundle + Sized {
    /// Builder type returned by [`Object::builder`](Self::builder).
    type Builder;
    /// Handler type returned after spawning the object into the ECS world.
    type Handler;

    /// Builds an object builder.
    fn builder() -> Self::Builder;

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
                .add(SnapshotStack::default())
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

    /// Ends this object's lifetime at the current scheduling time.
    fn remove(&self);

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

    /// Saves all tracked property values on this object's snapshot stack.
    fn save(&self);

    /// Pops the latest snapshot and creates a tween back to its values.
    fn restore(&self) -> Tween<Self::Object>;
}

/// Pushes the current tracked values onto an object's snapshot stack.
#[doc(hidden)]
pub fn save_object(world: &SceneWorld, entity: hecs::Entity) {
    let values = {
        let world = world.borrow();
        let inspection = *world
            .get::<&Inspection>(entity)
            .expect("Object handler must contain Inspection metadata.");

        let mut values = Vec::new();

        for trackable in (inspection.get)(&world, entity) {
            let type_id = (trackable.type_id)();

            for track_info in (trackable.get)() {
                values.push(SnapshotValue {
                    type_id,
                    track_info,
                    value: (track_info.get)(&world, entity),
                });
            }
        }

        values
    };

    world
        .borrow()
        .get::<&mut SnapshotStack>(entity)
        .expect("Object handler must contain a snapshot stack.")
        .0
        .push(values);
}

/// Pops an object's latest snapshot and builds a tween back to it.
#[doc(hidden)]
pub fn restore_object<Object>(
    world: &SceneWorld,
    entity: hecs::Entity,
    animator: AnimatorHandle,
) -> Tween<Object> {
    let snapshot = world
        .borrow()
        .get::<&mut SnapshotStack>(entity)
        .expect("Object handler must contain a snapshot stack.")
        .0
        .pop()
        .expect("Cannot restore an object without a saved snapshot.");

    let targets = {
        let world_ref = world.borrow();

        snapshot
            .into_iter()
            .map(|saved| {
                let from = (saved.track_info.get)(&world_ref, entity);
                (saved.track_info.set)(&world_ref, entity, saved.value.clone());

                (saved.type_id, saved.track_info, from, saved.value)
            })
            .collect()
    };

    Tween::from_targets(std::rc::Rc::clone(world), entity, targets, animator)
}

/// Ends an object subtree's lifetime at the supplied scheduling time.
#[doc(hidden)]
pub fn remove_object(world: &SceneWorld, entity: hecs::Entity, time: f32) {
    let world = world.borrow();
    let node = world
        .get::<&Node>(entity)
        .expect("Removed object must contain a Node component.");

    assert!(!node.is_root, "The scene root must not be removed.");
    drop(node);

    assert!(
        is_attached(&world, entity),
        "Removed object must belong to a container."
    );

    deactivate_subtree(&world, entity, time);
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
