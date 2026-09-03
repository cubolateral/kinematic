use kinematic_macros::{Object, Trackable};

use crate::core::{
    Easing,
    components::{Draw, Style, Transform, stroke_width_for_scale},
    types::{Color, Vector2},
};

type FontCache = std::collections::HashMap<std::path::PathBuf, skia_safe::Typeface>;

const WRITE_STAGGER: f32 = 0.25;
const WRITE_OUTLINE_PHASE: f32 = 0.25;

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
#[derive(Clone, Trackable)]
pub struct TextShape {
    /// Text displayed by the object.
    #[track]
    pub text: String,
    /// Font size in logical canvas units.
    #[track]
    pub size: f32,
    /// Horizontal line alignment from `-1.0` left to `1.0` right.
    #[track]
    pub align: f32,
    /// Normalized progress used by text writing effects.
    #[track]
    pub write_progress: f32,
    /// Initial scale applied to each written unit.
    #[track]
    pub write_scale: f32,
    /// Whether write units are words instead of characters.
    #[track]
    pub write_by_word: bool,
    /// Whether write units are sequenced from right to left.
    #[track]
    pub write_reverse: bool,
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
            align: 0.0,
            write_progress: 1.0,
            write_scale: 0.0,
            write_by_word: false,
            write_reverse: false,
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

struct TextLine<'a> {
    text: &'a str,
    width: f32,
    origin: (f32, f32),
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

fn inner_outline_path(path: &skia_safe::Path, width: f32) -> Option<skia_safe::Path> {
    let mut stroke_paint = text_paint(Color::WHITE, 1.0);
    stroke_paint.set_style(skia_safe::PaintStyle::Stroke);
    stroke_paint.set_stroke_width(width * 2.0);

    let mut stroke_builder = skia_safe::PathBuilder::new();
    if !skia_safe::path_utils::fill_path_with_paint(
        path,
        &stroke_paint,
        &mut stroke_builder,
        None,
        None,
    ) {
        return None;
    }

    path.op(&stroke_builder.detach(), skia_safe::PathOp::Intersect)
}

fn text_lines<'a>(shape: &'a TextShape, font: &skia_safe::Font) -> Vec<TextLine<'a>> {
    let lines: Vec<_> = shape
        .text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .map(|line| {
            let (width, _) = font.measure_str(line, None);

            (line, width)
        })
        .collect();
    let max_width = lines.iter().map(|(_, width)| *width).fold(0.0, f32::max);
    let (_, metrics) = font.metrics();
    let line_spacing = font.spacing();
    let block_height =
        metrics.descent - metrics.ascent + line_spacing * lines.len().saturating_sub(1) as f32;
    let first_baseline = -block_height * 0.5 - metrics.ascent;
    let alignment = (shape.align.clamp(-1.0, 1.0) + 1.0) * 0.5;

    lines
        .into_iter()
        .enumerate()
        .map(|(index, (text, width))| TextLine {
            text,
            width,
            origin: (
                -max_width * 0.5 + (max_width - width) * alignment,
                first_baseline + index as f32 * line_spacing,
            ),
        })
        .collect()
}

