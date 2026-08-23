use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw, Style, Transform},
    types::{Vector2, vec2},
};

#[derive(Trackable)]
/// Geometry of a rectangular object.
pub struct RectShape {
    #[track]
    pub size: Vector2,
}

impl Default for RectShape {
    fn default() -> Self {
        Self {
            size: vec2(256.0, 256.0),
        }
    }
}

#[derive(Object, hecs::Bundle)]
/// Built-in rectangular scene object.
pub struct Rect {
    #[trackable]
    pub shape: RectShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, canvas| {
                    let shape = world.get::<&RectShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let draw = world.get::<&Draw>(entity).unwrap();
                    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();
                    let rect = skia_safe::Rect::from_xywh(
                        -shape.size.x * 0.5,
                        -shape.size.y * 0.5,
                        shape.size.x,
                        shape.size.y,
                    );
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

                    canvas.draw_rect(rect, &paint);

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
                        canvas.draw_rect(rect, &paint);
                    }
                },
                ..Default::default()
            },
        }
    }
}
