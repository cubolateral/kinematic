use kinematic_macros::Trackable;

use crate::core::types::Vector2;

/// Local rendering callback and opacity for an entity.
///
/// The callbacks receive the current entity state and must not mutate the ECS world.
#[derive(Clone, Trackable)]
pub struct Draw {
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,

    /// Draws this entity in local coordinates with the supplied opacity.
    pub on_draw: fn(&hecs::World, hecs::Entity, &skia_safe::Canvas, f32),

    /// Returns the object's local bounding-box size.
    pub get_box: fn(&hecs::World, hecs::Entity) -> Vector2,
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _, _| {},
            get_box: |_, _| Vector2::ZERO,
            opacity: 1.0,
        }
    }
}
