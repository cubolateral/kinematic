use crate::core::components::{Draw3D, Transform3D};
use kinematic_macros::{Node, Object};

/// Container inheriting three-dimensional transforms.
#[derive(Default, Object, Node)]
#[object(spatial = "3d", builder = "group_3d")]
pub struct Group3D {
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}
