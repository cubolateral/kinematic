use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw2D, Morph, PARTICLE_COUNT, Style, Transform2D, draw_complete_styled_path,
        draw_styled_path, stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key},
    types::Vector2,
};

#[derive(Clone, Trackable)]
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
#[object(spatial = "2d", builder = "circle")]
#[morph]
/// Built-in circular scene object.
pub struct Circle {
    #[trackable]
    pub shape: CircleShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&CircleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let morph = world.get::<&Morph>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let path = skia_safe::Path::circle((0.0, 0.0), shape.radius, None);
                    if morph.particles_enabled && morph.progress < 1.0 {
                        let stroke_padding =
                            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale)
                                * 0.5;
                        let extent = shape.radius + stroke_padding;
                        let bounds = skia_safe::Rect::new(-extent, -extent, extent, extent);
                        let visual_key = particle_visual_key(
                            "Circle",
                            &style,
                            &[shape.radius, transform.scale.x, transform.scale.y],
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
                get_box: |world, entity| {
                    let shape = world.get::<&CircleShape>(entity).unwrap();
                    Vector2::splat(shape.radius * 2.0)
                },
                ..Default::default()
            },
        }
    }
}
