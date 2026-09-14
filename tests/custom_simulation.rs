use kinematic::{hecs, prelude::*};

#[derive(Clone, Trackable)]
struct Appearance {
    #[track]
    color: Color,
}

#[derive(Clone)]
struct State;

impl SimulationState for State {
    fn on_update(&mut self, _context: &SimulationContext<'_>) {}
}

impl SimulationState2D for State {
    fn on_draw(&mut self, world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
        let color = world.get::<&Appearance>(entity).unwrap().color;
        canvas.clear(skia_safe::Color4f::new(color.r, color.g, color.b, color.a));
    }

    fn get_box(&self, _world: &hecs::World, _entity: hecs::Entity) -> Vector2 {
        Vector2::ONE
    }
}

#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "custom_simulation")]
struct CustomSimulation {
    #[trackable]
    simulation: Simulation,
    #[trackable]
    appearance: Appearance,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl Default for CustomSimulation {
    fn default() -> Self {
        Self {
            simulation: Simulation::new_2d(State),
            appearance: Appearance {
                color: Color::WHITE,
            },
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, _opacity| {
                    world
                        .get::<&mut Simulation>(entity)
                        .unwrap()
                        .draw_2d(world, entity, canvas);
                },
                get_box: |world, entity| {
                    world
                        .get::<&Simulation>(entity)
                        .unwrap()
                        .box_2d(world, entity)
                },
                ..Draw2D::default()
            },
        }
    }
}

impl CustomSimulationHandler {
    fn update(&self) {
        schedule_simulation_update(self);
    }
}

#[test]
fn public_simulation_api_supports_custom_trackable_components() {
    let mut scene = Scene::new();
    let simulation = custom_simulation()
        .auto_update(false)
        .color(Color::CYAN)
        .build(&mut scene);
    scene.get_world_2d().add(&simulation);
    simulation.update();
    scene.update(0.0);

    assert_eq!(simulation.get_color(), Color::CYAN);
    assert_eq!(simulation.get_box(), Vector2::ONE);

    let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();
    scene.draw(surface.canvas());
    assert_eq!(surface.peek_pixels().unwrap().get_color((0, 0)).g(), 255);
}
