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
    fn on_draw(&self, canvas: &skia_safe::Canvas) {
        let origin = -(SIZE as f32 * CELL_2D) / 2.0;
        let paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.25, 0.85, 1.0, 1.0), None);
        for (y, row) in self.cells.iter().enumerate() {
            for (x, alive) in row.iter().enumerate() {
                if *alive {
                    canvas.draw_rect(
                        skia_safe::Rect::from_xywh(
                            x as f32 * CELL_2D + origin,
                            y as f32 * CELL_2D + origin,
                            CELL_2D - 2.0,
                            CELL_2D - 2.0,
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
    fn on_draw(&self, context: &mut RenderContext3D<'_>) -> Result<(), String> {
        let origin = -(SIZE as f32 - 1.0) * CELL_3D / 2.0;
        let material = Material {
            albedo: Color::new(0.25, 0.85, 1.0, 1.0),
            metallic: 0.15,
            roughness: 0.45,
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
                        Vector3::splat(CELL_3D * 0.42),
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

#[scene]
fn game_of_life_2d(s: &mut Scene) {
    let life = simulation_2d()
        .state(Life::glider())
        .auto_update(false)
        .build(s);
    s.get_world_2d().add(&life);

    for _ in 0..32 {
        life.update();
        s.wait(0.1);
    }

    s.wait(1.0);
}

#[scene]
fn game_of_life_3d(s: &mut Scene) {
    s.get_root().view_2d(false).immediate();
    let life = simulation_3d()
        .state(Life::glider())
        .auto_update(false)
        .rotation(Quaternion::from_rotation_y(0.35))
        .build(s);
    s.get_world_3d().add(&life);

    for _ in 0..32 {
        life.update();
        s.wait(0.1);
    }

    s.wait(1.0);
}

fn main() {
    app()
        .project("Game of Life", vec![game_of_life_2d, game_of_life_3d])
        .run();
}
