use kinematic_macros::{Node, Object};

use crate::core::{
    Tween,
    components::{__KinematicCamera3DTweenFields, Camera3D, Draw3D},
    types::{Quaternion, Vector3},
};

use super::super::canvas::{
    CanvasDimension, CanvasSettings, CanvasTexture, scene_identity, validate_canvas,
};
/// Independent 3D perspective viewport with its own camera.
#[derive(Object, Node)]
#[object(spatial = "none", builder = "canvas_3d", image_shader)]
pub struct Canvas3D {
    #[trackable]
    pub settings: CanvasSettings,
    #[trackable]
    pub camera: Camera3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Canvas3D {
    fn default() -> Self {
        Self {
            settings: CanvasSettings::new(CanvasDimension::Three),
            camera: Camera3D::default(),
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
    /// Moves the camera while turning it toward a target point.
    pub fn focus_camera(&self, position: Vector3, target: Vector3) -> Tween<Canvas3D> {
        let rotation =
            Quaternion::from_rotation_arc(Vector3::NEG_Z, (target - position).normalize_or_zero());
        self.camera_position(position).camera_rotation(rotation)
    }

    /// Returns the render output produced with this canvas's camera.
    pub fn texture(&self) -> CanvasTexture {
        CanvasTexture {
            scene: scene_identity(&self.world),
            entity: self.entity,
        }
    }

    /// Validates the resolution, camera transform, and perspective values.
    pub fn validate(&self) -> Result<(), String> {
        validate_canvas(&self.world.borrow(), self.entity)
    }
}
