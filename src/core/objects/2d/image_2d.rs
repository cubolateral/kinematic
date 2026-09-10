use std::path::Path;

use kinematic_macros::Object;

use crate::core::{
    components::{
        Draw2D, Morph, PARTICLE_COUNT, Style, Transform2D, draw_styled_path, stroke_width_for_scale,
    },
    objects::{CreationDraw, ImageSource, RectShape, particle_visual_key, rect_path},
    types::{Color, Vector2},
};

/// Image drawn inside a styled rectangular shape.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "image_2d")]
#[morph]
pub struct Image2D {
    #[trackable]
    pub shape: RectShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,

    source: ImageSource,
}

impl Default for Image2D {
    fn default() -> Self {
        Self {
            shape: RectShape::default(),
            style: Style {
                fill: Color::TRANSPARENT,
                ..Default::default()
            },
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: draw_image_2d,
                get_box: |world, entity| world.get::<&RectShape>(entity).unwrap().size.abs(),
                ..Default::default()
            },
            source: ImageSource::default(),
        }
    }
}

impl Image2DBuilder {
    /// Loads an image and adopts its pixel dimensions as the rectangle size.
    pub fn source(mut self, path: impl AsRef<Path>) -> Self {
        self.object.source = ImageSource::load(path);
        let (width, height) = self.object.source.dimensions();
        self.object.shape.size = Vector2::new(width as f32, height as f32);
        self
    }
}

impl Image2DHandler {
    /// Returns the source pixel at a position in the object's local rectangle.
    ///
    /// Positions outside the rectangle return transparent black.
    pub fn get_pixel_color(&self, point: Vector2) -> Color {
        let world = self.world.borrow();
        let source = world.get::<&ImageSource>(self.entity).unwrap();
        let shape = world.get::<&RectShape>(self.entity).unwrap();
        source.pixel_color(point, shape.size)
    }
}

fn draw_image_2d(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) {
    let source = world.get::<&ImageSource>(entity).unwrap();
    let Some(image) = source.image() else {
        return;
    };
    let shape = world.get::<&RectShape>(entity).unwrap();
    let size = shape.size;
    if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let style = world.get::<&Style>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();
    let morph = world.get::<&Morph>(entity).unwrap();
    let path = rect_path(&shape);
    let bounds = skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y);
    let draw_complete = |target: &skia_safe::Canvas, target_opacity: f32| {
        let mut paint = skia_safe::Paint::default();
        paint.set_alpha_f(target_opacity);
        let saved = target.save();
        target.clip_path(&path, None, true);
        target.draw_image_rect(image, None, bounds, &paint);
        target.restore_to_count(saved);
        draw_styled_path(&path, &style, transform.scale, target_opacity, target);
    };

    if morph.particles_enabled && morph.progress < 1.0 {
        let padding = stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5;
        let particle_bounds = skia_safe::Rect::new(
            bounds.left - padding,
            bounds.top - padding,
            bounds.right + padding,
            bounds.bottom + padding,
        );
        let visual_key = particle_visual_key(
            "Image2D",
            &style,
            &[size.x, size.y, transform.scale.x, transform.scale.y],
            &[source.path()],
        );
        let pixel_color = |point| source.pixel_color(point, size);
        if (CreationDraw {
            entity,
            cache_slot: 0,
            bounds: particle_bounds,
            visual_key,
            particle_count: PARTICLE_COUNT as usize,
            style: &style,
            pixel_color: Some(&pixel_color),
            morph: &morph,
            opacity,
            canvas,
        })
        .render(draw_complete)
        {
            return;
        }
    }

    draw_complete(canvas, opacity);
}

#[cfg(test)]
mod tests {
    use crate::{
        core::objects::image::write_test_image,
        prelude::{Color, Object2DHandler, Scene, image_2d, vec2},
    };

    #[test]
    fn source_sets_size_and_handler_samples_local_pixels() {
        let path = write_test_image();
        let mut scene = Scene::new();
        let image = image_2d().source(&path).build(&mut scene);
        std::fs::remove_file(path).unwrap();

        assert_eq!(image.get_box(), vec2(2.0, 1.0));
        assert_eq!(image.get_pixel_color(vec2(-0.5, 0.0)), Color::RED);
        assert_eq!(image.get_pixel_color(vec2(0.5, 0.0)), Color::BLUE);
        assert_eq!(image.get_pixel_color(vec2(2.0, 0.0)), Color::TRANSPARENT);
    }
}
