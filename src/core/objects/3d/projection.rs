use crate::core::{
    components::Transform3D,
    objects::{Canvas2DHandler, CanvasSettings, ObjectHandler, PlaneShape},
};
use kinematic_macros::Object;

use super::super::canvas::ProjectionSource;

#[derive(Clone, Copy)]
pub(crate) struct ProjectionSettings {
    pixels_per_unit: f32,
}

impl Default for ProjectionSettings {
    fn default() -> Self {
        Self {
            pixels_per_unit: 256.0,
        }
    }
}

/// Unlit plane sampling the premultiplied output of a Canvas2D.
#[derive(Default, Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "projection")]
pub struct Projection {
    #[trackable]
    pub shape: PlaneShape,
    #[trackable]
    pub transform: Transform3D,

    pub source: ProjectionSource,
    settings: ProjectionSettings,
}

impl ProjectionBuilder {
    /// Sets the Canvas2D-to-world scale used to size the projection plane.
    pub fn pixels_per_unit(mut self, pixels_per_unit: f32) -> Self {
        assert!(
            pixels_per_unit.is_finite() && pixels_per_unit > 0.0,
            "Projection pixels per unit must be finite and positive."
        );
        if self.object.source.0.is_some() {
            self.object.shape.size *= self.object.settings.pixels_per_unit / pixels_per_unit;
        }
        self.object.settings.pixels_per_unit = pixels_per_unit;
        self
    }

    pub fn source(mut self, canvas: &Canvas2DHandler) -> Self {
        let resolution = canvas
            .object_world()
            .borrow()
            .get::<&CanvasSettings>(canvas.get_id())
            .expect("Canvas2D handler must contain CanvasSettings.")
            .resolution;
        self.object.source = ProjectionSource(Some(canvas.get_texture()));
        let scale = self.object.settings.pixels_per_unit;
        self.object.shape.size =
            glam::vec2(resolution.0 as f32 / scale, resolution.1 as f32 / scale);
        self
    }
}
