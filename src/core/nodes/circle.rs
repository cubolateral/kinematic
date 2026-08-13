use crate::core::components::{Node, Transform};

pub struct CircleShape {
    pub radius: f32,
}

impl Default for CircleShape {
    fn default() -> Self {
        Self { radius: 100.0 }
    }
}

#[derive(hecs::Bundle)]
pub struct Circle {
    pub shape: CircleShape,
    pub transform: Transform,
    pub node: Node,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            shape: CircleShape::default(),
            transform: Default::default(),
            node: Node {
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
