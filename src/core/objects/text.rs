use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{Draw, ParticleStyle, Style, Transform, stroke_width_for_scale},
    objects::{CreationDraw, particle_visual_key},
    types::{Color, Vector2},
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

    /// Font used to render the text.
    pub font: Font,
}

impl Default for TextShape {
    fn default() -> Self {
        Self {
            text: "Text!".to_owned(),
            size: 64.0,
            align: 0.0,
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
    pub particles: ParticleStyle,
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

fn text_box(shape: &TextShape) -> Vector2 {
    let font = shape.font.skia_font(shape.size);
    let lines = text_lines(shape, &font);
    let (_, metrics) = font.metrics();
    let width = lines.iter().map(|line| line.width).fold(0.0, f32::max);
    let height =
        metrics.descent - metrics.ascent + font.spacing() * lines.len().saturating_sub(1) as f32;

    Vector2::new(width, height)
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

fn draw_text(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas, opacity: f32) {
    let shape = world.get::<&TextShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let particles = world.get::<&ParticleStyle>(entity).unwrap();
    let transform = world.get::<&Transform>(entity).unwrap();

    if particles.particles_enabled && style.progress < 1.0 {
        let size = text_box(&shape);
        let stroke_padding =
            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5;
        let bounds = skia_safe::Rect::new(
            -size.x * 0.5 - stroke_padding,
            -size.y * 0.5 - stroke_padding,
            size.x * 0.5 + stroke_padding,
            size.y * 0.5 + stroke_padding,
        );
        let font_path = shape.font.path().to_string_lossy();
        let visual_key = particle_visual_key(
            "Text",
            &style,
            &[
                shape.size,
                shape.align,
                transform.scale.x,
                transform.scale.y,
            ],
            &[&shape.text, &font_path],
        );

        if (CreationDraw {
            entity,
            bounds,
            visual_key,
            style: &style,
            particles: &particles,
            opacity,
            canvas,
        })
        .render(|target, target_opacity| {
            draw_complete_text(&shape, &style, target_opacity, transform.scale, target);
        }) {
            return;
        }
    }

    let progress = style.progress.clamp(0.0, 1.0);
    if progress > 0.0 {
        draw_complete_text(&shape, &style, opacity * progress, transform.scale, canvas);
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            particles: Default::default(),
            transform: Default::default(),
            draw: Draw {
                on_draw: draw_text,
                get_box: |world, entity| text_box(&world.get::<&TextShape>(entity).unwrap()),
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
}
