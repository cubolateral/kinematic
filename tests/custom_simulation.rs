use std::sync::{Arc, Mutex};

use kinematic::{hecs, prelude::*};

#[derive(Clone, Trackable)]
struct Settings {
    #[track]
    color: Color,
}

#[derive(Clone, Debug, PartialEq)]
enum Event {
    Mutation(usize, bool),
    Update(Vec<bool>),
}

#[derive(Clone)]
struct State {
    cells: Vec<bool>,
    events: Vec<Event>,
    observed: Arc<Mutex<(Vec<bool>, Vec<Event>)>>,
}

impl State {
    fn new(observed: Arc<Mutex<(Vec<bool>, Vec<Event>)>>) -> Self {
        Self {
            cells: vec![false; 4],
            events: Vec::new(),
            observed,
        }
    }

    fn set_cell(&mut self, index: usize, alive: bool) {
        self.cells[index] = alive;
        self.events.push(Event::Mutation(index, alive));
    }

    fn observe(&self) {
        *self.observed.lock().unwrap() = (self.cells.clone(), self.events.clone());
    }
}

impl SimulationState for State {
    fn on_update(&mut self, _context: &SimulationContext<'_>) {
        self.events.push(Event::Update(self.cells.clone()));
    }
}

impl SimulationState2D for State {
    fn on_draw(&mut self, _world: &hecs::World, _entity: hecs::Entity, canvas: &skia_safe::Canvas) {
        self.observe();
        canvas.clear(skia_safe::colors::WHITE);
    }

    fn box_size(&self, _world: &hecs::World, _entity: hecs::Entity) -> Vector2 {
        Vector2::ONE
    }
}

impl SimulationState3D for State {
    fn on_draw(
        &mut self,
        _world: &hecs::World,
        _entity: hecs::Entity,
        _context: &mut RenderContext3D<'_>,
    ) -> Result<(), String> {
        self.observe();
        Ok(())
    }

    fn box_size(&self, _world: &hecs::World, _entity: hecs::Entity) -> Vector3 {
        Vector3::ONE
    }
}

#[derive(Object)]
#[object(spatial = "2d", builder = "ca_2d", simulation = State)]
struct CA2D {
    #[trackable]
    settings: Settings,
    #[trackable]
    simulation: Simulation,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl CA2D {
    fn new(observed: Arc<Mutex<(Vec<bool>, Vec<Event>)>>) -> Self {
        Self {
            settings: Settings {
                color: Color::WHITE,
            },
            simulation: Simulation::new_2d(State::new(observed)),
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: Simulation::draw_2d,
                box_size: Simulation::box_2d,
                ..Draw2D::default()
            },
        }
    }
}

impl Default for CA2D {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new((Vec::new(), Vec::new()))))
    }
}

impl CA2DHandler {
    fn set_cell(&self, index: usize, alive: bool) {
        self.write_simulation(move |state| state.set_cell(index, alive));
    }
}

#[derive(Object)]
#[object(spatial = "3d", builder = "ca_3d", simulation = State)]
struct CA3D {
    #[trackable]
    simulation: Simulation,
    #[trackable]
    transform: Transform3D,
    #[trackable]
    draw: Draw3D,
}

impl Default for CA3D {
    fn default() -> Self {
        Self {
            simulation: Simulation::new_3d(State::new(Arc::new(Mutex::new((
                Vec::new(),
                Vec::new(),
            ))))),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: Simulation::draw_3d,
                box_size: Simulation::box_3d,
                ..Draw3D::default()
            },
        }
    }
}

#[derive(Object)]
#[object(spatial = "2d", builder = "custom_draw", simulation = State)]
struct CustomDraw {
    #[trackable]
    simulation: Simulation,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl Default for CustomDraw {
    fn default() -> Self {
        Self {
            simulation: Simulation::new(State::new(Arc::new(Mutex::new((Vec::new(), Vec::new()))))),
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: |_world, _entity, canvas, _opacity| {
                    canvas.clear(skia_safe::colors::CYAN);
                },
                box_size: |_, _| Vector2::ONE,
                ..Draw2D::default()
            },
        }
    }
}

fn draw(scene: &Scene) {
    let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();
    scene.draw(surface.canvas());
}

#[test]
fn custom_object_preserves_mutation_update_order_and_seek() {
    let observed = Arc::new(Mutex::new((Vec::new(), Vec::new())));
    let mut scene = Scene::new();
    let ca = scene.spawn_object(CA2D::new(Arc::clone(&observed)), "CA");
    scene.world_2d().add(&ca);
    ca.set_auto_update(false);
    ca.set_color(Color::CYAN);

    ca.set_cell(0, true);
    ca.update();
    ca.set_cell(1, true);
    scene.wait(3.0);
    ca.set_cell(2, true);
    ca.update();

    scene.update(3.0);
    draw(&scene);
    let cells = ca.read_simulation(|state| state.cells.clone());
    let expected = observed.lock().unwrap().clone();
    assert_eq!(cells, vec![true, true, true, false]);
    assert_eq!(ca.get_color(), Color::CYAN);
    assert_eq!(ca.box_size(), Vector2::ONE);
    assert_eq!(
        expected.1,
        vec![
            Event::Mutation(0, true),
            Event::Update(vec![true, false, false, false]),
            Event::Mutation(1, true),
            Event::Mutation(2, true),
            Event::Update(vec![true, true, true, false]),
        ]
    );

    scene.update(0.0);
    draw(&scene);
    assert_eq!(observed.lock().unwrap().0, vec![true, true, false, false]);
    scene.update(3.0);
    draw(&scene);
    assert_eq!(*observed.lock().unwrap(), expected);
}

#[test]
fn auto_update_runs_after_same_frame_mutations() {
    let observed = Arc::new(Mutex::new((Vec::new(), Vec::new())));
    let mut scene = Scene::new();
    let ca = scene.spawn_object(CA2D::new(Arc::clone(&observed)), "CA");
    let label = text_2d().build(&mut scene);
    scene.world_2d().add(&ca);
    scene.world_2d().add(&label);
    ca.set_cell(0, true);
    let signal_label = label.clone();
    ca.signal(move |ca, _frame| {
        let alive = ca.read_simulation(|state| state.cells.iter().filter(|cell| **cell).count());
        signal_label.set_text(format!("Alive: {alive}"));
    });

    scene.update(0.0);
    draw(&scene);
    assert_eq!(label.get_text(), "Alive: 1");
    assert_eq!(
        observed.lock().unwrap().1,
        vec![
            Event::Mutation(0, true),
            Event::Update(vec![true, false, false, false]),
        ]
    );
}

#[test]
fn simulation_drawing_is_opt_in_for_2d_3d_and_custom_callbacks() {
    let mut scene = Scene::new();
    let ca_2d = ca_2d()
        .color(Color::MAGENTA)
        .auto_update(false)
        .build(&mut scene);
    assert_eq!(ca_2d.get_color(), Color::MAGENTA);

    let ca_3d = ca_3d().build(&mut scene);
    assert_eq!(ca_3d.box_size(), Vector3::ONE);

    let custom = custom_draw().build(&mut scene);
    scene.world_2d().add(&custom);
    scene.update(0.0);
    let mut surface = skia_safe::surfaces::raster_n32_premul((1, 1)).unwrap();
    scene.draw(surface.canvas());
    assert_eq!(surface.peek_pixels().unwrap().get_color((0, 0)).g(), 255);
}
