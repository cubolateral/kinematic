use kinematic_macros::{Object, Trackable};

use crate::core::components::{Draw, Style, Transform};

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
    pub style: Style,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for CircleBundle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, vg| {
                    let shape = world.get::<&CircleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform>(entity).unwrap();
                    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();

                    let mut path = femtovg::Path::new();
                    path.circle(transform.position.x, transform.position.y, shape.radius);

                    vg.fill_path(
                        &path,
                        &femtovg::Paint::color(femtovg::Color::rgbaf(
                            fill_r, fill_g, fill_b, fill_a,
                        )),
                    );

                    if style.stroke_width > 0.0 {
                        let [stroke_r, stroke_g, stroke_b, stroke_a] = style.stroke.rgba();
                        vg.stroke_path(
                            &path,
                            &femtovg::Paint::color(femtovg::Color::rgbaf(
                                stroke_r, stroke_g, stroke_b, stroke_a,
                            ))
                            .with_line_width(style.stroke_width),
                        );
                    }
                },
                ..Default::default()
            },
        }
    }
}
