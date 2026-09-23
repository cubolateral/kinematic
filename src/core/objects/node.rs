use crate::core::{
    AnimatorHandle, SceneWorld, Tween,
    components::{Inspection, ObjectType, TreeNode, View},
    objects::{Object, ObjectHandler},
    types::{Vector2, Vector3},
};

/// Marker trait for objects whose handlers can own child objects.
pub trait Node: Object {}

/// Failure to retrieve a typed direct child from a container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildError {
    /// The container has no child at the requested index.
    NotFound { index: usize, len: usize },
    /// The child exists, but has a different concrete object type.
    TypeMismatch {
        index: usize,
        expected: &'static str,
        actual: &'static str,
    },
}

impl std::fmt::Display for ChildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { index, len } => {
                write!(
                    formatter,
                    "Child index {index} is out of bounds for {len} children."
                )
            }
            Self::TypeMismatch {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "Child at index {index} is {actual}, not {expected}."
            ),
        }
    }
}

impl std::error::Error for ChildError {}

/// Common child-management behavior for scene containers.
pub trait NodeHandler {
    #[doc(hidden)]
    fn container_world(&self) -> SceneWorld;

    #[doc(hidden)]
    fn container_entity(&self) -> hecs::Entity;

    #[doc(hidden)]
    fn container_time(&self) -> f32;

    #[doc(hidden)]
    fn container_animator(&self) -> AnimatorHandle;

    /// Adds an object subtree to this container at the current scheduling time.
    fn add(&self, handler: &impl ObjectHandler) {
        assert!(
            std::rc::Rc::ptr_eq(&self.container_world(), &handler.object_world()),
            "Added object must belong to this scene."
        );
        attach_child(
            &self.container_world(),
            self.container_entity(),
            handler.entity(),
            self.container_time(),
        );
    }

    /// Returns the entity ids of all direct children in insertion order.
    fn children(&self) -> Vec<hecs::Entity> {
        let scene_world = self.container_world();
        let world = scene_world.borrow();

        children(&world, self.container_entity())
    }

    /// Returns the entity id of the direct child at `index`.
    fn get_child_entity(&self, index: usize) -> Result<hecs::Entity, ChildError> {
        let children = self.children();

        children.get(index).copied().ok_or(ChildError::NotFound {
            index,
            len: children.len(),
        })
    }

    /// Returns the typed handler for a direct child at `index`.
    fn get_child<T: Object + 'static>(&self, index: usize) -> Result<T::Handler, ChildError> {
        let scene_world = self.container_world();
        let expected_name = T::inspection().object_name;
        let expected_type = std::any::TypeId::of::<T>();
        let entity = self.get_child_entity(index)?;
        let (actual_type, actual_name) = {
            let world = scene_world.borrow();
            let object_type = world
                .get::<&ObjectType>(entity)
                .expect("Child must contain an ObjectType component.");
            let inspection = world
                .get::<&Inspection>(entity)
                .expect("Child must contain an Inspection component.");

            (object_type.0, inspection.object_name)
        };

        if actual_type != expected_type {
            return Err(ChildError::TypeMismatch {
                index,
                expected: expected_name,
                actual: actual_name,
            });
        }

        Ok(T::handler(scene_world, entity, self.container_animator()))
    }
}

/// Handler for the scene's internal root container.
pub struct RootHandler {
    pub(crate) world: SceneWorld,
    pub(crate) entity: hecs::Entity,
    pub(crate) animator: AnimatorHandle,
}

impl Clone for RootHandler {
    fn clone(&self) -> Self {
        Self {
            world: std::rc::Rc::clone(&self.world),
            entity: self.entity,
            animator: self.animator.clone(),
        }
    }
}

impl RootHandler {
    /// Returns the ECS entity represented by the root.
    pub fn entity(&self) -> hecs::Entity {
        self.entity
    }

    /// Returns the root's user-facing name.
    pub fn name(&self) -> String {
        self.world
            .borrow()
            .get::<&crate::core::components::Name>(self.entity)
            .expect("Root must contain a Name component.")
            .get()
            .to_owned()
    }

    /// Returns whether the root currently renders World 2D.
    pub fn is_view_2d(&self) -> bool {
        self.world
            .borrow()
            .get::<&View>(self.entity)
            .expect("Root must contain a View component.")
            .view_2d
    }

    /// Animates the selected world. True selects 2D and false selects 3D.
    pub fn view_2d(&self, enabled: bool) -> Tween {
        View::view_2d_property()
            .handle(
                std::rc::Rc::clone(&self.world),
                self.entity,
                self.animator.clone(),
            )
            .to(enabled)
    }

