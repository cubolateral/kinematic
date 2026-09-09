use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw2D, Morph, PARTICLE_COUNT, Style, Transform2D, draw_complete_styled_path,
        draw_styled_path, stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key, regular_polygon_vertices},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of a regular polygon.
pub struct RegularPolygon2DShape {
    #[track]
    pub size: Vector2,
    #[track]
    pub sides: u32,
}

impl Default for RegularPolygon2DShape {
    fn default() -> Self {
        Self {
            size: vec2(256.0, 256.0),
            sides: 4,
        }
    }
}

fn polygon_points(shape: &RegularPolygon2DShape) -> Option<Vec<skia_safe::Point>> {
    Some(
        regular_polygon_vertices(shape.sides)?
            .into_iter()
            .map(|point| skia_safe::Point::new(point.x * shape.size.x, point.y * shape.size.y))
            .collect(),
    )
}

#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "regular_polygon_2d")]
#[morph]
/// Built-in regular polygon scene object.
pub struct RegularPolygon2D {
    #[trackable]
    pub shape: RegularPolygon2DShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for RegularPolygon2D {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&RegularPolygon2DShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let morph = world.get::<&Morph>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let Some(points) = polygon_points(&shape) else {
                        return;
                    };
                    let path = skia_safe::Path::polygon(&points, true, None, None);
                    let half_size = shape.size.abs() * 0.5;

                    if morph.particles_enabled && morph.progress < 1.0 {
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
                            "RegularPolygon2D",
                            &style,
                            &[
                                shape.size.x,
                                shape.size.y,
                                shape.sides as f32,
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
                get_box: |world, entity| {
                    world
                        .get::<&RegularPolygon2DShape>(entity)
                        .unwrap()
                        .size
                        .abs()
                },
                ..Default::default()
            },
        }
    }
}

/// Creates a three-sided [`RegularPolygon2D`] builder.
pub fn triangle() -> RegularPolygon2DBuilder {
    regular_polygon_2d().sides(3)
}

/// Creates a four-sided [`RegularPolygon2D`] builder.
pub fn square() -> RegularPolygon2DBuilder {
    regular_polygon_2d().sides(4)
}

#[cfg(test)]
mod tests {
    use super::polygon_points;
    use crate::core::{Scene, objects::*, types::vec2};

    #[test]
    fn regular_polygon_2d_builder_sets_trackable_sides() {
        let mut scene = Scene::new();
        let polygon = regular_polygon_2d()
            .size(vec2(320.0, 180.0))
            .sides(3)
            .build(&mut scene);

        assert_eq!(polygon.get_box(), vec2(320.0, 180.0));
        assert_eq!(polygon.get(RegularPolygon2DShape::sides_property()), 3);
        assert_eq!(
            scene
                .get_world()
                .get::<&RegularPolygon2DShape>(polygon.get_id())
                .unwrap()
                .size,
            vec2(320.0, 180.0)
        );
    }

    #[test]
    fn regular_polygon_2d_defaults_to_four_sides_and_rejects_less_than_three() {
        let default = RegularPolygon2DShape::default();

        assert_eq!(default.sides, 4);
        assert_eq!(polygon_points(&default).unwrap().len(), 4);
        assert!(
            polygon_points(&RegularPolygon2DShape {
                sides: 2,
                ..default
            })
            .is_none()
        );
    }

    #[test]
    fn polygon_presets_set_triangle_and_square_sides() {
        let mut scene = Scene::new();
        let triangle = triangle().build(&mut scene);
        let square = square().build(&mut scene);

        assert_eq!(triangle.get(RegularPolygon2DShape::sides_property()), 3);
        assert_eq!(square.get(RegularPolygon2DShape::sides_property()), 4);
    }
}
