use crate::core::{
    AnimatorHandle, SceneWorld,
    components::Node,
    objects::{Object, ObjectHandler},
};

/// Marker trait for objects whose handlers can own child objects.
pub trait Container: Object {}

/// Common child-management behavior for scene containers.
pub trait ContainerHandler {
    #[doc(hidden)]
    fn container_world(&self) -> SceneWorld;

    #[doc(hidden)]
    fn container_entity(&self) -> hecs::Entity;

    #[doc(hidden)]
    fn container_time(&self) -> f32;

    /// Adds an object subtree to this container at the current scheduling time.
    fn add(&self, handler: &impl ObjectHandler) {
        attach_child(
            &self.container_world(),
            self.container_entity(),
            handler.get_id(),
            self.container_time(),
        );
    }
}

/// Handler for the scene's internal root container.
pub struct RootHandler {
    pub(crate) world: SceneWorld,
    pub(crate) entity: hecs::Entity,
    pub(crate) animator: AnimatorHandle,
}

impl RootHandler {
    /// Returns the ECS entity represented by the root.
    pub fn get_id(&self) -> hecs::Entity {
        self.entity
    }

    /// Returns the root's user-facing name.
    pub fn get_name(&self) -> String {
        self.world
            .borrow()
            .get::<&crate::core::components::Name>(self.entity)
            .expect("Root must contain a Name component.")
            .get()
            .to_owned()
    }

    /// Adds an object subtree to the root at the current scheduling time.
    pub fn add(&self, handler: &impl ObjectHandler) {
        <Self as ContainerHandler>::add(self, handler);
    }
}

impl ContainerHandler for RootHandler {
    fn container_world(&self) -> SceneWorld {
        std::rc::Rc::clone(&self.world)
    }

    fn container_entity(&self) -> hecs::Entity {
        self.entity
    }

    fn container_time(&self) -> f32 {
        self.animator.time()
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
            .get::<&Node>(child)
            .expect("Added object must contain a Node component.")
            .is_root,
        "The scene root must not be added as a child."
    );
    assert!(
        !contains_entity(&world, child, parent),
        "Adding this object would create a container cycle."
    );

    for (container, node) in world.query::<(hecs::Entity, &Node)>().iter() {
        let Some(children) = &node.children else {
            continue;
        };
        if !children.contains(&child) {
            continue;
        }

        assert_eq!(
            container, parent,
            "An object must not belong to more than one container."
        );
        return;
    }

    world
        .get::<&mut Node>(parent)
        .expect("Container must contain a Node component.")
        .children
        .get_or_insert_with(Vec::new)
        .push(child);
    world
        .get::<&mut Node>(child)
        .expect("Added object must contain a Node component.")
        .parent = Some(parent);
    activate_subtree(&world, child, time);
}

pub(crate) fn contains_entity(
    world: &hecs::World,
    root: hecs::Entity,
    target: hecs::Entity,
) -> bool {
    if root == target {
        return true;
    }

    children(world, root)
        .iter()
        .copied()
        .any(|child| contains_entity(world, child, target))
}

pub(crate) fn is_attached(world: &hecs::World, entity: hecs::Entity) -> bool {
    world.query::<&Node>().iter().any(|node| {
        node.children
            .as_ref()
            .is_some_and(|children| children.contains(&entity))
    })
}

pub(crate) fn activate_subtree(world: &hecs::World, entity: hecs::Entity, time: f32) {
    world
        .get::<&mut Node>(entity)
        .expect("Added object must contain a Node component.")
        .activate(time);

    for child in children(world, entity) {
        activate_subtree(world, child, time);
    }
}

pub(crate) fn deactivate_subtree(world: &hecs::World, entity: hecs::Entity, time: f32) {
    world
        .get::<&mut Node>(entity)
        .expect("Removed object must contain a Node component.")
        .deactivate(time);

    for child in children(world, entity) {
        deactivate_subtree(world, child, time);
    }
}

