use kinematic_macros::{Container, Object};

use crate::core::components::{Draw, Transform};

/// Transformable scene object that groups an ordered set of child objects.
#[derive(Object, Container, hecs::Bundle)]
pub struct Group {
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            draw: Default::default(),
        }
    }
}