    /// Animates the selected world from an explicit starting value.
    pub fn view_2d_from(&self, from: bool, to: bool) -> Tween {
        View::view_2d_property()
            .handle(
                std::rc::Rc::clone(&self.world),
                self.entity,
                self.animator.clone(),
            )
            .from(from, to)
    }

    /// Adds a canvas to the internal root at the current scheduling time.
    pub(crate) fn add(&self, handler: &impl ObjectHandler) {
        self.animator.assert_finite_scope();
        assert!(
            std::rc::Rc::ptr_eq(&self.world, &handler.object_world()),
            "Added canvas must belong to this scene."
        );
        attach_child(
            &self.world,
            self.entity,
            handler.entity(),
            self.animator.time(),
        );
    }
}

pub(crate) fn attach_child(
    scene_world: &SceneWorld,
    parent: hecs::Entity,
    child: hecs::Entity,
    time: f32,
) {
    let world = scene_world.borrow();

    assert!(
        world.contains(child),
        "Added object must belong to this scene."
    );
    assert_ne!(parent, child, "A container must not be added to itself.");
    assert!(
        !world
            .get::<&TreeNode>(child)
            .expect("Added object must contain a Node component.")
            .is_root,
        "The scene root must not be added as a child."
    );
    assert!(
        !contains_entity(&world, child, parent),
        "Adding this object would create a container cycle."
    );

    use crate::core::{
        components::Transform3D,
        objects::{CanvasDimension, CanvasSettings},
    };
    let parent_is_root = world.get::<&TreeNode>(parent).unwrap().is_root;
    if !parent_is_root {
        assert!(
            world.get::<&CanvasSettings>(child).is_err(),
            "Canvases must be attached to the scene root."
        );
        let parent_3d = world.get::<&Transform3D>(parent).is_ok()
            || world
                .get::<&CanvasSettings>(parent)
                .is_ok_and(|s| s.dimension == CanvasDimension::Three);
        let child_3d = world.get::<&Transform3D>(child).is_ok();
        assert_eq!(
            parent_3d, child_3d,
            "Cannot mix 2D and 3D objects in a spatial container."
        );
    } else {
        assert!(
            world.get::<&CanvasSettings>(child).is_ok(),
            "Only canvases can be attached to the scene root; use World2D or World3D for objects."
        );
    }

    if let Some(container) = world.get::<&TreeNode>(child).unwrap().parent {
        assert_eq!(
            container, parent,
            "An object must not belong to more than one container."
        );
        return;
    }

    world
        .get::<&mut TreeNode>(parent)
        .expect("Container must contain a Node component.")
        .children
        .get_or_insert_with(Vec::new)
        .push(child);
    world
        .get::<&mut TreeNode>(child)
        .expect("Added object must contain a Node component.")
        .parent = Some(parent);
    activate_subtree(&world, child, time);
    crate::core::invalidate_lifetimes(&world);
}

pub(crate) fn contains_entity(
    world: &hecs::World,
    root: hecs::Entity,
    target: hecs::Entity,
) -> bool {
    if root == target {
        return true;
    }

    child_iter(world, root).any(|child| contains_entity(world, child, target))
}

pub(crate) fn is_attached(world: &hecs::World, entity: hecs::Entity) -> bool {
    world
        .get::<&TreeNode>(entity)
        .is_ok_and(|node| node.parent.is_some())
}

pub(crate) fn activate_subtree(world: &hecs::World, entity: hecs::Entity, time: f32) {
    world
        .get::<&mut TreeNode>(entity)
        .expect("Added object must contain a Node component.")
        .activate(time);

    for child in children(world, entity) {
        activate_subtree(world, child, time);
    }
}

pub(crate) fn deactivate_subtree(world: &hecs::World, entity: hecs::Entity, time: f32) {
    crate::core::invalidate_lifetimes(world);
    world
        .get::<&mut TreeNode>(entity)
        .expect("Removed object must contain a Node component.")
        .deactivate(time);

    for child in children(world, entity) {
        deactivate_subtree(world, child, time);
    }
}

pub(crate) fn child_iter(
    world: &hecs::World,
    entity: hecs::Entity,
) -> impl Iterator<Item = hecs::Entity> + '_ {
    let node = world
        .get::<&TreeNode>(entity)
        .expect("Scene object must contain a Node component.");
    let count = node.children.as_ref().map_or(0, Vec::len);
    (0..count).map(move |index| node.children.as_ref().unwrap()[index])
}

