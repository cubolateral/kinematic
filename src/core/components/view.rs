use kinematic_macros::Trackable;

/// Selects which built-in world the scene renders.
#[derive(Clone, Debug, Trackable)]
pub struct View {
    /// Renders World 2D when true and World 3D when false.
    #[track]
    pub view_2d: bool,
}

impl Default for View {
    fn default() -> Self {
        Self { view_2d: true }
    }
}
