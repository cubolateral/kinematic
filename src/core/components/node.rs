/// Rendering callback attached to an entity.
///
/// The callback receives the current entity state and must not mutate the ECS world.
pub struct Node {
    /// Draws this entity on the supplied canvas.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>),
}

impl Default for Node {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| {},
        }
    }
}
