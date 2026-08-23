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
        Self { radius: 128.0 }
    }
}

#[derive(Object, hecs::Bundle)]
/// Built-in circular scene object.
pub struct Circle {
    #[trackable]
    pub shape: CircleShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, canvas| {
                    let shape = world.get::<&CircleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let draw = world.get::<&Draw>(entity).unwrap();
                    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();
                    let mut paint = skia_safe::Paint::new(
                        skia_safe::Color4f::new(
                            fill_r,
                            fill_g,
                            fill_b,
                            fill_a * draw.opacity.clamp(0.0, 1.0),
                        ),
                        None,
                    );
                    paint.set_anti_alias(true);

                    canvas.draw_circle((0.0, 0.0), shape.radius, &paint);

                    if style.stroke_width > 0.0 {
                        let [stroke_r, stroke_g, stroke_b, stroke_a] = style.stroke.rgba();
                        paint.set_color4f(
                            skia_safe::Color4f::new(
                                stroke_r,
                                stroke_g,
                                stroke_b,
                                stroke_a * draw.opacity.clamp(0.0, 1.0),
                            ),
                            None,
                        );
                        paint.set_style(skia_safe::PaintStyle::Stroke);
                        paint.set_stroke_width(style.stroke_width);
                        canvas.draw_circle((0.0, 0.0), shape.radius, &paint);
                    }
                },
                ..Default::default()
            },
        }
    }
}
