use kinematic::{prelude::*, three_d};

const SIZE: usize = 12;
const CELL_2D: f32 = 24.0;
const CELL_3D: f32 = 0.14;

#[derive(Clone)]
struct Life {
    cells: [[bool; SIZE]; SIZE],
}

impl Life {
    fn glider() -> Self {
        let mut cells = [[false; SIZE]; SIZE];
        for (x, y) in [(2, 1), (3, 2), (1, 3), (2, 3), (3, 3)] {
            cells[y][x] = true;
        }
        Self { cells }
    }
}

impl SimulationState for Life {
    fn on_update(&mut self, _dt: f32) {
        let previous = self.cells;
        for y in 0..SIZE {
            for x in 0..SIZE {
                let mut neighbors = 0;
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if (dx, dy) == (0, 0) {
                            continue;
                        }
                        let nx = (x as i32 + dx).rem_euclid(SIZE as i32) as usize;
                        let ny = (y as i32 + dy).rem_euclid(SIZE as i32) as usize;
                        neighbors += previous[ny][nx] as u8;
                    }
                }
                self.cells[y][x] = neighbors == 3 || (previous[y][x] && neighbors == 2);
            }
        }
    }
}

impl SimulationState2D for Life {
    fn on_draw(&self, world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
        let appearance = world.get::<&Life2DAppearance>(entity).unwrap();
        let origin = -(SIZE as f32 * CELL_2D) / 2.0;
        let [r, g, b, a] = appearance.color.rgba();
        let paint = skia_safe::Paint::new(skia_safe::Color4f::new(r, g, b, a), None);
        let cell_size = CELL_2D * appearance.cell_scale;
        let inset = (CELL_2D - cell_size) * 0.5;
        for (y, row) in self.cells.iter().enumerate() {
            for (x, alive) in row.iter().enumerate() {
                if *alive {
                    canvas.draw_rect(
                        skia_safe::Rect::from_xywh(
                            x as f32 * CELL_2D + origin + inset,
                            y as f32 * CELL_2D + origin + inset,
                            cell_size,
                            cell_size,
                        ),
                        &paint,
                    );
                }
            }
        }
    }

    fn get_box(&self) -> Vector2 {
        Vector2::splat(SIZE as f32 * CELL_2D)
    }
}

impl SimulationState3D for Life {
    fn on_draw(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
        context: &mut RenderContext3D<'_>,
    ) -> Result<(), String> {
        let appearance = world.get::<&Life3DAppearance>(entity).unwrap();
        let origin = -(SIZE as f32 - 1.0) * CELL_3D / 2.0;
        let material = Material {
            albedo: appearance.color,
            metallic: appearance.metallic,
            roughness: appearance.roughness,
            ..Material::default()
        };
        for (y, row) in self.cells.iter().enumerate() {
            for (x, alive) in row.iter().enumerate() {
                if *alive {
                    let position = vec3(
                        x as f32 * CELL_3D + origin,
                        -(y as f32 * CELL_3D + origin),
                        0.0,
                    );
                    let local = glam::Mat4::from_scale_rotation_translation(
                        Vector3::splat(CELL_3D * 0.5 * appearance.cell_scale),
                        Quaternion::IDENTITY,
                        position,
                    );
                    context.render_material(
                        GeometryKey::new::<Life>(0),
                        three_d::CpuMesh::cube,
                        local,
                        &material,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn get_box(&self) -> Vector3 {
        vec3(SIZE as f32 * CELL_3D, SIZE as f32 * CELL_3D, CELL_3D)
    }
}

#[derive(Clone, Trackable)]
struct Life2DAppearance {
    #[track]
    color: Color,
    #[track(min = 0.0, max = 1.0)]
    cell_scale: f32,
}

impl Default for Life2DAppearance {
    fn default() -> Self {
        Self {
            color: Color::new(0.25, 0.85, 1.0, 1.0),
            cell_scale: 0.9,
        }
    }
}

#[derive(Clone, Trackable)]
struct Life3DAppearance {
    #[track]
    color: Color,
    #[track(min = 0.0, max = 1.0)]
    cell_scale: f32,
    #[track(min = 0.0, max = 1.0)]
    metallic: f32,
    #[track(min = 0.0, max = 1.0)]
    roughness: f32,
}

impl Default for Life3DAppearance {
    fn default() -> Self {
        Self {
            color: Color::new(0.25, 0.85, 1.0, 1.0),
            cell_scale: 0.84,
            metallic: 0.15,
            roughness: 0.45,
        }
    }
}

#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "life_2d")]
struct Life2D {
    #[trackable]
    simulation: Simulation,
    #[trackable]
    appearance: Life2DAppearance,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl Default for Life2D {
    fn default() -> Self {
        Self {
            simulation: Simulation::new_2d(Life::glider()),
            appearance: Life2DAppearance::default(),
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, _opacity| {
                    world
                        .get::<&Simulation>(entity)
                        .unwrap()
                        .draw_2d(world, entity, canvas);
                },
                get_box: |world, entity| world.get::<&Simulation>(entity).unwrap().box_2d(),
                ..Draw2D::default()
            },
        }
    }
}

impl Life2DHandler {
    fn update(&self) {
        schedule_simulation_update(self);
    }
}

#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "life_3d")]
struct Life3D {
    #[trackable]
    simulation: Simulation,
    #[trackable]
    appearance: Life3DAppearance,
    #[trackable]
    transform: Transform3D,
    #[trackable]
    draw: Draw3D,
}

impl Default for Life3D {
    fn default() -> Self {
        Self {
            simulation: Simulation::new_3d(Life::glider()),
            appearance: Life3DAppearance::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_life_3d,
                get_box: |world, entity| world.get::<&Simulation>(entity).unwrap().box_3d(),
                ..Draw3D::default()
            },
        }
    }
}

impl Life3DHandler {
    fn update(&self) {
        schedule_simulation_update(self);
    }
}

fn draw_life_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let previous = context.set_current_transform(global_matrix3d(world, entity));
    let result = world
        .get::<&Simulation>(entity)
        .unwrap()
        .draw_3d(world, entity, context);
    context.set_current_transform(previous);
    result
}

#[scene]
fn game_of_life_2d(s: &mut Scene) {
    let life = life_2d()
        .auto_update(false)
        .color(Color::CYAN)
        .cell_scale(0.9)
        .build(s);
    s.get_world_2d().add(&life);

    for _ in 0..32 {
        life.update();
        s.wait(0.1);
    }

    life.color(Color::MAGENTA)
        .cell_scale(0.65)
        .duration(1.0)
        .play();
}

#[scene]
fn game_of_life_3d(s: &mut Scene) {
    s.get_root().view_2d(false).immediate();
    let life = life_3d()
        .auto_update(false)
        .color(Color::CYAN)
        .cell_scale(0.84)
        .metallic(0.15)
        .roughness(0.45)
        .rotation(Quaternion::from_rotation_y(0.35))
        .build(s);
    s.get_world_3d().add(&life);

    for _ in 0..32 {
        life.update();
        s.wait(0.1);
    }

    life.color(Color::MAGENTA)
        .cell_scale(0.65)
        .metallic(0.8)
        .roughness(0.2)
        .duration(1.0)
        .play();
}

fn main() {
    app()
        .project("Game of Life", vec![game_of_life_2d, game_of_life_3d])
        .run();
}
