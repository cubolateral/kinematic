use crate::core::components::{Draw3D, Transform3D};
use crate::core::types::Vector3;
use kinematic_macros::{Node, Object, TrackEnum, Trackable};

/// Child ordering used by a three-dimensional container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, TrackEnum)]
pub enum LayoutDirection3D {
    #[default]
    Horizontal,
    HorizontalReverse,
    Vertical,
    VerticalReverse,
    Depth,
    DepthReverse,
}

/// Procedural spacing and anchoring for a three-dimensional container.
#[derive(Clone, Debug, Default, Trackable)]
pub struct Layout3D {
    #[track]
    pub gap: Vector3,
    #[track]
    pub direction: LayoutDirection3D,
}

/// A three-dimensional node that lays out its children procedurally.
#[derive(Default, Object, Node)]
#[object(spatial = "3d", builder = "container_3d")]
pub struct Container3D {
    #[trackable]
    pub layout: Layout3D,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}
