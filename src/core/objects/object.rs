use super::render::object_box;
use crate::core::{
    AnimatorHandle, SceneWorld, SignalFrame, SignalHandle, TrackProperty, TrackTarget, TrackValue,
    TrackValueType, Trackable, Tween,
    components::{
        Animation, Draw2D, Draw3D, Inspection, Morph, Name, ObjectType, Transform2D, TreeNode,
    },
    objects::{deactivate_subtree, is_attached},
    types::Vector2,
};

#[derive(Clone)]
struct SnapshotValue {
    target: TrackTarget,
    value: TrackValue,
}

/// Captures every tracked value from one typed object handler.
#[derive(Clone)]
pub struct Snapshot<Handler> {
    values: Vec<SnapshotValue>,
    handler: std::marker::PhantomData<Handler>,
}

#[derive(Default)]
struct Snapshots {
    initial: Vec<SnapshotValue>,
    saved: Vec<Vec<SnapshotValue>>,
}

/// Marker trait for object types that can be spawned into the scene.
///
/// The derive macro generates a lowercase builder function and the typed handler
/// returned by that builder.
pub trait Object: hecs::DynamicBundle + Sized + 'static {
    /// Handler type returned after spawning the object into the ECS world.
    type Handler: ObjectHandler<Object = Self>;

    #[doc(hidden)]
    const SPATIAL_2D: bool = false;

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
        animator.assert_finite_scope();
        let mut builder = hecs::EntityBuilder::new();
        builder
            .add_bundle(object)
            .add(Animation::default())
            .add(TreeNode::default())
            .add(ObjectType(std::any::TypeId::of::<Self>()))
            .add(Snapshots::default())
            .add(name)
            .add(Self::inspection());
        if Self::SPATIAL_2D {
            builder.add(Morph::default());
        }
        let entity = world.borrow_mut().spawn(builder.build());
        let initial = snapshot_values(&world, entity);
        world
            .borrow()
            .get::<&mut Snapshots>(entity)
            .expect("Spawned object must contain snapshots.")
            .initial = initial;

        Self::handler(world, entity, animator)
    }
}

/// Common access to the entity represented by a typed object handler.
pub trait ObjectHandler: Clone {
    /// Object type represented by this handler.
    type Object: Object;

    #[doc(hidden)]
    fn object_world(&self) -> SceneWorld;

    #[doc(hidden)]
    fn object_animator(&self) -> AnimatorHandle;

    /// Returns the ECS entity represented by this handler.
    fn entity(&self) -> hecs::Entity;

    /// Returns the object's user-facing name.
    fn name(&self) -> String {
        self.object_world()
            .borrow()
            .get::<&Name>(self.entity())
            .expect("Object handler must contain a Name component.")
            .get()
            .to_owned()
    }

    /// Replaces the object's user-facing name.
    fn set_name(&self, name: impl Into<String>) {
        self.object_world()
            .borrow()
            .get::<&mut Name>(self.entity())
            .expect("Object handler must contain a Name component.")
            .set(name);
    }

    /// Ends this object's lifetime at the current scheduling time.
    fn remove(&self) {
        let animator = self.object_animator();
        animator.assert_finite_scope();
        remove_object(&self.object_world(), self.entity(), animator.time());
    }

