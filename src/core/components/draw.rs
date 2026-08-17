use kinematic_macros::Trackable;

/// Local rendering callbacks and group opacity for an entity.
///
/// The callbacks receive the current entity state and must not mutate the ECS world.
#[derive(Trackable)]
pub struct Draw {
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,

    /// Returns the local drawing rectangle as `[x, y, width, height]`.
    pub get_rect:
        fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>) -> [f32; 4],
    /// Draws this entity in local coordinates on the supplied canvas.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>),
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            get_rect: |_, _, _| [0.0; 4],
            on_draw: |_, _, _| {},
            opacity: 1.0,
        }
    }
}
