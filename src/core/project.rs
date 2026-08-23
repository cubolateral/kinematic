use crate::core::SceneBuilder;

/// Top-level configuration used to initialize an editor project.
pub struct Project {
    /// Name displayed by the host application.
    pub name: &'static str,
    /// Logical canvas size used by the scene preview.
    pub resolution: (u32, u32),
    /// Maximum animation and preview frame rate.
    pub fps: u32,
    /// Declarative scene definition used to populate the runtime scene.
    pub scene: Box<dyn SceneBuilder>,
}
