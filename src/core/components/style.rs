use kinematic_macros::Trackable;

use crate::core::types::{Color, Vector2};

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
    #[track]
    pub stroke_width: f32,
    /// Normalized creation progress of the object.
    #[track]
    pub progress: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Color::default(),
            stroke: Color::default(),
            stroke_width: 0.0,
            progress: 1.0,
        }
    }
}

/// Draws a closed path using its normalized creation opacity.
pub(crate) fn draw_styled_path(
    path: &skia_safe::Path,
    style: &Style,
    scale: Vector2,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let progress = style.progress.clamp(0.0, 1.0);
    if progress <= 0.0 {
        return;
    }

    draw_complete_styled_path(path, style, scale, opacity * progress, canvas);
}

/// Draws a complete closed path without applying style progress.
pub(crate) fn draw_complete_styled_path(
    path: &skia_safe::Path,
    style: &Style,
    scale: Vector2,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();
    let mut paint = skia_safe::Paint::new(
        skia_safe::Color4f::new(fill_r, fill_g, fill_b, fill_a * opacity),
        None,
    );
    paint.set_anti_alias(true);
    canvas.draw_path(path, &paint);

    if style.stroke_width <= 0.0 {
        return;
    }

    let [stroke_r, stroke_g, stroke_b, stroke_a] = style.stroke.rgba();
    paint.set_color4f(
        skia_safe::Color4f::new(stroke_r, stroke_g, stroke_b, stroke_a * opacity),
        None,
    );
    paint.set_style(skia_safe::PaintStyle::Stroke);
    paint.set_stroke_width(stroke_width_for_scale(style.stroke_width, scale));
    canvas.draw_path(path, &paint);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_is_fully_created_by_default() {
        assert_eq!(Style::default().progress, 1.0);
    }
}
