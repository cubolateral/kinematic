use kinematic_macros::Trackable;

use crate::core::components::RenderContext3D;

const CHECKPOINT_INTERVAL_SECONDS: u64 = 2;

/// Frame-dependent state owned by a simulation object.
///
/// [`Self::on_update`] runs with a fixed
/// `dt` of `1.0 / fps`. The complete value is cloned for seek checkpoints, so
/// every value that influences future updates must live in this state.
///
/// Use [`SimulationState2D`] or [`SimulationState3D`] to provide read-only
/// drawing for the corresponding simulation object.
pub trait SimulationState: Clone + Send + Sync + 'static {
    /// Advances the state by one project frame.
    fn on_update(&mut self, dt: f32);
}

/// A simulation state that can draw itself in a two-dimensional scene.
///
/// Drawing receives the object's local Skia canvas. The scene renderer applies
/// [`crate::core::components::Transform2D`] and composites opacity before this
/// callback. It must not mutate simulation state.
pub trait SimulationState2D: SimulationState {
    /// Draws the current state in the simulation object's local coordinates.
    fn on_draw(&self, canvas: &skia_safe::Canvas);

    /// Returns the simulation's local bounding-box size.
    fn get_box(&self) -> glam::Vec2 {
        glam::Vec2::ZERO
    }
}

/// A simulation state that can draw itself in a three-dimensional scene.
///
/// Submit geometry with transforms local to the simulation. [`RenderContext3D`]
/// combines them with the simulation object's global transform. Drawing must
/// not mutate simulation state.
pub trait SimulationState3D: SimulationState {
    /// Draws the current state using the supplied global object transform.
    fn on_draw(&self, context: &mut RenderContext3D<'_>) -> Result<(), String>;

    /// Returns the simulation's local bounding-box size.
    fn get_box(&self) -> glam::Vec3 {
        glam::Vec3::ZERO
    }
}

trait ErasedSimulation: Send + Sync {
    fn clone_box(&self) -> Box<dyn ErasedSimulation>;
    fn on_update(&mut self, dt: f32);
    fn draw_2d(&self, _canvas: &skia_safe::Canvas) {}
    fn box_2d(&self) -> glam::Vec2 {
        glam::Vec2::ZERO
    }
    fn draw_3d(&self, _context: &mut RenderContext3D<'_>) -> Result<(), String> {
        Ok(())
    }
    fn box_3d(&self) -> glam::Vec3 {
        glam::Vec3::ZERO
    }
}

impl Clone for Box<dyn ErasedSimulation> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
struct State2D<S>(S);

impl<S: SimulationState2D> ErasedSimulation for State2D<S> {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, dt: f32) {
        self.0.on_update(dt);
    }

    fn draw_2d(&self, canvas: &skia_safe::Canvas) {
        self.0.on_draw(canvas);
    }

    fn box_2d(&self) -> glam::Vec2 {
        self.0.get_box()
    }
}

#[derive(Clone)]
struct State3D<S>(S);

impl<S: SimulationState3D> ErasedSimulation for State3D<S> {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, dt: f32) {
        self.0.on_update(dt);
    }

    fn draw_3d(&self, context: &mut RenderContext3D<'_>) -> Result<(), String> {
        self.0.on_draw(context)
    }

    fn box_3d(&self) -> glam::Vec3 {
        self.0.get_box()
    }
}

#[derive(Clone)]
struct EmptyState;

impl ErasedSimulation for EmptyState {
    fn clone_box(&self) -> Box<dyn ErasedSimulation> {
        Box::new(self.clone())
    }

    fn on_update(&mut self, _dt: f32) {}
}

#[derive(Clone)]
struct Checkpoint {
    frame: u64,
    state: Box<dyn ErasedSimulation>,
}

/// Runtime state and timeline controls shared by 2D and 3D simulations.
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
    manual_updates: Vec<f32>,
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
            manual_updates: self.manual_updates.clone(),
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
            manual_updates: Vec::new(),
        }
    }
}

impl Simulation {
    pub(crate) fn new_2d<S: SimulationState2D>(state: S) -> Self {
        Self::new(Box::new(State2D(state)))
    }

    pub(crate) fn new_3d<S: SimulationState3D>(state: S) -> Self {
        Self::new(Box::new(State3D(state)))
    }

    fn new(state: Box<dyn ErasedSimulation>) -> Self {
        Self {
            auto_update: true,
            current_frame: 0,
            initial_frame_processed: false,
            initial: state.clone(),
            current: state.clone(),
            checkpoints: vec![Checkpoint { frame: 0, state }],
            manual_updates: Vec::new(),
        }
    }

