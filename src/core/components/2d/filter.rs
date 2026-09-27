use kinematic_macros::Trackable;

use crate::core::types::{Color, Vector2};

/// Image filters applied to a two-dimensional object and its subtree.
#[derive(Clone, Trackable)]
pub struct Filter {
    /// Gaussian blur sigma in local canvas units.
    #[track(min = 0.0)]
    pub blur: f32,
    /// Shadow offset in local canvas units.
    #[track]
    pub shadow_offset: Vector2,
    /// Shadow blur sigma in local canvas units.
    #[track(min = 0.0)]
    pub shadow_blur: f32,
    /// Shadow color.
    #[track]
    pub shadow_color: Color,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            blur: 0.0,
            shadow_offset: Vector2::ZERO,
            shadow_blur: 0.0,
            shadow_color: Color::TRANSPARENT,
        }
    }
}
