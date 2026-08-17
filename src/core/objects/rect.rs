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
                on_draw: |world, entity, vg| {
                    let shape = world.get::<&RectShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform>(entity).unwrap();
                    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();

                    vg.save();
                    vg.translate(transform.position.x, transform.position.y);
                    vg.rotate(transform.rotation);
                    vg.scale(transform.scale.x, transform.scale.y);

                    let mut path = femtovg::Path::new();
                    path.rect(
                        -shape.size.x * 0.5,
                        -shape.size.y * 0.5,
                        shape.size.x,
                        shape.size.y,
                    );

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

                    vg.restore();
                },
                ..Default::default()
            },
        }
    }
}
