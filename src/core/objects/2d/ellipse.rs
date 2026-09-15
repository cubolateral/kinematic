use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw2D, Style, Transform2D, draw_styled_path, styled_bounds},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of an elliptical object.
pub struct EllipseShape {
    #[track]
    pub radius: Vector2,
}

impl Default for EllipseShape {
    fn default() -> Self {
        Self {
            radius: vec2(128.0, 64.0),
        }
    }
}

#[derive(Object)]
#[object(spatial = "2d", builder = "ellipse")]
/// Built-in elliptical scene object.
pub struct Ellipse {
    #[trackable]
    pub shape: EllipseShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&EllipseShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let radius = shape.radius.abs();
                    let bounds = skia_safe::Rect::new(-radius.x, -radius.y, radius.x, radius.y);
                    let path = skia_safe::Path::oval(bounds, None);
                    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
                },
                box_size: |world, entity| {
                    world.get::<&EllipseShape>(entity).unwrap().radius.abs() * 2.0
                },
                visual_bounds: |world, entity| {
                    let radius = world.get::<&EllipseShape>(entity).unwrap().radius.abs();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    styled_bounds(
                        skia_safe::Rect::new(-radius.x, -radius.y, radius.x, radius.y),
                        &style,
                        transform.scale,
                    )
                },
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Scene, objects::*, types::vec2};

    #[test]
    fn ellipse_builder_sets_trackable_radii() {
        let mut scene = Scene::new();
        let ellipse = ellipse().radius(vec2(160.0, 80.0)).build(&mut scene);

        assert_eq!(ellipse.box_size(), vec2(320.0, 160.0));
        assert_eq!(
            ellipse.get(EllipseShape::radius_property()),
            vec2(160.0, 80.0)
        );
    }
}
