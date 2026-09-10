use kinematic_macros::Trackable;

use crate::core::{objects::GlobalTransform, types::Vector2};

/// View transformation owned by a two-dimensional canvas.
#[derive(Clone, Debug, Trackable)]
pub struct Camera2D {
    /// Position observed at the center of the viewport.
    #[track]
    pub camera_position: Vector2,
    /// Magnification applied to the scene.
    #[track]
    pub camera_zoom: f32,
    /// Rotation of the view in radians.
    #[track]
    pub camera_rotation: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            camera_position: Vector2::ZERO,
            camera_zoom: 1.0,
            camera_rotation: 0.0,
        }
    }
}

impl Camera2D {
    pub(crate) fn transform(&self) -> GlobalTransform {
        GlobalTransform {
            position: self.camera_position,
            rotation: self.camera_rotation,
            scale: Vector2::splat(self.camera_zoom.recip()),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.camera_position.is_finite()
            || !self.camera_rotation.is_finite()
            || !self.camera_zoom.is_finite()
            || self.camera_zoom <= 0.0
        {
            return Err("Canvas2D camera requires finite values and positive zoom.".into());
        }
        Ok(())
    }
}
