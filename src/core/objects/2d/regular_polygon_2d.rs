use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw2D, Style, Transform2D, draw_styled_path, styled_bounds},
    objects::regular_polygon_vertices,
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of a regular polygon.
pub struct RegularPolygon2DShape {
    #[track]
    pub size: Vector2,
    #[track(min = 3, max = 256)]
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

#[derive(Object)]
#[object(spatial = "2d", builder = "regular_polygon_2d")]
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
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let Some(points) = polygon_points(&shape) else {
                        return;
                    };
                    let path = skia_safe::Path::polygon(&points, true, None, None);
                    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
                },
                box_size: |world, entity| {
                    world
                        .get::<&RegularPolygon2DShape>(entity)
                        .unwrap()
                        .size
                        .abs()
                },
                visual_bounds: |world, entity| {
                    let shape = world.get::<&RegularPolygon2DShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let Some(points) = polygon_points(&shape) else {
                        return skia_safe::Rect::default();
                    };
                    styled_bounds(
                        *skia_safe::Path::polygon(&points, true, None, None).bounds(),
                        &style,
                        transform.scale,
                    )
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

        assert_eq!(polygon.box_size(), vec2(320.0, 180.0));
        assert_eq!(polygon.get(RegularPolygon2DShape::sides_property()), 3);
        assert_eq!(
            scene
                .world()
                .get::<&RegularPolygon2DShape>(polygon.entity())
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