pub(crate) fn children(world: &hecs::World, entity: hecs::Entity) -> Vec<hecs::Entity> {
    world
        .get::<&TreeNode>(entity)
        .expect("Scene object must contain a Node component.")
        .children
        .clone()
        .unwrap_or_default()
}

pub(crate) fn layout_offset_2d(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    let Some(parent) = world
        .get::<&TreeNode>(entity)
        .ok()
        .and_then(|node| node.parent)
    else {
        return Vector2::ZERO;
    };
    let Ok(layout) = world.get::<&crate::core::objects::Layout2D>(parent) else {
        return Vector2::ZERO;
    };
    // ponytail: This is O(children) per child; cache only if large layouts become measurable.
    let children: Vec<_> = child_iter(world, parent)
        .filter(|child| {
            world
                .get::<&TreeNode>(*child)
                .is_ok_and(|node| node.is_activated)
        })
        .collect();
    let Some(index) = children.iter().position(|child| *child == entity) else {
        return Vector2::ZERO;
    };
    let sizes: Vec<_> = children
        .iter()
        .map(|child| crate::core::objects::object_box(world, *child))
        .collect();
    match layout.direction {
        crate::core::objects::LayoutDirection2D::Horizontal
        | crate::core::objects::LayoutDirection2D::HorizontalReverse => {
            let reverse =
                layout.direction == crate::core::objects::LayoutDirection2D::HorizontalReverse;
            let (x, _) = axis_position(&sizes, index, layout.gap.x, reverse, |size| size.x);
            Vector2::new(x, 0.0)
        }
        crate::core::objects::LayoutDirection2D::Vertical
        | crate::core::objects::LayoutDirection2D::VerticalReverse => {
            let reverse =
                layout.direction == crate::core::objects::LayoutDirection2D::VerticalReverse;
            let (y, _) = axis_position(&sizes, index, layout.gap.y, reverse, |size| size.y);
            Vector2::new(0.0, y)
        }
    }
}

pub(crate) fn layout_offset_3d(world: &hecs::World, entity: hecs::Entity) -> Vector3 {
    let Some(parent) = world
        .get::<&TreeNode>(entity)
        .ok()
        .and_then(|node| node.parent)
    else {
        return Vector3::ZERO;
    };
    let Ok(layout) = world.get::<&crate::core::objects::Layout3D>(parent) else {
        return Vector3::ZERO;
    };
    // ponytail: This is O(children) per child; cache only if large layouts become measurable.
    let children: Vec<_> = child_iter(world, parent)
        .filter(|child| {
            world
                .get::<&TreeNode>(*child)
                .is_ok_and(|node| node.is_activated)
        })
        .collect();
    let Some(index) = children.iter().position(|child| *child == entity) else {
        return Vector3::ZERO;
    };
    let sizes: Vec<_> = children
        .iter()
        .map(|child| crate::core::objects::object_box3d(world, *child))
        .collect();
    match layout.direction {
        crate::core::objects::LayoutDirection3D::Horizontal
        | crate::core::objects::LayoutDirection3D::HorizontalReverse => {
            let reverse =
                layout.direction == crate::core::objects::LayoutDirection3D::HorizontalReverse;
            let (x, _) = axis_position(&sizes, index, layout.gap.x, reverse, |size| size.x);
            Vector3::new(x, 0.0, 0.0)
        }
        crate::core::objects::LayoutDirection3D::Vertical
        | crate::core::objects::LayoutDirection3D::VerticalReverse => {
            let reverse =
                layout.direction == crate::core::objects::LayoutDirection3D::VerticalReverse;
            let (y, _) = axis_position(&sizes, index, layout.gap.y, reverse, |size| size.y);
            Vector3::new(0.0, y, 0.0)
        }
        crate::core::objects::LayoutDirection3D::Depth
        | crate::core::objects::LayoutDirection3D::DepthReverse => {
            let reverse = layout.direction == crate::core::objects::LayoutDirection3D::DepthReverse;
            let (z, _) = axis_position(&sizes, index, layout.gap.z, reverse, |size| size.z);
            Vector3::new(0.0, 0.0, z)
        }
    }
}

fn axis_position<T>(
    sizes: &[T],
    target: usize,
    gap: f32,
    reverse: bool,
    extent: impl Fn(&T) -> f32,
) -> (f32, f32) {
    let total = sizes.iter().map(&extent).sum::<f32>() + gap * sizes.len().saturating_sub(1) as f32;
    let mut cursor = -total * 0.5;
    for order in 0..sizes.len() {
        let index = if reverse {
            sizes.len() - order - 1
        } else {
            order
        };
        let size = extent(&sizes[index]);
        if index == target {
            return (cursor + size * 0.5, total);
        }
        cursor += size + gap;
    }
    (0.0, total)
}