fn draw_complete_text(
    shape: &TextShape,
    style: &Style,
    opacity: f32,
    scale: Vector2,
    canvas: &skia_safe::Canvas,
) {
    let font = shape.font.skia_font(shape.size);
    let mut paint = text_paint(style.fill, opacity);
    let lines = text_lines(shape, &font);

    for line in &lines {
        canvas.draw_str(line.text, line.origin, &font, &paint);
    }

    if style.stroke_width <= 0.0 {
        return;
    }

    paint.set_color4f(text_paint(style.stroke, opacity).color4f(), None);
    paint.set_style(skia_safe::PaintStyle::Stroke);
    paint.set_stroke_width(stroke_width_for_scale(style.stroke_width, scale));

    for line in lines {
        canvas.draw_str(line.text, line.origin, &font, &paint);
    }
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

fn draw_written_text(
    shape: &TextShape,
    style: &Style,
    opacity: f32,
    object_scale: Vector2,
    canvas: &skia_safe::Canvas,
) {
    let font = shape.font.skia_font(shape.size);
    let layout_paint = text_paint(style.fill, opacity);
    let lines = text_lines(shape, &font);
    let units: Vec<_> = lines
        .iter()
        .enumerate()
        .flat_map(|(line_index, line)| {
            write_units(line.text, shape.write_by_word)
                .into_iter()
                .map(move |(start, end)| (line_index, start, end))
        })
        .collect();

    if units.is_empty() {
        return;
    }

    let (_, metrics) = font.metrics();
    let progress = shape.write_progress.clamp(0.0, 1.0);
    let unit_count = units.len() as f32;
    let sequence_duration = 1.0 + (unit_count - 1.0) * WRITE_STAGGER;

    for (unit_index, (line_index, start, end)) in units.into_iter().enumerate() {
        let sequence_index = if shape.write_reverse {
            unit_count as usize - 1 - unit_index
        } else {
            unit_index
        };
        let unit_start = sequence_index as f32 * WRITE_STAGGER;
        let local_progress = (progress * sequence_duration - unit_start).clamp(0.0, 1.0);

        let line = &lines[line_index];
        let unit = &line.text[start..end];
        let prefix = &line.text[..start];
        let (prefix_width, _) = font.measure_str(prefix, Some(&layout_paint));
        let (unit_width, _) = font.measure_str(unit, Some(&layout_paint));
        let x = line.origin.0 + prefix_width;
        let center = (
            x + unit_width * 0.5,
            line.origin.1 + (metrics.ascent + metrics.descent) * 0.5,
        );
        let scale_progress = Easing::OutBack.evaluate(local_progress);
        let scale = shape.write_scale + (1.0 - shape.write_scale) * scale_progress;
        let outline_progress = (local_progress / WRITE_OUTLINE_PHASE).clamp(0.0, 1.0);
        let fill_progress =
            ((local_progress - WRITE_OUTLINE_PHASE) / (1.0 - WRITE_OUTLINE_PHASE)).clamp(0.0, 1.0);
        let fill = interpolate_color(Color::TRANSPARENT, style.fill, fill_progress);
        let save_count = canvas.save();

        canvas.translate(center);
        canvas.scale((scale, scale));
        canvas.translate((-center.0, -center.1));

        let mut paint = text_paint(fill, opacity);
        canvas.draw_str(unit, (x, line.origin.1), &font, &paint);

        if shape.write_outline_width > 0.0 {
            let outline = color_with_alpha(style.fill, outline_progress);
            let glyph_path =
                skia_safe::utils::text_utils::get_path(unit, (x, line.origin.1), &font);

            if let Some(outline_path) = inner_outline_path(&glyph_path, shape.write_outline_width) {
                paint = text_paint(outline, opacity);
                canvas.draw_path(&outline_path, &paint);
            }
        }

        if style.stroke_width > 0.0 {
            let stroke = color_with_alpha(style.stroke, fill_progress);
            paint = text_paint(stroke, opacity);
            paint.set_style(skia_safe::PaintStyle::Stroke);
            paint.set_stroke_width(stroke_width_for_scale(
                style.stroke_width,
                object_scale * scale,
            ));
            canvas.draw_str(unit, (x, line.origin.1), &font, &paint);
        }

        canvas.restore_to_count(save_count);
    }
}

fn draw_text(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas, opacity: f32) {
    let shape = world.get::<&TextShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let transform = world.get::<&Transform>(entity).unwrap();

    if shape.write_progress >= 1.0 {
        draw_complete_text(&shape, &style, opacity, transform.scale, canvas);
    } else {
        draw_written_text(&shape, &style, opacity, transform.scale, canvas);
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
                    let lines = text_lines(&shape, &font);
                    let (_, metrics) = font.metrics();
                    let width = lines.iter().map(|line| line.width).fold(0.0, f32::max);
                    let height = metrics.descent - metrics.ascent
                        + font.spacing() * lines.len().saturating_sub(1) as f32;

                    Vector2::new(width, height)
                },
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approximately_equal(left: f32, right: f32) -> bool {
        (left - right).abs() < 0.001
    }

    #[test]
    fn lays_out_multiple_lines_with_interpolated_alignment() {
        let mut shape = TextShape {
            text: "Longest!\nShort!".to_owned(),
            ..Default::default()
        };
        let font = shape.font.skia_font(shape.size);

        shape.align = -1.0;
        let left = text_lines(&shape, &font);
        assert!(approximately_equal(left[0].origin.0, left[1].origin.0));

        shape.align = 0.0;
        let center = text_lines(&shape, &font);
        assert!(approximately_equal(
            center[0].origin.0 + center[0].width * 0.5,
            center[1].origin.0 + center[1].width * 0.5,
        ));

        shape.align = 1.0;
        let right = text_lines(&shape, &font);
        assert!(approximately_equal(
            right[0].origin.0 + right[0].width,
            right[1].origin.0 + right[1].width,
        ));
        assert!(approximately_equal(
            right[1].origin.1 - right[0].origin.1,
            font.spacing(),
        ));
    }

    #[test]
    fn inner_write_outline_matches_the_glyph_boundary() {
        let shape = TextShape::default();
        let font = shape.font.skia_font(shape.size);
        let glyph = skia_safe::utils::text_utils::get_path("Border!", (0.0, 0.0), &font);
        let outline = inner_outline_path(&glyph, 4.0).unwrap();
        let glyph_bounds = glyph.compute_tight_bounds();
        let outline_bounds = outline.compute_tight_bounds();

        assert!(approximately_equal(glyph_bounds.left, outline_bounds.left,));
        assert!(approximately_equal(glyph_bounds.top, outline_bounds.top,));
        assert!(approximately_equal(
            glyph_bounds.right,
            outline_bounds.right,
        ));
        assert!(approximately_equal(
            glyph_bounds.bottom,
            outline_bounds.bottom,
        ));
    }
}
