use crate::core::components::{Camera2D, Draw2D};
use kinematic_macros::{Container, Object};

use super::super::canvas::{
    CanvasDimension, CanvasSettings, CanvasTexture, scene_identity, validate_canvas,
};
/// Independent 2D Skia viewport with its own camera.
#[derive(Object, Container, hecs::Bundle)]
#[object(spatial = "none", builder = "canvas_2d")]
pub struct Canvas2D {
    #[trackable]
    pub settings: CanvasSettings,
    #[trackable]
    pub camera: Camera2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Canvas2D {
    fn default() -> Self {
        Self {
            settings: CanvasSettings::new(CanvasDimension::Two),
            camera: Camera2D::default(),
            draw: Draw2D::default(),
        }
    }
}

impl Canvas2DBuilder {
    /// Sets the fixed render-target resolution for this canvas.
    pub fn resolution(mut self, resolution: (u32, u32)) -> Self {
        self.object.settings.resolution = resolution;
        self
    }
}

impl Canvas2DHandler {
    /// Returns the render output produced with this canvas's camera.
    pub fn get_texture(&self) -> CanvasTexture {
        CanvasTexture {
            scene: scene_identity(&self.world),
            entity: self.entity,
        }
    }

    /// Validates the resolution and camera values of this canvas.
    pub fn validate(&self) -> Result<(), String> {
        validate_canvas(&self.world.borrow(), self.entity)
    }
}
