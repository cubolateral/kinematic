use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw, ParticleStyle, Style, Transform, draw_complete_styled_path, draw_styled_path,
        stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of a triangular object.
pub struct TriangleShape {
    #[track]
    pub size: Vector2,
}

impl Default for TriangleShape {
    fn default() -> Self {
        Self {
            size: vec2(256.0, 256.0),
        }
    }
}

#[derive(Object, hecs::Bundle)]
/// Built-in triangular scene object.
pub struct Triangle {
    #[trackable]
    pub shape: TriangleShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub particles: ParticleStyle,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            particles: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&TriangleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let particles = world.get::<&ParticleStyle>(entity).unwrap();
                    let transform = world.get::<&Transform>(entity).unwrap();
                    let half_size = shape.size * 0.5;
                    let path = skia_safe::Path::polygon(
                        &[
                            (0.0, -half_size.y).into(),
                            (-half_size.x, half_size.y).into(),
                            (half_size.x, half_size.y).into(),
                        ],
                        true,
                        None,
                        None,
                    );

                    if particles.particles_enabled && style.progress < 1.0 {
                        let stroke_padding =
                            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale)
                                * 0.5;
                        let bounds = skia_safe::Rect::new(
                            -half_size.x - stroke_padding,
                            -half_size.y - stroke_padding,
                            half_size.x + stroke_padding,
                            half_size.y + stroke_padding,
                        );
                        let visual_key = particle_visual_key(
                            "Triangle",
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
                            bounds,
                            visual_key,
                            style: &style,
                            particles: &particles,
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
                get_box: |world, entity| world.get::<&TriangleShape>(entity).unwrap().size,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Scene, objects::*, types::vec2};

    #[test]
    fn triangle_alias_builds_a_sized_triangle() {
        let mut scene = Scene::new();
        let builder: TriangleBuilder = triangle().size(vec2(320.0, 180.0));
        let triangle = builder.build(&mut scene);

        assert_eq!(triangle.get_box(), vec2(320.0, 180.0));
        assert_eq!(
            scene
                .get_world()
                .get::<&TriangleShape>(triangle.get_id())
                .unwrap()
                .size,
            vec2(320.0, 180.0)
        );
    }
}
