use crate::core::{
    AnimatorHandle, SceneWorld, TrackInfo, TrackProperty, TrackValue, TrackValueType, Trackable,
    Tween,
    components::{Animation, Draw, Inspection, Name, Node, Transform},
    objects::{CameraTransform, deactivate_subtree, is_attached},
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

    /// Returns the object's position in scene coordinates.
    fn get_global_position(&self) -> Vector2;

    /// Returns the object's accumulated rotation in radians.
    fn get_global_rotation(&self) -> f32;

    /// Returns the object's accumulated scale without introducing skew.
    fn get_global_scale(&self) -> Vector2;

    /// Returns the object's opacity combined with its ancestor opacities.
    fn get_global_opacity(&self) -> f32;

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

#[derive(Clone, Copy)]
pub(crate) struct GlobalTransform {
    pub(crate) position: Vector2,
    pub(crate) rotation: f32,
    pub(crate) scale: Vector2,
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            position: Vector2::ZERO,
            rotation: 0.0,
            scale: Vector2::ONE,
        }
    }
}

impl GlobalTransform {
    pub(crate) fn append(self, local: Self) -> Self {
        let position = local.position * self.scale;
        let (sin, cos) = self.rotation.sin_cos();
        let position = Vector2::new(
            position.x * cos - position.y * sin,
            position.x * sin + position.y * cos,
        );

        Self {
            position: self.position + position,
            rotation: self.rotation + local.rotation,
            scale: self.scale * local.scale,
        }
    }
}

pub(crate) fn local_transform(world: &hecs::World, entity: hecs::Entity) -> GlobalTransform {
    if let Ok(transform) = world.get::<&Transform>(entity) {
        return GlobalTransform {
            position: transform.position,
            rotation: transform.rotation,
            scale: transform.scale,
        };
    }

    if let Ok(transform) = world.get::<&CameraTransform>(entity) {
        let inverse_zoom = if transform.zoom.abs() <= f32::EPSILON {
            0.0
        } else {
            transform.zoom.recip()
        };

        return GlobalTransform {
            position: transform.position,
            rotation: transform.rotation,
            scale: Vector2::splat(inverse_zoom),
        };
    }

    GlobalTransform::default()
}

pub(crate) fn global_transform(world: &hecs::World, entity: hecs::Entity) -> GlobalTransform {
    let mut lineage = vec![entity];
    let mut current = entity;

    while let Some(parent) = world
        .get::<&Node>(current)
        .expect("Scene object must contain a Node component.")
        .parent
    {
        lineage.push(parent);
        current = parent;
    }

    lineage
        .into_iter()
        .rev()
        .fold(GlobalTransform::default(), |global, current| {
            global.append(local_transform(world, current))
        })
}

/// Returns an object's position in scene coordinates.
#[doc(hidden)]
pub fn object_global_position(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    global_transform(world, entity).position
}

/// Returns an object's accumulated rotation in radians.
#[doc(hidden)]
pub fn object_global_rotation(world: &hecs::World, entity: hecs::Entity) -> f32 {
    global_transform(world, entity).rotation
}

/// Returns an object's accumulated scale without introducing skew.
#[doc(hidden)]
pub fn object_global_scale(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    global_transform(world, entity).scale
}

/// Returns an object's opacity combined with its ancestor opacities.
#[doc(hidden)]
pub fn object_global_opacity(world: &hecs::World, entity: hecs::Entity) -> f32 {
    let mut opacity = 1.0;
    let mut current = Some(entity);

    while let Some(entity) = current {
        if let Ok(draw) = world.get::<&Draw>(entity) {
            opacity *= draw.opacity.clamp(0.0, 1.0);
        }

        current = world
            .get::<&Node>(entity)
            .expect("Scene object must contain a Node component.")
            .parent;
    }

    opacity
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
