use kinematic_macros::Trackable;

use crate::core::types::Color;

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
