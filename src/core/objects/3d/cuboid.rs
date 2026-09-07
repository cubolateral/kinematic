use crate::core::{
    components::{Material, Transform3D},
    types::Vector3,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct CuboidShape {
    #[track]
    pub size: Vector3,
}

impl Default for CuboidShape {
    fn default() -> Self {
        Self { size: Vector3::ONE }
    }
}

/// Cuboid centered on its local origin.
#[derive(Default, Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "cuboid")]
pub struct Cuboid {
    #[trackable]
    pub shape: CuboidShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
}
