use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw2D, Morph, Style, Transform2D, draw_complete_styled_path, draw_styled_path,
        stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key},
    types::{Quad, Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of a rectangular object.
pub struct RectShape {
    #[track]
    pub size: Vector2,
    #[track]
    pub round: Quad,
}

impl Default for RectShape {
    fn default() -> Self {
        Self {
            size: vec2(256.0, 256.0),
            round: Quad::default(),
        }
    }
}

pub(crate) fn rect_path(shape: &RectShape) -> skia_safe::Path {
    let rect = skia_safe::Rect::from_xywh(
        -shape.size.x * 0.5,
        -shape.size.y * 0.5,
        shape.size.x,
        shape.size.y,
    );
    let radii = shape.round.to_array().map(|radius| {
        let radius = if radius.is_finite() {
            radius.max(0.0)
        } else {
            0.0
        };
        skia_safe::Vector::new(radius, radius)
    });
    skia_safe::Path::rrect(skia_safe::RRect::new_rect_radii(rect, &radii), None)
}

#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "rect")]
#[morph]
/// Built-in rectangular scene object.
pub struct Rect {
    #[trackable]
    pub shape: RectShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&RectShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let morph = world.get::<&Morph>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();

                    let rect = skia_safe::Rect::from_xywh(
                        -shape.size.x * 0.5,
                        -shape.size.y * 0.5,
                        shape.size.x,
                        shape.size.y,
                    );
                    let path = rect_path(&shape);
                    if morph.particles_enabled && morph.progress < 1.0 {
                        let stroke_padding =
                            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale)
                                * 0.5;
                        let bounds = skia_safe::Rect::new(
                            rect.left - stroke_padding,
                            rect.top - stroke_padding,
                            rect.right + stroke_padding,
                            rect.bottom + stroke_padding,
                        );
                        let visual_key = particle_visual_key(
                            "Rect",
                            &style,
                            &[
                                shape.size.x,
                                shape.size.y,
                                shape.round.a,
                                shape.round.b,
                                shape.round.c,
                                shape.round.d,
                                transform.scale.x,
                                transform.scale.y,
                            ],
                            &[],
                        );

                        if (CreationDraw {
                            entity,
                            cache_slot: 0,
                            bounds,
                            visual_key,
                            style: &style,
                            pixel_color: None,
                            morph: &morph,
                            opacity,
                            canvas,
                        })
                        .render(|target, target_opacity| {
                            draw_complete_styled_path(
                                &path,
                                &style,
                                transform.scale,
                                target_opacity,
                                target,
                            );
                        }) {
                            return;
                        }
                    }

                    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
                },
                get_box: |world, entity| world.get::<&RectShape>(entity).unwrap().size,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn builder_accepts_corner_shorthand() {
        let mut scene = Scene::new();
        let values = [
            rect().round(10).build(&mut scene),
            rect().round([10, 30]).build(&mut scene),
            rect().round([10, 20, 30]).build(&mut scene),
            rect().round([10, 20, 30, 40]).build(&mut scene),
        ];
        let expected = [
            Quad::new(10.0, 10.0, 10.0, 10.0),
            Quad::new(10.0, 30.0, 10.0, 30.0),
            Quad::new(10.0, 20.0, 30.0, 20.0),
            Quad::new(10.0, 20.0, 30.0, 40.0),
        ];
        {
            let world = scene.get_world();
            for (rect, expected) in values.iter().zip(expected) {
                assert_eq!(
                    world.get::<&RectShape>(rect.get_id()).unwrap().round,
                    expected
                );
            }
        }

        values[0]
            .round(20.0)
            .round_from([10.0, 30.0], [10.0, 20.0, 30.0, 40.0])
            .round_a(5.0)
            .immediate();
    }

    #[test]
    fn rounded_rect_clears_its_corner() {
        let mut scene = Scene::new_with_resolution((64, 64));
        let rect = rect()
            .size(vec2(40.0, 40.0))
            .round(12)
            .position(vec2(32.0, 32.0))
            .fill(Color::RED)
            .build(&mut scene);
        scene.get_world_2d().add(&rect);
        scene.update(0.0);
        let mut surface = skia_safe::surfaces::raster_n32_premul((64, 64)).unwrap();

        scene.draw(surface.canvas());

        let pixels = surface.peek_pixels().unwrap();
        assert_eq!(pixels.get_color((12, 12)).a(), 0);
        assert_eq!(pixels.get_color((32, 32)).r(), 255);
    }
}
