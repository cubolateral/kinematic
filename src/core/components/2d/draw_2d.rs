use kinematic_macros::Trackable;

use crate::core::types::Vector2;

/// Local two-dimensional rendering settings and callbacks for an entity.
///
/// The callback receives the current entity state and must not mutate the ECS world.
#[derive(Clone, Trackable)]
pub struct Draw2D {
    /// Whether this entity and, for containers, its subtree are drawn.
    #[track]
    pub visibility: bool,
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,
    /// Stacking order among sibling objects. Higher values are drawn in front.
    #[track]
    pub z_index: i32,

    /// Draws this entity in local coordinates with the supplied opacity.
    pub on_draw: fn(&hecs::World, hecs::Entity, &skia_safe::Canvas, f32),

    /// Returns the object's local bounding-box size.
    pub get_box: fn(&hecs::World, hecs::Entity) -> Vector2,
}

impl Default for Draw2D {
    fn default() -> Self {
        Self {
            visibility: true,
            opacity: 1.0,
            z_index: 0,
            on_draw: |_, _, _, _| {},
            get_box: |_, _| Vector2::ZERO,
        }
    }
}
