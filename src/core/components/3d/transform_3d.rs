use kinematic_macros::Trackable;

use crate::core::{
    normalized_quaternion,
    types::{Quaternion, Vector3},
};

/// Local three-dimensional position, rotation and scale.
#[derive(Clone, Debug, Trackable)]
pub struct Transform3D {
    #[track]
    pub position: Vector3,
    #[track]
    pub rotation: Quaternion,
    #[track]
    pub scale: Vector3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Vector3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Vector3::ONE,
        }
    }
}

impl Transform3D {
    pub(crate) fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            normalized_quaternion(self.rotation),
            self.position,
        )
    }
}
