use crate::core::{
    Animator, Scheduling, Task, TrackableInfo, Tween,
    components::{Animation, Draw2D, Inspection, Name, Node, View},
    objects::{
        Canvas2D, Canvas2DHandler, Canvas3D, Canvas3DHandler, Object, ObjectHandler, RootHandler,
        camera_matrix2d, canvas_2d, canvas_3d, children_by_z_index, draw_entity,
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
pub(crate) struct SceneIdentity(pub(crate) u64);

fn root_trackables(_: &hecs::World, _: hecs::Entity) -> &'static [TrackableInfo] {
    static TRACKABLES: [TrackableInfo; 1] = [View::INFO];
    &TRACKABLES
}

pub struct Scene {
    name: &'static str,
    world: SceneWorld,
    root: hecs::Entity,
    world_2d: hecs::Entity,
    world_3d: hecs::Entity,
    animator_time: std::rc::Rc<std::cell::Cell<f32>>,
    animator: Animator,
}

impl Scene {
    /// Creates a scene with 1920x1080 default worlds.
    pub fn new() -> Self {
        Self::new_with_resolution((1920, 1080))
    }

    /// Creates a scene whose built-in worlds use `resolution`.
    pub fn new_with_resolution(resolution: (u32, u32)) -> Self {
        Self::new_named("Scene", resolution)
    }

    #[doc(hidden)]
    pub fn new_named(name: &'static str, resolution: (u32, u32)) -> Self {
        let animator_time = std::rc::Rc::new(std::cell::Cell::new(0.0));
        let animator = Animator::with_scene_time(std::rc::Rc::clone(&animator_time));
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        static NEXT_SCENE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_SCENE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = world.borrow_mut().spawn(
            hecs::EntityBuilder::new()
                .add(SceneIdentity(id))
                .add(Animation::default())
                .add(Draw2D::default())
                .add(Inspection {
                    object_name: "Root",
                    get: root_trackables,
                })
                .add(Name::new("Root"))
                .add(Node::default())
                .add(View::default())
                .build(),
        );

        {
            let world = world.borrow();
            let mut node = world
                .get::<&mut Node>(root)
                .expect("Root must contain a Node component.");

            node.is_root = true;
            node.activate(0.0);
        }

        let mut scene = Self {
            name,
            world,
            root,
            world_2d: root,
            world_3d: root,
            animator_time: std::rc::Rc::clone(&animator_time),
            animator,
        };

        let world_2d = canvas_2d()
            .name("World 2D")
            .resolution(resolution)
            .build(&mut scene);
        let world_3d = canvas_3d()
            .name("World 3D")
            .resolution(resolution)
            .build(&mut scene);
        scene.get_root().add(&world_2d);
        scene.get_root().add(&world_3d);
        scene.world_2d = world_2d.get_id();
        scene.world_3d = world_3d.get_id();
        scene
    }

    pub(crate) fn get_view(&self) -> crate::core::objects::CanvasTexture {
        if self.get_root().is_view_2d() {
            self.get_world_2d().get_texture()
        } else {
            self.get_world_3d().get_texture()
        }
    }

    /// Restores signal overrides, updates nodes and tracks, then evaluates active signals.
    ///
    /// This updates scene state only; rendering remains in [`Self::draw`].
    pub fn update(&self, time: f32) {
        let signals = self.animator.handle().signals();
        let world = self.world.borrow_mut();

        signals.restore_overrides(&world);

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
        drop(world);

        signals.evaluate(&self.world, time);
    }

    /// Draws the built-in 2D world without applying its output-size translation.
    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        let world = self.world.borrow();
        let save_count = canvas.save();

        if let Some(view) =
            camera_matrix2d(&world, self.world_2d).and_then(|camera| camera.invert())
        {
            canvas.concat(&view);
        }

        for child in children_by_z_index(&world, self.world_2d) {
            draw_entity(&world, child, canvas);
        }
        canvas.restore_to_count(save_count);
    }

    pub(crate) fn draw_outline(
        &self,
        entity: hecs::Entity,
        canvas: &skia_safe::Canvas,
        thickness: f32,
    ) {
        let output = self.get_view();
        let world = self.world.borrow();
        if self.output_is_active_2d(&world, output.entity) {
            crate::core::objects::draw_canvas_outline2d(
                &world,
                output.entity,
                entity,
                thickness,
                canvas,
            );
        }
    }

    pub(crate) fn pick(&self, point: Vector2) -> Option<hecs::Entity> {
        let output = self.get_view();
        let world = self.world.borrow();
        if self.output_is_active_2d(&world, output.entity) {
            crate::core::objects::pick_canvas2d(&world, output.entity, point)
        } else {
            None
        }
    }

    fn output_is_active_2d(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&Node>(entity).is_ok_and(|n| n.is_activated)
            && world
                .get::<&crate::core::objects::CanvasSettings>(entity)
                .is_ok_and(|s| s.dimension == crate::core::objects::CanvasDimension::Two)
    }

    /// Populates the scene and compiles the builder's animation timeline.
    ///
    /// Returns the duration of the resulting timeline.
    pub fn build(&mut self, builder: &mut dyn SceneBuilder) -> f32 {
        self.animator = Animator::with_scene_time(std::rc::Rc::clone(&self.animator_time));

        builder.build(self);

        self.animator.take_schedule().compile(self)
    }

    pub(crate) fn get_duration(&self) -> f32 {
        self.animator.duration()
    }

    pub(crate) fn get_name(&self) -> &'static str {
        self.name
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

    /// Delays a sequential group by the specified number of timeline seconds.
    pub fn delay(&mut self, duration: f32, schedule: impl FnOnce(&mut Scene)) {
        self.chain(|scene| {
            scene.wait(duration);
            schedule(scene);
        });
    }

    /// Adds a sequential group to the current scene timeline.
    pub fn chain(&mut self, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Sequential, false, schedule);
    }

    /// Adds a simultaneous group to the current scene timeline.
    pub fn all(&mut self, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Parallel, false, schedule);
    }

    /// Loops one finite animation cycle without advancing the scene timeline.
    ///
    /// The closure runs once. Only its animations repeat, until the scene or
    /// object's lifetime ends. Use a `for` loop for finite repetition and waits
    /// or other finite animations to establish the scene duration.
    /// Cycles cannot create, attach, remove objects, or contain another repeat.
    pub fn repeat(&mut self, schedule: impl FnOnce(&mut Scene)) {
        let values = {
            let world = self.world.borrow();
            let mut values = Vec::new();
            for (entity, inspection) in world.query::<(hecs::Entity, &Inspection)>().iter() {
                for component in (inspection.get)(&world, entity) {
                    for info in (component.get)() {
                        values.push((entity, info, (info.get)(&world, entity)));
                    }
                }
            }
            values
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.schedule_group(Scheduling::Sequential, true, schedule);
        }));
        // A background cycle does not advance the outer construction values.
        let world = self.world.borrow();
        for (entity, info, value) in values {
            (info.set)(&world, entity, value);
        }
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }

    fn schedule_group(
        &mut self,
        scheduling: Scheduling,
        repeating: bool,
        schedule: impl FnOnce(&mut Scene),
    ) {
        let group = self.animator.group(scheduling, repeating);
        let previous = group.handle().activate();
        let parent = std::mem::replace(&mut self.animator, group);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| schedule(self)));
        let group = std::mem::replace(&mut self.animator, parent);
        group.handle().restore(previous);
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
        let schedule = group.take_schedule();
        self.animator.handle().schedule(if repeating {
            schedule.repeated()
        } else {
            schedule
        });
    }

    #[doc(hidden)]
    pub fn spawn_object<T: Object>(&mut self, object: T, name: impl Into<String>) -> T::Handler {
        T::spawn(
            std::rc::Rc::clone(&self.world),
            self.animator.handle().active(),
            object,
            Name::new(name),
        )
    }

    /// Returns the built-in 2D canvas.
    pub fn get_world_2d(&self) -> Canvas2DHandler {
        Canvas2D::handler(
            std::rc::Rc::clone(&self.world),
            self.world_2d,
            self.animator.handle().active(),
        )
    }

    /// Returns the built-in 3D canvas.
    pub fn get_world_3d(&self) -> Canvas3DHandler {
        Canvas3D::handler(
            std::rc::Rc::clone(&self.world),
            self.world_3d,
            self.animator.handle().active(),
        )
    }

    /// Attaches an additional 2D canvas to this scene.
    pub fn add_canvas_2d(&self, canvas: &Canvas2DHandler) {
        self.get_root().add(canvas);
    }

    /// Attaches an additional 3D canvas to this scene.
    pub fn add_canvas_3d(&self, canvas: &Canvas3DHandler) {
        self.get_root().add(canvas);
    }

    /// Returns the internal root that owns the scene's canvases.
    pub fn get_root(&self) -> RootHandler {
        RootHandler {
            world: std::rc::Rc::clone(&self.world),
            entity: self.root,
            animator: self.animator.handle().active(),
        }
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
        effects::{Effect, creation, uncreation},
        objects::*,
        types::*,
    };

    use super::*;

    #[test]
    fn signal_overrides_a_tween_after_tracks_are_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        object.signal(|handler| handler.set_position(vec2(25.0, 0.0)));
        object
            .position_x(100.0)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();
        scene.animator.take_schedule().compile(&scene);

        scene.update(1.0);

        assert_eq!(object.get_position(), vec2(25.0, 0.0));
    }

    #[test]
    fn signal_interval_supports_forward_and_backward_seeks() {
        let mut scene = Scene::new();
        let object = circle().position(vec2(3.0, 0.0)).build(&mut scene);
        scene.get_world_2d().add(&object);
        scene.wait(1.0);
        let signaled = object.clone();
        let signal = object.signal(move |_| signaled.set_position(vec2(9.0, 0.0)));
        scene.wait(1.0);
        signal.stop();
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.5);
        assert_eq!(object.get_position(), vec2(3.0, 0.0));
        scene.update(1.5);
        assert_eq!(object.get_position(), vec2(9.0, 0.0));
        scene.update(2.0);
        assert_eq!(object.get_position(), vec2(3.0, 0.0));
        scene.update(1.5);
        assert_eq!(object.get_position(), vec2(9.0, 0.0));
        scene.update(0.5);
        assert_eq!(object.get_position(), vec2(3.0, 0.0));
    }

    #[test]
    fn stopping_a_signal_uses_the_current_animator_time() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        let signaled = object.clone();
        let signal = object.signal(move |_| signaled.set_position(vec2(4.0, 0.0)));
        scene.wait(1.0);
        signal.stop();
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.999);
        assert_eq!(object.get_position(), vec2(4.0, 0.0));
        scene.update(1.0);
        assert_eq!(object.get_position(), Vector2::ZERO);
    }

    #[test]
    fn multiple_signals_save_a_property_original_only_once() {
        let mut scene = Scene::new();
        let object = circle().position(vec2(3.0, 0.0)).build(&mut scene);
        scene.get_world_2d().add(&object);
        let first = object.clone();
        let first_signal = object.signal(move |_| first.set_position(vec2(10.0, 0.0)));
        let second = object.clone();
        let second_signal = object.signal(move |_| second.set_position(vec2(20.0, 0.0)));
        scene.wait(1.0);
        first_signal.stop();
        second_signal.stop();
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.5);
        assert_eq!(object.get_position(), vec2(20.0, 0.0));
        scene.update(1.0);
        assert_eq!(object.get_position(), vec2(3.0, 0.0));
    }

    #[test]
    fn signal_does_not_run_while_its_target_is_inactive() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let callback_calls = std::rc::Rc::clone(&calls);
        object.signal(move |_| callback_calls.set(callback_calls.get() + 1));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);

        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn signal_can_read_another_handler_without_borrowing_the_scene_world() {
        let mut scene = Scene::new();
        let tracked = circle().position(vec2(7.0, 8.0)).build(&mut scene);
        scene.get_world_2d().add(&tracked);
        let canvas = scene.get_world_2d();
        let signaled_canvas = canvas.clone();
        let tracked = tracked.clone();
        canvas.signal(move |_| signaled_canvas.set_camera_position(tracked.get_global_position()));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);

        assert_eq!(canvas.get_camera_position(), vec2(7.0, 8.0));
    }

    #[test]
    fn signal_does_not_advance_scene_duration() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        object.signal(|_| {});

        assert_eq!(scene.animator.take_schedule().compile(&scene), 0.0);
    }

    #[test]
    #[should_panic(expected = "create, attach, or remove objects outside repeat")]
    fn signal_is_rejected_inside_repeat() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);

        scene.repeat(|_| {
            object.signal(|_| {});
        });
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_create_a_tween_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        let signaled = object.clone();
        object.signal(move |_| {
            let _ = signaled.position(vec2(1.0, 2.0));
        });
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_remove_an_object_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        let signaled = object.clone();
        object.signal(move |_| signaled.remove());
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_attach_an_object_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let canvas = scene.get_world_2d();
        let child = circle().build(&mut scene);
        let parent = canvas.clone();
        canvas.signal(move |_| parent.add(&child));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_play_an_existing_tween_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        let queued = object.position(vec2(1.0, 2.0));
        object.set_position(Vector2::ZERO);
        let mut queued = Some(queued);
        object.signal(move |_| queued.take().unwrap().play());
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    fn finite_for_loops_rebuild_relative_targets() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        for _ in 0..3 {
            object
                .position_x_by(10.0)
                .duration(1.0)
                .easing(Easing::Linear)
                .play();
        }
        assert_eq!(scene.animator.take_schedule().compile(&scene), 3.0);
        scene.update(2.5);
        assert_eq!(object.get(Transform2D::position_property()).x, 25.0);
    }

    #[test]
    fn container_visibility_hides_its_2d_subtree() {
        let mut scene = Scene::new();
        let group = group_2d().visibility(false).build(&mut scene);
        let object = rect().size(vec2(10.0, 10.0)).build(&mut scene);
        group.add(&object);
        scene.get_world_2d().add(&group);

        assert_eq!(scene.pick(Vector2::ZERO), None);
        group.visibility(true).immediate();
        assert_eq!(scene.pick(Vector2::ZERO), Some(object.get_id()));
    }

    #[test]
    fn delay_offsets_tweens_tasks_and_scopes() {
        let mut scene = Scene::new();
        let tween_object = circle().build(&mut scene);
        let task_object = circle().build(&mut scene);
        let scope_object = circle().build(&mut scene);
        scene.get_world_2d().add(&tween_object);
        scene.get_world_2d().add(&task_object);
        scene.get_world_2d().add(&scope_object);

        scene.all(|scene| {
            tween_object
                .position_x(100.0)
                .delay(0.5)
                .duration(1.0)
                .easing(Easing::Linear)
                .play();
            scene.play(
                task_object
                    .position_x(100.0)
                    .duration(1.0)
                    .easing(Easing::Linear)
                    .task()
                    .delay(1.0),
            );
            scene.delay(1.5, |_| {
                scope_object
                    .position_x(100.0)
                    .duration(1.0)
                    .easing(Easing::Linear)
                    .play();
            });
        });

        assert_eq!(scene.animator.take_schedule().compile(&scene), 2.5);
        for (time, tween_x, task_x, scope_x) in [
            (0.25, 0.0, 0.0, 0.0),
            (0.75, 25.0, 0.0, 0.0),
            (1.25, 75.0, 25.0, 0.0),
            (1.75, 100.0, 75.0, 25.0),
        ] {
            scene.update(time);
            assert_eq!(
                tween_object.get(Transform2D::position_property()).x,
                tween_x
            );
            assert_eq!(task_object.get(Transform2D::position_property()).x, task_x);
            assert_eq!(
                scope_object.get(Transform2D::position_property()).x,
                scope_x
            );
        }
    }

    #[test]
    fn task_repeat_and_groups_share_the_same_schedule() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        let cycle = Task::Repeat(vec![Task::All(vec![
            object
                .opacity_from(0.0, 1.0)
                .duration(1.0)
                .easing(Easing::Linear)
                .task(),
            Task::Chain(vec![Task::Wait(1.0)]),
        ])]);
        scene.play(Task::Chain(vec![Task::Wait(2.0), cycle, Task::Wait(3.0)]));
        assert_eq!(scene.animator.take_schedule().compile(&scene), 5.0);
        scene.update(4.5);
        assert_eq!(object.get(Draw2D::opacity_property()), 0.5);
    }

    #[test]
    fn repeat_is_seekable_and_does_not_advance_the_outer_timeline() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        scene.wait(3.0);
        scene.all(|scene| {
            scene.repeat(|scene| {
                scene.chain(|_| {
                    object
                        .position_y(10.0)
                        .duration(1.0)
                        .easing(Easing::Linear)
                        .play();
                    object
                        .position_y(0.0)
                        .duration(1.0)
                        .easing(Easing::Linear)
                        .play();
                });
            });
            object
                .opacity(0.0)
                .duration(4.0)
                .easing(Easing::Linear)
                .play();
        });
        scene.wait(3.0);
        assert_eq!(scene.animator.take_schedule().compile(&scene), 10.0);
        assert_eq!(scene.get_duration(), 10.0);
        for (time, y) in [
            (3.5, 5.0),
            (6.5, 5.0),
            (3.0, 0.0),
            (4.0, 10.0),
            (101.5, 5.0),
            (1.0, 0.0),
        ] {
            scene.update(time);
            assert_eq!(object.get(Transform2D::position_property()).y, y);
        }
        scene.update(5.0);
        assert_eq!(object.get(Draw2D::opacity_property()), 0.5);
        let world = scene.get_world();
        let animation = world.get::<&Animation>(object.get_id()).unwrap();
        let track = animation
            .tracks
            .iter()
            .find(|track| track.track.repeat.is_some())
            .unwrap();
        assert!(track.track.keyframes.len() <= 5);
    }

    #[test]
    fn repeat_restores_construction_values_and_holds_at_the_start_of_each_cycle() {
        let mut scene = Scene::new();
        let object = circle().position(vec2(0.0, 2.0)).build(&mut scene);
        scene.get_world_2d().add(&object);
        object
            .position_y(4.0)
            .duration(1.0)
            .easing(Easing::Linear)
            .play();
        scene.repeat(|scene| {
            scene.wait(1.0);
            object
                .position_y(8.0)
                .duration(1.0)
                .easing(Easing::Linear)
                .play();
        });
        assert_eq!(object.get(Transform2D::position_property()).y, 4.0);
        assert_eq!(scene.get_duration(), 1.0);
        scene.wait(6.0);
        scene.animator.take_schedule().compile(&scene);
        for (time, y) in [(0.5, 3.0), (2.5, 6.0), (3.5, 4.0), (1.5, 4.0), (6.5, 6.0)] {
            scene.update(time);
            assert_eq!(object.get(Transform2D::position_property()).y, y);
        }
    }

    #[test]
    fn repeated_axis_rotation_preserves_signed_multiple_turns() {
        let mut scene = Scene::new();
        let object = cube().build(&mut scene);
        scene.get_world_3d().add(&object);
        scene.repeat(|_| {
            object
                .rotate_y(-2.0 * std::f32::consts::TAU)
                .duration(4.0)
                .easing(Easing::Linear)
                .play();
        });
        scene.wait(12.0);
        scene.animator.take_schedule().compile(&scene);
        for time in [0.5, 4.5, 100.5, 0.5] {
            scene.update(time);
            let direction = object.get(Transform3D::rotation_property()) * Vector3::X;
            assert!(direction.abs_diff_eq(Vector3::Z, 1e-5));
        }
        scene.update(4.0);
        let direction = object.get(Transform3D::rotation_property()) * Vector3::X;
        assert!(direction.abs_diff_eq(Vector3::X, 1e-5));
    }

    #[test]
    fn nested_parallel_branches_compile_in_property_time_order() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.get_world_2d().add(&object);
        scene.all(|scene| {
            scene.chain(|scene| {
                scene.wait(2.0);
                scene.all(|scene| {
                    scene.all(|_| {
                        object
                            .position_from(vec2(10.0, 0.0), vec2(20.0, 0.0))
                            .duration(1.0)
                            .easing(Easing::Linear)
                            .play();
                    });
                });
            });
            object
                .position_from(Vector2::ZERO, vec2(10.0, 0.0))
                .duration(1.0)
                .easing(Easing::Linear)
                .play();
        });
        assert_eq!(scene.animator.take_schedule().compile(&scene), 3.0);
        for (time, x) in [(2.5, 15.0), (0.5, 5.0), (1.5, 10.0)] {
            scene.update(time);
            assert_eq!(object.get(Transform2D::position_property()).x, x);
        }
    }

    #[test]
    #[should_panic(expected = "Repeat overlaps another animation")]
    fn repeat_rejects_competing_property_animation() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.repeat(|_| {
            object.opacity(0.0).play();
        });
        scene.wait(4.0);
        object.opacity(0.5).play();
        scene.animator.take_schedule().compile(&scene);
    }

    #[test]
    #[should_panic(expected = "Repeat overlaps another animation")]
    fn repeat_rejects_another_cycle_on_the_same_property() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.repeat(|_| {
            object.opacity(0.0).play();
        });
        scene.repeat(|_| {
            object.opacity(0.5).play();
        });
        scene.animator.take_schedule().compile(&scene);
    }

    #[test]
    fn rejected_repeat_restores_the_active_scope_and_values() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.wait(2.0);
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scene.repeat(|scene| {
                object.opacity(0.0).play();
                scene.repeat(|_| {});
            });
        }));
        assert!(error.is_err());
        assert_eq!(object.get(Draw2D::opacity_property()), 1.0);
        assert_eq!(scene.get_duration(), 2.0);
        object.opacity(0.5).play();
        assert_eq!(scene.get_duration(), 3.0);
    }

    #[test]
    #[should_panic(expected = "finite, positive duration")]
    fn repeat_rejects_zero_duration_cycles() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.repeat(|_| {
            object.opacity(0.0).immediate();
        });
    }

    #[test]
    #[should_panic(expected = "create, attach, or remove objects outside repeat")]
    fn repeat_rejects_lifetime_changes_through_existing_handlers() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.repeat(|_| {
            object.remove();
        });
    }

    #[test]
    fn scene_creates_project_sized_worlds_and_defaults_to_world_2d() {
        let scene = Scene::new_with_resolution((1280, 720));
        let world_2d = scene.get_world_2d();
        let world_3d = scene.get_world_3d();
        let world = scene.get_world();

        assert_eq!(world_2d.get_name(), "World 2D");
        assert_eq!(world_3d.get_name(), "World 3D");
        assert_eq!(
            world
                .get::<&CanvasSettings>(world_2d.get_id())
                .unwrap()
                .resolution,
            (1280, 720)
        );
        assert_eq!(
            world
                .get::<&CanvasSettings>(world_3d.get_id())
                .unwrap()
                .resolution,
            (1280, 720)
        );
        assert_eq!(scene.get_view(), world_2d.get_texture());
        let inspection = world.get::<&Inspection>(scene.get_root().get_id()).unwrap();
        assert_eq!(
            (inspection.get)(&world, scene.get_root().get_id())[0].name,
            "View"
        );
        assert_eq!(
            children(&world, scene.get_root().get_id()),
            vec![world_2d.get_id(), world_3d.get_id()]
        );
    }

    #[crate::scene]
    fn delayed_object_scene(scene: &mut Scene) {
        let circle = circle().build(scene);

        scene.wait(32.0);
        scene.get_world_2d().add(&circle);
        scene.wait(1.0);
    }

    #[test]
    fn object_builder_sets_component_values_and_preserves_object_defaults() {
        let default = Text::default();
        let mut scene = Scene::new();
        let handler = text()
            .opacity(0.5)
            .z_index(-7)
            .position(vec2(10.0, 20.0))
            .text("Kinematic!")
            .build(&mut scene);
        let world = scene.get_world();
        let draw = world.get::<&Draw2D>(handler.get_id()).unwrap();
        let transform = world.get::<&Transform2D>(handler.get_id()).unwrap();
        let shape = world.get::<&TextShape>(handler.get_id()).unwrap();

        assert_eq!(draw.opacity, 0.5);
        assert_eq!(draw.z_index, -7);
        assert_eq!(transform.position, vec2(10.0, 20.0));
        assert_eq!(shape.text, "Kinematic!");
        assert!(std::ptr::fn_addr_eq(draw.on_draw, default.draw.on_draw,));
    }

    #[test]
    fn object_builder_sets_vector_axes_and_color_channels_individually() {
        let default = RegularPolygon2D::default();
        let mut scene = Scene::new();
        let handler = regular_polygon_2d()
            .position_y(24.0)
            .scale_x(2.0)
            .fill_r(0.25)
            .build(&mut scene);
        let world = scene.get_world();
        let transform = world.get::<&Transform2D>(handler.get_id()).unwrap();
        let style = world.get::<&Style>(handler.get_id()).unwrap();

        assert_eq!(transform.position, vec2(default.transform.position.x, 24.0));
        assert_eq!(transform.scale, vec2(2.0, default.transform.scale.y));
        assert_eq!(
            style.fill,
            Color::new(
                0.25,
                default.style.fill.g,
                default.style.fill.b,
                default.style.fill.a,
            )
        );
    }

    #[test]
    fn object_names_default_to_the_type_and_remain_mutable() {
        let mut scene = Scene::new();
        let circle = circle().build(&mut scene);
        let label = text().name("Caption").build(&mut scene);
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
        let text: TextHandler = text().build(&mut scene);
        let circle: CircleHandler = circle().build(&mut scene);

        let _ = text.opacity(0.25);
        let _ = circle.position(vec2(10.0, 20.0));
        let _ = circle.position_x(10.0);
        let _ = circle.fill(Color::RED);
        let _ = circle.fill_r(0.75);
        let world = scene.get_world();
        let draw = world.get::<&Draw2D>(text.get_id()).unwrap();

        assert_eq!(draw.opacity, 0.25);
    }

    #[test]
    fn object_handlers_animate_properties_and_generate_from_shortcuts() {
        let mut scene = Scene::new();
        let circle: CircleHandler = circle().build(&mut scene);
        scene.get_world_2d().add(&circle);

        scene.play(
            circle
                .animate(Transform2D::position_property(), vec2(10.0, 20.0))
                .duration(1.0)
                .task(),
        );
        circle
            .position_from(Vector2::ZERO, vec2(20.0, 30.0))
            .immediate();
        scene.play(circle.opacity_from(0.0, 1.0).duration(1.0).task());

        scene.animator.take_schedule().compile(&scene);
        scene.update(0.5);
        let world = scene.get_world();
        let transform = world.get::<&Transform2D>(circle.get_id()).unwrap();

        assert_eq!(transform.position, vec2(5.0, 10.0));
    }

    #[test]
    fn integer_tracks_interpolate_during_tweens() {
        let mut scene = Scene::new();
        let object = rect().z_index(-10).build(&mut scene);
        let polygon = regular_polygon_2d().sides(4).build(&mut scene);
        scene.get_world_2d().add(&object);
        scene.get_world_2d().add(&polygon);

        scene.all(|_| {
            object
                .z_index(10)
                .duration(2.0)
                .easing(Easing::Linear)
                .play();
            polygon.sides(8).duration(2.0).easing(Easing::Linear).play();
        });

        scene.animator.take_schedule().compile(&scene);
        scene.update(0.5);
        assert_eq!(object.get(Draw2D::z_index_property()), -5);
        assert_eq!(polygon.get(RegularPolygon2DShape::sides_property()), 5);
        scene.update(1.0);
        assert_eq!(object.get(Draw2D::z_index_property()), 0);
        assert_eq!(polygon.get(RegularPolygon2DShape::sides_property()), 6);
        scene.update(2.0);
        assert_eq!(object.get(Draw2D::z_index_property()), 10);
        assert_eq!(polygon.get(RegularPolygon2DShape::sides_property()), 8);
    }

    #[test]
    fn handler_tweens_add_relative_scalar_and_vector_targets() {
        let mut scene = Scene::new();
        let circle = circle().build(&mut scene);
        scene.get_world_2d().add(&circle);

        circle
            .position_by(vec2(10.0, 20.0))
            .position_x_by(6.0)
            .opacity_by(-0.5)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();

        scene.animator.take_schedule().compile(&scene);
        scene.update(1.0);
        let world = scene.get_world();
        let transform = world.get::<&Transform2D>(circle.get_id()).unwrap();
        let draw = world.get::<&Draw2D>(circle.get_id()).unwrap();

        assert_eq!(transform.position, vec2(8.0, 10.0));
        assert_eq!(draw.opacity, 0.75);
    }

    #[test]
    fn quaternion_axis_rotation_preserves_a_complete_turn() {
        let mut scene = Scene::new();
        let cube = cube().build(&mut scene);
        scene.get_world_3d().add(&cube);

        cube.rotate_y(std::f32::consts::TAU)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();

        scene.animator.take_schedule().compile(&scene);

        scene.update(1.0);
        let halfway = scene
            .get_world()
            .get::<&Transform3D>(cube.get_id())
            .unwrap()
            .rotation
            * Vector3::X;
        assert!(halfway.abs_diff_eq(-Vector3::X, 1e-5));

        scene.update(2.0);
        let complete = scene
            .get_world()
            .get::<&Transform3D>(cube.get_id())
            .unwrap()
            .rotation
            * Vector3::X;
        assert!(complete.abs_diff_eq(Vector3::X, 1e-5));
    }

    #[test]
    fn handler_tween_play_registers_in_scene_animator() {
        struct HandlerTweenScene;

        impl SceneBuilder for HandlerTweenScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().fill(Color::RED).build(scene);
                scene.get_world_2d().add(&circle);
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
        let mut query = world.query::<(&Transform2D, &Style)>();
        let circle = query.iter().next().unwrap();
        assert_eq!(circle.0.position.x, 64.0);
        assert_eq!(circle.1.fill, Color::new(0.75, 0.0, 0.25, 1.0));
    }

    #[test]
    fn object_handler_restores_saved_states_in_lifo_order() {
        struct SnapshotScene;

        impl SceneBuilder for SnapshotScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().fill(Color::RED).build(scene);
                scene.get_world_2d().add(&circle);

                circle.save();
                circle.position_x(100.0).fill(Color::BLUE).play();
                circle.save();
                circle.position_x(200.0).fill(Color::GREEN).play();
                circle.restore().play();
                circle.restore().play();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut SnapshotScene), 4.0);

        scene.update(3.0);
        {
            let world = scene.get_world();
            let mut query = world.query::<(&Transform2D, &Style)>();
            let (transform, style) = query.iter().next().unwrap();

            assert_eq!(transform.position.x, 100.0);
            assert_eq!(style.fill, Color::BLUE);
        }

        scene.update(4.0);
        let world = scene.get_world();
        let mut query = world.query::<(&Transform2D, &Style)>();
        let (transform, style) = query.iter().next().unwrap();

        assert_eq!(transform.position.x, 0.0);
        assert_eq!(style.fill, Color::RED);
    }

    #[test]
    #[should_panic(expected = "Cannot restore an object without a saved snapshot.")]
    fn object_handler_rejects_restore_without_a_snapshot() {
        let mut scene = Scene::new();
        let circle = circle().build(&mut scene);

        let _ = circle.restore();
    }

    #[test]
    fn immediate_restore_preserves_the_preceding_track_transition() {
        struct ImmediateRestoreScene;

        impl SceneBuilder for ImmediateRestoreScene {
            fn build(&mut self, scene: &mut Scene) {
                let camera = scene.get_world_2d();

                camera.save();
                camera.camera_rotation(1.0).play();
                camera.restore().immediate();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut ImmediateRestoreScene), 1.0);

        scene.update(0.5);
        {
            let world = scene.get_world();
            let mut query = world.query::<&Camera2D>();
            let camera = query.iter().next().unwrap();

            assert_eq!(camera.camera_rotation, 0.5);
        }

        scene.update(1.0);
        let world = scene.get_world();
        let mut query = world.query::<&Camera2D>();
        let camera = query.iter().next().unwrap();

        assert_eq!(camera.camera_rotation, 0.0);
    }

    #[test]
    fn handler_tween_merges_updates_to_the_same_track() {
        struct ComponentTweenScene;

        impl SceneBuilder for ComponentTweenScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().build(scene);
                scene.get_world_2d().add(&circle);
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
        let mut query = world.query::<(&Node, &Transform2D)>();
        let (_, circle) = query.iter().find(|(node, _)| !node.is_root).unwrap();

        assert_eq!(circle.position, vec2(128.0, 64.0));
    }

    #[test]
    fn creation_effects_toggle_particles_at_their_boundaries() {
        struct CreationScene;

        impl SceneBuilder for CreationScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().build(scene);
                scene.get_world_2d().add(&circle);

                creation().duration(2.0).play(&circle);
                uncreation().duration(2.0).play(&circle);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut CreationScene), 4.0);

        scene.update(0.0);
        {
            let world = scene.get_world();
            let mut query = world.query::<(&Morph,)>();
            let (morph,) = query.iter().next().unwrap();

            assert_eq!(morph.progress, 0.0);
            assert!(morph.particles_enabled);
        }

        scene.update(4.0);
        let world = scene.get_world();
        let mut query = world.query::<(&Morph,)>();
        let (morph,) = query.iter().next().unwrap();

        assert_eq!(morph.progress, 0.0);
        assert!(!morph.particles_enabled);
    }

    #[test]
    fn effect_state_is_not_exposed_by_object_inspection() {
        let mut scene = Scene::new();
        let rect = rect().build(&mut scene);
        let world = scene.get_world();
        let inspection = world.get::<&Inspection>(rect.get_id()).unwrap();
        let components = (inspection.get)(&world, rect.get_id());

        assert!(components.iter().all(|component| component.name != "Morph"));
        let style = components
            .iter()
            .find(|component| component.name == "Style")
            .unwrap();
        assert!((style.get)().iter().all(|track| track.name != "progress"));
    }

    #[test]
    fn creation_particles_form_the_object_silhouette_before_completion() {
        struct CreationScene;

        impl SceneBuilder for CreationScene {
            fn build(&mut self, scene: &mut Scene) {
                let rect = rect().size(vec2(16.0, 16.0)).fill(Color::RED).build(scene);
                scene.get_world_2d().add(&rect);

                creation().play(&rect);
            }
        }

        let mut scene = Scene::new();
        scene.build(&mut CreationScene);
        let mut surface = skia_safe::surfaces::raster_n32_premul((64, 64)).unwrap();

        scene.update(0.0);
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((32.0, 32.0));
        scene.draw(surface.canvas());
        assert_eq!(surface.peek_pixels().unwrap().get_color((32, 32)).a(), 0);

        scene.update(2.4);
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();
        assert!(pixels.get_color((32, 32)).a() > 0);
        assert_eq!(pixels.get_color((8, 8)).a(), 0);

        scene.update(2.5);
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        scene.draw(surface.canvas());
        let center = surface.peek_pixels().unwrap().get_color((32, 32));
        assert_eq!(center.r(), 255);
        assert_eq!(center.a(), 255);
    }

    #[test]
    fn creation_renders_particle_silhouettes_for_every_styled_object() {
        struct StyledObjectsScene;

        impl SceneBuilder for StyledObjectsScene {
            fn build(&mut self, scene: &mut Scene) {
                scene.all(|scene| {
                    let circle = circle()
                        .radius(8.0)
                        .position(vec2(-32.0, 0.0))
                        .fill(Color::RED)
                        .build(scene);
                    let rect = rect()
                        .size(vec2(16.0, 16.0))
                        .fill(Color::GREEN)
                        .build(scene);
                    let text = text()
                        .text("A")
                        .size(20.0)
                        .position(vec2(32.0, 0.0))
                        .fill(Color::BLUE)
                        .build(scene);
                    scene.get_world_2d().add(&circle);
                    scene.get_world_2d().add(&rect);
                    scene.get_world_2d().add(&text);

                    creation().play(&circle);
                    creation().play(&rect);
                    creation().play(&text);
                });
            }
        }

        let mut scene = Scene::new();
        scene.build(&mut StyledObjectsScene);
        scene.update(2.4);
        let mut surface = skia_safe::surfaces::raster_n32_premul((128, 64)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((64.0, 32.0));
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();

        let has_alpha =
            |left, right| (0..64).any(|y| (left..right).any(|x| pixels.get_color((x, y)).a() > 0));

        assert!(has_alpha(20, 44));
        assert!(has_alpha(52, 76));
        assert!(has_alpha(84, 108));
    }

    #[test]
    fn nested_groups_keep_scene_objects_on_their_local_timeline() {
        struct NestedGroupsScene;

        impl SceneBuilder for NestedGroupsScene {
            fn build(&mut self, scene: &mut Scene) {
                let root = circle().build(scene);
                scene.get_world_2d().add(&root);
                scene.wait(1.0);

                scene.chain(|scene| {
                    let chain_object = circle().build(scene);
                    scene.get_world_2d().add(&chain_object);
                    scene.wait(2.0);

                    scene.all(|scene| {
                        let parallel_object = circle().build(scene);
                        scene.get_world_2d().add(&parallel_object);
                        scene.wait(4.0);

                        scene.chain(|scene| {
                            let nested_object = circle().build(scene);
                            scene.get_world_2d().add(&nested_object);
                            scene.wait(1.0);

                            let nested_end_object = circle().build(scene);
                            scene.get_world_2d().add(&nested_end_object);
                        });

                        scene.chain(|scene| {
                            let short_chain_object = circle().build(scene);
                            scene.get_world_2d().add(&short_chain_object);
                            scene.wait(0.5);

                            let short_chain_end_object = circle().build(scene);
                            scene.get_world_2d().add(&short_chain_end_object);
                        });

                        let parallel_end_object = circle().build(scene);
                        scene.get_world_2d().add(&parallel_end_object);
                    });

                    scene.wait(1.0);
                    let chain_end_object = circle().build(scene);
                    scene.get_world_2d().add(&chain_end_object);
                });
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut NestedGroupsScene), 8.0);

        let world = scene.get_world();
        let mut lifetimes: Vec<_> = world
            .query::<(hecs::Entity, &Node)>()
            .iter()
            .filter(|(entity, node)| {
                !node.is_root && world.get::<&CanvasSettings>(*entity).is_err()
            })
            .map(|(_, node)| node.lifetime)
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
                [3.0, f32::INFINITY],
                [3.5, f32::INFINITY],
                [4.0, f32::INFINITY],
                [8.0, f32::INFINITY],
            ]
        );
    }

    #[test]
    fn draw_renders_objects_directly_into_the_supplied_skia_canvas() {
        let mut scene = Scene::new();
        let rect = rect()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .opacity(0.5)
            .position(vec2(4.0, 0.0))
            .build(&mut scene);
        scene.get_world_2d().add(&rect);

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
                let circle = circle().build(scene);
                let rect = rect().build(scene);
                scene.get_world_2d().add(&circle);

                scene.wait(1.0);
                scene.get_world_2d().add(&rect);
                scene.wait(2.0);
                circle.remove();
            }
        }

        let mut scene = Scene::new();
        let duration = scene.build(&mut LifetimeScene);

        assert_eq!(duration, 3.0);

        let lifetimes = || {
            let world = scene.get_world();
            let mut lifetimes: Vec<_> = world
                .query::<(hecs::Entity, &Node)>()
                .iter()
                .filter(|(entity, node)| {
                    !node.is_root && world.get::<&CanvasSettings>(*entity).is_err()
                })
                .map(|(_, node)| node.lifetime)
                .collect();
            lifetimes.sort_by(|left, right| left[0].total_cmp(&right[0]));
            lifetimes
        };

        assert_eq!(lifetimes(), vec![[0.0, 3.0], [1.0, f32::INFINITY]]);

        let active_count = |time| {
            scene.update(time);
            scene
                .get_world()
                .query::<(hecs::Entity, &Node)>()
                .iter()
                .filter(|(entity, node)| {
                    !node.is_root
                        && node.is_activated
                        && scene.get_world().get::<&CanvasSettings>(*entity).is_err()
                })
                .count()
        };

        assert_eq!(active_count(0.0), 1);
        assert_eq!(active_count(1.0), 2);
        assert_eq!(active_count(3.0), 1);
        assert_eq!(active_count(0.5), 1);
    }

    #[test]
    fn object_added_in_parallel_uses_the_group_start() {
        struct ParallelLifetimeScene;

        impl SceneBuilder for ParallelLifetimeScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().build(scene);

                scene.all(|scene| {
                    scene.wait(5.0);
                    scene.get_world_2d().add(&circle);
                    scene.wait(2.0);
                });
                scene.wait(1.0);
            }
        }

        let mut scene = Scene::new();

        assert_eq!(scene.build(&mut ParallelLifetimeScene), 6.0);

        let world = scene.get_world();
        let mut query = world.query::<(hecs::Entity, &Node)>();
        let (_, node) = query
            .iter()
            .find(|(entity, node)| !node.is_root && world.get::<&CanvasSettings>(*entity).is_err())
            .unwrap();

        assert_eq!(node.lifetime, [0.0, f32::INFINITY]);
        assert!(node.is_activated);
    }

    #[test]
    fn scene_macro_preserves_the_create_time_as_the_node_start() {
        let scene = delayed_object_scene((1920, 1080));

        assert_eq!(scene.get_name(), "delayed_object_scene");
        assert_eq!(scene.get_duration(), 33.0);

        scene.update(31.0);
        {
            let world = scene.get_world();
            let mut query = world.query::<(hecs::Entity, &Node)>();
            let (_, node) = query
                .iter()
                .find(|(entity, node)| {
                    !node.is_root && world.get::<&CanvasSettings>(*entity).is_err()
                })
                .unwrap();

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
