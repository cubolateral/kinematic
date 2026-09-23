use kinematic_macros::{TrackEnum, Trackable};

use crate::core::{
    normalized_quaternion,
    types::{Quaternion, Vector3},
};

/// Projection used by a three-dimensional camera.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, TrackEnum)]
pub enum Camera3DMode {
    #[default]
    Perspective,
    Orthogonal,
}

/// Camera owned by a three-dimensional canvas.
#[derive(Clone, Debug, Trackable)]
pub struct Camera3D {
    /// Projection mode.
    #[track]
    pub camera_mode: Camera3DMode,
    /// Position in canvas coordinates. Defaults to `(0, 0, 3)`.
    #[track]
    pub camera_position: Vector3,
    /// Orientation looking along local negative Z, with positive Y up.
    #[track]
    pub camera_rotation: Quaternion,
    /// Vertical field of view in radians, or view height in orthogonal mode.
    #[track(min = 0.000001, max = 3.1415915)]
    pub camera_fov: f32,
    /// Near clipping plane distance.
    #[track(min = 0.000001)]
    pub camera_near: f32,
    /// Far clipping plane distance.
    #[track(min = 0.000001)]
    pub camera_far: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            camera_mode: Camera3DMode::Perspective,
            camera_position: Vector3::new(0.0, 0.0, 3.0),
            camera_rotation: Quaternion::IDENTITY,
            camera_fov: 45.0_f32.to_radians(),
            camera_near: 0.1,
            camera_far: 1000.0,
        }
    }
}

impl Camera3D {
    pub(crate) fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(
            normalized_quaternion(self.camera_rotation),
            self.camera_position,
        )
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        match self.camera_mode {
            Camera3DMode::Perspective
                if !self.camera_fov.is_finite()
                    || !(0.0..std::f32::consts::PI).contains(&self.camera_fov)
                    || self.camera_fov == 0.0 =>
            {
                return Err("Camera FOV must be between zero and pi radians.".into());
            }
            Camera3DMode::Orthogonal if !self.camera_fov.is_finite() || self.camera_fov <= 0.0 => {
                return Err("Orthogonal camera height must be positive and finite.".into());
            }
            _ => {}
        }
        if !self.camera_near.is_finite()
            || !self.camera_far.is_finite()
            || self.camera_near <= 0.0
            || self.camera_far <= self.camera_near
        {
            return Err("Camera planes must satisfy 0 < near < far.".into());
        }
        if !self.matrix().is_finite() {
            return Err("Camera transform must be finite.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orthogonal_height_can_exceed_the_perspective_fov_range() {
        let orthogonal = Camera3D {
            camera_mode: Camera3DMode::Orthogonal,
            camera_fov: 10.0,
            ..Default::default()
        };
        assert!(orthogonal.validate().is_ok());
        assert!(
            Camera3D {
                camera_mode: Camera3DMode::Perspective,
                ..orthogonal
            }
            .validate()
            .is_err()
        );
    }
}
