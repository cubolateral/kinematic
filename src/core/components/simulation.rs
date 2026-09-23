use kinematic_macros::Trackable;
use std::{any::Any, sync::Arc};

use crate::core::{
    Trackable,
    components::{Animation, RenderContext3D},
    frame_dt,
    objects::global_matrix3d,
};

const CHECKPOINT_INTERVAL_SECONDS: u64 = 2;

/// Frame-dependent state owned by an object with a [`Simulation`] component.
///
/// [`Self::on_update`] runs with a fixed
/// `dt` of `1.0 / fps`. The complete value is cloned for seek checkpoints.
/// Timeline-controlled inputs may live in trackable components and be read
/// through [`SimulationContext::get`].
///
/// Use [`SimulationState2D`] or [`SimulationState3D`] to draw the corresponding
/// object.
pub trait SimulationState: Clone + Send + Sync + 'static {
    /// Advances the state by one project frame.
    fn on_update(&mut self, context: &SimulationContext<'_>);
}

/// Values available while advancing one fixed simulation frame.
pub struct SimulationContext<'a> {
    world: &'a hecs::World,
    entity: hecs::Entity,
    /// Absolute project time of this simulation step.
    pub time: f32,
    /// Zero-based frame within the object's lifetime.
    pub frame: u64,
    /// Fixed duration of one project frame.
    pub dt: f32,
}

impl SimulationContext<'_> {
    /// Samples a trackable component at this step's project time.
    pub fn get<T: Trackable + hecs::Component + Clone>(&self) -> T {
        let component = (*self
            .world
            .get::<&T>(self.entity)
            .unwrap_or_else(|_| panic!("Simulation entity must contain the requested component.")))
        .clone();
        let Ok(animation) = self.world.get::<&Animation>(self.entity) else {
            return component;
        };

        let mut snapshot = hecs::World::new();
        let entity = snapshot.spawn((component,));
        for track in (T::info().get)() {
            if let Some(value) = animation.sample(std::any::TypeId::of::<T>(), track, self.time) {
                (track.set)(&snapshot, entity, value);
            }
        }
        snapshot.remove_one::<T>(entity).unwrap()
    }
}

/// A simulation state that can draw itself in a two-dimensional scene.
///
/// Drawing receives the scene world, simulation entity, and object's local Skia
/// canvas. The scene renderer applies [`crate::core::components::Transform2D`]
/// and composites opacity before this callback. Mutations made while drawing
/// are visual runtime state: they are not synchronized to the timeline and may
/// be discarded when seeking rebuilds the simulation.
pub trait SimulationState2D: SimulationState {
    /// Draws the current state in the object's local coordinates.
    fn on_draw(&mut self, world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas);

    /// Returns the simulation's local bounding-box size.
    fn box_size(&self, _world: &hecs::World, _entity: hecs::Entity) -> glam::Vec2 {
        glam::Vec2::ZERO
    }
}

/// A simulation state that can draw itself in a three-dimensional scene.
///
/// Drawing receives the scene world and simulation entity. Submit geometry with
/// transforms local to the simulation. [`RenderContext3D`] combines them with
/// the object's global transform. Mutations made while drawing are
/// visual runtime state: they are not synchronized to the timeline and may be
/// discarded when seeking rebuilds the simulation.
pub trait SimulationState3D: SimulationState {
    /// Draws the current state using the supplied global object transform.
    fn on_draw(
        &mut self,
        world: &hecs::World,
        entity: hecs::Entity,
        context: &mut RenderContext3D<'_>,
    ) -> Result<(), String>;

    /// Returns the simulation's local bounding-box size.
    fn box_size(&self, _world: &hecs::World, _entity: hecs::Entity) -> glam::Vec3 {
        glam::Vec3::ZERO
    }
}

trait ErasedSimulation: Send + Sync {
    fn clone_box(&self) -> Box<dyn ErasedSimulation>;
    fn on_update(&mut self, context: &SimulationContext<'_>);
    fn state(&self) -> &dyn Any;
    fn state_mut(&mut self) -> &mut dyn Any;
    fn draw_2d(
        &mut self,
        _world: &hecs::World,
        _entity: hecs::Entity,
        _canvas: &skia_safe::Canvas,
    ) {
    }
    fn box_2d(&self, _world: &hecs::World, _entity: hecs::Entity) -> glam::Vec2 {
        glam::Vec2::ZERO
    }
    fn draw_3d(
        &mut self,
        _world: &hecs::World,
        _entity: hecs::Entity,
        _context: &mut RenderContext3D<'_>,
    ) -> Result<(), String> {
        Ok(())
    }
    fn box_3d(&self, _world: &hecs::World, _entity: hecs::Entity) -> glam::Vec3 {
        glam::Vec3::ZERO
    }
}

impl Clone for Box<dyn ErasedSimulation> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
struct State<S>(S);

impl<S: SimulationState> ErasedSimulation for State<S> {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, context: &SimulationContext<'_>) {
        self.0.on_update(context);
    }

