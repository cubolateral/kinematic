use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw, Morph, Style, Transform, draw_complete_styled_path, draw_styled_path,
        stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of a line, including optional arrowheads at either end.
pub struct LineShape {
    /// Starting point in local coordinates.
    #[track]
    pub from: Vector2,
    /// Ending point in local coordinates.
    #[track]
    pub to: Vector2,
    /// Width of the line body.
    #[track]
    pub thickness: f32,
    /// Size of the arrowhead at the starting point.
    #[track]
    pub from_arrow_size: f32,
    /// Size of the arrowhead at the ending point.
    #[track]
    pub to_arrow_size: f32,
}

impl Default for LineShape {
    fn default() -> Self {
        Self {
            from: vec2(-128.0, 0.0),
            to: vec2(128.0, 0.0),
            thickness: 8.0,
            from_arrow_size: 0.0,
            to_arrow_size: 0.0,
        }
    }
}

struct LineGeometry {
    path: skia_safe::Path,
    bounds: skia_safe::Rect,
}

fn line_geometry(shape: &LineShape) -> Option<LineGeometry> {
    let offset = shape.to - shape.from;
    let length = offset.length();
    let thickness = shape.thickness.max(0.0);

    if !length.is_finite() || length <= f32::EPSILON || thickness <= 0.0 {
        return None;
    }

    let direction = offset / length;
    let normal = vec2(-direction.y, direction.x);
    let half_thickness = thickness * 0.5;
    let from_arrow_size = shape.from_arrow_size.max(0.0);
    let to_arrow_size = shape.to_arrow_size.max(0.0);
    let requested_arrow_length = from_arrow_size + to_arrow_size;
    let arrow_scale = if requested_arrow_length > length {
        length / requested_arrow_length
    } else {
        1.0
    };
    let from_length = from_arrow_size * arrow_scale;
    let to_length = to_arrow_size * arrow_scale;
    let from_half_width = (from_length * 0.5).max(half_thickness);
    let to_half_width = (to_length * 0.5).max(half_thickness);
    let body_from = shape.from + direction * from_length;
    let body_to = shape.to - direction * to_length;
    let mut points = Vec::with_capacity(10);

    if from_length > 0.0 {
        points.push(shape.from);
        points.push(body_from + normal * from_half_width);
    } else {
        points.push(shape.from + normal * half_thickness);
    }

    points.push(body_from + normal * half_thickness);
    points.push(body_to + normal * half_thickness);

    if to_length > 0.0 {
        points.push(body_to + normal * to_half_width);
        points.push(shape.to);
        points.push(body_to - normal * to_half_width);
    } else {
        points.push(shape.to - normal * half_thickness);
    }

    points.push(body_to - normal * half_thickness);
    points.push(body_from - normal * half_thickness);

    if from_length > 0.0 {
        points.push(body_from - normal * from_half_width);
    }

    let left = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let top = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let right = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let bottom = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let path_points: Vec<_> = points
        .iter()
        .map(|point| skia_safe::Point::new(point.x, point.y))
        .collect();

    Some(LineGeometry {
        path: skia_safe::Path::polygon(&path_points, true, None, None),
        bounds: skia_safe::Rect::new(left, top, right, bottom),
    })
}

fn line_box(shape: &LineShape) -> Vector2 {
    let Some(geometry) = line_geometry(shape) else {
        return Vector2::ZERO;
    };
    let horizontal_extent = geometry.bounds.left.abs().max(geometry.bounds.right.abs());
    let vertical_extent = geometry.bounds.top.abs().max(geometry.bounds.bottom.abs());

    vec2(horizontal_extent * 2.0, vertical_extent * 2.0)
}

#[derive(Object, hecs::Bundle)]
/// Built-in line scene object with independently animatable arrowheads.
pub struct Line {
    #[trackable]
    pub shape: LineShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&LineShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let morph = world.get::<&Morph>(entity).unwrap();
                    let transform = world.get::<&Transform>(entity).unwrap();
                    let Some(geometry) = line_geometry(&shape) else {
                        return;
                    };

                    if morph.particles_enabled && morph.progress < 1.0 {
                        let stroke_padding =
                            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale)
                                * 0.5;
                        let bounds = skia_safe::Rect::new(
                            geometry.bounds.left - stroke_padding,
                            geometry.bounds.top - stroke_padding,
                            geometry.bounds.right + stroke_padding,
                            geometry.bounds.bottom + stroke_padding,
                        );
                        let visual_key = particle_visual_key(
                            "Line",
                            &style,
                            &[
                                shape.from.x,
                                shape.from.y,
                                shape.to.x,
                                shape.to.y,
                                shape.thickness,
                                shape.from_arrow_size,
                                shape.to_arrow_size,
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
                            morph: &morph,
                            opacity,
                            canvas,
                        })
                        .render(|target, target_opacity| {
                            draw_complete_styled_path(
                                &geometry.path,
                                &style,
                                transform.scale,
                                target_opacity,
                                target,
                            );
                        }) {
                            return;
                        }
                    }

                    draw_styled_path(&geometry.path, &style, transform.scale, opacity, canvas);
                },
                get_box: |world, entity| line_box(&world.get::<&LineShape>(entity).unwrap()),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::line_geometry;
    use crate::core::{Scene, objects::*, types::vec2};

    #[test]
    fn line_alias_builds_the_configured_line() {
        let mut scene = Scene::new();
        let line = line()
            .from(vec2(-100.0, 20.0))
            .to(vec2(100.0, 20.0))
            .thickness(10.0)
            .from_arrow_size(10.0)
            .to_arrow_size(40.0)
            .build(&mut scene);
        let world = scene.get_world();
        let shape = world.get::<&LineShape>(line.get_id()).unwrap();

        assert_eq!(shape.from, vec2(-100.0, 20.0));
        assert_eq!(shape.to, vec2(100.0, 20.0));
        assert_eq!(shape.thickness, 10.0);
        assert_eq!(shape.from_arrow_size, 10.0);
        assert_eq!(shape.to_arrow_size, 40.0);
    }

    #[test]
    fn negative_arrow_sizes_are_ignored_when_building_geometry() {
        let shape = LineShape {
            from_arrow_size: -10.0,
            to_arrow_size: 32.0,
            ..Default::default()
        };
        let geometry = line_geometry(&shape).unwrap();

        assert_eq!(geometry.bounds.left, -128.0);
        assert_eq!(geometry.bounds.right, 128.0);
        assert_eq!(geometry.bounds.top, -16.0);
        assert_eq!(geometry.bounds.bottom, 16.0);
    }
}
