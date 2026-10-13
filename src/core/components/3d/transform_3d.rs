use kinematic_macros::Trackable;

use crate::core::{
    normalized_quaternion,
    types::{Quaternion, Vector3},
};

/// Local three-dimensional position, rotation and scale.
#[derive(Clone, Debug, Trackable)]
pub struct Transform3D {
    /// Position relative to the parent in local coordinates.
    #[track]
    pub position: Vector3,
    /// Pivot relative to half the entity size. Values are not clamped.
    #[track]
    pub origin: Vector3,
    /// Local orientation as a quaternion. Use k.axis_angle to construct a rotation.
    #[track]
    pub rotation: Quaternion,
    /// Local scale multiplier on each axis.
    #[track]
    pub scale: Vector3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Vector3::ZERO,
            origin: Vector3::ZERO,
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

    pub(crate) fn matrix_with_origin(&self, size: Vector3) -> glam::Mat4 {
        self.matrix() * glam::Mat4::from_translation(-self.origin * size * 0.5)
    }
}
