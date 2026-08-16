use kinematic_macros::Trackable;

/// Rendering callback and opacity for an entity.
///
/// The callback receives the current entity state and must not mutate the ECS world.
#[derive(Trackable)]
pub struct Draw {
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,

    /// Draws this entity on the supplied canvas.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>),
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| {},
            opacity: 1.0,
        }
    }
}
