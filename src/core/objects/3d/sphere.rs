use crate::core::components::{Material, Transform3D};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct SphereShape {
    #[track]
    pub radius: f32,
    pub segments: u32,
}

impl Default for SphereShape {
    fn default() -> Self {
        Self {
            radius: 0.5,
            segments: 32,
        }
    }
}

/// Sphere centered on its local origin.
#[derive(Default, Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "sphere")]
pub struct Sphere {
    #[trackable]
    pub shape: SphereShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
}