    fn state(&self) -> &dyn Any {
        &self.0
    }

    fn state_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}

#[derive(Clone)]
struct State2D<S>(S);

impl<S: SimulationState2D> ErasedSimulation for State2D<S> {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, context: &SimulationContext<'_>) {
        self.0.on_update(context);
    }

    fn state(&self) -> &dyn Any {
        &self.0
    }

    fn state_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }

    fn draw_2d(&mut self, world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
        self.0.on_draw(world, entity, canvas);
    }

    fn box_2d(&self, world: &hecs::World, entity: hecs::Entity) -> glam::Vec2 {
        self.0.box_size(world, entity)
    }
}

#[derive(Clone)]
struct State3D<S>(S);

impl<S: SimulationState3D> ErasedSimulation for State3D<S> {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, context: &SimulationContext<'_>) {
        self.0.on_update(context);
    }

    fn state(&self) -> &dyn Any {
        &self.0
    }

    fn state_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }

    fn draw_3d(
        &mut self,
        world: &hecs::World,
        entity: hecs::Entity,
        context: &mut RenderContext3D<'_>,
    ) -> Result<(), String> {
        self.0.on_draw(world, entity, context)
    }

    fn box_3d(&self, world: &hecs::World, entity: hecs::Entity) -> glam::Vec3 {
        self.0.box_size(world, entity)
    }
}

#[derive(Clone)]
struct EmptyState;

impl ErasedSimulation for EmptyState {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, _context: &SimulationContext<'_>) {}

    fn state(&self) -> &dyn Any {
        self
    }

    fn state_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone)]
struct Checkpoint {
    frame: u64,
    state: Box<dyn ErasedSimulation>,
}

#[derive(Clone)]
enum SimulationOperation {
    Update {
        time: f32,
    },
    Mutation {
        time: f32,
        apply: Arc<dyn Fn(&mut dyn Any) + Send + Sync>,
    },
}

impl SimulationOperation {
    fn time(&self) -> f32 {
        match self {
            Self::Update { time } | Self::Mutation { time, .. } => *time,
        }
    }
}

/// Type-erased runtime state and timeline controls for a scene object.
///
/// Only [`Self::auto_update`] is trackable. Current state, frame position, and
/// checkpoints are transient runtime data and are not persisted with a project.
#[derive(Trackable)]
pub struct Simulation {
    /// Whether [`SimulationState::on_update`] runs automatically on project frames.
    ///
    /// Drawing is unaffected, so a paused simulation keeps showing its latest
    /// state. During seek replay, the value is sampled from its historical track
    /// at every frame.
    #[track]
    pub auto_update: bool,

    current_frame: u64,
    initial_frame_processed: bool,
    initial: Box<dyn ErasedSimulation>,
    current: Box<dyn ErasedSimulation>,
    checkpoints: Vec<Checkpoint>,
    operations: Vec<SimulationOperation>,
}

impl Clone for Simulation {
    fn clone(&self) -> Self {
        Self {
            auto_update: self.auto_update,
            current_frame: self.current_frame,
            initial_frame_processed: self.initial_frame_processed,
            initial: self.initial.clone(),
            current: self.current.clone(),
            checkpoints: self.checkpoints.clone(),
            operations: self.operations.clone(),
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        let state: Box<dyn ErasedSimulation> = Box::new(EmptyState);
        Self {
            auto_update: true,
            current_frame: 0,
            initial_frame_processed: false,
            initial: state.clone(),
            current: state.clone(),
            checkpoints: vec![Checkpoint { frame: 0, state }],
            operations: Vec::new(),
        }
    }
}

impl Simulation {
    /// Creates runtime storage for a simulation with custom drawing.
    pub fn new<S: SimulationState>(state: S) -> Self {
        Self::from_erased(Box::new(State(state)))
    }

    /// Creates runtime storage whose state supplies two-dimensional drawing.
    pub fn new_2d<S: SimulationState2D>(state: S) -> Self {
        Self::from_erased(Box::new(State2D(state)))
    }

    /// Creates runtime storage whose state supplies three-dimensional drawing.
    pub fn new_3d<S: SimulationState3D>(state: S) -> Self {
        Self::from_erased(Box::new(State3D(state)))
    }

