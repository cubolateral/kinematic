use crate::core::components::Transform3D;
use kinematic_macros::{Container, Object};

/// Container inheriting three-dimensional transforms.
#[derive(Default, Object, Container, hecs::Bundle)]
#[object(spatial = "3d", builder = "group_3d")]
pub struct Group3D {
    #[trackable]
    pub transform: Transform3D,
}
