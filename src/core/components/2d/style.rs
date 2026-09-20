use kinematic_macros::Trackable;

use crate::core::types::{Color, Vector2};

const PATH_REBASE_THRESHOLD: f32 = 32_768.0;

/// Fill and stroke properties for a style entity.
#[derive(Clone, Trackable, Debug)]
pub struct Style {
    /// Color used to fill the style.
    #[track]
    pub fill: Color,
    /// Color used to outline the style.
    #[track]
    pub stroke: Color,
    /// Width used to outline the style.
    #[track(min = 0.0)]
    pub stroke_width: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Color::default(),
            stroke: Color::default(),
            stroke_width: 0.0,
        }
    }
}

/// Draws a closed path.
pub(crate) fn draw_styled_path(
    path: &skia_safe::Path,
    style: &Style,
    scale: Vector2,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    draw_complete_styled_path(path, style, scale, opacity, canvas);
}

/// Draws a complete closed path without applying style progress.
pub(crate) fn draw_complete_styled_path(
    path: &skia_safe::Path,
    style: &Style,
    scale: Vector2,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let matrix = canvas.local_to_device_as_3x3();
    let device_bounds = matrix.map_rect(*path.bounds()).0;
    let oversized = [
        device_bounds.left,
        device_bounds.top,
        device_bounds.right,
        device_bounds.bottom,
    ]
    .into_iter()
    .any(|value| value.abs() > PATH_REBASE_THRESHOLD);
    let rebased = oversized
        .then(|| {
            let bounds = skia_safe::Rect::from_irect(canvas.device_clip_bounds()?);
            let center = skia_safe::Point::new(bounds.center_x(), bounds.center_y());
            let anchor = matrix.invert()?.map_point(center);
            let shifted =
                path.with_transform(&skia_safe::Matrix::translate((-anchor.x, -anchor.y)));
            let transformed = shifted.with_transform(&skia_safe::Matrix::new_all(
                matrix.scale_x(),
                matrix.skew_x(),
                center.x,
                matrix.skew_y(),
                matrix.scale_y(),
                center.y,
                0.0,
                0.0,
                1.0,
            ));
            let stroke_width = stroke_width_for_scale(style.stroke_width.max(0.0), scale)
                * matrix.map_radius(1.0).unwrap_or(1.0);
            let mut bounds = bounds;
            bounds.outset((stroke_width * 0.5, stroke_width * 0.5));
            let clipped = transformed.op(
                &skia_safe::Path::rect(bounds, None),
                skia_safe::PathOp::Intersect,
            );
            let saved = canvas.save();
            canvas.reset_matrix();
            Some((clipped.unwrap_or(transformed), saved))
        })
        .flatten();
    let path = rebased.as_ref().map_or(path, |(path, _)| path);

    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();
    let mut paint = skia_safe::Paint::new(
        skia_safe::Color4f::new(fill_r, fill_g, fill_b, fill_a * opacity),
        None,
    );
    paint.set_anti_alias(true);
    canvas.draw_path(path, &paint);

    if style.stroke_width > 0.0 {
        let [stroke_r, stroke_g, stroke_b, stroke_a] = style.stroke.rgba();
        paint.set_color4f(
            skia_safe::Color4f::new(stroke_r, stroke_g, stroke_b, stroke_a * opacity),
            None,
        );
        paint.set_style(skia_safe::PaintStyle::Stroke);
        let mut stroke_width = stroke_width_for_scale(style.stroke_width, scale);
        if oversized {
            stroke_width *= matrix.map_radius(1.0).unwrap_or(1.0);
        }
        paint.set_stroke_width(stroke_width);
        canvas.draw_path(path, &paint);
    }

    if let Some((_, saved)) = rebased {
        canvas.restore_to_count(saved);
    }
}

/// Compensates a stroke width for the entity scale applied by the canvas.
pub(crate) fn stroke_width_for_scale(stroke_width: f32, scale: Vector2) -> f32 {
    let scale = scale.length();

    if scale > f32::EPSILON {
        stroke_width / scale
    } else {
        stroke_width
    }
}

pub(crate) fn styled_bounds(
    mut bounds: skia_safe::Rect,
    style: &Style,
    scale: Vector2,
) -> skia_safe::Rect {
    let padding = stroke_width_for_scale(style.stroke_width.max(0.0), scale) * 0.5;
    bounds.outset((padding, padding));
    bounds
}
