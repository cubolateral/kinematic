use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw, Style, Transform},
    types::Vector2,
};

type FontCache = std::collections::HashMap<std::path::PathBuf, skia_safe::Typeface>;

static FONT_CACHE: std::sync::LazyLock<std::sync::Mutex<FontCache>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(FontCache::new()));

/// Font file used to render a text shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Font {
    path: std::path::PathBuf,
}

impl Font {
    /// Creates a font from a TTF or OTF file.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: Self::resolve_path(path.into()),
        }
    }

    fn resolve_path(path: std::path::PathBuf) -> std::path::PathBuf {
        if path.is_absolute() {
            return path;
        }

        if path.exists() {
            return std::fs::canonicalize(&path).unwrap_or(path);
        }

        let crate_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&path);

        std::fs::canonicalize(&crate_path).unwrap_or(crate_path)
    }

    /// Returns the font file path.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    fn skia_font(&self, size: f32) -> skia_safe::Font {
        let path = self.path.clone();
        let mut cache = FONT_CACHE.lock().unwrap();

        let typeface = cache.entry(path.clone()).or_insert_with(|| {
            let data = std::fs::read(&path).unwrap_or_else(|error| {
                panic!("Font at `{}` could not be read: {error}.", path.display())
            });

            skia_safe::FontMgr::new()
                .new_from_data(&data, None)
                .unwrap_or_else(|| panic!("Font at `{}` could not be parsed.", path.display()))
        });

        skia_safe::Font::new(typeface.clone(), size)
    }
}

/// Content and typography of a text object.
#[derive(Trackable)]
pub struct TextShape {
    /// Text displayed by the object.
    #[track]
    pub text: String,
    /// Font size in logical canvas units.
    #[track]
    pub size: f32,

    /// Font used to render the text.
    pub font: Font,
}

impl Default for TextShape {
    fn default() -> Self {
        Self {
            text: "Text".to_owned(),
            size: 64.0,
            font: Font::new("assets/fonts/JetBrainsMono-Regular.ttf"),
        }
    }
}

/// Built-in text scene object.
#[derive(Object, hecs::Bundle)]
pub struct Text {
    #[trackable]
    pub shape: TextShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: |world, entity, canvas| {
                    let shape = world.get::<&TextShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let draw = world.get::<&Draw>(entity).unwrap();
                    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();
                    let font = shape.font.skia_font(shape.size);
                    let mut paint = skia_safe::Paint::new(
                        skia_safe::Color4f::new(
                            fill_r,
                            fill_g,
                            fill_b,
                            fill_a * draw.opacity.clamp(0.0, 1.0),
                        ),
                        None,
                    );
                    paint.set_anti_alias(true);
                    let (width, _) = font.measure_str(&shape.text, Some(&paint));
                    let (_, metrics) = font.metrics();
                    let origin = (-width * 0.5, -(metrics.ascent + metrics.descent) * 0.5);

                    canvas.draw_str(&shape.text, origin, &font, &paint);

                    if style.stroke_width > 0.0 {
                        let [stroke_r, stroke_g, stroke_b, stroke_a] = style.stroke.rgba();
                        paint.set_color4f(
                            skia_safe::Color4f::new(
                                stroke_r,
                                stroke_g,
                                stroke_b,
                                stroke_a * draw.opacity.clamp(0.0, 1.0),
                            ),
                            None,
                        );
                        paint.set_style(skia_safe::PaintStyle::Stroke);
                        paint.set_stroke_width(style.stroke_width);
                        canvas.draw_str(&shape.text, origin, &font, &paint);
                    }
                },
                get_box: |world, entity| {
                    let shape = world.get::<&TextShape>(entity).unwrap();
                    let font = shape.font.skia_font(shape.size);
                    let (width, _) = font.measure_str(&shape.text, None);
                    let (_, metrics) = font.metrics();

                    Vector2::new(width, metrics.descent - metrics.ascent)
                },
                ..Default::default()
            },
        }
    }
}
