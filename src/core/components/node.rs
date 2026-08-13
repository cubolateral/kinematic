pub struct Node {
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>),
}

impl Default for Node {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| {},
        }
    }
}
