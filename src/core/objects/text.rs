use kinematic_macros::{Object, Trackable};

use crate::core::components::{Draw, Style, Transform};

type FontCache = std::collections::HashMap<(usize, std::path::PathBuf), femtovg::FontId>;

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

    fn id(&self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) -> femtovg::FontId {
        let path = self.path.clone();
        let canvas = vg as *const femtovg::Canvas<femtovg::renderer::OpenGl> as usize;
        let key = (canvas, path.clone());
        let mut cache = FONT_CACHE.lock().unwrap();

        *cache.entry(key).or_insert_with(|| {
            let data = std::fs::read(&path).unwrap_or_else(|error| {
                panic!("Font at `{}` could not be read: {error}.", path.display())
            });

            vg.add_font_mem(&data).unwrap_or_else(|error| {
                panic!(
                    "Font at `{}` could not be parsed: {error:?}.",
                    path.display()
                )
            })
        })
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
                get_rect: |world, entity, vg| {
                    let shape = world.get::<&TextShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let font = shape.font.id(vg);
                    let paint = femtovg::Paint::color(femtovg::Color::white())
                        .with_font(&[font])
                        .with_font_size(shape.size)
                        .with_text_align(femtovg::Align::Center)
                        .with_text_baseline(femtovg::Baseline::Middle);
                    let metrics = vg
                        .measure_text(0.0, 0.0, &shape.text, &paint)
                        .expect("Text bounds must be measured.");
                    let font_metrics = vg
                        .measure_font(&paint)
                        .expect("Font bounds must be measured.");
                    let height = font_metrics.ascender() - font_metrics.descender();
                    let padding = style.stroke_width.max(0.0) * 0.5 + 1.0;

                    [
                        metrics.x - padding,
                        -height * 0.5 - padding,
                        metrics.width() + padding * 2.0,
                        height + padding * 2.0,
                    ]
                },
                on_draw: |world, entity, vg| {
                    let shape = world.get::<&TextShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let [fill_r, fill_g, fill_b, fill_a] = style.fill.rgba();
                    let font = shape.font.id(vg);

                    let fill_paint = femtovg::Paint::color(femtovg::Color::rgbaf(
                        fill_r, fill_g, fill_b, fill_a,
                    ))
                    .with_font(&[font])
                    .with_font_size(shape.size)
                    .with_text_align(femtovg::Align::Center)
                    .with_text_baseline(femtovg::Baseline::Middle);

                    vg.fill_text(0.0, 0.0, &shape.text, &fill_paint)
                        .expect("Text fill must render.");

                    if style.stroke_width > 0.0 {
                        let [stroke_r, stroke_g, stroke_b, stroke_a] = style.stroke.rgba();
                        let stroke_paint = femtovg::Paint::color(femtovg::Color::rgbaf(
                            stroke_r, stroke_g, stroke_b, stroke_a,
                        ))
                        .with_line_width(style.stroke_width)
                        .with_font(&[font])
                        .with_font_size(shape.size)
                        .with_text_align(femtovg::Align::Center)
                        .with_text_baseline(femtovg::Baseline::Middle);

                        vg.stroke_text(0.0, 0.0, &shape.text, &stroke_paint)
                            .expect("Text stroke must render.");
                    }
                },
                ..Default::default()
            },
        }
    }
}