    fn from_erased(state: Box<dyn ErasedSimulation>) -> Self {
        Self {
            auto_update: true,
            current_frame: 0,
            initial_frame_processed: false,
            initial: state.clone(),
            current: state.clone(),
            checkpoints: vec![Checkpoint { frame: 0, state }],
            operations: Vec::new(),
        }
    }

    pub(crate) fn read_state<S: SimulationState, R>(&self, read: impl FnOnce(&S) -> R) -> R {
        read(
            self.current
                .state()
                .downcast_ref::<S>()
                .expect("Simulation state must match the state associated with its object."),
        )
    }

    pub(crate) fn write_state<S: SimulationState>(&mut self, write: impl FnOnce(&mut S)) {
        write(
            self.current
                .state_mut()
                .downcast_mut::<S>()
                .expect("Simulation state must match the state associated with its object."),
        );
    }

    pub(crate) fn schedule_update(&mut self, time: f32) {
        self.operations.push(SimulationOperation::Update { time });
        self.reset_replay();
    }

    pub(crate) fn schedule_mutation<S: SimulationState>(
        &mut self,
        time: f32,
        mutation: impl Fn(&mut S) + Send + Sync + 'static,
    ) {
        assert!(
            self.initial.state_mut().is::<S>(),
            "Simulation state must match the state associated with its object."
        );
        self.operations.push(SimulationOperation::Mutation {
            time,
            apply: Arc::new(move |state| {
                mutation(
                    state
                        .downcast_mut::<S>()
                        .expect("Simulation mutation state type must remain stable."),
                );
            }),
        });
        self.reset_replay();
    }

    fn reset_replay(&mut self) {
        self.current_frame = 0;
        self.initial_frame_processed = false;
        self.current = self.initial.clone();
        self.checkpoints.truncate(1);
    }

    pub(crate) fn seek(
        &mut self,
        world: &hecs::World,
        entity: hecs::Entity,
        target_frame: u64,
        fps: u32,
        start_time: f32,
        mut auto_update_at: impl FnMut(f32) -> bool,
    ) {
        if target_frame == self.current_frame && self.initial_frame_processed {
            return;
        }

        if target_frame < self.current_frame || target_frame > self.current_frame + 1 {
            let checkpoint = self
                .checkpoints
                .iter()
                .rev()
                .find(|checkpoint| checkpoint.frame <= target_frame);
            if let Some(checkpoint) = checkpoint {
                self.current_frame = checkpoint.frame;
                self.current = checkpoint.state.clone();
                self.initial_frame_processed = checkpoint.frame > 0;
            } else {
                self.current_frame = 0;
                self.current = self.initial.clone();
                self.initial_frame_processed = false;
            }
        }

        let dt = frame_dt(fps);
        let checkpoint_interval = u64::from(fps.max(1)) * CHECKPOINT_INTERVAL_SECONDS;
        if !self.initial_frame_processed {
            self.run_frame(
                self.frame_operations(0, dt, start_time),
                auto_update_at(start_time),
                SimulationContext {
                    world,
                    entity,
                    time: start_time,
                    frame: 0,
                    dt,
                },
            );
            self.initial_frame_processed = true;
        }
        while self.current_frame < target_frame {
            self.current_frame += 1;
            let time = start_time + self.current_frame as f32 * dt;
            self.run_frame(
                self.frame_operations(self.current_frame, dt, start_time),
                auto_update_at(time),
                SimulationContext {
                    world,
                    entity,
                    time,
                    frame: self.current_frame,
                    dt,
                },
            );
            if self.current_frame.is_multiple_of(checkpoint_interval)
                && self
                    .checkpoints
                    .last()
                    .is_none_or(|checkpoint| checkpoint.frame < self.current_frame)
            {
                self.checkpoints.push(Checkpoint {
                    frame: self.current_frame,
                    state: self.current.clone(),
                });
            }
        }
    }

    fn frame_operations(&self, frame: u64, dt: f32, start_time: f32) -> Vec<SimulationOperation> {
        let epsilon = f32::EPSILON * (start_time.abs() + frame as f32 * dt + 1.0) * 4.0;
        if frame == 0 {
            return self
                .operations
                .iter()
                .filter(|operation| (operation.time() - start_time).abs() <= epsilon)
                .cloned()
                .collect();
        }

        let previous = start_time + (frame - 1) as f32 * dt;
        let current = start_time + frame as f32 * dt;
        self.operations
            .iter()
            .filter(|operation| {
                operation.time() > previous + epsilon && operation.time() <= current + epsilon
            })
            .cloned()
            .collect()
    }

