use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw2D, Filter, Style, Transform2D, draw_styled_path, styled_bounds},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
#[trackable(name = "Triangle Shape")]
/// Geometry of a triangle with a horizontally adjustable apex.
pub struct TriangleShape {
    /// Base width and height in local coordinates.
    #[track]
    pub size: Vector2,
    /// Horizontal apex position from left (0) to right (1).
    #[track(min = 0.0, max = 1.0)]
    pub apex: f32,
}

impl Default for TriangleShape {
    fn default() -> Self {
        Self {
            size: vec2(256.0, 256.0),
            apex: 0.5,
        }
    }
}

fn triangle_path(shape: &TriangleShape) -> skia_safe::Path {
    let half = shape.size * 0.5;
    let points = [
        skia_safe::Point::new(-half.x, half.y),
        skia_safe::Point::new((shape.apex - 0.5) * shape.size.x, -half.y),
        skia_safe::Point::new(half.x, half.y),
    ];

    skia_safe::Path::polygon(&points, true, None, None)
}

#[derive(Object)]
#[object(spatial = "2d", builder = "triangle")]
/// Built-in triangle scene object with an independently animatable apex.
pub struct Triangle {
    #[trackable]
    pub shape: TriangleShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
    #[trackable]
    pub filter: Filter,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&TriangleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let path = triangle_path(&shape);

                    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
                },
                box_size: |world, entity| world.get::<&TriangleShape>(entity).unwrap().size.abs(),
                visual_bounds: |world, entity| {
                    let shape = world.get::<&TriangleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();

                    styled_bounds(*triangle_path(&shape).bounds(), &style, transform.scale)
                },
                ..Default::default()
            },
            filter: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        Scene,
        objects::{two_d::triangle::triangle_path, *},
        types::vec2,
    };

    #[test]
    fn apex_moves_the_tip_without_changing_the_base_or_bounds() {
        let mut scene = Scene::new();
        let triangle = triangle().size(vec2(200.0, 100.0)).build(&mut scene);

        assert_eq!(triangle.get(TriangleShape::apex_property()), 0.5);
        assert_eq!(triangle.box_size(), vec2(200.0, 100.0));

        for (requested, expected) in [(-1.0, 0.0), (0.5, 0.5), (2.0, 1.0)] {
            triangle.apex(requested).immediate();

            let world = scene.world();
            let shape = world.get::<&TriangleShape>(triangle.entity()).unwrap();
            let path = triangle_path(&shape);
            let points = path.points();

            assert_eq!(shape.apex, expected);
            assert_eq!(points[0], skia_safe::Point::new(-100.0, 50.0));
            assert_eq!(
                points[1],
                skia_safe::Point::new((expected - 0.5) * 200.0, -50.0)
            );
            assert_eq!(points[2], skia_safe::Point::new(100.0, 50.0));
            assert_eq!(
                *path.bounds(),
                skia_safe::Rect::from_ltrb(-100.0, -50.0, 100.0, 50.0)
            );
        }
    }
}
