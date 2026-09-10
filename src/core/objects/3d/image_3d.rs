use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use kinematic_macros::Object;

use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
        validate_transformation,
    },
    objects::{ImageSource, PlaneShape, global_matrix3d},
    types::{Color, Vector2},
};

#[derive(Clone, Default)]
struct Image3DTexture(Arc<Mutex<Option<three_d::Texture2DRef>>>);

#[derive(Clone, Copy)]
struct Image3DSettings {
    pixels_per_unit: f32,
}

impl Default for Image3DSettings {
    fn default() -> Self {
        Self {
            pixels_per_unit: 256.0,
        }
    }
}

/// Textured plane in local XY with its front facing positive Z.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "image_3d")]
pub struct Image3D {
    #[trackable]
    pub shape: PlaneShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,

    source: ImageSource,
    texture: Image3DTexture,
    settings: Image3DSettings,
}

impl Default for Image3D {
    fn default() -> Self {
        Self {
            shape: PlaneShape::default(),
            material: Material {
                unlit: true,
                ..Default::default()
            },
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_image_3d,
                get_box: |world, entity| {
                    world
                        .get::<&PlaneShape>(entity)
                        .unwrap()
                        .size
                        .abs()
                        .extend(0.0)
                },
                ..Default::default()
            },
            source: ImageSource::default(),
            texture: Image3DTexture::default(),
            settings: Image3DSettings::default(),
        }
    }
}

impl Image3DBuilder {
    /// Sets the number of source pixels represented by one world unit.
    pub fn pixels_per_unit(mut self, pixels_per_unit: f32) -> Self {
        assert!(
            pixels_per_unit.is_finite() && pixels_per_unit > 0.0,
            "Image3D pixels per unit must be finite and positive."
        );
        if self.object.source.image().is_some() {
            self.object.shape.size *= self.object.settings.pixels_per_unit / pixels_per_unit;
        }
        self.object.settings.pixels_per_unit = pixels_per_unit;
        self
    }

    /// Loads an image and sizes the plane from its pixels-per-unit setting.
    pub fn source(mut self, path: impl AsRef<Path>) -> Self {
        self.object.source = ImageSource::load(path);
        self.object.texture = Image3DTexture::default();
        let (width, height) = self.object.source.dimensions();
        let scale = self.object.settings.pixels_per_unit;
        self.object.shape.size = Vector2::new(width as f32 / scale, height as f32 / scale);
        self
    }
}

impl Image3DHandler {
    /// Returns the source pixel at a position in the plane's local rectangle.
    ///
    /// Positions outside the rectangle return transparent black.
    pub fn get_pixel_color(&self, point: Vector2) -> Color {
        let world = self.world.borrow();
        let source = world.get::<&ImageSource>(self.entity).unwrap();
        let shape = world.get::<&PlaneShape>(self.entity).unwrap();
        source.pixel_color(point, shape.size)
    }
}

fn draw_image_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&PlaneShape>(entity).unwrap();
    let size = (shape.size * 0.5).extend(1.0);
    validate_dimensions(size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(size);
    validate_transformation(transformation)?;
    if transformation.determinant().abs() <= f32::EPSILON {
        return Ok(());
    }

    let source = world.get::<&ImageSource>(entity).unwrap();
    let texture = world.get::<&Image3DTexture>(entity).unwrap();
    let mut texture = texture.0.lock().unwrap();
    if texture.is_none() {
        let cpu_texture = source
            .cpu_texture()
            .ok_or("Image3D requires a source image.")?;
        *texture = Some(three_d::Texture2DRef::from_cpu_texture(
            context.three_d,
            &cpu_texture,
        ));
    }
    let texture = texture.as_ref().unwrap();
    context.render_textured_material(
        GeometryKey::new::<Image3D>(0),
        three_d::CpuMesh::square,
        transformation,
        texture.clone(),
        &world.get::<&Material>(entity).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        core::objects::image::write_test_image,
        prelude::{Color, Object3DHandler, Scene, image_3d, vec2, vec3},
    };

    #[test]
    fn source_sets_world_size_and_handler_samples_local_pixels() {
        let path = write_test_image();
        let mut scene = Scene::new();
        let image = image_3d()
            .pixels_per_unit(2.0)
            .source(&path)
            .build(&mut scene);
        std::fs::remove_file(path).unwrap();

        assert_eq!(image.get_box(), vec3(1.0, 0.5, 0.0));
        assert_eq!(image.get_pixel_color(vec2(-0.25, 0.0)), Color::RED);
        assert_eq!(image.get_pixel_color(vec2(0.25, 0.0)), Color::BLUE);
        assert_eq!(image.get_pixel_color(vec2(1.0, 0.0)), Color::TRANSPARENT);
    }
}