    /// Runs a callback after animation tracks while this object is active.
    ///
    /// The callback receives a fresh typed clone of this handler and the current
    /// project frame on every evaluation.
    fn signal(&self, callback: impl FnMut(Self, SignalFrame) + 'static) -> SignalHandle
    where
        Self: Sized + 'static,
    {
        let handler = self.clone();
        let mut callback = callback;
        self.object_animator()
            .signal(self.entity(), move |frame| callback(handler.clone(), frame))
    }

    /// Reads a typed trackable property from the object.
    fn get<T: TrackValueType>(&self, property: TrackProperty<T>) -> T {
        property
            .handle(self.object_world(), self.entity(), self.object_animator())
            .get()
    }

    /// Creates a tween from the current property value to a target value.
    fn animate<T: TrackValueType>(&self, property: TrackProperty<T>, to: T) -> Tween<Self::Object> {
        property
            .handle(self.object_world(), self.entity(), self.object_animator())
            .set_for::<Self::Object>(to)
    }

    /// Creates a tween from an explicit starting value to a target value.
    fn animate_from<T: TrackValueType>(
        &self,
        property: TrackProperty<T>,
        from: T,
        to: T,
    ) -> Tween<Self::Object> {
        property
            .handle(self.object_world(), self.entity(), self.object_animator())
            .from_for::<Self::Object>(from, to)
    }

    /// Reads a builder-defined shader uniform with its original Rust type.
    fn get_uniform<T: TrackValueType>(&self, name: &str) -> T {
        let value = crate::core::objects::shader_uniform(
            &self.object_world().borrow(),
            self.entity(),
            name,
        )
        .unwrap_or_else(|error| panic!("{error}"));
        T::from_track_value(value)
            .unwrap_or_else(|| panic!("Shader uniform `{name}` has a different type."))
    }

    /// Writes a builder-defined shader uniform without creating a timeline task.
    fn set_uniform<T: TrackValueType>(&self, name: &str, value: T) {
        let value = value.into_track_value();
        crate::core::objects::validate_shader_track_value(&value)
            .unwrap_or_else(|error| panic!("{error}"));
        let world = self.object_world();
        let previous = crate::core::objects::shader_uniform(&world.borrow(), self.entity(), name)
            .unwrap_or_else(|error| panic!("{error}"));
        crate::core::objects::set_shader_uniform(&world.borrow(), self.entity(), name, value)
            .unwrap_or_else(|error| panic!("{error}"));
        self.object_animator()
            .record_signal_uniform_override(self.entity(), name, previous);
    }

    /// Creates a tween from the current uniform value to `to`.
    fn uniform<T: TrackValueType>(&self, name: &str, to: T) -> Tween<Self::Object> {
        let world = self.object_world();
        let animator = self.object_animator();
        animator.assert_timeline_mutation();
        let from = crate::core::objects::shader_uniform(&world.borrow(), self.entity(), name)
            .unwrap_or_else(|error| panic!("{error}"));
        let to = to.into_track_value();
        crate::core::objects::validate_shader_track_value(&to)
            .unwrap_or_else(|error| panic!("{error}"));
        let target = TrackTarget::uniform(name);
        assert!(
            std::mem::discriminant(&from) == std::mem::discriminant(&to),
            "Shader uniform `{name}` must keep its builder-defined type."
        );
        target.set(&world.borrow(), self.entity(), to.clone());
        Tween::from_targets(world, self.entity(), vec![(target, from, to)], animator)
    }

    /// Creates a tween between explicit values for a builder-defined uniform.
    fn uniform_from<T: TrackValueType>(&self, name: &str, from: T, to: T) -> Tween<Self::Object> {
        let world = self.object_world();
        let animator = self.object_animator();
        animator.assert_timeline_mutation();
        let current = crate::core::objects::shader_uniform(&world.borrow(), self.entity(), name)
            .unwrap_or_else(|error| panic!("{error}"));
        let from = from.into_track_value();
        let to = to.into_track_value();
        crate::core::objects::validate_shader_track_value(&from)
            .unwrap_or_else(|error| panic!("{error}"));
        crate::core::objects::validate_shader_track_value(&to)
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(
            std::mem::discriminant(&current) == std::mem::discriminant(&from)
                && std::mem::discriminant(&from) == std::mem::discriminant(&to),
            "Shader uniform `{name}` must keep its builder-defined type."
        );
        let target = TrackTarget::uniform(name);
        target.set(&world.borrow(), self.entity(), to.clone());
        Tween::from_targets(world, self.entity(), vec![(target, from, to)], animator)
    }

    /// Captures the current values of every tracked property.
    fn snapshot(&self) -> Snapshot<Self>
    where
        Self: Sized,
    {
        Snapshot {
            values: snapshot_values(&self.object_world(), self.entity()),
            handler: std::marker::PhantomData,
        }
    }

    /// Creates a tween from the current values to a compatible snapshot.
    fn restore_snapshot(&self, state: Snapshot<Self>) -> Tween<Self::Object>
    where
        Self: Sized,
    {
        let world = self.object_world();
        let animator = self.object_animator();
        animator.assert_timeline_mutation();
        tween_to_values(&world, self.entity(), state.values, animator)
    }

    /// Saves all tracked property values on this object's snapshot stack.
    fn save(&self) {
        save_object(&self.object_world(), self.entity());
    }

    /// Pops the latest snapshot and creates a tween back to its values.
    fn restore(&self) -> Tween<Self::Object> {
        let animator = self.object_animator();
        animator.assert_timeline_mutation();
        restore_object(&self.object_world(), self.entity(), animator)
    }

    /// Creates a tween back to the tracked values configured by the builder.
    fn reset(&self) -> Tween<Self::Object> {
        let animator = self.object_animator();
        animator.assert_timeline_mutation();
        reset_object(&self.object_world(), self.entity(), animator)
    }
}

/// Spatial access for objects in a two-dimensional scene.
pub trait Object2DHandler: ObjectHandler {
    /// Returns the object's local bounding-box size.
    fn box_size(&self) -> Vector2 {
        object_box(&self.object_world().borrow(), self.entity())
    }

