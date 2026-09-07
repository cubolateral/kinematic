use crate::core::{
    components::{Material, Transform3D},
    types::Vector2,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct PlaneShape {
    #[track]
    pub size: Vector2,
}

impl Default for PlaneShape {
    fn default() -> Self {
        Self { size: Vector2::ONE }
    }
}

/// Plane in local XY with its front facing positive Z.
#[derive(Default, Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "plane")]
pub struct Plane {
    #[trackable]
    pub shape: PlaneShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
}
