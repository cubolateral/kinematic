use crate::core::{
    Animator, Scheduling, Task, Tween,
    components::{Animation, Name, Node},
    objects::{
        Group, GroupHandler, Object, ObjectHandler, draw_entity, draw_entity_outline, pick_entity,
    },
    types::Vector2,
};

/// Shared ECS world used by scenes and their handlers.
pub type SceneWorld = std::rc::Rc<std::cell::RefCell<hecs::World>>;

/// Builds scene entities and schedules their animation timeline.
pub trait SceneBuilder {
    fn build(&mut self, scene: &mut Scene);
}

/// Runtime ECS scene containing render nodes and compiled animation tracks.
pub struct Scene {
    world: SceneWorld,
    root: hecs::Entity,
    animator_time: std::rc::Rc<std::cell::Cell<f32>>,
    animator: Animator,
}

impl Scene {
    /// Creates an empty scene.
    pub fn new() -> Self {
        let animator_time = std::rc::Rc::new(std::cell::Cell::new(0.0));
        let animator = Animator::with_scene_time(std::rc::Rc::clone(&animator_time));
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        let root = Group::spawn(
            std::rc::Rc::clone(&world),
            animator.handle(),
            Group::default(),
            Name::new("Root"),
        )
        .get_id();

        {
            let world = world.borrow();
            let mut node = world
                .get::<&mut Node>(root)
                .expect("Root group must contain a Node component.");

            node.is_root = true;
            node.activate(0.0);
        }

        Self {
            world,
            root,
            animator_time: std::rc::Rc::clone(&animator_time),
            animator,
        }
    }

    /// Evaluates every animation track at `time` and writes its value to the ECS world.
    ///
    /// This updates scene state only; rendering remains in [`Self::draw`].
    pub fn update(&self, time: f32) {
        let world = self.world.borrow_mut();

        for node in world.query::<&mut Node>().iter() {
            node.update(time);
        }

        for (entity, node, animation) in world
            .query::<(hecs::Entity, &Node, &mut Animation)>()
            .iter()
        {
            if !node.is_activated {
                continue;
            }

            for track in &mut animation.tracks {
                track.track.update(&world, entity, time);
            }
        }
    }

    /// Draws the root group tree using the scene state produced by [`Self::update`].
    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        let world = self.world.borrow();