    /// Returns the object's position in scene coordinates.
    fn global_position(&self) -> Vector2 {
        object_global_position(&self.object_world().borrow(), self.entity())
    }

    /// Returns the object's accumulated rotation in radians.
    fn global_rotation(&self) -> f32 {
        object_global_rotation(&self.object_world().borrow(), self.entity())
    }

    /// Returns the object's accumulated scale without introducing skew.
    fn global_scale(&self) -> Vector2 {
        object_global_scale(&self.object_world().borrow(), self.entity())
    }

    /// Returns the object's opacity combined with its ancestor opacities.
    fn global_opacity(&self) -> f32 {
        object_global_opacity(&self.object_world().borrow(), self.entity())
    }
}

/// Pushes the current tracked values onto an object's snapshot stack.
#[doc(hidden)]
pub fn save_object(world: &SceneWorld, entity: hecs::Entity) {
    let values = snapshot_values(world, entity);

    world
        .borrow()
        .get::<&mut Snapshots>(entity)
        .expect("Object handler must contain snapshots.")
        .saved
        .push(values);
}

fn snapshot_values(world: &SceneWorld, entity: hecs::Entity) -> Vec<SnapshotValue> {
    let world = world.borrow();
    let inspection = world
        .get::<&Inspection>(entity)
        .expect("Object handler must contain Inspection metadata.")
        .clone();

    let mut values = Vec::new();

    for trackable in inspection.trackables(&world, entity) {
        let type_id = (trackable.type_id)();

        for track_info in (trackable.get)() {
            values.push(SnapshotValue {
                target: TrackTarget::property(type_id, track_info),
                value: (track_info.get)(&world, entity),
            });
        }
    }

    for (name, value) in crate::core::objects::shader_uniforms(&world, entity) {
        values.push(SnapshotValue {
            target: TrackTarget::uniform(name),
            value,
        });
    }

    values
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
        .get::<&mut Snapshots>(entity)
        .expect("Object handler must contain snapshots.")
        .saved
        .pop()
        .expect("Cannot restore an object without a saved snapshot.");

    tween_to_values(world, entity, snapshot, animator)
}

fn reset_object<Object>(
    world: &SceneWorld,
    entity: hecs::Entity,
    animator: AnimatorHandle,
) -> Tween<Object> {
    let snapshot = world
        .borrow()
        .get::<&Snapshots>(entity)
        .expect("Object handler must contain snapshots.")
        .initial
        .clone();

    tween_to_values(world, entity, snapshot, animator)
}

fn tween_to_values<Object>(
    world: &SceneWorld,
    entity: hecs::Entity,
    values: Vec<SnapshotValue>,
    animator: AnimatorHandle,
) -> Tween<Object> {
    let targets = {
        let world_ref = world.borrow();

        values
            .into_iter()
            .map(|saved| {
                let from = saved.target.get(&world_ref, entity);
                saved.target.set(&world_ref, entity, saved.value.clone());

                (saved.target, from, saved.value)
            })
            .collect()
    };

    Tween::from_targets(std::rc::Rc::clone(world), entity, targets, animator)
}

/// Refreshes builder-state snapshots after shader components are attached.
#[doc(hidden)]
pub fn initialize_object_snapshots(world: &SceneWorld, entity: hecs::Entity) {
    let initial = snapshot_values(world, entity);
    world
        .borrow()
        .get::<&mut Snapshots>(entity)
        .expect("Spawned object must contain snapshots.")
        .initial = initial;
}

/// Ends an object subtree's lifetime at the supplied scheduling time.
#[doc(hidden)]
pub fn remove_object(world: &SceneWorld, entity: hecs::Entity, time: f32) {
    let world = world.borrow();
    let node = world
        .get::<&TreeNode>(entity)
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
        let layout = crate::core::objects::layout_offset_2d(world, entity);
        let origin = crate::core::objects::object_box(world, entity) * transform.origin * 0.5;
        let origin = origin * transform.scale;
        let (sin, cos) = transform.rotation.sin_cos();
        let origin = Vector2::new(
            origin.x * cos - origin.y * sin,
            origin.x * sin + origin.y * cos,
        );
        return GlobalTransform {
            position: layout + transform.position - origin,
            rotation: transform.rotation,
            scale: transform.scale,
        };
    }

    GlobalTransform::default()
}

