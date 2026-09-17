use crate::core::components::{Draw2D, Transform2D};
use crate::core::types::Vector2;
use kinematic_macros::{Node, Object, TrackEnum, Trackable};

/// Child ordering used by a two-dimensional container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, TrackEnum)]
pub enum LayoutDirection2D {
    #[default]
    Horizontal,
    HorizontalReverse,
    Vertical,
    VerticalReverse,
}

/// Procedural spacing and anchoring for a two-dimensional container.
#[derive(Clone, Debug, Default, Trackable)]
pub struct Layout2D {
    #[track]
    pub gap: Vector2,
    #[track]
    pub direction: LayoutDirection2D,
}

/// A two-dimensional node that lays out its children procedurally.
#[derive(Default, Object, Node)]
#[object(spatial = "2d", builder = "container_2d")]
pub struct Container2D {
    #[trackable]
    pub layout: Layout2D,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}