#[cfg(test)]
mod tests {
    use kinematic_macros::{Node, Object};

    use crate::core::{
        Scene,
        components::*,
        objects::*,
        types::{Color, vec2, vec3},
    };

    #[derive(Object, Node)]
    #[object(spatial = "2d", builder = "test_container")]
    struct TestContainer {
        #[trackable]
        transform: Transform2D,
        #[trackable]
        draw: Draw2D,
    }

    impl Default for TestContainer {
        fn default() -> Self {
            Self {
                transform: Default::default(),
                draw: Default::default(),
            }
        }
    }

    #[test]
    fn derived_container_initializes_children_on_first_add() {
        let mut scene = Scene::new();
        let container = test_container().build(&mut scene);
        let child = rect().build(&mut scene);

        assert!(
            scene
                .world()
                .get::<&TreeNode>(container.entity())
                .unwrap()
                .children
                .is_none()
        );

        container.add(&child);

        assert_eq!(
            scene
                .world()
                .get::<&TreeNode>(container.entity())
                .unwrap()
                .children
                .as_deref(),
            Some([child.entity()].as_slice())
        );
    }

    #[test]
    fn container_2d_layout_is_procedural_and_preserves_child_positions() {
        let mut scene = Scene::new();
        let container = container_2d().gap(vec2(2.0, 4.0)).build(&mut scene);
        let first = rect()
            .size(vec2(10.0, 6.0))
            .position(vec2(3.0, 0.0))
            .build(&mut scene);
        let second = rect().size(vec2(20.0, 8.0)).build(&mut scene);
        container.add(&first);
        container.add(&second);
        scene.world_2d().add(&container);

        assert_eq!(first.global_position(), vec2(-8.0, 0.0));
        assert_eq!(second.global_position(), vec2(6.0, 0.0));
        assert_eq!(first.get(Transform2D::position_property()), vec2(3.0, 0.0));

        container.set_direction(LayoutDirection2D::HorizontalReverse);
        assert_eq!(first.global_position(), vec2(14.0, 0.0));
        assert_eq!(second.global_position(), vec2(-6.0, 0.0));
    }

    #[test]
    fn container_3d_uses_sizes_and_gap() {
        let mut scene = Scene::new();
        let container = container_3d()
            .gap(vec3(0.0, 0.0, 1.0))
            .direction(LayoutDirection3D::Depth)
            .build(&mut scene);
        let first = prism().size(vec3(1.0, 1.0, 2.0)).build(&mut scene);
        let second = prism().size(vec3(1.0, 1.0, 4.0)).build(&mut scene);
        container.add(&first);
        container.add(&second);
        scene.world_3d().add(&container);

        assert_eq!(first.global_position(), vec3(0.0, 0.0, -2.5));
        assert_eq!(second.global_position(), vec3(0.0, 0.0, 1.5));
    }

    #[test]
    fn non_group_containers_draw_their_children() {
        let mut scene = Scene::new();
        let container = test_container().position(vec2(4.0, 0.0)).build(&mut scene);
        let child = rect()
            .size(vec2(4.0, 4.0))
            .fill(Color::RED)
            .build(&mut scene);

        container.add(&child);
        scene.world_2d().add(&container);

        let image_info = skia_safe::ImageInfo::new(
            (16, 16),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::colors::TRANSPARENT);
        canvas.translate((8.0, 8.0));

        scene.draw(canvas);

        let pixels = surface.peek_pixels().unwrap();
        assert_eq!(pixels.get_color((12, 8)).r(), 255);
        assert_eq!(pixels.get_color((8, 8)).a(), 0);
    }

