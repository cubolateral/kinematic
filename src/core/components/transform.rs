use kinematic_macros::Trackable;

use crate::core::types::Vector2;

#[derive(Trackable, Debug)]
/// Spatial transformation of an entity in logical canvas coordinates.
pub struct Transform {
    /// Position of the entity.
    #[track]
    pub position: Vector2,
    /// Scale of the entity on each axis.
    #[track]
    pub scale: Vector2,
    /// Rotation of the entity in radians.
    #[track]
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector2::ZERO,
            scale: Vector2::ONE,
            rotation: 0.0,
        }
    }
}
