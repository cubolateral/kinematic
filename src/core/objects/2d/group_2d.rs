use kinematic_macros::{Container, Object};

use crate::core::components::{Draw2D, Transform2D};

/// Transformable scene object that groups an ordered set of child objects.
#[derive(Object, Container, hecs::Bundle)]
#[object(spatial = "2d", builder = "group_2d")]
#[morph]
pub struct Group2D {
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Group2D {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            draw: Default::default(),
        }
    }
}
