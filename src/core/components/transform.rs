use kinematic_macros::Trackable;

use crate::core::types::Vector2;

#[derive(Trackable, Default, Debug)]
/// Position of an entity in logical canvas coordinates.
pub struct Transform {
    #[track]
    pub position: Vector2,
}
