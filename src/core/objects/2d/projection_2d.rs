use crate::core::{
    components::{Draw2D, Transform2D},
    objects::{ProjectionCanvas, ProjectionSource, RectShape},
};
use kinematic_macros::Object;

/// 2D rectangle sampling the premultiplied output of a canvas.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "projection_2d")]
pub struct Projection2D {
    #[trackable]
    pub shape: RectShape,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,

    pub source: ProjectionSource,
}

impl Default for Projection2D {
    fn default() -> Self {
        Self {
            shape: RectShape::default(),
            transform: Transform2D::default(),
            draw: Draw2D {
                get_box: |world, entity| world.get::<&RectShape>(entity).unwrap().size.abs(),
                ..Default::default()
            },
            source: ProjectionSource::default(),
        }
    }
}

impl Projection2DBuilder {
    /// Sets the canvas rendered by this projection and adopts its pixel dimensions.
    pub fn source(mut self, canvas: &impl ProjectionCanvas) -> Self {
        let resolution = canvas.projection_resolution();
        self.object.source = ProjectionSource(Some(canvas.projection_texture()));
        self.object.shape.size = glam::vec2(resolution.0 as f32, resolution.1 as f32);
        self
    }
}

pub(crate) fn draw_projection_2d(
    world: &hecs::World,
    entity: hecs::Entity,
    image: &skia_safe::Image,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) {
    let size = world.get::<&RectShape>(entity).unwrap().size.abs();
    if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let destination = skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y);
    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(opacity);
    canvas.draw_image_rect(image, None, destination, &paint);
}
