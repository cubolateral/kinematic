use kinematic_macros::{Object, Trackable};

use crate::core::components::{
    Draw2D, Filter, Style, Transform2D, draw_styled_path, styled_bounds,
};

#[derive(Clone, Trackable)]
/// Geometry of a circular object.
pub struct CircleShape {
    #[track(min = 0.0)]
    pub radius: f32,
}

impl Default for CircleShape {
    fn default() -> Self {
        Self { radius: 128.0 }
    }
}

#[derive(Object)]
#[object(spatial = "2d", builder = "circle")]
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
    #[trackable]
    pub filter: Filter,
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
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let path = skia_safe::Path::circle((0.0, 0.0), shape.radius, None);
                    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
                },
                box_size: |world, entity| {
                    let radius = world.get::<&CircleShape>(entity).unwrap().radius;
                    glam::Vec2::splat(radius * 2.0)
                },
                visual_bounds: |world, entity| {
                    let shape = world.get::<&CircleShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let radius = shape.radius.max(0.0);
                    styled_bounds(
                        skia_safe::Rect::new(-radius, -radius, radius, radius),
                        &style,
                        transform.scale,
                    )
                },
                ..Default::default()
            },
            filter: Default::default(),
        }
    }
}