    fn run_frame(
        &mut self,
        operations: Vec<SimulationOperation>,
        auto_update: bool,
        context: SimulationContext<'_>,
    ) {
        let has_update = operations
            .iter()
            .any(|operation| matches!(operation, SimulationOperation::Update { .. }));
        for operation in operations {
            match operation {
                SimulationOperation::Update { .. } => self.current.on_update(&context),
                SimulationOperation::Mutation { apply, .. } => {
                    apply(self.current.state_mut());
                }
            }
        }
        if auto_update && !has_update {
            self.current.on_update(&context);
        }
    }

    /// Draws the current 2D state as a [`Draw2D`](super::Draw2D) callback.
    pub fn draw_2d(
        world: &hecs::World,
        entity: hecs::Entity,
        canvas: &skia_safe::Canvas,
        _opacity: f32,
    ) {
        world
            .get::<&mut Self>(entity)
            .expect("Simulated object must contain a Simulation component.")
            .current
            .draw_2d(world, entity, canvas);
    }

    /// Returns the current 2D state's bounds as a [`Draw2D`](super::Draw2D) callback.
    pub fn box_2d(world: &hecs::World, entity: hecs::Entity) -> glam::Vec2 {
        world
            .get::<&Self>(entity)
            .expect("Simulated object must contain a Simulation component.")
            .current
            .box_2d(world, entity)
    }

    /// Returns centered visual bounds for the current 2D simulation state.
    pub fn visual_bounds_2d(world: &hecs::World, entity: hecs::Entity) -> skia_safe::Rect {
        let size = Self::box_2d(world, entity);
        skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y)
    }

    /// Draws the current 3D state as a [`Draw3D`](super::Draw3D) callback.
    pub fn draw_3d(
        world: &hecs::World,
        entity: hecs::Entity,
        context: &mut RenderContext3D<'_>,
    ) -> Result<(), String> {
        let previous = context.set_current_transform(global_matrix3d(world, entity));
        let result = world
            .get::<&mut Self>(entity)
            .expect("Simulated object must contain a Simulation component.")
            .current
            .draw_3d(world, entity, context);
        context.set_current_transform(previous);
        result
    }

