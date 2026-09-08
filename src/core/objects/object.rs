use crate::core::{
    AnimatorHandle, SceneWorld, TrackInfo, TrackProperty, TrackValue, TrackValueType, Trackable,
    Tween,
    components::{Animation, Draw2D, Draw3D, Inspection, Morph, Name, Node, Transform2D},
    objects::{CameraTransform2D, deactivate_subtree, is_attached},
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
/// The derive macro generates a lowercase builder function and the typed handler
/// returned by that builder.
pub trait Object: hecs::DynamicBundle + Sized {
    /// Handler type returned after spawning the object into the ECS world.
    type Handler;

    #[doc(hidden)]
    const MORPHABLE: bool = false;

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
        let mut builder = hecs::EntityBuilder::new();
        builder
            .add_bundle(object)
            .add(Animation::default())
            .add(Node::default())
            .add(SnapshotStack::default())
            .add(name)
            .add(Self::inspection());
        if Self::MORPHABLE {
            builder.add(Morph::default());
        }
        let entity = world.borrow_mut().spawn(builder.build());

        Self::handler(world, entity, animator)
    }
}

/// Common access to the entity represented by a typed object handler.
pub trait ObjectHandler {
    /// Object type represented by this handler.
    type Object: Object;

    #[doc(hidden)]
    fn object_world(&self) -> SceneWorld;

    /// Returns the ECS entity represented by this handler.
    fn get_id(&self) -> hecs::Entity;

    /// Returns the object's user-facing name.
    fn get_name(&self) -> String;

    /// Replaces the object's user-facing name.
    fn set_name(&self, name: impl Into<String>);

    /// Ends this object's lifetime at the current scheduling time.
    fn remove(&self);

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

/// Spatial access for objects in a two-dimensional scene.
pub trait Object2DHandler: ObjectHandler {
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
    if let Ok(transform) = world.get::<&Transform2D>(entity) {
        return GlobalTransform {
            position: transform.position,
            rotation: transform.rotation,
            scale: transform.scale,
        };
    }

    if let Ok(transform) = world.get::<&CameraTransform2D>(entity) {
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
        if let Ok(draw) = world.get::<&Draw2D>(entity) {
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

/// Explicit capability for objects supporting particle morph and creation.
pub trait Morphable: Object {}

/// Spatial access for three-dimensional objects. Bounds are local extents.
pub trait Object3DHandler: ObjectHandler {
    fn get_box(&self) -> crate::core::types::Vector3 {
        object_box3d(&self.object_world().borrow(), self.get_id())
    }
    fn get_global_position(&self) -> crate::core::types::Vector3 {
        global_matrix3d(&self.object_world().borrow(), self.get_id())
            .transform_point3(glam::Vec3::ZERO)
    }
    fn get_global_rotation(&self) -> crate::core::types::Quaternion {
        global_rotation3d(&self.object_world().borrow(), self.get_id())
    }
    fn get_global_scale(&self) -> crate::core::types::Vector3 {
        let world = self.object_world();
        let world = world.borrow();
        let mut scale = glam::Vec3::ONE;
        let mut current = Some(self.get_id());
        while let Some(entity) = current {
            if let Ok(transform) = world.get::<&crate::core::components::Transform3D>(entity) {
                scale *= transform.scale;
            }
            current = world.get::<&Node>(entity).ok().and_then(|node| node.parent);
        }
        scale
    }
}

/// Returns an object's complete three-dimensional scene transform.
#[doc(hidden)]
pub fn global_matrix3d(world: &hecs::World, entity: hecs::Entity) -> glam::Mat4 {
    let local = if let Ok(transform) = world.get::<&crate::core::objects::CameraTransform3D>(entity)
    {
        glam::Mat4::from_rotation_translation(
            crate::core::normalized_quaternion(transform.rotation),
            transform.position,
        )
    } else {
        world
            .get::<&crate::core::components::Transform3D>(entity)
            .map_or(glam::Mat4::IDENTITY, |transform| transform.matrix())
    };
    match world.get::<&Node>(entity).ok().and_then(|node| node.parent) {
        Some(parent) => global_matrix3d(world, parent) * local,
        None => local,
    }
}

pub(crate) fn global_rotation3d(world: &hecs::World, entity: hecs::Entity) -> glam::Quat {
    let local = if let Ok(transform) = world.get::<&crate::core::objects::CameraTransform3D>(entity)
    {
        crate::core::normalized_quaternion(transform.rotation)
    } else {
        world
            .get::<&crate::core::components::Transform3D>(entity)
            .map_or(glam::Quat::IDENTITY, |transform| {
                crate::core::normalized_quaternion(transform.rotation)
            })
    };
    match world.get::<&Node>(entity).ok().and_then(|node| node.parent) {
        Some(parent) => (global_rotation3d(world, parent) * local).normalize(),
        None => local,
    }
}

pub(crate) fn object_box3d(world: &hecs::World, entity: hecs::Entity) -> glam::Vec3 {
    bounds3d(world, entity).map_or(glam::Vec3::ZERO, |(min, max)| max - min)
}

fn bounds3d(world: &hecs::World, entity: hecs::Entity) -> Option<(glam::Vec3, glam::Vec3)> {
    let size = world
        .get::<&Draw3D>(entity)
        .ok()
        .map(|draw| (draw.get_box)(world, entity));
    let (mut min, mut max) = size.map_or(
        (
            glam::Vec3::splat(f32::INFINITY),
            glam::Vec3::splat(f32::NEG_INFINITY),
        ),
        |size| (-size * 0.5, size * 0.5),
    );
    for child in crate::core::objects::children(world, entity) {
        if !world.get::<&Node>(child).is_ok_and(|n| n.is_activated) {
            continue;
        }
        let Some((child_min, child_max)) = bounds3d(world, child) else {
            continue;
        };
        let matrix = world
            .get::<&crate::core::components::Transform3D>(child)
            .map_or(glam::Mat4::IDENTITY, |t| t.matrix());
        for x in [child_min.x, child_max.x] {
            for y in [child_min.y, child_max.y] {
                for z in [child_min.z, child_max.z] {
                    let p = matrix.transform_point3(glam::vec3(x, y, z));
                    min = min.min(p);
                    max = max.max(p);
                }
            }
        }
    }
    min.is_finite().then_some((min, max))
}
