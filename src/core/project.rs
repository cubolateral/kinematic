use crate::core::Scene;

/// Top-level configuration used to initialize an editor project.
pub struct Project {
    /// Name displayed by the host application.
    pub name: &'static str,
    /// Logical canvas size used by the scene preview.
    pub resolution: (u32, u32),
    /// Maximum animation and preview frame rate.
    pub fps: u32,
    /// Ordered factories used to create the runtime scenes.
    pub scenes: Vec<fn() -> Scene>,
}

impl Project {
    pub(crate) fn validate(&self) {
        assert!(
            !self.scenes.is_empty(),
            "Project must contain at least one scene."
        );
    }
}
