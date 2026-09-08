use kinematic_macros::{Container, Object};

use crate::core::components::Draw3D;

use super::super::canvas::{
    CanvasDimension, CanvasSettings, CanvasTexture, scene_identity, validate_canvas,
};
use super::super::{Camera3DHandler, ObjectHandler};

/// Independent 3D perspective viewport. Requires an explicitly assigned camera.
#[derive(Object, Container, hecs::Bundle)]
#[object(spatial = "none", builder = "canvas_3d")]
pub struct Canvas3D {
    #[trackable]
    pub settings: CanvasSettings,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Canvas3D {
    fn default() -> Self {
        Self {
            settings: CanvasSettings::new(CanvasDimension::Three),
            draw: Draw3D::default(),
        }
    }
}

impl Canvas3DBuilder {
    /// Sets the fixed render-target resolution for this canvas.
    pub fn resolution(mut self, resolution: (u32, u32)) -> Self {
        self.object.settings.resolution = resolution;
        self
    }
}

impl Canvas3DHandler {
    pub fn get_texture(&self) -> CanvasTexture {
        CanvasTexture {
            scene: scene_identity(&self.world),
            entity: self.entity,
        }
    }

    /// Assigns a 3D camera. Membership is checked when validating or rendering.
    pub fn set_camera(&self, camera: &Camera3DHandler) {
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
