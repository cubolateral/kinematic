use crate::core::{
    Animator, Scheduling, Task, Tween,
    components::{Animation, Draw, Node, Transform},
    objects::{Object, ObjectHandler},
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
    animator_time: std::rc::Rc<std::cell::Cell<f32>>,
    animator: Animator,
}

impl Scene {
    /// Creates an empty scene.
    pub fn new() -> Self {
        let animator_time = std::rc::Rc::new(std::cell::Cell::new(0.0));

        Self {
            world: std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new())),
            animator_time: std::rc::Rc::clone(&animator_time),
            animator: Animator::with_scene_time(animator_time),
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

    /// Draws each entity using the scene state produced by [`Self::update`].
    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        let world = self.world.borrow();

        for (entity, node, draw, transform) in world
            .query::<(hecs::Entity, &Node, &Draw, &Transform)>()
            .iter()
        {
            if !node.is_activated {
                continue;
            }

            if draw.opacity <= 0.0 {
                continue;
            }

            let save_count = canvas.save();

            canvas.translate((transform.position.x, transform.position.y));
            canvas.rotate(transform.rotation.to_degrees(), None);
            canvas.scale((transform.scale.x, transform.scale.y));

            (draw.on_draw)(&world, entity, canvas);

            canvas.restore_to_count(save_count);
        }
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

    /// Starts building an inactive object connected to this scene.
    ///
    /// Calling the generated builder's `build` method spawns the inactive object
    /// and returns its typed handler. Use [`Self::add`] to begin its lifetime.
    ///
    /// ```
    /// use kinematic::prelude::*;
    ///
    /// let mut scene = Scene::new();
    /// let text: TextHandler = scene
    ///     .create::<Text>()
    ///     .opacity(1.0)
    ///     .position(Vector2::ZERO)
    ///     .build();
    /// scene.add(&text);
    /// let _ = text.position_x(100.0);
    /// ```
    pub fn create<T: Object>(&mut self) -> T::Builder {
        T::builder(
            std::rc::Rc::clone(&self.world),
            self.animator.handle().active(),
        )
    }

    /// Begins an object's lifetime at the animator's current timeline position.
    pub fn add(&mut self, handler: &impl ObjectHandler) {
        let world = self.world.borrow();
        let mut node = world
            .get::<&mut Node>(handler.entity())
            .expect("Added object must belong to this scene.");

        node.activate(self.animator.handle().time());
    }

    /// Ends an object's lifetime at the animator's current timeline position.
    pub fn destroy(&mut self, handler: impl ObjectHandler) {
        let world = self.world.borrow();
        let mut node = world
            .get::<&mut Node>(handler.entity())
            .expect("Destroyed object must belong to this scene.");

        node.deactivate(self.animator.handle().time());
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
        let circle = scene.create::<Circle>().build();

        scene.wait(32.0);
        scene.add(&circle);
        scene.wait(1.0);
    }

    #[test]
    fn object_builder_sets_component_values_and_preserves_object_defaults() {
        let default = Text::default();
        let mut scene = Scene::new();
        let handler = scene
            .create::<Text>()
            .opacity(0.5)
            .position(vec2(10.0, 20.0))
            .text("Kinematic!".to_owned())
            .build();
        let world = scene.get_world();
        let draw = world.get::<&Draw>(handler.entity()).unwrap();
        let transform = world.get::<&Transform>(handler.entity()).unwrap();
        let shape = world.get::<&TextShape>(handler.entity()).unwrap();

        assert_eq!(draw.opacity, 0.5);
        assert_eq!(transform.position, vec2(10.0, 20.0));
        assert_eq!(shape.text, "Kinematic!");
        assert!(std::ptr::fn_addr_eq(draw.on_draw, default.draw.on_draw,));
    }

    #[test]
    fn object_handler_exposes_trackable_fields_directly() {
        let mut scene = Scene::new();
        let text: TextHandler = scene.create::<Text>().build();
        let circle: CircleHandler = scene.create::<Circle>().build();

        let _ = text.opacity(0.25);
        let _ = circle.position(vec2(10.0, 20.0));
        let _ = circle.position_x(10.0);
        let _ = circle.fill(Color::RED);
        let _ = circle.fill_r(0.75);
        let world = scene.get_world();
        let mut query = world.query::<&Draw>();

        assert_eq!(query.iter().next().unwrap().opacity, 0.25);
    }

