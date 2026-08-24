use kinematic_macros::{Object, Trackable};

use crate::core::{
    Easing,
    components::{Draw, Style, Transform},
    types::{Color, Vector2},
};

type FontCache = std::collections::HashMap<std::path::PathBuf, skia_safe::Typeface>;

const REVEAL_STAGGER: f32 = 0.25;
const REVEAL_OUTLINE_PHASE: f32 = 0.25;

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
    /// Normalized progress used by text writing effects.
    #[track]
    pub write_progress: f32,
    /// Initial scale applied to each written unit.
    #[track]
    pub write_scale: f32,
    /// Whether write units are words instead of characters.
    #[track]
    pub write_by_word: bool,
    /// Temporary outline width used while writing text without a stroke.
    #[track]
    pub write_outline_width: f32,

    /// Font used to render the text.
    pub font: Font,
}

impl Default for TextShape {
    fn default() -> Self {
        Self {
            text: "Text!".to_owned(),
            size: 64.0,
            write_progress: 1.0,
            write_scale: 0.0,
            write_by_word: false,
            write_outline_width: 1.0,
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

fn text_paint(color: Color, opacity: f32) -> skia_safe::Paint {
    let [r, g, b, a] = color.rgba();
    let mut paint = skia_safe::Paint::new(
        skia_safe::Color4f::new(r, g, b, a * opacity.clamp(0.0, 1.0)),
        None,
    );
    paint.set_anti_alias(true);
    paint
}

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color::new(
        from.r + (to.r - from.r) * progress,
        from.g + (to.g - from.g) * progress,
        from.b + (to.b - from.b) * progress,
        from.a + (to.a - from.a) * progress,
    )
}

fn color_with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, color.a * alpha)
}

fn text_origin(shape: &TextShape, font: &skia_safe::Font, paint: &skia_safe::Paint) -> (f32, f32) {
    let (width, _) = font.measure_str(&shape.text, Some(paint));
    let (_, metrics) = font.metrics();

    (-width * 0.5, -(metrics.ascent + metrics.descent) * 0.5)
}

fn draw_complete_text(shape: &TextShape, style: &Style, draw: &Draw, canvas: &skia_safe::Canvas) {
    let font = shape.font.skia_font(shape.size);
    let mut paint = text_paint(style.fill, draw.opacity);
    let origin = text_origin(shape, &font, &paint);

    canvas.draw_str(&shape.text, origin, &font, &paint);

    if style.stroke_width <= 0.0 {
        return;
    }

    paint.set_color4f(text_paint(style.stroke, draw.opacity).color4f(), None);
    paint.set_style(skia_safe::PaintStyle::Stroke);
    paint.set_stroke_width(style.stroke_width);
    canvas.draw_str(&shape.text, origin, &font, &paint);
}

fn write_units(text: &str, by_word: bool) -> Vec<(usize, usize)> {
    if !by_word {
        return text
            .char_indices()
            .filter_map(|(start, character)| {
                (!character.is_whitespace()).then_some((start, start + character.len_utf8()))
            })
            .collect();
    }

    let mut units = vec![];
    let mut start = None;

    for (index, character) in text.char_indices() {
        if character.is_whitespace() {
            if let Some(start) = start.take() {
                units.push((start, index));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }

    if let Some(start) = start {
        units.push((start, text.len()));
    }

    units
}

fn draw_written_text(shape: &TextShape, style: &Style, draw: &Draw, canvas: &skia_safe::Canvas) {
    let units = write_units(&shape.text, shape.write_by_word);

    if units.is_empty() {
        return;
    }

    let font = shape.font.skia_font(shape.size);
    let layout_paint = text_paint(style.fill, draw.opacity);
    let origin = text_origin(shape, &font, &layout_paint);
    let progress = shape.write_progress.clamp(0.0, 1.0);
    let unit_count = units.len() as f32;
    let sequence_duration = 1.0 + (unit_count - 1.0) * REVEAL_STAGGER;

    for (unit_index, (start, end)) in units.into_iter().enumerate() {
        let unit_start = unit_index as f32 * REVEAL_STAGGER;
        let local_progress = (progress * sequence_duration - unit_start).clamp(0.0, 1.0);

        let unit = &shape.text[start..end];
        let prefix = &shape.text[..start];
        let (prefix_width, _) = font.measure_str(prefix, Some(&layout_paint));
        let (unit_width, _) = font.measure_str(unit, Some(&layout_paint));
        let x = origin.0 + prefix_width;
        let center = (x + unit_width * 0.5, 0.0);
        let scale_progress = Easing::OutBack.evaluate(local_progress);
        let scale = shape.write_scale + (1.0 - shape.write_scale) * scale_progress;
        let outline_progress = (local_progress / REVEAL_OUTLINE_PHASE).clamp(0.0, 1.0);
        let fill_progress = ((local_progress - REVEAL_OUTLINE_PHASE)
            / (1.0 - REVEAL_OUTLINE_PHASE))
            .clamp(0.0, 1.0);
        let fill = interpolate_color(Color::TRANSPARENT, style.fill, fill_progress);
        let save_count = canvas.save();

        canvas.translate(center);
        canvas.scale((scale, scale));
        canvas.translate((-center.0, -center.1));

        let mut paint = text_paint(fill, draw.opacity);
        canvas.draw_str(unit, (x, origin.1), &font, &paint);

        let has_stroke = style.stroke_width > 0.0;
        let outline_width = if has_stroke {
            style.stroke_width
        } else {
            shape.write_outline_width
        };

        if outline_width > 0.0 {
            let outline = color_with_alpha(style.stroke, outline_progress);
            let outline_path = skia_safe::utils::text_utils::get_path(unit, (x, origin.1), &font);
            let outline_save_count = canvas.save();

            canvas.clip_path(&outline_path, None, true);
            paint.set_color4f(text_paint(outline, draw.opacity).color4f(), None);
            paint.set_style(skia_safe::PaintStyle::Stroke);
            paint.set_stroke_width(outline_width * 2.0);
            canvas.draw_str(unit, (x, origin.1), &font, &paint);
            canvas.restore_to_count(outline_save_count);
        }

        canvas.restore_to_count(save_count);
    }
}

fn draw_text(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    let shape = world.get::<&TextShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let draw = world.get::<&Draw>(entity).unwrap();

    if shape.write_progress >= 1.0 {
        draw_complete_text(&shape, &style, &draw, canvas);
    } else {
        draw_written_text(&shape, &style, &draw, canvas);
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: draw_text,
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