    #[test]
    fn containers_reject_cycles_and_multiple_parents() {
        let mut scene = Scene::new();
        let first = test_container().build(&mut scene);
        let second = test_container().build(&mut scene);
        let child = rect().build(&mut scene);

        first.add(&second);
        second.add(&child);

        let cycle = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| second.add(&first)));
        assert!(cycle.is_err());

        let duplicate =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| first.add(&child)));
        assert!(duplicate.is_err());
    }

    #[test]
    fn removing_a_container_updates_its_complete_subtree() {
        let mut scene = Scene::new();
        let container = test_container().build(&mut scene);
        let child = rect().build(&mut scene);

        container.add(&child);
        scene.world_2d().add(&container);
        scene.wait(2.0);
        container.remove();

        let world = scene.world();
        assert_eq!(
            world.get::<&TreeNode>(container.entity()).unwrap().lifetime,
            [0.0, 2.0]
        );
        assert_eq!(
            world.get::<&TreeNode>(child.entity()).unwrap().lifetime,
            [0.0, 2.0]
        );
    }

    #[test]
    fn container_bounds_include_transformed_children() {
        let mut scene = Scene::new();
        let container = test_container().build(&mut scene);
        let child = rect()
            .size(vec2(10.0, 20.0))
            .position(vec2(30.0, -10.0))
            .scale(vec2(2.0, 1.0))
            .rotation(std::f32::consts::FRAC_PI_2)
            .build(&mut scene);

        container.add(&child);

        let size = container.box_size();
        assert!((size.x - 20.0).abs() < 0.001);
        assert!((size.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn group_origin_moves_its_children_to_the_selected_edge() {
        let mut scene = Scene::new();
        let group = group_2d().origin(vec2(1.0, 0.0)).build(&mut scene);
        let children = [-4.0, 0.0, 4.0].map(|x| {
            let child = rect()
                .size(vec2(4.0, 4.0))
                .position(vec2(x, 0.0))
                .build(&mut scene);
            group.add(&child);
            child
        });

        assert_eq!(children[2].global_position(), vec2(-2.0, 0.0));
        assert_eq!(group.get(Transform2D::origin_property()), vec2(1.0, 0.0));
    }

    #[test]
    fn container_opacity_composites_the_subtree_once() {
        let mut scene = Scene::new();
        let first = rect()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .build(&mut scene);
        let second = rect()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .build(&mut scene);
        let container = test_container().opacity(0.5).build(&mut scene);

        container.add(&first);
        container.add(&second);
        scene.world_2d().add(&container);

        let image_info = skia_safe::ImageInfo::new(
            (16, 16),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::colors::TRANSPARENT);
        canvas.translate((8.0, 8.0));

        scene.draw(canvas);

        let center = surface.peek_pixels().unwrap().get_color((8, 8));
        assert_eq!(center.r(), 255);
        assert!((127..=128).contains(&center.a()));
    }

    #[test]
    fn higher_z_index_draws_and_picks_in_front() {
        let mut scene = Scene::new();
        let front = rect()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .z_index(1)
            .build(&mut scene);
        let back = rect()
            .size(vec2(8.0, 8.0))
            .fill(Color::BLUE)
            .z_index(-1)
            .build(&mut scene);

        scene.world_2d().add(&front);
        scene.world_2d().add(&back);

        let image_info = skia_safe::ImageInfo::new(
            (16, 16),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        surface.canvas().translate((8.0, 8.0));

        scene.draw(surface.canvas());

        let center = surface.peek_pixels().unwrap().get_color((8, 8));
        assert_eq!(center.r(), 255);
        assert_eq!(center.b(), 0);
        assert_eq!(scene.pick(vec2(0.0, 0.0)), Some(front.entity()));
    }

    #[test]
    fn handlers_compose_global_values_without_skew() {
        let mut scene = Scene::new();
        let outer = group_2d()
            .position(vec2(10.0, 20.0))
            .scale(vec2(2.0, 3.0))
            .rotation(std::f32::consts::FRAC_PI_2)
            .opacity(0.5)
            .build(&mut scene);
        let inner = group_2d()
            .position(vec2(4.0, 5.0))
            .scale(vec2(5.0, 7.0))
            .rotation(0.25)
            .opacity(0.4)
            .build(&mut scene);
        let child = rect()
            .position(vec2(1.0, 2.0))
            .scale(vec2(0.5, 0.25))
            .rotation(0.125)
            .opacity(0.5)
            .build(&mut scene);

        inner.add(&child);
        outer.add(&inner);
        scene.world_2d().add(&outer);

        let inner_position = vec2(-5.0, 28.0);
        let scaled_child_position = vec2(10.0, 42.0);
        let rotation = std::f32::consts::FRAC_PI_2 + 0.25;
        let (sin, cos) = rotation.sin_cos();
        let expected_position = inner_position
            + vec2(
                scaled_child_position.x * cos - scaled_child_position.y * sin,
                scaled_child_position.x * sin + scaled_child_position.y * cos,
            );

        assert!(
            child
                .global_position()
                .abs_diff_eq(expected_position, 0.0001)
        );
        assert!((child.global_rotation() - (std::f32::consts::FRAC_PI_2 + 0.375)).abs() < 0.0001);
        assert!(child.global_scale().abs_diff_eq(vec2(5.0, 5.25), 0.0001));
        assert!((child.global_opacity() - 0.1).abs() < 0.0001);
        assert_eq!(scene.pick(expected_position), Some(child.entity()));
    }
}
