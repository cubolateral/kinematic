use crate::core::components::Draw2D;
use kinematic_macros::{Container, Object};

use super::super::canvas::{
    CanvasDimension, CanvasSettings, CanvasTexture, scene_identity, validate_canvas,
};
use super::super::{Camera2DHandler, ObjectHandler};

/// Independent 2D Skia viewport. An omitted camera uses the identity view.
#[derive(Object, Container, hecs::Bundle)]
#[object(spatial = "none", builder = "canvas_2d")]
pub struct Canvas2D {
    #[trackable]
    pub settings: CanvasSettings,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Canvas2D {
    fn default() -> Self {
        Self {
            settings: CanvasSettings::new(CanvasDimension::Two),
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
    pub fn get_texture(&self) -> CanvasTexture {
        CanvasTexture {
            scene: scene_identity(&self.world),
            entity: self.entity,
        }
    }

    /// Assigns a 2D camera. Membership is checked when validating or rendering.
    pub fn set_camera(&self, camera: &Camera2DHandler) {
        assert!(
            std::rc::Rc::ptr_eq(&self.world, &camera.object_world()),
            "Camera must belong to the same scene."
        );
        self.world
            .borrow()
            .get::<&mut CanvasSettings>(self.entity)
            .unwrap()
            .camera = Some(camera.get_id());
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_canvas(&self.world.borrow(), self.entity)
    }
}
