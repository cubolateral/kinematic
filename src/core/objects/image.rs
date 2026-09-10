use std::{path::Path, sync::Arc};

use crate::core::types::{Color, Vector2};

/// Decoded image data shared by the built-in image objects.
#[derive(Clone, Default)]
pub(crate) struct ImageSource {
    image: Option<skia_safe::Image>,
    pixels: Arc<Vec<[u8; 4]>>,
    width: u32,
    height: u32,
    path: String,
}

impl ImageSource {
    pub(crate) fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let bytes = std::fs::read(path)
            .unwrap_or_else(|error| panic!("Failed to read image '{}': {error}.", path.display()));
        let image = skia_safe::Image::from_encoded(skia_safe::Data::new_copy(&bytes))
            .unwrap_or_else(|| panic!("Failed to decode image '{}'.", path.display()));
        let width = image.width() as u32;
        let height = image.height() as u32;
        let info = skia_safe::ImageInfo::new(
            (width as i32, height as i32),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Unpremul,
            skia_safe::ColorSpace::new_srgb(),
        );
        let mut pixels = vec![[0; 4]; width as usize * height as usize];
        assert!(
            image.read_pixels(
                &info,
                &mut pixels,
                width as usize * 4,
                (0, 0),
                skia_safe::image::CachingHint::Allow,
            ),
            "Failed to read pixels from image '{}'.",
            path.display()
        );

        Self {
            image: Some(image),
            pixels: Arc::new(pixels),
            width,
            height,
            path: path.to_string_lossy().into_owned(),
        }
    }

    pub(crate) fn image(&self) -> Option<&skia_safe::Image> {
        self.image.as_ref()
    }

    pub(crate) fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn pixel_color(&self, point: Vector2, size: Vector2) -> Color {
        if !point.is_finite()
            || !size.is_finite()
            || size.x <= 0.0
            || size.y <= 0.0
            || self.width == 0
            || self.height == 0
        {
            return Color::TRANSPARENT;
        }

        let uv = point / size + Vector2::splat(0.5);
        if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
            return Color::TRANSPARENT;
        }

        let x = (uv.x * self.width as f32).floor() as u32;
        let y = (uv.y * self.height as f32).floor() as u32;
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        let [r, g, b, a] = self.pixels[(y * self.width + x) as usize];
        Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    pub(crate) fn cpu_texture(&self) -> Option<three_d::CpuTexture> {
        self.image.as_ref()?;
        let mut texture = three_d::CpuTexture {
            name: self.path.clone(),
            data: three_d::TextureData::RgbaU8(self.pixels.as_ref().clone()),
            width: self.width,
            height: self.height,
            ..Default::default()
        };
        texture.data.to_linear_srgb();
        Some(texture)
    }
}

#[cfg(test)]
pub(crate) fn write_test_image() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_IMAGE: AtomicU64 = AtomicU64::new(0);

    let mut surface = skia_safe::surfaces::raster_n32_premul((2, 1)).unwrap();
    surface.canvas().clear(skia_safe::colors::RED);
    surface.canvas().draw_rect(
        skia_safe::Rect::from_xywh(1.0, 0.0, 1.0, 1.0),
        &skia_safe::Paint::new(skia_safe::colors::BLUE, None),
    );
    let data = surface
        .image_snapshot()
        .encode(None, skia_safe::EncodedImageFormat::PNG, None)
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "kinematic-image-test-{}-{}.png",
        std::process::id(),
        NEXT_IMAGE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, data.as_bytes()).unwrap();
    path
}