pub(crate) fn children(world: &hecs::World, entity: hecs::Entity) -> Vec<hecs::Entity> {
    world
        .get::<&Node>(entity)
        .expect("Scene object must contain a Node component.")
        .children
        .clone()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use kinematic_macros::{Container, Object};

    use crate::core::{
        Scene,
        components::*,
        objects::*,
        types::{Color, vec2},
    };

    #[derive(Object, Container, hecs::Bundle)]
    struct TestContainer {
        #[trackable]
        transform: Transform,
        #[trackable]
        draw: Draw,
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
        let container = TestContainer::builder().build(&mut scene);
        let child = Rect::builder().build(&mut scene);

        assert!(
            scene
                .get_world()
                .get::<&Node>(container.get_id())
                .unwrap()
                .children
                .is_none()
        );

        container.add(&child);

        assert_eq!(
            scene
                .get_world()
                .get::<&Node>(container.get_id())
                .unwrap()
                .children
                .as_deref(),
            Some([child.get_id()].as_slice())
        );
    }

    #[test]
    fn non_group_containers_draw_their_children() {
        let mut scene = Scene::new();
        let container = TestContainer::builder()
            .position(vec2(4.0, 0.0))
            .build(&mut scene);
        let child = Rect::builder()
            .size(vec2(4.0, 4.0))
            .fill(Color::RED)
            .build(&mut scene);

        container.add(&child);
        scene.get_root().add(&container);

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
        let first = TestContainer::builder().build(&mut scene);
        let second = TestContainer::builder().build(&mut scene);
        let child = Rect::builder().build(&mut scene);

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
        let container = TestContainer::builder().build(&mut scene);
        let child = Rect::builder().build(&mut scene);

        container.add(&child);
        scene.get_root().add(&container);
        scene.wait(2.0);
        container.remove();

        let world = scene.get_world();
        assert_eq!(
            world.get::<&Node>(container.get_id()).unwrap().lifetime,
            [0.0, 2.0]
        );
        assert_eq!(
            world.get::<&Node>(child.get_id()).unwrap().lifetime,
            [0.0, 2.0]
        );
    }

    #[test]
    fn container_bounds_include_transformed_children() {
        let mut scene = Scene::new();
        let container = TestContainer::builder().build(&mut scene);
        let child = Rect::builder()
            .size(vec2(10.0, 20.0))
            .position(vec2(30.0, -10.0))
            .scale(vec2(2.0, 1.0))
            .rotation(std::f32::consts::FRAC_PI_2)
            .build(&mut scene);

        container.add(&child);

        let size = container.get_box();
        assert!((size.x - 20.0).abs() < 0.001);
        assert!((size.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn container_opacity_composites_the_subtree_once() {
        let mut scene = Scene::new();
        let first = Rect::builder()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .build(&mut scene);
        let second = Rect::builder()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .build(&mut scene);
        let container = TestContainer::builder().opacity(0.5).build(&mut scene);

        container.add(&first);
        container.add(&second);
        scene.get_root().add(&container);

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
    fn handlers_compose_global_values_without_skew() {
        let mut scene = Scene::new();
        let outer = Group::builder()
            .position(vec2(10.0, 20.0))
            .scale(vec2(2.0, 3.0))
            .rotation(std::f32::consts::FRAC_PI_2)
            .opacity(0.5)
            .build(&mut scene);
        let inner = Group::builder()
            .position(vec2(4.0, 5.0))
            .scale(vec2(5.0, 7.0))
            .rotation(0.25)
            .opacity(0.4)
            .build(&mut scene);
        let child = Rect::builder()
            .position(vec2(1.0, 2.0))
            .scale(vec2(0.5, 0.25))
            .rotation(0.125)
            .opacity(0.5)
            .build(&mut scene);

        inner.add(&child);
        outer.add(&inner);
        scene.get_root().add(&outer);

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
                .get_global_position()
                .abs_diff_eq(expected_position, 0.0001)
        );
        assert!(
            (child.get_global_rotation() - (std::f32::consts::FRAC_PI_2 + 0.375)).abs() < 0.0001
        );
        assert!(
            child
                .get_global_scale()
                .abs_diff_eq(vec2(5.0, 5.25), 0.0001)
        );
        assert!((child.get_global_opacity() - 0.1).abs() < 0.0001);
    }
}