pub(crate) fn global_transform(world: &hecs::World, entity: hecs::Entity) -> GlobalTransform {
    let mut lineage = vec![entity];
    let mut current = entity;

    while let Some(parent) = world
        .get::<&TreeNode>(current)
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
            .get::<&TreeNode>(entity)
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

/// Spatial access for three-dimensional objects. Bounds are local extents.
pub trait Object3DHandler: ObjectHandler {
    fn box_size(&self) -> crate::core::types::Vector3 {
        object_box3d(&self.object_world().borrow(), self.entity())
    }
    fn global_position(&self) -> crate::core::types::Vector3 {
        global_matrix3d(&self.object_world().borrow(), self.entity())
            .transform_point3(glam::Vec3::ZERO)
    }
    fn global_rotation(&self) -> crate::core::types::Quaternion {
        global_rotation3d(&self.object_world().borrow(), self.entity())
    }
    fn global_scale(&self) -> crate::core::types::Vector3 {
        let world = self.object_world();
        let world = world.borrow();
        let mut scale = glam::Vec3::ONE;
        let mut current = Some(self.entity());
        while let Some(entity) = current {
            if let Ok(transform) = world.get::<&crate::core::components::Transform3D>(entity) {
                scale *= transform.scale;
            }
            current = world
                .get::<&TreeNode>(entity)
                .ok()
                .and_then(|node| node.parent);
        }
        scale
    }
}

/// Returns an object's complete three-dimensional scene transform.
#[doc(hidden)]
pub fn global_matrix3d(world: &hecs::World, entity: hecs::Entity) -> glam::Mat4 {
    let local = local_matrix3d(world, entity);
    match world
        .get::<&TreeNode>(entity)
        .ok()
        .and_then(|node| node.parent)
    {
        Some(parent) => global_matrix3d(world, parent) * local,
        None => local,
    }
}

fn local_matrix3d(world: &hecs::World, entity: hecs::Entity) -> glam::Mat4 {
    world
        .get::<&crate::core::components::Transform3D>(entity)
        .map_or(glam::Mat4::IDENTITY, |transform| {
            glam::Mat4::from_translation(crate::core::objects::layout_offset_3d(world, entity))
                * transform.matrix_with_origin(object_box3d(world, entity))
        })
}

pub(crate) fn global_rotation3d(world: &hecs::World, entity: hecs::Entity) -> glam::Quat {
    let local = world
        .get::<&crate::core::components::Transform3D>(entity)
        .map_or(glam::Quat::IDENTITY, |transform| {
            crate::core::normalized_quaternion(transform.rotation)
        });
    match world
        .get::<&TreeNode>(entity)
        .ok()
        .and_then(|node| node.parent)
    {
        Some(parent) => (global_rotation3d(world, parent) * local).normalize(),
        None => local,
    }
}

pub(crate) fn object_box3d(world: &hecs::World, entity: hecs::Entity) -> glam::Vec3 {
    bounds3d_inner(world, entity, false).map_or(glam::Vec3::ZERO, |(min, max)| max - min)
}

pub(crate) fn bounds3d(
    world: &hecs::World,
    entity: hecs::Entity,
) -> Option<(glam::Vec3, glam::Vec3)> {
    bounds3d_inner(world, entity, true)
}

fn bounds3d_inner(
    world: &hecs::World,
    entity: hecs::Entity,
    shader_padding: bool,
) -> Option<(glam::Vec3, glam::Vec3)> {
    let size = world
        .get::<&Draw3D>(entity)
        .ok()
        .map(|draw| (draw.box_size)(world, entity));
    let padding = if shader_padding {
        crate::core::objects::effective_mesh_shader(world, entity)
            .ok()
            .flatten()
            .map_or(0.0, |shader| shader.bounds_padding)
    } else {
        0.0
    };
    let (mut min, mut max) = size.map_or(
        (
            glam::Vec3::splat(f32::INFINITY),
            glam::Vec3::splat(f32::NEG_INFINITY),
        ),
        |size| {
            let padding = if size == glam::Vec3::ZERO {
                0.0
            } else {
                padding
            };
            (
                -size * 0.5 - glam::Vec3::splat(padding),
                size * 0.5 + glam::Vec3::splat(padding),
            )
        },
    );
    for child in crate::core::objects::child_iter(world, entity) {
        if !world.get::<&TreeNode>(child).is_ok_and(|n| n.is_activated) {
            continue;
        }
        let Some((child_min, child_max)) = bounds3d_inner(world, child, shader_padding) else {
            continue;
        };
        let matrix = local_matrix3d(world, child);
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
