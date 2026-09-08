use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw2D, Morph, PARTICLE_COUNT, Style, Transform2D, draw_complete_styled_path,
        draw_styled_path, stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
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
                    let path = skia_safe::Path::rect(rect, None);
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
                            particle_count: PARTICLE_COUNT as usize,
                            style: &style,
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
