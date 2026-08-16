use kinematic_macros::{Object, Trackable};

use crate::core::components::{Draw, Transform};

#[derive(Trackable)]
/// Geometry of a circular object.
pub struct CircleShape {
    #[track]
    pub radius: f32,
}

impl Default for CircleShape {
    fn default() -> Self {
        Self { radius: 100.0 }
    }
}

#[derive(Object, hecs::Bundle)]
/// ECS bundle for the built-in circular scene object.
pub struct CircleBundle {
    #[trackable]
    pub shape: CircleShape,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for CircleBundle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, vg| {
                    let shape = world.get::<&CircleShape>(entity).unwrap();
                    let transform = world.get::<&Transform>(entity).unwrap();

                    let mut path = femtovg::Path::new();
                    path.circle(transform.x, transform.y, shape.radius);
                    vg.fill_path(&path, &femtovg::Paint::color(femtovg::Color::white()));
                },
                ..Default::default()
            },
        }
    }
}