    pub(crate) fn schedule_update(&mut self, time: f32) {
        self.manual_updates.push(time);
    }

    pub(crate) fn seek(
        &mut self,
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

        let dt = 1.0 / fps.max(1) as f32;
        let checkpoint_interval = u64::from(fps.max(1)) * CHECKPOINT_INTERVAL_SECONDS;
        if !self.initial_frame_processed {
            let manual = self.manual_steps(0, dt, start_time);
            let steps = manual.max(u32::from(auto_update_at(start_time)));
            self.run_steps(steps, dt);
            self.initial_frame_processed = true;
        }
        while self.current_frame < target_frame {
            self.current_frame += 1;
            let time = start_time + self.current_frame as f32 * dt;
            let manual = self.manual_steps(self.current_frame, dt, start_time);
            let steps = manual.max(u32::from(auto_update_at(time)));
            self.run_steps(steps, dt);
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

    fn manual_steps(&self, frame: u64, dt: f32, start_time: f32) -> u32 {
        let epsilon = f32::EPSILON * (start_time.abs() + frame as f32 * dt + 1.0) * 4.0;
        if frame == 0 {
            return self
                .manual_updates
                .iter()
                .filter(|time| (**time - start_time).abs() <= epsilon)
                .count() as u32;
        }

        let previous = start_time + (frame - 1) as f32 * dt;
        let current = start_time + frame as f32 * dt;
        self.manual_updates
            .iter()
            .filter(|time| **time > previous + epsilon && **time <= current + epsilon)
            .count() as u32
    }

    fn run_steps(&mut self, steps: u32, dt: f32) {
        for _ in 0..steps {
            self.current.on_update(dt);
        }
    }

    pub(crate) fn draw_2d(&self, canvas: &skia_safe::Canvas) {
        self.current.draw_2d(canvas);
    }

    pub(crate) fn box_2d(&self) -> glam::Vec2 {
        self.current.box_2d()
    }

    pub(crate) fn draw_3d(&self, context: &mut RenderContext3D<'_>) -> Result<(), String> {
        self.current.draw_3d(context)
    }

    pub(crate) fn box_3d(&self) -> glam::Vec3 {
        self.current.box_3d()
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

    impl SimulationState for Counter {
        fn on_update(&mut self, dt: f32) {
            self.steps += 1;
            self.elapsed += dt;
        }
    }

    impl SimulationState2D for Counter {
        fn on_draw(&self, _canvas: &skia_safe::Canvas) {
            *self.observed.lock().unwrap() = (self.steps, self.elapsed);
        }
    }

    fn observed(simulation: &Simulation, value: &Arc<Mutex<(u64, f32)>>) -> (u64, f32) {
        let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();
        simulation.draw_2d(surface.canvas());
        *value.lock().unwrap()
    }

    #[test]
    fn seek_replays_from_the_initial_state_deterministically() {
        let value = Arc::new(Mutex::new((0, 0.0)));
        let mut simulation = Simulation::new_2d(Counter {
            steps: 0,
            elapsed: 0.0,
            observed: Arc::clone(&value),
        });

        simulation.seek(110, 20, 0.0, |_| true);
        let first = observed(&simulation, &value);
        assert!(
            simulation
                .checkpoints
                .iter()
                .any(|checkpoint| checkpoint.frame == 20 * CHECKPOINT_INTERVAL_SECONDS)
        );
        simulation.seek(3, 20, 0.0, |_| true);
        assert_eq!(observed(&simulation, &value).0, 4);
        simulation.seek(110, 20, 0.0, |_| true);

        let replayed = observed(&simulation, &value);
        assert_eq!(replayed.0, 111);
        assert!((replayed.1 - 5.55).abs() < 1e-5);
        assert_eq!(first, replayed);
    }

    #[test]
    fn auto_update_only_controls_automatic_state_updates() {
        let value = Arc::new(Mutex::new((0, 0.0)));
        let mut simulation = Simulation::new_2d(Counter {
            steps: 0,
            elapsed: 0.0,
            observed: Arc::clone(&value),
        });

        simulation.seek(6, 10, 0.0, |time| time < 0.3 || time > 0.5);

        assert_eq!(observed(&simulation, &value).0, 4);
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
        simulation.seek(0, 10, 0.0, |_| auto_update);
        observed(&simulation, &value).0
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