    /// Returns the current 3D state's bounds as a [`Draw3D`](super::Draw3D) callback.
    pub fn box_3d(world: &hecs::World, entity: hecs::Entity) -> glam::Vec3 {
        world
            .get::<&Self>(entity)
            .expect("Simulated object must contain a Simulation component.")
            .current
            .box_3d(world, entity)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[derive(Clone)]
    struct Counter {
        steps: u64,
        elapsed: f32,
        observed: Arc<Mutex<(u64, f32)>>,
    }

    #[derive(Clone)]
    struct RandomCounter {
        random: crate::core::Random,
        observed: Arc<Mutex<u64>>,
    }

    #[derive(Clone)]
    struct DrawCounter {
        draws: u64,
        observed: Arc<Mutex<u64>>,
    }

    struct DrawSetting(u32);

    #[derive(Clone)]
    struct ComponentReader(Arc<Mutex<Option<u32>>>);

    impl SimulationState for ComponentReader {
        fn on_update(&mut self, _context: &SimulationContext<'_>) {}
    }

    impl SimulationState2D for ComponentReader {
        fn on_draw(
            &mut self,
            world: &hecs::World,
            entity: hecs::Entity,
            _canvas: &skia_safe::Canvas,
        ) {
            *self.0.lock().unwrap() = Some(world.get::<&DrawSetting>(entity).unwrap().0);
        }
    }

    impl SimulationState for Counter {
        fn on_update(&mut self, context: &SimulationContext<'_>) {
            self.steps += 1;
            self.elapsed += context.dt;
        }
    }

    impl SimulationState2D for Counter {
        fn on_draw(
            &mut self,
            _world: &hecs::World,
            _entity: hecs::Entity,
            _canvas: &skia_safe::Canvas,
        ) {
            *self.observed.lock().unwrap() = (self.steps, self.elapsed);
        }
    }

    impl SimulationState for RandomCounter {
        fn on_update(&mut self, _context: &SimulationContext<'_>) {
            *self.observed.lock().unwrap() = self.random.u64();
        }
    }

    impl SimulationState2D for RandomCounter {
        fn on_draw(
            &mut self,
            _world: &hecs::World,
            _entity: hecs::Entity,
            _canvas: &skia_safe::Canvas,
        ) {
        }
    }

    impl SimulationState for DrawCounter {
        fn on_update(&mut self, _context: &SimulationContext<'_>) {}
    }

    impl SimulationState2D for DrawCounter {
        fn on_draw(
            &mut self,
            _world: &hecs::World,
            _entity: hecs::Entity,
            _canvas: &skia_safe::Canvas,
        ) {
            self.draws += 1;
            *self.observed.lock().unwrap() = self.draws;
        }
    }

    fn observed(simulation: &mut Simulation, value: &Arc<Mutex<(u64, f32)>>) -> (u64, f32) {
        let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        simulation.current.draw_2d(&world, entity, surface.canvas());
        *value.lock().unwrap()
    }

    fn seek(
        simulation: &mut Simulation,
        target_frame: u64,
        fps: u32,
        auto_update_at: impl FnMut(f32) -> bool,
    ) {
        let mut world = hecs::World::new();
        let entity = world.spawn((Animation::default(),));
        simulation.seek(&world, entity, target_frame, fps, 0.0, auto_update_at);
    }

    #[test]
    fn drawing_can_read_components_from_the_simulation_entity() {
        let observed = Arc::new(Mutex::new(None));
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Simulation::new_2d(ComponentReader(Arc::clone(&observed))),
            DrawSetting(42),
        ));
        let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();

        Simulation::draw_2d(&world, entity, surface.canvas(), 1.0);

        assert_eq!(*observed.lock().unwrap(), Some(42));
    }

    #[test]
    fn drawing_can_mutate_visual_runtime_state() {
        let observed = Arc::new(Mutex::new(0));
        let mut simulation = Simulation::new_2d(DrawCounter {
            draws: 0,
            observed: Arc::clone(&observed),
        });
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();

        simulation.current.draw_2d(&world, entity, surface.canvas());
        simulation.current.draw_2d(&world, entity, surface.canvas());

        assert_eq!(*observed.lock().unwrap(), 2);
    }

    #[test]
    fn seek_replays_from_the_initial_state_deterministically() {
        let value = Arc::new(Mutex::new((0, 0.0)));
        let mut simulation = Simulation::new_2d(Counter {
            steps: 0,
            elapsed: 0.0,
            observed: Arc::clone(&value),
        });

        seek(&mut simulation, 110, 20, |_| true);
        let first = observed(&mut simulation, &value);
        assert!(
            simulation
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.frame == 20 * CHECKPOINT_INTERVAL_SECONDS)
        );
        seek(&mut simulation, 3, 20, |_| true);
        assert_eq!(observed(&mut simulation, &value).0, 4);
        seek(&mut simulation, 110, 20, |_| true);

        let replayed = observed(&mut simulation, &value);
        assert_eq!(replayed.0, 111);
        assert!((replayed.1 - 5.55).abs() < 1e-5);
        assert_eq!(first, replayed);
    }

    #[test]
    fn seek_restores_random_state_from_checkpoints() {
        let value = Arc::new(Mutex::new(0));
        let mut simulation = Simulation::new_2d(RandomCounter {
            random: crate::core::Random::new(42),
            observed: Arc::clone(&value),
        });

        seek(&mut simulation, 110, 20, |_| true);
        let first = *value.lock().unwrap();
        seek(&mut simulation, 3, 20, |_| true);
        assert_ne!(*value.lock().unwrap(), first);
        seek(&mut simulation, 110, 20, |_| true);

        assert_eq!(*value.lock().unwrap(), first);
    }

    #[test]
    fn auto_update_only_controls_automatic_state_updates() {
        let value = Arc::new(Mutex::new((0, 0.0)));
        let mut simulation = Simulation::new_2d(Counter {
            steps: 0,
            elapsed: 0.0,
            observed: Arc::clone(&value),
        });

        seek(&mut simulation, 6, 10, |time| time < 0.3 || time > 0.5);

        assert_eq!(observed(&mut simulation, &value).0, 4);
    }

    fn steps(auto_update: bool, manual_updates: u32) -> u64 {
        let value = Arc::new(Mutex::new((0, 0.0)));
        let mut simulation = Simulation::new_2d(Counter {
            steps: 0,
            elapsed: 0.0,
            observed: Arc::clone(&value),
        });
        for _ in 0..manual_updates {
            simulation.schedule_update(0.0);
        }
        seek(&mut simulation, 0, 10, |_| auto_update);
        observed(&mut simulation, &value).0
    }

    #[test]
    fn explicit_steps_merge_with_the_automatic_step() {
        assert_eq!(steps(false, 0), 0);
        assert_eq!(steps(false, 1), 1);
        assert_eq!(steps(true, 0), 1);
        assert_eq!(steps(true, 1), 1);
        assert_eq!(steps(false, 3), 3);
        assert_eq!(steps(true, 3), 3);
    }
}
