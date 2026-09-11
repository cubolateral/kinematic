use kinematic_macros::Object;

use crate::core::{
    components::{
        Draw2D, Draw3D, RenderContext3D, Simulation, SimulationState2D, SimulationState3D,
        Transform2D, Transform3D,
    },
    objects::{ObjectHandler, global_matrix3d},
};

/// A frame-dependent simulation placed in a two-dimensional scene.
///
/// Create one with [`simulation_2d`]. It behaves like a regular spatial object
/// and contains [`Transform2D`], [`Draw2D`], and scene-node metadata.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "simulation_2d")]
pub struct Simulation2D {
    #[trackable]
    pub simulation: Simulation,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Simulation2D {
    fn default() -> Self {
        Self {
            simulation: Simulation::default(),
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, _opacity| {
                    world.get::<&Simulation>(entity).unwrap().draw_2d(canvas);
                },
                get_box: |world, entity| world.get::<&Simulation>(entity).unwrap().box_2d(),
                ..Draw2D::default()
            },
        }
    }
}

impl Simulation2DBuilder {
    /// Sets the state advanced and drawn by this simulation.
    pub fn state<S: SimulationState2D>(mut self, state: S) -> Self {
        self.object.simulation = Simulation::new_2d(state);
        self
    }
}

impl Simulation2DHandler {
    /// Schedules one explicit simulation step at the current scene time.
    ///
    /// Multiple calls in the same frame schedule multiple steps. If automatic
    /// updating is enabled, one explicit call shares its automatic step instead
    /// of adding a duplicate.
    pub fn update(&self) {
        schedule_update(self);
    }
}

/// A frame-dependent simulation placed in a three-dimensional scene.
///
/// Create one with [`simulation_3d`]. It behaves like a regular spatial object
/// and contains [`Transform3D`], [`Draw3D`], and scene-node metadata.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "simulation_3d")]
pub struct Simulation3D {
    #[trackable]
    pub simulation: Simulation,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Simulation3D {
    fn default() -> Self {
        Self {
            simulation: Simulation::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_simulation_3d,
                get_box: |world, entity| world.get::<&Simulation>(entity).unwrap().box_3d(),
                ..Draw3D::default()
            },
        }
    }
}

impl Simulation3DBuilder {
    /// Sets the state advanced and drawn by this simulation.
    pub fn state<S: SimulationState3D>(mut self, state: S) -> Self {
        self.object.simulation = Simulation::new_3d(state);
        self
    }
}

impl Simulation3DHandler {
    /// Schedules one explicit simulation step at the current scene time.
    ///
    /// Multiple calls in the same frame schedule multiple steps. If automatic
    /// updating is enabled, one explicit call shares its automatic step instead
    /// of adding a duplicate.
    pub fn update(&self) {
        schedule_update(self);
    }
}

fn schedule_update(handler: &impl ObjectHandler) {
    let animator = handler.object_animator();
    animator.assert_finite_scope();
    handler
        .object_world()
        .borrow()
        .get::<&mut Simulation>(handler.get_id())
        .expect("Simulation handler must contain a Simulation component.")
        .schedule_update(animator.time());
}

fn draw_simulation_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let previous = context.set_current_transform(global_matrix3d(world, entity));
    let result = world.get::<&Simulation>(entity).unwrap().draw_3d(context);
    context.set_current_transform(previous);
    result
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::prelude::*;

    #[derive(Clone)]
    struct Counter {
        steps: u64,
        observed: Arc<Mutex<u64>>,
    }

    #[derive(Clone)]
    struct Solid;

    impl SimulationState for Solid {
        fn on_update(&mut self, _dt: f32) {}
    }

    impl SimulationState2D for Solid {
        fn on_draw(&self, canvas: &skia_safe::Canvas) {
            canvas.draw_rect(
                skia_safe::Rect::from_xywh(-4.0, -4.0, 8.0, 8.0),
                &skia_safe::Paint::new(skia_safe::colors::WHITE, None),
            );
        }

        fn get_box(&self) -> Vector2 {
            Vector2::splat(8.0)
        }
    }

    impl SimulationState for Counter {
        fn on_update(&mut self, _dt: f32) {
            self.steps += 1;
            *self.observed.lock().unwrap() = self.steps;
        }
    }

    impl SimulationState2D for Counter {
        fn on_draw(&self, _canvas: &skia_safe::Canvas) {
            *self.observed.lock().unwrap() = self.steps;
        }

        fn get_box(&self) -> glam::Vec2 {
            glam::Vec2::splat(10.0)
        }
    }

    fn draw(scene: &Scene) {
        let mut surface = skia_safe::surfaces::raster_n32_premul((16, 16)).unwrap();
        scene.draw(surface.canvas());
    }

    #[test]
    fn scene_replays_the_historical_auto_update_track_when_seeking() {
        let observed = Arc::new(Mutex::new(0));
        let mut scene = Scene::new();
        struct Setup(Arc<Mutex<u64>>);

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let simulation = simulation_2d()
                    .state(Counter {
                        steps: 0,
                        observed: Arc::clone(&self.0),
                    })
                    .position(vec2(100.0, 0.0))
                    .build(scene);
                scene.get_world_2d().add(&simulation);
                scene.wait(0.3);
                simulation.auto_update(false).immediate();
                scene.wait(0.3);
            }
        }

        scene.build(&mut Setup(Arc::clone(&observed)));
        scene.set_fps(10);

        scene.update(0.6);
        draw(&scene);
        assert_eq!(*observed.lock().unwrap(), 3);

        scene.update(0.1);
        draw(&scene);
        assert_eq!(*observed.lock().unwrap(), 2);

        scene.update(0.6);
        draw(&scene);
        assert_eq!(*observed.lock().unwrap(), 3);
    }

    #[test]
    fn handler_can_schedule_multiple_steps_in_one_frame() {
        let observed = Arc::new(Mutex::new(0));
        let mut scene = Scene::new();
        struct Setup(Arc<Mutex<u64>>);

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let simulation = simulation_2d()
                    .state(Counter {
                        steps: 0,
                        observed: Arc::clone(&self.0),
                    })
                    .build(scene);
                scene.get_world_2d().add(&simulation);
                simulation.auto_update(false).immediate();
                for _ in 0..4 {
                    simulation.update();
                }
            }
        }

        scene.build(&mut Setup(Arc::clone(&observed)));
        scene.update(0.0);
        draw(&scene);

        assert_eq!(*observed.lock().unwrap(), 4);
    }

    #[test]
    fn simulation_2d_inherits_group_transform_and_composited_opacity() {
        let mut scene = Scene::new();
        struct Setup;

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let group = group_2d()
                    .position(vec2(4.0, 0.0))
                    .opacity(0.5)
                    .build(scene);
                let simulation = simulation_2d()
                    .state(Solid)
                    .position(vec2(4.0, 0.0))
                    .opacity(0.5)
                    .build(scene);
                group.add(&simulation);
                scene.get_world_2d().add(&group);
            }
        }

        scene.build(&mut Setup);
        scene.update(0.0);
        let image_info = skia_safe::ImageInfo::new(
            (32, 32),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((16.0, 16.0));
        scene.draw(surface.canvas());

        let pixels = surface.peek_pixels().unwrap();
        let inside = pixels.get_color((24, 16));
        assert_eq!(inside.r(), 255);
        assert!((63..=64).contains(&inside.a()));
        assert_eq!(pixels.get_color((16, 16)).a(), 0);
    }
}
