use kinematic_macros::Trackable;

use crate::core::types::{Color, Vector2};

/// Fill and stroke properties for a style entity.
#[derive(Trackable, Default, Debug)]
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
