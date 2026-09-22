use crate::core::scene_file::{SceneFile, ScheduledEvent, TimeEvent};
use crate::core::{
    Animator, Scheduling, Task, TrackHandle, TrackValueType, TrackableInfo, Tween,
    components::{Animation, Draw2D, Inspection, Name, Simulation, TreeNode, View},
    frame_index,
    objects::{
        Canvas2D, Canvas2DBuilder, Canvas2DHandler, Canvas3D, Canvas3DBuilder, Canvas3DHandler,
        Object, ObjectHandler, RootHandler, camera_matrix2d, canvas_2d, canvas_3d,
        children_by_z_index, draw_entity,
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
pub(crate) struct SceneIdentity(pub(crate) u64, std::sync::atomic::AtomicU64);

pub(crate) fn invalidate_lifetimes(world: &hecs::World) {
    if let Some(identity) = world.query::<&SceneIdentity>().iter().next() {
        identity
            .1
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

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
    time: std::cell::Cell<f32>,
    animator: Animator,
    scene_file: SceneFile,
    scheduled_events: Vec<ScheduledEvent>,
    persist_events: bool,
    revision: std::cell::Cell<u64>,
    plan_revision: std::cell::Cell<u64>,
    runtime: std::cell::RefCell<Option<Runtime>>,
    fps: u32,
}

#[derive(Default)]
struct Runtime {
    animated: Vec<hecs::Entity>,
    simulations: Vec<hecs::Entity>,
    boundaries: Vec<(f32, hecs::Entity)>,
    cursor: usize,
    initialized: bool,
    generation: u64,
}

impl Scene {
    /// Creates a scene with 1920x1080 default worlds.
    pub fn new() -> Self {
        Self::new_with_resolution((1920, 1080))
    }

    /// Creates a scene whose built-in worlds use `resolution`.
    pub fn new_with_resolution(resolution: (u32, u32)) -> Self {
        Self::new_inner("Scene", resolution, false, None)
    }

    /// Creates a scene using fully configured builders for its built-in canvases.
    pub fn new_with_canvases(world_2d: Canvas2DBuilder, world_3d: Canvas3DBuilder) -> Self {
        Self::new_inner("Scene", (0, 0), false, Some((world_2d, world_3d)))
    }

    #[doc(hidden)]
    pub fn new_named(name: &'static str, resolution: (u32, u32)) -> Self {
        Self::new_inner(name, resolution, true, None)
    }

    /// Creates a named scene using fully configured built-in canvas builders.
    #[doc(hidden)]
    pub fn new_named_with_canvases(
        name: &'static str,
        world_2d: Canvas2DBuilder,
        world_3d: Canvas3DBuilder,
    ) -> Self {
        Self::new_inner(name, (0, 0), true, Some((world_2d, world_3d)))
    }

    fn new_inner(
        name: &'static str,
        resolution: (u32, u32),
        persist_events: bool,
        canvases: Option<(Canvas2DBuilder, Canvas3DBuilder)>,
    ) -> Self {
        let animator_time = std::rc::Rc::new(std::cell::Cell::new(0.0));
        let animator = Animator::with_scene_time(std::rc::Rc::clone(&animator_time));
        let world = std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new()));
        static NEXT_SCENE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_SCENE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = world.borrow_mut().spawn(
            hecs::EntityBuilder::new()
                .add(SceneIdentity(id, std::sync::atomic::AtomicU64::new(0)))
                .add(Animation::default())
                .add(Draw2D::default())
                .add(Inspection::new("Root", root_trackables))
                .add(Name::new("Root"))
                .add(TreeNode::default())
                .add(View::default())
                .build(),
        );

        {
            let world = world.borrow();
            let mut node = world
                .get::<&mut TreeNode>(root)
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
            time: std::cell::Cell::new(0.0),
            animator,
            scene_file: if persist_events {
                SceneFile::load(name)
            } else {
                SceneFile::default()
            },
            scheduled_events: Vec::new(),
            persist_events,
            revision: std::cell::Cell::new(0),
            plan_revision: std::cell::Cell::new(0),
            runtime: std::cell::RefCell::new(None),
            fps: 60,
        };

        let (world_2d, world_3d) = canvases.unwrap_or_else(|| {
            (
                canvas_2d().name("World 2D").resolution(resolution),
                canvas_3d().name("World 3D").resolution(resolution),
            )
        });
        let world_2d = world_2d.build(&mut scene);
        let world_3d = world_3d.build(&mut scene);
        scene.root().add(&world_2d);
        scene.root().add(&world_3d);
        scene.world_2d = world_2d.entity();
        scene.world_3d = world_3d.entity();
        scene
    }

    pub(crate) fn view_texture(&self) -> crate::core::objects::CanvasTexture {
        if self.root().is_view_2d() {
            self.world_2d().texture()
        } else {
            self.world_3d().texture()
        }
    }

    /// Updates nodes, tracks and simulations, then evaluates active signals.
    ///
    /// This updates scene state only; rendering remains in [`Self::draw`].
    pub fn update(&self, time: f32) {
        self.time.set(time);
        let generation = self.structure_revision();
        if self
            .runtime
            .borrow()
            .as_ref()
            .is_none_or(|runtime| runtime.generation != generation)
        {
            self.compile_runtime();
        }
        let signals = self.animator.handle().signals();
        let world = self.world.borrow_mut();

        signals.restore_overrides(&world);

        let mut runtime = self.runtime.borrow_mut();
        if let Some(runtime) = runtime.as_mut() {
            let cursor = runtime.boundaries.partition_point(|(at, _)| *at <= time);
            if !runtime.initialized {
                for (_, entity) in &runtime.boundaries {
                    if let Ok(mut node) = world.get::<&mut TreeNode>(*entity) {
                        node.update(time);
                    }
                }
            } else {
                for (_, entity) in
                    &runtime.boundaries[runtime.cursor.min(cursor)..runtime.cursor.max(cursor)]
                {
                    if let Ok(mut node) = world.get::<&mut TreeNode>(*entity) {
                        node.update(time);
                    }
                }
            }
            if !runtime.initialized || cursor != runtime.cursor {
                self.plan_revision
                    .set(self.plan_revision.get().wrapping_add(1));
            }
            runtime.cursor = cursor;
            runtime.initialized = true;
            for entity in &runtime.animated {
                if world
                    .get::<&TreeNode>(*entity)
                    .is_ok_and(|node| node.is_activated)
                    && let Ok(mut animation) = world.get::<&mut Animation>(*entity)
                {
                    for track in &mut animation.tracks {
                        track.track.update(&world, *entity, time);
                    }
                }
            }

            for entity in &runtime.simulations {
                let Ok(node) = world.get::<&TreeNode>(*entity) else {
                    continue;
                };
                if !node.is_activated {
                    continue;
                }
                let start_time = node.lifetime[0];
                drop(node);

                let local_time = (time - start_time).max(0.0);
                let target_frame = frame_index(local_time, self.fps);
                let animation = world.get::<&Animation>(*entity).ok();
                let fallback = world
                    .get::<&Simulation>(*entity)
                    .map_or(true, |simulation| simulation.auto_update);
                let track_info = <Simulation as crate::core::Trackable>::track(0);
                if let Ok(mut simulation) = world.get::<&mut Simulation>(*entity) {
                    simulation.seek(
                        &world,
                        *entity,
                        target_frame,
                        self.fps,
                        start_time,
                        |sample_time| {
                            animation
                                .as_ref()
                                .and_then(|animation| {
                                    animation.sample(
                                        std::any::TypeId::of::<Simulation>(),
                                        track_info,
                                        sample_time,
                                    )
                                })
                                .and_then(bool::from_track_value)
                                .unwrap_or(fallback)
                        },
                    );
                }
            }
        }
        drop(runtime);
        drop(world);

        signals.evaluate(&self.world, time, self.fps);
        self.revision.set(self.revision.get().wrapping_add(1));
    }

    pub(crate) fn invalidate(&self) {
        self.revision.set(self.revision.get().wrapping_add(1));
        self.plan_revision
            .set(self.plan_revision.get().wrapping_add(1));
    }

    fn structure_revision(&self) -> u64 {
        self.world
            .borrow()
            .get::<&SceneIdentity>(self.root)
            .unwrap()
            .1
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn render_key(&self) -> (u64, u64) {
        (
            self.world
                .borrow()
                .get::<&SceneIdentity>(self.root)
                .unwrap()
                .0,
            self.revision.get().wrapping_add(self.structure_revision()),
        )
    }

    pub(crate) fn plan_revision(&self) -> u64 {
        self.plan_revision
            .get()
            .wrapping_add(self.structure_revision())
    }

    pub(crate) fn compile_runtime(&self) {
        let world = self.world.borrow();
        let mut runtime = Runtime {
            generation: world
                .get::<&SceneIdentity>(self.root)
                .unwrap()
                .1
                .load(std::sync::atomic::Ordering::Relaxed),
            ..Runtime::default()
        };
        for (entity, node) in world.query::<(hecs::Entity, &TreeNode)>().iter() {
            for at in node.lifetime {
                if at.is_finite() {
                    runtime.boundaries.push((at, entity));
                }
            }
        }
        runtime.boundaries.sort_by(|a, b| a.0.total_cmp(&b.0));
        runtime.animated.extend(
            world
                .query::<(hecs::Entity, &Animation)>()
                .iter()
                .filter(|(_, animation)| !animation.tracks.is_empty())
                .map(|(entity, _)| entity),
        );
        runtime.simulations.extend(
            world
                .query::<(hecs::Entity, &Simulation)>()
                .iter()
                .map(|(entity, _)| entity),
        );
        *self.runtime.borrow_mut() = Some(runtime);
        self.invalidate();
    }

    pub(crate) fn set_fps(&mut self, fps: u32) {
        self.fps = fps.max(1);
    }

    /// Draws the built-in 2D world without applying its output-size translation.
    pub fn draw(&self, canvas: &skia_safe::Canvas) {
        let world = self.world.borrow();
        let save_count = canvas.save();
        let camera_base = canvas.local_to_device();

        if let Some(view) =
            camera_matrix2d(&world, self.world_2d).and_then(|camera| camera.invert())
        {
            canvas.concat(&view);
        }

        for child in children_by_z_index(&world, self.world_2d) {
            draw_entity(&world, child, canvas, Some(&camera_base));
        }
        canvas.restore_to_count(save_count);
    }

    #[cfg(test)]
    pub(crate) fn selection_outline(&self, entity: hecs::Entity) -> Option<Vec<[[f32; 2]; 2]>> {
        let view_2d = self.root().is_view_2d();
        let output = self.view_texture();
        let world = self.world.borrow();
        if !view_2d {
            return crate::core::objects::outline_segments3d(&world, output.entity, entity);
        }
        if !self.output_is_active_2d(&world, output.entity) {
            return None;
        }
        let points = crate::core::objects::outline_points(&world, output.entity, entity)?;
        let size = world
            .get::<&crate::core::objects::CanvasSettings>(output.entity)
            .ok()?
            .resolution;
        let points = points.map(|p| {
            [
                p.x / size.0.max(1) as f32 + 0.5,
                p.y / size.1.max(1) as f32 + 0.5,
            ]
        });
        Some(
            (0..4)
                .map(|index| [points[index], points[(index + 1) % 4]])
                .collect(),
        )
    }

    pub(crate) fn nearest_canvas(
        &self,
        mut entity: hecs::Entity,
    ) -> Option<(hecs::Entity, crate::core::objects::CanvasDimension)> {
        let world = self.world.borrow();
        loop {
            if let Ok(settings) = world.get::<&crate::core::objects::CanvasSettings>(entity) {
                return Some((entity, settings.dimension));
            }
            entity = world.get::<&TreeNode>(entity).ok()?.parent?;
        }
    }

    pub(crate) fn editor_2d_selection_outline(
        &self,
        canvas: hecs::Entity,
        entity: hecs::Entity,
        camera_view: bool,
    ) -> Option<[skia_safe::Point; 4]> {
        let world = self.world.borrow();
        let mut points = crate::core::objects::outline_points_in_world(&world, canvas, entity)?;
        if camera_view && crate::core::objects::object_follows_camera(&world, canvas, entity) {
            crate::core::objects::camera_matrix2d(&world, canvas)?
                .invert()?
                .map_points_inplace(&mut points);
        }
        Some(points)
    }

    pub(crate) fn editor_2d_camera_outline(
        &self,
        canvas: hecs::Entity,
        camera_view: bool,
    ) -> Option<[skia_safe::Point; 4]> {
        crate::core::objects::camera_outline_points2d(&self.world.borrow(), canvas, camera_view)
    }

    pub(crate) fn pick_editor_2d(
        &self,
        canvas: hecs::Entity,
        point: Vector2,
        camera_view: bool,
    ) -> Option<hecs::Entity> {
        let world = self.world.borrow();
        crate::core::objects::pick_canvas2d_in_world(&world, canvas, point, camera_view)
    }

    pub(crate) fn pick_editor_3d(
        &self,
        canvas: hecs::Entity,
        camera: &crate::core::components::Camera3D,
        resolution: (u32, u32),
        point: Vector2,
    ) -> Option<hecs::Entity> {
        crate::core::objects::pick_canvas3d_with_camera(
            &self.world.borrow(),
            canvas,
            camera,
            resolution,
            point,
        )
    }

    pub(crate) fn editor_3d_selection_segments(
        &self,
        canvas: hecs::Entity,
        target: hecs::Entity,
    ) -> Option<Vec<[glam::Vec3; 2]>> {
        crate::core::objects::outline_segments3d_in_world(&self.world.borrow(), canvas, target)
    }

    #[cfg(test)]
    pub(crate) fn pick(&self, point: Vector2) -> Option<hecs::Entity> {
        let view_2d = self.root().is_view_2d();
        let output = self.view_texture();
        let world = self.world.borrow();
        if view_2d && self.output_is_active_2d(&world, output.entity) {
            crate::core::objects::pick_canvas2d(&world, output.entity, point)
        } else if !view_2d {
            crate::core::objects::pick_canvas3d(&world, output.entity, point)
        } else {
            None
        }
    }

    #[cfg(test)]
    fn output_is_active_2d(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&TreeNode>(entity).is_ok_and(|n| n.is_activated)
            && world
                .get::<&crate::core::objects::CanvasSettings>(entity)
                .is_ok_and(|s| s.dimension == crate::core::objects::CanvasDimension::Two)
    }

    /// Populates the scene and compiles the builder's animation timeline.
    ///
    /// Returns the duration of the resulting timeline.
    pub fn build(&mut self, builder: &mut dyn SceneBuilder) -> f32 {
        self.animator = Animator::with_scene_time(std::rc::Rc::clone(&self.animator_time));
        self.scheduled_events.clear();

        builder.build(self);

        self.animator.take_schedule().compile(self)
    }

    pub(crate) fn duration(&self) -> f32 {
        self.animator.duration()
    }

    pub(crate) fn time(&self) -> f32 {
        self.time.get()
    }

    pub(crate) fn name(&self) -> &'static str {
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

    /// Creates an invisible typed track initialized with `initial`.
    ///
    /// The value is converted through [`TrackValueType`], follows the scene
    /// timeline during seeking, and can be read with [`TrackHandle::get`].
    ///
    /// ```
    /// # use kinematic::prelude::*;
    /// # let mut scene = Scene::new();
    /// let counter = scene.track(0_u32);
    /// counter.set(10).duration(2.0).play();
    /// ```
    pub fn track<T: TrackValueType>(&mut self, initial: T) -> TrackHandle<T> {
        crate::core::standalone_track(
            std::rc::Rc::clone(&self.world),
            self.animator.handle().active(),
            initial,
        )
    }

    /// Waits for the specified duration on the current scene timeline.
    pub fn wait(&mut self, duration: f32) {
        self.play(Task::Wait(duration));
    }

    /// Waits for the duration stored for a named scene event.
    ///
    /// The duration is relative to the active scheduling position, like
    /// [`Self::wait`]. Named scenes persist durations in
    /// `.kinematic/scenes/<scene_name>.ron`. Missing events are created with
    /// zero duration. The Timeline displays the wait as a draggable event span;
    /// committing a drag rebuilds that scene through its factory.
    ///
    /// Events cannot be used inside [`Self::repeat`].
    ///
    /// ```ignore
    /// s.event("intro");
    /// ```
    pub fn event(&mut self, name: &str) {
        self.animator.handle().assert_event_scope();
        let (file_index, duration) = match self
            .scene_file
            .events
            .iter()
            .enumerate()
            .find(|(_, event)| event.name == name)
        {
            Some((index, event)) => (index, event.duration.max(0.0)),
            None => {
                self.scene_file.events.push(TimeEvent {
                    name: name.to_owned(),
                    duration: 0.0,
                });
                self.save_events();
                (self.scene_file.events.len() - 1, 0.0)
            }
        };
        self.scheduled_events.push(ScheduledEvent {
            file_index,
            name: name.to_owned(),
            creation_time: self.animator.handle().time(),
            duration,
        });
        self.wait(duration.max(0.0));
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
        self.schedule_group(Scheduling::Sequential, false, false, schedule);
    }

    /// Adds a simultaneous group to the current scene timeline.
    pub fn all(&mut self, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Parallel, false, false, schedule);
    }

    /// Runs a group without advancing or extending the current timeline.
    pub fn parallel(&mut self, schedule: impl FnOnce(&mut Scene)) {
        self.schedule_group(Scheduling::Sequential, false, true, schedule);
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
                for component in inspection.trackables(&world, entity) {
                    for info in (component.get)() {
                        values.push((entity, info, (info.get)(&world, entity)));
                    }
                }
            }
            values
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.schedule_group(Scheduling::Sequential, true, false, schedule);
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
        parallel: bool,
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
        let mut schedule = group.take_schedule();
        schedule.parallel = parallel;
        self.animator.handle().schedule(if repeating {
            schedule.repeated()
        } else {
            schedule
        });
    }

    #[doc(hidden)]
    pub fn spawn_object<T: Object>(&mut self, object: T, name: impl Into<String>) -> T::Handler {
        self.spawn_object_with_shaders(object, name, None, None)
    }

    #[doc(hidden)]
    pub fn spawn_object_with_shaders<T: Object>(
        &mut self,
        object: T,
        name: impl Into<String>,
        image_shader: Option<crate::core::objects::ImageShaderData>,
        mesh_shader: Option<crate::core::objects::MeshShaderData>,
    ) -> T::Handler {
        let handler = T::spawn(
            std::rc::Rc::clone(&self.world),
            self.animator.handle().active(),
            object,
            Name::new(name),
        );
        if let Some(shader) = image_shader {
            self.world
                .borrow_mut()
                .insert_one(
                    crate::core::objects::ObjectHandler::entity(&handler),
                    shader,
                )
                .expect("Spawned object must accept its image shader.");
        }
        if let Some(shader) = mesh_shader {
            self.world
                .borrow_mut()
                .insert_one(
                    crate::core::objects::ObjectHandler::entity(&handler),
                    shader,
                )
                .expect("Spawned object must accept its mesh shader.");
        }
        crate::core::objects::initialize_object_snapshots(
            &self.world,
            crate::core::objects::ObjectHandler::entity(&handler),
        );
        handler
    }

    /// Returns the built-in 2D canvas.
    pub fn world_2d(&self) -> Canvas2DHandler {
        Canvas2D::handler(
            std::rc::Rc::clone(&self.world),
            self.world_2d,
            self.animator.handle().active(),
        )
    }

    /// Returns the built-in 3D canvas.
    pub fn world_3d(&self) -> Canvas3DHandler {
        Canvas3D::handler(
            std::rc::Rc::clone(&self.world),
            self.world_3d,
            self.animator.handle().active(),
        )
    }

    /// Attaches an additional 2D canvas to this scene.
    pub fn add_canvas_2d(&self, canvas: &Canvas2DHandler) {
        self.root().add(canvas);
    }

    /// Attaches an additional 3D canvas to this scene.
    pub fn add_canvas_3d(&self, canvas: &Canvas3DHandler) {
        self.root().add(canvas);
    }

    /// Returns the internal root that owns the scene's canvases.
    pub fn root(&self) -> RootHandler {
        RootHandler {
            world: std::rc::Rc::clone(&self.world),
            entity: self.root,
            animator: self.animator.handle().active(),
        }
    }

    /// Read-only access to the underlying ECS world.
    pub fn world(&self) -> std::cell::Ref<'_, hecs::World> {
        self.world.borrow()
    }

    /// Mutable access to the underlying ECS world.
    pub fn world_mut(&self) -> std::cell::RefMut<'_, hecs::World> {
        self.invalidate();
        *self.runtime.borrow_mut() = None;
        self.world.borrow_mut()
    }

    pub(crate) fn events(&self) -> &[ScheduledEvent] {
        &self.scheduled_events
    }

    pub(crate) fn set_event_duration(&mut self, index: usize, duration: f32) {
        let event = self
            .scene_file
            .events
            .get_mut(index)
            .expect("Event index must belong to this scene.");
        event.duration = duration.max(0.0);
        self.save_events();
    }

    fn save_events(&self) {
        if self.persist_events {
            self.scene_file.save(self.name);
        }
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
    fn revision_changes_for_updates_edits_and_rebuilds_but_not_overlays() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        scene.update(0.0);
        let rendered = scene.render_key();
        assert!(scene.selection_outline(object.entity()).is_some());
        assert_eq!(scene.pick(Vector2::ZERO), Some(object.entity()));
        let mut surface = skia_safe::surfaces::raster_n32_premul((16, 16)).unwrap();
        scene.draw(surface.canvas());
        assert_eq!(scene.render_key(), rendered);
        scene.invalidate();
        let edited = scene.render_key();
        assert_ne!(edited, rendered);
        scene.update(0.5);
        assert_ne!(scene.render_key(), edited);
        assert_ne!(Scene::new().render_key().0, rendered.0);
    }

    #[test]
    fn preview_hit_testing_and_debug_follow_the_root_view_dimension() {
        let mut scene = Scene::new_with_resolution((64, 64));
        let rectangle = rect().size(vec2(16.0, 16.0)).build(&mut scene);
        scene.world_2d().add(&rectangle);
        let near_cube = cube().build(&mut scene);
        let far_cube = cube().position(vec3(0.0, 0.0, -2.0)).build(&mut scene);
        scene.world_3d().add(&near_cube);
        scene.world_3d().add(&far_cube);

        assert_eq!(scene.pick(Vector2::ZERO), Some(rectangle.entity()));
        assert_eq!(
            scene.selection_outline(rectangle.entity()).unwrap().len(),
            4
        );

        scene.root().view_2d(false).immediate();

        assert_eq!(scene.pick(Vector2::ZERO), Some(near_cube.entity()));
        assert!(scene.pick(vec2(31.0, 31.0)).is_none());
        assert_eq!(
            scene.selection_outline(near_cube.entity()).unwrap().len(),
            12
        );
        assert!(scene.selection_outline(rectangle.entity()).is_none());
    }

    #[test]
    fn editor_2d_supports_free_and_camera_locked_coordinates() {
        let mut scene = Scene::new_with_resolution((64, 64));
        let canvas = canvas_2d()
            .resolution((100, 80))
            .camera_position(vec2(100.0, 0.0))
            .camera_zoom(2.0)
            .build(&mut scene);
        let group = group_2d().build(&mut scene);
        let rectangle = rect()
            .size(vec2(10.0, 10.0))
            .position(vec2(20.0, 0.0))
            .build(&mut scene);
        group.add(&rectangle);
        canvas.add(&group);
        scene.add_canvas_2d(&canvas);

        assert_eq!(
            scene.nearest_canvas(rectangle.entity()),
            Some((canvas.entity(), CanvasDimension::Two))
        );
        assert_eq!(
            scene.pick_editor_2d(canvas.entity(), vec2(20.0, 0.0), false),
            Some(rectangle.entity())
        );

        let camera = scene
            .editor_2d_camera_outline(canvas.entity(), false)
            .unwrap();
        assert_eq!(camera[0], skia_safe::Point::new(75.0, -20.0));
        assert_eq!(camera[2], skia_safe::Point::new(125.0, 20.0));
        assert!(
            scene
                .editor_2d_selection_outline(canvas.entity(), rectangle.entity(), false)
                .is_some()
        );
        assert_eq!(
            scene.pick_editor_2d(canvas.entity(), vec2(-160.0, 0.0), true),
            Some(rectangle.entity())
        );
        let locked_camera = scene
            .editor_2d_camera_outline(canvas.entity(), true)
            .unwrap();
        assert_eq!(locked_camera[0], skia_safe::Point::new(-50.0, -40.0));
        assert_eq!(locked_camera[2], skia_safe::Point::new(50.0, 40.0));
        let locked_selection = scene
            .editor_2d_selection_outline(canvas.entity(), rectangle.entity(), true)
            .unwrap();
        assert_eq!(locked_selection[0], skia_safe::Point::new(-170.0, -10.0));
        assert_eq!(locked_selection[2], skia_safe::Point::new(-150.0, 10.0));
    }

    #[test]
    fn free_editor_2d_shades_the_world_outside_the_camera() {
        let scene = Scene::new_with_resolution((100, 80));
        let canvas = scene.world_2d().entity();
        let world = scene.world();
        let mut surface = skia_safe::surfaces::raster_n32_premul((200, 160)).unwrap();

        draw_canvas2d_editor_with_images(
            &world,
            canvas,
            surface.canvas(),
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            (200, 160),
            [0.0; 2],
            1.0,
            [1.0; 2],
            false,
        );

        let pixels = surface.peek_pixels().unwrap();
        assert_eq!(pixels.get_color((100, 80)).a(), 0);
        assert!((63..=64).contains(&pixels.get_color((10, 10)).a()));
    }

    #[test]
    fn camera_2d_draws_the_far_end_of_a_large_object_when_zoomed() {
        let mut scene = Scene::new_with_resolution((100, 80));
        let line = line_2d()
            .from(vec2(0.0, 0.0))
            .to(vec2(100_000_000.0, 0.0))
            .thickness(10.0)
            .fill(Color::RED)
            .build(&mut scene);
        scene.world_2d().add(&line);
        scene
            .world_2d()
            .camera_position(vec2(100_000_000.0, 0.0))
            .camera_zoom(100.0)
            .immediate();
        let mut surface = skia_safe::surfaces::raster_n32_premul((100, 80)).unwrap();

        surface.canvas().translate((50.0, 40.0));
        scene.draw(surface.canvas());

        assert!(surface.peek_pixels().unwrap().get_color((40, 40)).r() > 0);
    }

    #[test]
    fn compiled_runtime_visits_tracks_and_crossed_lifetimes_and_handles_later_removal() {
        let mut scene = Scene::new();
        let static_object = circle().build(&mut scene);
        scene.world_2d().add(&static_object);
        let animated = circle().build(&mut scene);
        scene.world_2d().add(&animated);
        animated
            .position_x(10.0)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();
        scene.animator.take_schedule().compile(&scene);
        assert_eq!(
            scene.runtime.borrow().as_ref().unwrap().animated,
            vec![animated.entity()]
        );
        scene.update(0.5);
        let plan_revision = scene.plan_revision();
        scene.update(1.0);
        assert_eq!(scene.plan_revision(), plan_revision);
        assert_eq!(animated.get_position().x, 5.0);
        static_object.remove();
        scene.update(2.0);
        assert!(
            !scene
                .world()
                .get::<&TreeNode>(static_object.entity())
                .unwrap()
                .is_activated
        );
        scene.update(0.5);
        assert!(
            scene
                .world()
                .get::<&TreeNode>(static_object.entity())
                .unwrap()
                .is_activated
        );
        assert_eq!(animated.get_position().x, 2.5);
    }

    fn scene_with_event(name: &'static str, event_name: &str, duration: f32) -> Scene {
        let mut scene = Scene::new_named(name, (1920, 1080));
        scene.event(event_name);
        let index = scene
            .scene_file
            .events
            .iter()
            .position(|event| event.name == event_name)
            .unwrap();
        scene.set_event_duration(index, duration);
        Scene::new_named(name, (1920, 1080))
    }

    #[test]
    fn event_waits_for_its_duration() {
        let mut scene = scene_with_event("event_duration", "intro", 5.0);

        scene.wait(2.0);
        scene.event("intro");
        assert_eq!(scene.events()[0].creation_time, 2.0);
        assert_eq!(scene.events()[0].duration, 5.0);
        scene.wait(1.0);
        scene.event("intro");

        assert_eq!(scene.duration(), 13.0);
    }

    #[test]
    fn event_uses_the_active_chain_and_parallel_times() {
        let mut scene = scene_with_event("event_groups", "intro", 5.0);

        scene.wait(1.0);
        scene.all(|scene| {
            scene.chain(|scene| {
                scene.wait(2.0);
                scene.event("intro");
            });
            scene.wait(3.0);
        });

        assert_eq!(scene.duration(), 8.0);
    }

    #[test]
    #[should_panic(expected = "Events cannot be used inside repeat cycles.")]
    fn event_is_rejected_inside_repeat() {
        let mut scene = Scene::new_named("event_repeat", (1920, 1080));

        scene.repeat(|scene| scene.event("intro"));
    }

    #[test]
    fn signal_overrides_a_tween_after_tracks_are_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        object.signal(|handler, _| handler.set_position(vec2(25.0, 0.0)));
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
    fn signal_receives_project_frame_timing() {
        let mut scene = Scene::new();
        scene.set_fps(24);
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        let observed = std::rc::Rc::new(std::cell::Cell::new(None));
        let callback_observed = std::rc::Rc::clone(&observed);
        object.signal(move |_, frame| callback_observed.set(Some(frame)));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.5);

        let frame = observed.get().unwrap();
        assert_eq!(frame.time, 0.5);
        assert_eq!(frame.index, 12);
        assert_eq!(frame.dt, 1.0 / 24.0);
    }

    #[test]
    fn signal_interval_supports_forward_and_backward_seeks() {
        let mut scene = Scene::new();
        let object = circle().position(vec2(3.0, 0.0)).build(&mut scene);
        scene.world_2d().add(&object);
        scene.wait(1.0);
        let signaled = object.clone();
        let signal = object.signal(move |_, _| signaled.set_position(vec2(9.0, 0.0)));
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
        scene.world_2d().add(&object);
        let signaled = object.clone();
        let signal = object.signal(move |_, _| signaled.set_position(vec2(4.0, 0.0)));
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
        scene.world_2d().add(&object);
        let first = object.clone();
        let first_signal = object.signal(move |_, _| first.set_position(vec2(10.0, 0.0)));
        let second = object.clone();
        let second_signal = object.signal(move |_, _| second.set_position(vec2(20.0, 0.0)));
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
        object.signal(move |_, _| callback_calls.set(callback_calls.get() + 1));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);

        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn signal_can_read_another_handler_without_borrowing_the_scene_world() {
        let mut scene = Scene::new();
        let tracked = circle().position(vec2(7.0, 8.0)).build(&mut scene);
        scene.world_2d().add(&tracked);
        let canvas = scene.world_2d();
        let signaled_canvas = canvas.clone();
        let tracked = tracked.clone();
        canvas.signal(move |_, _| signaled_canvas.set_camera_position(tracked.global_position()));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);

        assert_eq!(canvas.get_camera_position(), vec2(7.0, 8.0));
    }

    #[test]
    fn signal_does_not_advance_scene_duration() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        object.signal(|_, _| {});

        assert_eq!(scene.animator.take_schedule().compile(&scene), 0.0);
    }

    #[test]
    #[should_panic(expected = "create, attach, or remove objects outside repeat")]
    fn signal_is_rejected_inside_repeat() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);

        scene.repeat(|_| {
            object.signal(|_, _| {});
        });
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_create_a_tween_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        let signaled = object.clone();
        object.signal(move |_, _| {
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
        scene.world_2d().add(&object);
        let signaled = object.clone();
        object.signal(move |_, _| signaled.remove());
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_attach_an_object_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let canvas = scene.world_2d();
        let child = circle().build(&mut scene);
        let parent = canvas.clone();
        canvas.signal(move |_, _| parent.add(&child));
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    #[should_panic(expected = "Signals cannot alter the scene structure or timeline")]
    fn signal_cannot_play_an_existing_tween_while_it_is_evaluated() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        let queued = object.position(vec2(1.0, 2.0));
        object.set_position(Vector2::ZERO);
        let mut queued = Some(queued);
        object.signal(move |_, _| queued.take().unwrap().play());
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.0);
    }

    #[test]
    fn finite_for_loops_rebuild_relative_targets() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
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
        scene.world_2d().add(&group);

        assert_eq!(scene.pick(Vector2::ZERO), None);
        group.visibility(true).immediate();
        assert_eq!(scene.pick(Vector2::ZERO), Some(object.entity()));
    }

    #[test]
    fn delay_offsets_tweens_tasks_and_scopes() {
        let mut scene = Scene::new();
        let tween_object = circle().build(&mut scene);
        let task_object = circle().build(&mut scene);
        let scope_object = circle().build(&mut scene);
        scene.world_2d().add(&tween_object);
        scene.world_2d().add(&task_object);
        scene.world_2d().add(&scope_object);

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
        scene.world_2d().add(&object);
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
        scene.world_2d().add(&object);
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
        assert_eq!(scene.duration(), 10.0);
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
        let world = scene.world();
        let animation = world.get::<&Animation>(object.entity()).unwrap();
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
        scene.world_2d().add(&object);
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
        assert_eq!(scene.duration(), 1.0);
        scene.wait(6.0);
        scene.animator.take_schedule().compile(&scene);
        for (time, y) in [(0.5, 3.0), (2.5, 6.0), (3.5, 4.0), (1.5, 4.0), (6.5, 6.0)] {
            scene.update(time);
            assert_eq!(object.get(Transform2D::position_property()).y, y);
        }
    }

    #[test]
    fn standalone_track_is_seekable_and_readable_from_a_signal() {
        let mut scene = Scene::new();
        let value = scene.track(0.0_f32);
        scene.repeat(|_| {
            value.set(10.0).duration(2.0).easing(Easing::Linear).play();
        });
        assert_eq!(value.get(), 0.0);

        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);
        let observed = std::rc::Rc::new(std::cell::Cell::new(0.0));
        let signal_value = value.clone();
        let signal_observed = std::rc::Rc::clone(&observed);
        object.signal(move |_, _| signal_observed.set(signal_value.get()));
        scene.wait(6.0);
        scene.animator.take_schedule().compile(&scene);

        for (time, expected) in [(0.5, 2.5), (2.5, 2.5), (1.5, 7.5), (0.5, 2.5)] {
            scene.update(time);
            assert_eq!(value.get(), expected);
            assert_eq!(observed.get(), expected);
        }
    }

    #[test]
    fn repeated_axis_rotation_preserves_signed_multiple_turns() {
        let mut scene = Scene::new();
        let object = cube().build(&mut scene);
        scene.world_3d().add(&object);
        scene.repeat(|_| {
            object
                .rotation_y_by(-2.0 * std::f32::consts::TAU)
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
        scene.world_2d().add(&object);
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
    fn parallel_tween_does_not_extend_the_timeline() {
        let mut scene = Scene::new();
        let object = circle().build(&mut scene);
        scene.world_2d().add(&object);

        scene.parallel(|_| {
            object
                .position_x(10.0)
                .duration(5.0)
                .easing(Easing::Linear)
                .play();
        });
        scene.wait(1.0);

        assert_eq!(scene.animator.take_schedule().compile(&scene), 1.0);
        scene.update(0.5);
        assert_eq!(object.get_position().x, 1.0);
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
        assert_eq!(scene.duration(), 2.0);
        object.opacity(0.5).play();
        assert_eq!(scene.duration(), 3.0);
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
        let world_2d = scene.world_2d();
        let world_3d = scene.world_3d();
        let world = scene.world();

        assert_eq!(world_2d.name(), "World 2D");
        assert_eq!(world_3d.name(), "World 3D");
        assert_eq!(
            world
                .get::<&CanvasSettings>(world_2d.entity())
                .unwrap()
                .resolution,
            (1280, 720)
        );
        assert_eq!(
            world
                .get::<&CanvasSettings>(world_3d.entity())
                .unwrap()
                .resolution,
            (1280, 720)
        );
        assert_eq!(scene.view_texture(), world_2d.texture());
        let inspection = world.get::<&Inspection>(scene.root().entity()).unwrap();
        assert_eq!(
            (inspection.get)(&world, scene.root().entity())[0].name,
            "View"
        );
        assert_eq!(
            children(&world, scene.root().entity()),
            vec![world_2d.entity(), world_3d.entity()]
        );
    }

    #[crate::scene]
    fn delayed_object_scene(scene: &mut Scene) {
        let circle = circle().build(scene);

        scene.wait(32.0);
        scene.world_2d().add(&circle);
        scene.wait(1.0);
    }

    #[test]
    fn object_builder_sets_component_values_and_preserves_object_defaults() {
        let default = Text2D::default();
        let mut scene = Scene::new();
        let handler = text_2d()
            .opacity(0.5)
            .z_index(-7)
            .position(vec2(10.0, 20.0))
            .text("Kinematic!")
            .build(&mut scene);
        let world = scene.world();
        let draw = world.get::<&Draw2D>(handler.entity()).unwrap();
        let transform = world.get::<&Transform2D>(handler.entity()).unwrap();
        let shape = world.get::<&TextShape>(handler.entity()).unwrap();

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
        let world = scene.world();
        let transform = world.get::<&Transform2D>(handler.entity()).unwrap();
        let style = world.get::<&Style>(handler.entity()).unwrap();

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
        let label = text_2d().name("Caption").build(&mut scene);
        let root = scene.root();

        assert_eq!(circle.name(), "Circle");
        assert_eq!(label.name(), "Caption");
        assert_eq!(root.name(), "Root");

        circle.set_name("Primary Circle");

        assert_eq!(circle.name(), "Primary Circle");
        assert_eq!(
            scene.world().get::<&Name>(circle.entity()).unwrap().get(),
            "Primary Circle"
        );
    }

    #[test]
    fn object_handler_exposes_trackable_fields_directly() {
        let mut scene = Scene::new();
        let text: Text2DHandler = text_2d().build(&mut scene);
        let circle: CircleHandler = circle().build(&mut scene);

        let _ = text.opacity(0.25);
        let _ = circle.position(vec2(10.0, 20.0));
        let _ = circle.position_x(10.0);
        let _ = circle.fill(Color::RED);
        let _ = circle.fill_r(0.75);
        let world = scene.world();
        let draw = world.get::<&Draw2D>(text.entity()).unwrap();

        assert_eq!(draw.opacity, 0.25);
    }

    #[test]
    fn track_limits_apply_to_builders_setters_and_tween_variants() {
        let mut scene = Scene::new();
        let circle = circle().opacity(2.0).build(&mut scene);

        assert_eq!(circle.get_opacity(), 1.0);
        circle.set_opacity(-1.0_f32);
        assert_eq!(circle.get_opacity(), 0.0);
        let _ = circle.opacity(2.0);
        assert_eq!(circle.get_opacity(), 1.0);
        let _ = circle.opacity_from(-1.0, 2.0);
        assert_eq!(circle.get_opacity(), 1.0);
        let _ = circle.opacity_by(-2.0);
        assert_eq!(circle.get_opacity(), 0.0);
    }

    #[test]
    fn object_handlers_animate_properties_and_generate_from_shortcuts() {
        let mut scene = Scene::new();
        let circle: CircleHandler = circle().build(&mut scene);
        scene.world_2d().add(&circle);

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
        let world = scene.world();
        let transform = world.get::<&Transform2D>(circle.entity()).unwrap();

        assert_eq!(transform.position, vec2(5.0, 10.0));
    }

    #[test]
    fn integer_tracks_interpolate_during_tweens() {
        let mut scene = Scene::new();
        let object = rect().z_index(-10).build(&mut scene);
        let polygon = regular_polygon_2d().sides(4).build(&mut scene);
        scene.world_2d().add(&object);
        scene.world_2d().add(&polygon);

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
        scene.world_2d().add(&circle);

        circle
            .position_by(vec2(10.0, 20.0))
            .position_x_by(6.0)
            .opacity_by(-0.5)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();

        scene.animator.take_schedule().compile(&scene);
        scene.update(1.0);
        let world = scene.world();
        let transform = world.get::<&Transform2D>(circle.entity()).unwrap();
        let draw = world.get::<&Draw2D>(circle.entity()).unwrap();

        assert_eq!(transform.position, vec2(8.0, 10.0));
        assert_eq!(draw.opacity, 0.75);
    }

    #[test]
    fn quaternion_axis_rotation_preserves_a_complete_turn() {
        let mut scene = Scene::new();
        let cube = cube().build(&mut scene);
        scene.world_3d().add(&cube);

        cube.rotation_y_by(std::f32::consts::TAU)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();

        scene.animator.take_schedule().compile(&scene);

        scene.update(1.0);
        let halfway = scene
            .world()
            .get::<&Transform3D>(cube.entity())
            .unwrap()
            .rotation
            * Vector3::X;
        assert!(halfway.abs_diff_eq(-Vector3::X, 1e-5));

        scene.update(2.0);
        let complete = scene
            .world()
            .get::<&Transform3D>(cube.entity())
            .unwrap()
            .rotation
            * Vector3::X;
        assert!(complete.abs_diff_eq(Vector3::X, 1e-5));
    }

    #[test]
    fn fixed_and_relative_euler_axes_combine_in_one_tween() {
        let mut scene = Scene::new();
        let initial = Quaternion::from_euler(glam::EulerRot::XYZ, 0.1, 0.2, 0.3);
        let fixed = cube().rotation(initial).build(&mut scene);
        let relative = cube().build(&mut scene);
        scene.world_3d().add(&fixed);
        scene.world_3d().add(&relative);

        scene.all(|_| {
            fixed
                .rotation_x(0.4)
                .rotation_y(0.5)
                .duration(2.0)
                .easing(Easing::Linear)
                .play();
            relative
                .rotation_x_by(std::f32::consts::FRAC_PI_2)
                .rotation_y_by(std::f32::consts::FRAC_PI_2)
                .duration(2.0)
                .easing(Easing::Linear)
                .play();
        });

        scene.animator.take_schedule().compile(&scene);
        let fixed_target = Quaternion::from_euler(glam::EulerRot::XYZ, 0.4, 0.5, 0.3);
        let relative_target = Quaternion::from_rotation_x(std::f32::consts::FRAC_PI_2)
            * Quaternion::from_rotation_y(std::f32::consts::FRAC_PI_2);

        scene.update(0.0);
        assert!(
            fixed
                .get(Transform3D::rotation_property())
                .abs_diff_eq(initial, 1e-5)
        );
        assert!(
            relative
                .get(Transform3D::rotation_property())
                .abs_diff_eq(Quaternion::IDENTITY, 1e-5)
        );
        scene.update(2.0);
        assert!(
            fixed
                .get(Transform3D::rotation_property())
                .abs_diff_eq(fixed_target, 1e-5)
        );
        assert!(
            relative
                .get(Transform3D::rotation_property())
                .abs_diff_eq(relative_target, 1e-5)
        );
    }

    #[test]
    fn absolute_axis_rotation_uses_the_requested_axis_and_angle() {
        let mut scene = Scene::new();
        let cube = cube().build(&mut scene);
        scene.world_3d().add(&cube);
        let axis = Vector3::new(1.0, 2.0, 3.0).normalize();
        cube.rotation_axis(axis, 0.75)
            .duration(1.0)
            .easing(Easing::Linear)
            .play();
        scene.animator.take_schedule().compile(&scene);
        scene.update(1.0);
        assert!(
            cube.get(Transform3D::rotation_property())
                .abs_diff_eq(Quaternion::from_axis_angle(axis, 0.75), 1e-5)
        );
    }

    #[test]
    fn handler_tween_play_registers_in_scene_animator() {
        struct HandlerTweenScene;

        impl SceneBuilder for HandlerTweenScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().fill(Color::RED).build(scene);
                scene.world_2d().add(&circle);
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
        let world = scene.world();
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
                scene.world_2d().add(&circle);

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
            let world = scene.world();
            let mut query = world.query::<(&Transform2D, &Style)>();
            let (transform, style) = query.iter().next().unwrap();

            assert_eq!(transform.position.x, 100.0);
            assert_eq!(style.fill, Color::BLUE);
        }

        scene.update(4.0);
        let world = scene.world();
        let mut query = world.query::<(&Transform2D, &Style)>();
        let (transform, style) = query.iter().next().unwrap();

        assert_eq!(transform.position.x, 0.0);
        assert_eq!(style.fill, Color::RED);
    }

    #[test]
    fn object_handler_resets_to_reusable_builder_state() {
        let mut scene = Scene::new();
        let circle = circle().position_x(10.0).fill(Color::RED).build(&mut scene);
        scene.world_2d().add(&circle);

        circle.position_x(100.0).fill(Color::BLUE).play();
        circle.reset().play();
        circle.position_x(200.0).fill(Color::GREEN).play();
        circle.reset().play();

        assert_eq!(scene.animator.take_schedule().compile(&scene), 4.0);
        scene.update(4.0);
        assert_eq!(circle.get_position().x, 10.0);
        assert_eq!(circle.get_fill(), Color::RED);
    }

    #[test]
    fn object_state_tweens_every_track_on_another_instance() {
        let mut scene = Scene::new();
        let source = circle()
            .position(vec2(100.0, 40.0))
            .radius(64.0)
            .fill(Color::BLUE)
            .opacity(0.4)
            .build(&mut scene);
        let target = circle()
            .position(vec2(-100.0, -40.0))
            .radius(32.0)
            .fill(Color::RED)
            .opacity(1.0)
            .build(&mut scene);
        scene.world_2d().add(&source);
        scene.world_2d().add(&target);

        target.restore_snapshot(source.snapshot()).play();
        scene.animator.take_schedule().compile(&scene);

        scene.update(0.5);
        assert_eq!(target.get_position(), Vector2::ZERO);
        assert_eq!(target.get_radius(), 48.0);
        assert_eq!(target.get_fill(), Color::new(0.5, 0.0, 0.5, 1.0));
        assert_eq!(target.get_opacity(), 0.7);

        scene.update(1.0);
        assert_eq!(target.get_position(), source.get_position());
        assert_eq!(target.get_radius(), source.get_radius());
        assert_eq!(target.get_fill(), source.get_fill());
        assert_eq!(target.get_opacity(), source.get_opacity());
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
                let camera = scene.world_2d();

                camera.save();
                camera.camera_rotation(1.0).play();
                camera.restore().immediate();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut ImmediateRestoreScene), 1.0);

        scene.update(0.5);
        {
            let world = scene.world();
            let mut query = world.query::<&Camera2D>();
            let camera = query.iter().next().unwrap();

            assert_eq!(camera.camera_rotation, 0.5);
        }

        scene.update(1.0);
        let world = scene.world();
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
                scene.world_2d().add(&circle);
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
        let world = scene.world();
        let mut query = world.query::<(&TreeNode, &Transform2D)>();
        let (_, circle) = query.iter().find(|(node, _)| !node.is_root).unwrap();

        assert_eq!(circle.position, vec2(128.0, 64.0));
    }

    #[test]
    fn creation_effects_toggle_particles_at_their_boundaries() {
        struct CreationScene;

        impl SceneBuilder for CreationScene {
            fn build(&mut self, scene: &mut Scene) {
                let circle = circle().build(scene);
                scene.world_2d().add(&circle);

                creation().duration(2.0).play(&circle);
                uncreation().duration(2.0).play(&circle);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut CreationScene), 4.0);

        scene.update(0.0);
        {
            let world = scene.world();
            let mut query = world.query::<(&Morph,)>();
            let (morph,) = query.iter().next().unwrap();

            assert_eq!(morph.progress, 0.0);
            assert!(morph.particles_enabled);
        }

        scene.update(3.0);
        {
            let world = scene.world();
            let mut query = world.query::<(&Morph, &Draw2D)>();
            let (morph, draw) = query.iter().next().unwrap();

            assert!(morph.particles_enabled);
            assert_eq!(draw.opacity, 1.0);
        }

        scene.update(4.0);
        {
            let world = scene.world();
            let mut query = world.query::<(&Morph, &Draw2D)>();
            let (morph, draw) = query.iter().next().unwrap();

            assert_eq!(morph.progress, 0.0);
            assert!(!morph.particles_enabled);
            assert_eq!(draw.opacity, 0.0);
        }

        scene.update(3.0);
        let world = scene.world();
        let mut query = world.query::<(&Morph, &Draw2D)>();
        let (morph, draw) = query.iter().next().unwrap();

        assert!(morph.particles_enabled);
        assert_eq!(draw.opacity, 1.0);
    }

    #[test]
    fn effect_state_is_not_exposed_by_object_inspection() {
        let mut scene = Scene::new();
        let rect = rect().build(&mut scene);
        let world = scene.world();
        let inspection = world.get::<&Inspection>(rect.entity()).unwrap();
        let components = (inspection.get)(&world, rect.entity());

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
                scene.world_2d().add(&rect);

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
                    let text = text_2d()
                        .text("A")
                        .size(20.0)
                        .position(vec2(32.0, 0.0))
                        .fill(Color::BLUE)
                        .build(scene);
                    scene.world_2d().add(&circle);
                    scene.world_2d().add(&rect);
                    scene.world_2d().add(&text);

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
                scene.world_2d().add(&root);
                scene.wait(1.0);

                scene.chain(|scene| {
                    let chain_object = circle().build(scene);
                    scene.world_2d().add(&chain_object);
                    scene.wait(2.0);

                    scene.all(|scene| {
                        let parallel_object = circle().build(scene);
                        scene.world_2d().add(&parallel_object);
                        scene.wait(4.0);

                        scene.chain(|scene| {
                            let nested_object = circle().build(scene);
                            scene.world_2d().add(&nested_object);
                            scene.wait(1.0);

                            let nested_end_object = circle().build(scene);
                            scene.world_2d().add(&nested_end_object);
                        });

                        scene.chain(|scene| {
                            let short_chain_object = circle().build(scene);
                            scene.world_2d().add(&short_chain_object);
                            scene.wait(0.5);

                            let short_chain_end_object = circle().build(scene);
                            scene.world_2d().add(&short_chain_end_object);
                        });

                        let parallel_end_object = circle().build(scene);
                        scene.world_2d().add(&parallel_end_object);
                    });

                    scene.wait(1.0);
                    let chain_end_object = circle().build(scene);
                    scene.world_2d().add(&chain_end_object);
                });
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut NestedGroupsScene), 8.0);

        let world = scene.world();
        let mut lifetimes: Vec<_> = world
            .query::<(hecs::Entity, &TreeNode)>()
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
        scene.world_2d().add(&rect);

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
                scene.world_2d().add(&circle);

                scene.wait(1.0);
                scene.world_2d().add(&rect);
                scene.wait(2.0);
                circle.remove();
            }
        }

        let mut scene = Scene::new();
        let duration = scene.build(&mut LifetimeScene);

        assert_eq!(duration, 3.0);

        let lifetimes = || {
            let world = scene.world();
            let mut lifetimes: Vec<_> = world
                .query::<(hecs::Entity, &TreeNode)>()
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
                .world()
                .query::<(hecs::Entity, &TreeNode)>()
                .iter()
                .filter(|(entity, node)| {
                    !node.is_root
                        && node.is_activated
                        && scene.world().get::<&CanvasSettings>(*entity).is_err()
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
                    scene.world_2d().add(&circle);
                    scene.wait(2.0);
                });
                scene.wait(1.0);
            }
        }

        let mut scene = Scene::new();

        assert_eq!(scene.build(&mut ParallelLifetimeScene), 6.0);

        let world = scene.world();
        let mut query = world.query::<(hecs::Entity, &TreeNode)>();
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

        assert_eq!(scene.name(), "delayed_object_scene");
        assert_eq!(scene.duration(), 33.0);

        scene.update(31.0);
        {
            let world = scene.world();
            let mut query = world.query::<(hecs::Entity, &TreeNode)>();
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
        let world = scene.world();
        let mut query = world.query::<&TreeNode>();

        assert!(
            query
                .iter()
                .find(|node| !node.is_root)
                .unwrap()
                .is_activated
        );
    }
}
