use crate::core::{
    components::{Draw2D, Style, Transform2D, draw_styled_path},
    objects::{ProjectionCanvas, ProjectionSource, RectShape, rect_path},
    types::Color,
};
use kinematic_macros::Object;

/// 2D rectangle sampling the premultiplied output of a canvas.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "projection_2d")]
pub struct Projection2D {
    #[trackable]
    pub shape: RectShape,
    #[trackable]
    pub style: Style,
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
            style: Style {
                fill: Color::TRANSPARENT,
                ..Default::default()
            },
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
    let shape = world.get::<&RectShape>(entity).unwrap();
    let size = shape.size;
    if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let path = rect_path(&shape);
    let destination = skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y);
    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(opacity);

    let save_count = canvas.save();
    canvas.clip_path(&path, None, true);
    canvas.draw_image_rect(image, None, destination, &paint);
    canvas.restore_to_count(save_count);

    let style = world.get::<&Style>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();
    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
}

#[cfg(test)]
mod tests {
    use super::draw_projection_2d;
    use crate::prelude::*;

    #[test]
    fn projection_clips_rounding_and_draws_style_over_the_image() {
        let mut scene = Scene::new();
        let projection = projection_2d()
            .size(vec2(40.0, 40.0))
            .round(12)
            .fill(Color::new(1.0, 0.0, 0.0, 0.5))
            .stroke(Color::WHITE)
            .stroke_width(4.0)
            .build(&mut scene);
        let mut source = skia_safe::surfaces::raster_n32_premul((40, 40)).unwrap();
        source.canvas().clear(skia_safe::colors::BLUE);
        let image = source.image_snapshot();
        let mut target = skia_safe::surfaces::raster_n32_premul((64, 64)).unwrap();
        target.canvas().translate((32.0, 32.0));

        draw_projection_2d(
            &scene.get_world(),
            projection.get_id(),
            &image,
            target.canvas(),
            1.0,
        );

        let pixels = target.peek_pixels().unwrap();
        assert_eq!(pixels.get_color((12, 12)).a(), 0);
        let center = pixels.get_color((32, 32));
        assert!(center.r().abs_diff(128) <= 1);
        assert_eq!(center.g(), 0);
        assert!(center.b().abs_diff(127) <= 1);
        let stroke = pixels.get_color((32, 12));
        assert!(
            stroke.r() >= 250 && stroke.g() >= 250 && stroke.b() >= 250,
            "Stroke pixel was {stroke:?}."
        );
    }
}