        draw_entity(&world, self.root, canvas);
    }

    pub(crate) fn draw_outline(&self, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
        let world = self.world.borrow();

        draw_entity_outline(&world, self.root, entity, canvas);
    }

    pub(crate) fn pick(&self, point: Vector2) -> Option<hecs::Entity> {
        let world = self.world.borrow();

        pick_entity(&world, self.root, point)
    }

    /// Populates the scene and compiles the builder's animation timeline.
    ///
    /// Returns the duration of the resulting timeline.
    pub fn build(&mut self, builder: &mut dyn SceneBuilder) -> f32 {
        self.animator = Animator::with_scene_time(std::rc::Rc::clone(&self.animator_time));

        builder.build(self);

        let tasks = self.animator.tasks();
        Animator::get_duration_for_tasks(&tasks, self)
    }

    /// Adds a task to the current scene timeline.
    pub fn play(&mut self, task: Task) {
        self.animator.handle().play(task);
    }

    /// Plays a tween on the current scene timeline.
    pub fn tween<Object>(&mut self, tween: Tween<Object>) {
        self.play(tween.task());
    }

    /// Waits for the specified duration on the current scene timeline.
    pub fn wait(&mut self, duration: f32) {
        self.play(Task::Wait(duration));
    }

    /// Adds a sequential group to the current scene timeline.
    pub fn chain(&mut self, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Sequential, schedule, Task::Chain);
    }

    /// Adds a simultaneous group to the current scene timeline.
    pub fn all(&mut self, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Parallel, schedule, Task::All);
    }

    /// Repeats a sequential group on the current scene timeline.
    pub fn repeat(&mut self, repetitions: usize, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Sequential, schedule, |tasks| {
            Task::Repeat(repetitions, tasks)
        });
    }

    fn schedule_group(
        &mut self,
        scheduling: Scheduling,
        schedule: impl FnOnce(&mut Scene),
        build_task: impl FnOnce(Vec<Task>) -> Task,
    ) {
        let group = self.animator.group(scheduling);
        let previous = group.handle().activate();
        let parent = std::mem::replace(&mut self.animator, group);

        schedule(self);

        let group = std::mem::replace(&mut self.animator, parent);
        group.handle().restore(previous);
        self.play(build_task(group.tasks()));
    }

    #[doc(hidden)]
    pub fn spawn_object<T: Object>(&mut self, object: T, name: String) -> T::Handler {
        T::spawn(
            std::rc::Rc::clone(&self.world),
            self.animator.handle().active(),
            object,
            Name::new(name),
        )
    }

    /// Returns the root group that owns the scene's drawable object tree.
    pub fn get_root(&self) -> GroupHandler {
        Group::handler(
            std::rc::Rc::clone(&self.world),
            self.root,
            self.animator.handle().active(),
        )
    }

    /// Read-only access to the underlying ECS world.
    pub fn get_world(&self) -> std::cell::Ref<'_, hecs::World> {
        self.world.borrow()
    }

    /// Mutable access to the underlying ECS world.
    pub fn get_world_mut(&self) -> std::cell::RefMut<'_, hecs::World> {
        self.world.borrow_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        Easing,
        components::*,
        effects::{Effect, Unwrite, Write, WriteBy},
        objects::*,
        types::*,
    };

    use super::*;

    #[crate::scene]
    fn delayed_object_scene(scene: &mut Scene) {
        let circle = Circle::builder().build(scene);

        scene.wait(32.0);
        scene.get_root().add(&circle);
        scene.wait(1.0);
    }

    #[test]
    fn object_builder_sets_component_values_and_preserves_object_defaults() {
        let default = Text::default();
        let mut scene = Scene::new();
        let handler = Text::builder()
            .opacity(0.5)
            .position(vec2(10.0, 20.0))
            .text("Kinematic!".to_owned())
            .build(&mut scene);
        let world = scene.get_world();
        let draw = world.get::<&Draw>(handler.get_id()).unwrap();
        let transform = world.get::<&Transform>(handler.get_id()).unwrap();
        let shape = world.get::<&TextShape>(handler.get_id()).unwrap();

        assert_eq!(draw.opacity, 0.5);
        assert_eq!(transform.position, vec2(10.0, 20.0));
        assert_eq!(shape.text, "Kinematic!");
        assert!(std::ptr::fn_addr_eq(draw.on_draw, default.draw.on_draw,));
    }

    #[test]
    fn object_names_default_to_the_type_and_remain_mutable() {
        let mut scene = Scene::new();
        let circle = Circle::builder().build(&mut scene);
        let label = Text::builder().name("Caption").build(&mut scene);
        let root = scene.get_root();

        assert_eq!(circle.get_name(), "Circle");
        assert_eq!(label.get_name(), "Caption");
        assert_eq!(root.get_name(), "Root");

        circle.set_name("Primary Circle");

        assert_eq!(circle.get_name(), "Primary Circle");
        assert_eq!(
            scene
                .get_world()
                .get::<&Name>(circle.get_id())
                .unwrap()
                .get(),
            "Primary Circle"
        );
    }

    #[test]
    fn object_handler_exposes_trackable_fields_directly() {
        let mut scene = Scene::new();
        let text: TextHandler = Text::builder().build(&mut scene);
        let circle: CircleHandler = Circle::builder().build(&mut scene);

        let _ = text.opacity(0.25);
        let _ = circle.position(vec2(10.0, 20.0));
        let _ = circle.position_x(10.0);
        let _ = circle.fill(Color::RED);
        let _ = circle.fill_r(0.75);
        let world = scene.get_world();
        let draw = world.get::<&Draw>(text.get_id()).unwrap();

        assert_eq!(draw.opacity, 0.25);
    }

    #[test]
    fn object_handlers_animate_properties_and_generate_from_shortcuts() {
        let mut scene = Scene::new();
        let circle: CircleHandler = Circle::builder().build(&mut scene);
        scene.get_root().add(&circle);

        scene.play(
            circle
                .animate(Transform::position_property(), vec2(10.0, 20.0))
                .duration(1.0)
                .task(),
        );
        scene.play(circle.position_from(Vector2::ZERO, vec2(20.0, 30.0)).task());
        scene.play(circle.opacity_from(0.0, 1.0).duration(1.0).task());

        let tasks = scene.animator.tasks();
        Animator::get_duration_for_tasks(&tasks, &mut scene);
        scene.update(0.5);
        let world = scene.get_world();
        let transform = world.get::<&Transform>(circle.get_id()).unwrap();

        assert_eq!(transform.position, vec2(10.0, 15.0));
    }

    #[test]
    fn handler_tween_play_registers_in_scene_animator() {
        struct HandlerTweenScene;

        impl SceneBuilder for HandlerTweenScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = Circle::builder().fill(Color::RED).build(scene);
                scene.get_root().add(&circle);
                circle
                    .position_x(256.0)
                    .fill(Color::BLUE)
                    .duration(1.0)
                    .easing(Easing::InQuad)
                    .play();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut HandlerTweenScene), 1.0);

        scene.update(0.5);
        let world = scene.get_world();
        let mut query = world.query::<(&Transform, &Style)>();
        let circle = query.iter().next().unwrap();
        assert_eq!(circle.0.position.x, 64.0);
        assert_eq!(circle.1.fill, Color::new(0.75, 0.0, 0.25, 1.0));
    }

    #[test]
    fn handler_tween_merges_updates_to_the_same_track() {
        struct ComponentTweenScene;

        impl SceneBuilder for ComponentTweenScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = Circle::builder().build(scene);
                scene.get_root().add(&circle);
                circle
                    .position_x(128.0)
                    .position_y(64.0)
                    .duration(1.0)
                    .play();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut ComponentTweenScene), 1.0);

        scene.update(1.0);
        let world = scene.get_world();
        let mut query = world.query::<(&Node, &Transform)>();
        let (_, circle) = query.iter().find(|(node, _)| !node.is_root).unwrap();

        assert_eq!(circle.position, vec2(128.0, 64.0));
    }

    #[test]
    fn text_effect_configuration_is_applied_at_its_start() {
        struct TextEffectScene;

        impl SceneBuilder for TextEffectScene {
            fn build(&mut self, scene: &mut Scene) {
                let text = Text::builder().build(scene);
                scene.get_root().add(&text);

                Write::new(WriteBy::Letter)
                    .scale(0.25)
                    .outline_width(2.0)
                    .play(scene, &text);

                Unwrite::new(WriteBy::Word)
                    .scale(0.5)
                    .outline_width(3.0)
                    .play(scene, &text);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut TextEffectScene), 2.0);

        scene.update(0.0);
        {
            let world = scene.get_world();
            let mut query = world.query::<&TextShape>();
            let shape = query.iter().next().unwrap();

            assert_eq!(shape.write_progress, 0.0);
            assert_eq!(shape.write_scale, 0.25);
            assert!(!shape.write_by_word);
            assert_eq!(shape.write_outline_width, 2.0);
        }

        scene.update(1.0);
        let world = scene.get_world();
        let mut query = world.query::<&TextShape>();
        let shape = query.iter().next().unwrap();

        assert_eq!(shape.write_progress, 1.0);
        assert_eq!(shape.write_scale, 0.5);
        assert!(shape.write_by_word);
        assert_eq!(shape.write_outline_width, 3.0);
    }

    #[test]
    fn nested_groups_keep_scene_objects_on_their_local_timeline() {
        struct NestedGroupsScene;

        impl SceneBuilder for NestedGroupsScene {
            fn build(&mut self, scene: &mut Scene) {
                let root = Circle::builder().build(scene);
                scene.get_root().add(&root);
                scene.wait(1.0);

                scene.chain(|scene| {
                    let chain_object = Circle::builder().build(scene);
                    scene.get_root().add(&chain_object);
                    scene.wait(2.0);

                    scene.all(|scene| {
                        let parallel_object = Circle::builder().build(scene);
                        scene.get_root().add(&parallel_object);
                        scene.wait(4.0);

                        scene.chain(|scene| {
                            let nested_object = Circle::builder().build(scene);
                            scene.get_root().add(&nested_object);
                            scene.wait(1.0);

                            let nested_end_object = Circle::builder().build(scene);
                            scene.get_root().add(&nested_end_object);
                        });

                        scene.repeat(2, |scene| {
                            let repeated_object = Circle::builder().build(scene);
                            scene.get_root().add(&repeated_object);
                            scene.wait(0.5);

                            let repeated_end_object = Circle::builder().build(scene);
                            scene.get_root().add(&repeated_end_object);
                        });

                        let parallel_end_object = Circle::builder().build(scene);
                        scene.get_root().add(&parallel_end_object);
                    });

                    scene.wait(1.0);
                    let chain_end_object = Circle::builder().build(scene);
                    scene.get_root().add(&chain_end_object);
                });
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut NestedGroupsScene), 8.0);

        let world = scene.get_world();
        let mut lifetimes: Vec<_> = world
            .query::<&Node>()
            .iter()
            .filter(|node| !node.is_root)
            .map(|node| node.lifetime)
            .collect();
        lifetimes.sort_by(|left, right| left[0].total_cmp(&right[0]));

        assert_eq!(
            lifetimes,
            vec![
                [0.0, f32::INFINITY],
                [1.0, f32::INFINITY],
                [3.0, f32::INFINITY],
                [3.0, f32::INFINITY],
                [3.0, f32::INFINITY],
                [3.5, f32::INFINITY],
                [4.0, f32::INFINITY],
                [7.0, f32::INFINITY],
                [8.0, f32::INFINITY],
            ]
        );
    }

    #[test]
    fn draw_renders_objects_directly_into_the_supplied_skia_canvas() {
        let mut scene = Scene::new();
        let rect = Rect::builder()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .opacity(0.5)
            .position(vec2(4.0, 0.0))
            .build(&mut scene);
        scene.get_root().add(&rect);

        let image_info = skia_safe::ImageInfo::new(
            (32, 32),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::colors::TRANSPARENT);
        canvas.translate((16.0, 16.0));

        scene.draw(canvas);

        let pixels = surface.peek_pixels().unwrap();
        let inside = pixels.get_color((20, 16));
        let outside = pixels.get_color((12, 16));

        assert_eq!(inside.r(), 255);
        assert!((127..=128).contains(&inside.a()));
        assert_eq!(outside.a(), 0);
    }

    #[test]
    fn object_lifetime_follows_the_animator_time_and_reactivates_when_seeking_back() {
        struct LifetimeScene;

        impl SceneBuilder for LifetimeScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = Circle::builder().build(scene);
                let rect = Rect::builder().build(scene);
                scene.get_root().add(&circle);

                scene.wait(1.0);
                scene.get_root().add(&rect);
                scene.wait(2.0);
                scene.get_root().remove(&circle);
            }
        }

        let mut scene = Scene::new();
        let duration = scene.build(&mut LifetimeScene);

        assert_eq!(duration, 3.0);

        let lifetimes = || {
            let world = scene.get_world();
            let mut lifetimes: Vec<_> = world
                .query::<&Node>()
                .iter()
                .filter(|node| !node.is_root)
                .map(|node| node.lifetime)
                .collect();
            lifetimes.sort_by(|left, right| left[0].total_cmp(&right[0]));
            lifetimes
        };

        assert_eq!(lifetimes(), vec![[0.0, 3.0], [1.0, f32::INFINITY]]);

        let active_count = |time| {
            scene.update(time);
            scene
                .get_world()
                .query::<&Node>()
                .iter()
                .filter(|node| !node.is_root && node.is_activated)
                .count()
        };

        assert_eq!(active_count(0.0), 1);
        assert_eq!(active_count(1.0), 2);
        assert_eq!(active_count(3.0), 1);
        assert_eq!(active_count(0.5), 1);
    }

    #[test]
    fn object_created_after_parallel_work_starts_at_the_latest_group_time() {
        struct ParallelLifetimeScene;

        impl SceneBuilder for ParallelLifetimeScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = Circle::builder().build(scene);

                scene.all(|scene| {
                    scene.wait(5.0);
                    scene.get_root().add(&circle);
                    scene.wait(2.0);
                });
                scene.wait(1.0);
            }
        }

        let mut scene = Scene::new();

        assert_eq!(scene.build(&mut ParallelLifetimeScene), 6.0);

        let world = scene.get_world();
        let mut query = world.query::<&Node>();
        let node = query.iter().find(|node| !node.is_root).unwrap();

        assert_eq!(node.lifetime, [5.0, f32::INFINITY]);
        assert!(!node.is_activated);
    }

    #[test]
    fn scene_macro_preserves_the_create_time_as_the_node_start() {
        let mut scene = Scene::new();

        assert_eq!(scene.build(delayed_object_scene().as_mut()), 33.0);

        scene.update(31.0);
        {
            let world = scene.get_world();
            let mut query = world.query::<&Node>();
            let node = query.iter().find(|node| !node.is_root).unwrap();

            assert_eq!(node.lifetime, [32.0, f32::INFINITY]);
            assert!(!node.is_activated);
        }

        scene.update(32.0);
        let world = scene.get_world();
        let mut query = world.query::<&Node>();

        assert!(
            query
                .iter()
                .find(|node| !node.is_root)
                .unwrap()
                .is_activated
        );
    }
}