    #[test]
    fn object_handlers_animate_properties_and_generate_from_shortcuts() {
        let mut scene = Scene::new();
        let circle: CircleHandler = scene.create::<Circle>().build();

        scene.play(
            circle
                .animate(Transform::position_property(), vec2(10.0, 20.0))
                .duration(1.0)
                .task(),
        );
        scene.play(circle.position_from(Vector2::ZERO, vec2(20.0, 30.0)).task());
        scene.play(circle.opacity_from(0.0, 1.0).duration(1.0).task());

        scene.update(0.5);
        let world = scene.get_world();
        let transform = world.get::<&Transform>(circle.entity()).unwrap();

        assert_eq!(transform.position, vec2(5.0, 10.0));
    }

    #[test]
    fn handler_tween_play_registers_in_scene_animator() {
        struct HandlerTweenScene;

        impl SceneBuilder for HandlerTweenScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = scene.create::<Circle>().fill(Color::RED).build();
                scene.add(&circle);
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
                let circle = scene.create::<Circle>().build();
                scene.add(&circle);
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
        let mut query = world.query::<&Transform>();
        let circle = query.iter().next().unwrap();
        assert_eq!(circle.position, vec2(128.0, 64.0));
    }

    #[test]
    fn text_effect_configuration_is_applied_at_its_start() {
        struct TextEffectScene;

        impl SceneBuilder for TextEffectScene {
            fn build(&mut self, scene: &mut Scene) {
                let text = scene.create::<Text>().build();
                scene.add(&text);

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
                let root = scene.create::<Circle>().build();
                scene.add(&root);
                scene.wait(1.0);

                scene.chain(|scene| {
                    let chain_object = scene.create::<Circle>().build();
                    scene.add(&chain_object);
                    scene.wait(2.0);

                    scene.all(|scene| {
                        let parallel_object = scene.create::<Circle>().build();
                        scene.add(&parallel_object);
                        scene.wait(4.0);

                        scene.chain(|scene| {
                            let nested_object = scene.create::<Circle>().build();
                            scene.add(&nested_object);
                            scene.wait(1.0);

                            let nested_end_object = scene.create::<Circle>().build();
                            scene.add(&nested_end_object);
                        });

                        scene.repeat(2, |scene| {
                            let repeated_object = scene.create::<Circle>().build();
                            scene.add(&repeated_object);
                            scene.wait(0.5);

                            let repeated_end_object = scene.create::<Circle>().build();
                            scene.add(&repeated_end_object);
                        });

                        let parallel_end_object = scene.create::<Circle>().build();
                        scene.add(&parallel_end_object);
                    });

                    scene.wait(1.0);
                    let chain_end_object = scene.create::<Circle>().build();
                    scene.add(&chain_end_object);
                });
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut NestedGroupsScene), 8.0);

        let world = scene.get_world();
        let mut lifetimes: Vec<_> = world
            .query::<&Node>()
            .iter()
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
        let rect = scene
            .create::<Rect>()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .opacity(0.5)
            .position(vec2(4.0, 0.0))
            .build();
        scene.add(&rect);

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
                let circle = scene.create::<Circle>().build();
                let rect = scene.create::<Rect>().build();
                scene.add(&circle);

                scene.wait(1.0);
                scene.add(&rect);
                scene.wait(2.0);
                scene.destroy(circle);
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
                .filter(|node| node.is_activated)
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
                let circle = scene.create::<Circle>().build();

                scene.all(|scene| {
                    scene.wait(5.0);
                    scene.add(&circle);
                    scene.wait(2.0);
                });
                scene.wait(1.0);
            }
        }

        let mut scene = Scene::new();

        assert_eq!(scene.build(&mut ParallelLifetimeScene), 6.0);

        let world = scene.get_world();
        let mut query = world.query::<&Node>();
        let node = query.iter().next().unwrap();

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
            let node = query.iter().next().unwrap();

            assert_eq!(node.lifetime, [32.0, f32::INFINITY]);
            assert!(!node.is_activated);
        }

        scene.update(32.0);
        let world = scene.get_world();
        let mut query = world.query::<&Node>();

        assert!(query.iter().next().unwrap().is_activated);
    }
}
