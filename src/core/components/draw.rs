use kinematic_macros::Trackable;

/// Local rendering callback and opacity for an entity.
///
/// The callbacks receive the current entity state and must not mutate the ECS world.
#[derive(Trackable)]
pub struct Draw {
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,

    /// Draws this entity in local coordinates on the supplied canvas.
    pub on_draw: fn(&hecs::World, hecs::Entity, &skia_safe::Canvas),
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| {},
            opacity: 1.0,
        }
    }
}
