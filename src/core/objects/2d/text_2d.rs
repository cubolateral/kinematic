use crate::core::{
    components::{Draw2D, Filter, Style, Transform2D, draw_complete_styled_path},
    types::Vector2,
};
use kinematic_macros::{Object, Trackable};

type FontCache = std::collections::HashMap<std::path::PathBuf, skia_safe::Typeface>;

static FONT_CACHE: std::sync::LazyLock<std::sync::Mutex<FontCache>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(FontCache::new()));

/// Font file used to render a text shape.
#[derive(Clone, Debug)]
pub struct Font {
    path: std::path::PathBuf,
    face: Option<skia_safe::Typeface>,
}

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.face.as_ref().map(|face| face.unique_id())
                == other.face.as_ref().map(|face| face.unique_id())
    }
}
impl Eq for Font {}

impl Font {
    /// Creates a font from a TTF or OTF file.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: Self::resolve_path(path.into()),
            face: None,
        }
    }

    pub(crate) fn from_data(path: std::path::PathBuf, bytes: Vec<u8>) -> Result<Self, String> {
        let face = skia_safe::FontMgr::new()
            .new_from_data(&bytes, None)
            .ok_or("Font data is invalid.")?;

        Ok(Self {
            path,
            face: Some(face),
        })
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

    pub(crate) fn resource_key(&self) -> String {
        format!(
            "{}:{:?}",
            self.path.display(),
            self.face.as_ref().map(|face| face.unique_id())
        )
    }

    /// Returns the font file path.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub(crate) fn skia_font(&self, size: f32) -> skia_safe::Font {
        if let Some(face) = &self.face {
            return skia_safe::Font::new(face.clone(), size);
        }

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

impl From<&str> for Font {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for Font {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<&std::path::Path> for Font {
    fn from(path: &std::path::Path) -> Self {
        Self::new(path)
    }
}

impl From<std::path::PathBuf> for Font {
    fn from(path: std::path::PathBuf) -> Self {
        Self::new(path)
    }
}

/// Fonts bundled with Kinematic.
pub mod fonts {
    /// CaskaydiaMono Nerd Font.
    pub const CASKAYDIA_MONO: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/CaskaydiaMonoNerdFont-Regular.ttf"
    );
    /// Hack Nerd Font Mono.
    pub const HACK_MONO: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/HackNerdFontMono-Regular.ttf"
    );
    /// JetBrainsMono Nerd Font.
    pub const JETBRAINS_MONO: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/JetBrainsMonoNerdFont-Regular.ttf"
    );
}

/// Content and typography of a text object.
#[derive(Clone, Trackable)]
pub struct TextShape {
    /// Text displayed by the object.
    #[track]
    pub text: String,
    /// Font size in logical canvas units.
    #[track(min = 0.0)]
    pub size: f32,
    /// Horizontal line alignment from `-1.0` left to `1.0` right.
    #[track(min = -1.0, max = 1.0)]
    pub align: f32,
    /// Extra glyph thickness in logical canvas units.
    #[track(min = 0.0)]
    pub thickness: f32,

    /// Font used to render the text.
    pub font: Font,
}

impl Default for TextShape {
    fn default() -> Self {
        Self {
            text: "Text!".to_owned(),
            size: 64.0,
            align: 0.0,
            thickness: 0.0,
            font: Font::new(fonts::HACK_MONO),
        }
    }
}

#[derive(Clone, Trackable)]
pub(crate) struct TextFade {
    #[track]
    pub text: String,
    #[track(min = 0.0, max = 1.0)]
    pub progress: f32,
    /// Keeps the transition active even when easing reaches full opacity early.
    #[track]
    pub active: bool,
}

impl Default for TextFade {
    fn default() -> Self {
        Self {
            text: String::new(),
            progress: 1.0,
            active: false,
        }
    }
}

impl TextFade {
    pub(crate) fn incoming_text(&self, current: &str) -> Option<&str> {
        (self.active && self.text != current).then_some(&self.text)
    }

    fn incoming_shape(&self, shape: &TextShape) -> Option<TextShape> {
        self.incoming_text(&shape.text).map(|text| TextShape {
            text: text.to_owned(),
            ..shape.clone()
        })
    }
}

/// Built-in text scene object.
#[derive(Object)]
#[object(spatial = "2d", builder = "text_2d")]
pub struct Text2D {
    #[trackable]
    pub shape: TextShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
    #[trackable]
    pub filter: Filter,

    pub(crate) fade: TextFade,
}

impl Text2DBuilder {
    /// Sets a bundled font or the path to a user-provided TTF or OTF file.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.object.shape.font = font.into();

        self
    }
}

struct TextLine<'a> {
    text: &'a str,
    width: f32,
    origin: (f32, f32),
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

pub(crate) fn text_box(shape: &TextShape) -> Vector2 {
    let font = shape.font.skia_font(shape.size);
    let lines = text_lines(shape, &font);
    let (_, metrics) = font.metrics();
    let width = lines.iter().map(|line| line.width).fold(0.0, f32::max);
    let height =
        metrics.descent - metrics.ascent + font.spacing() * lines.len().saturating_sub(1) as f32;

    let thickness = shape.thickness.max(0.0);

    Vector2::new(width + thickness, height + thickness)
}

/// Draws text in local 2D coordinates from reusable rendering data.
pub fn draw_text_2d(
    shape: &TextShape,
    style: &Style,
    scale: Vector2,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    draw_complete_styled_path(&text_path(shape), style, scale, opacity, canvas);
}

pub(crate) fn text_path(shape: &TextShape) -> skia_safe::Path {
    let font = shape.font.skia_font(shape.size);
    let mut builder = skia_safe::PathBuilder::new();

    for line in text_lines(shape, &font) {
        let glyphs = font.text_to_glyphs_vec(line.text);
        let mut positions = vec![skia_safe::Point::default(); glyphs.len()];

        font.get_pos(
            &glyphs,
            &mut positions,
            Some(skia_safe::Point::new(line.origin.0, line.origin.1)),
        );

        for (glyph, position) in glyphs.into_iter().zip(positions) {
            if let Some(path) = font.get_path(glyph) {
                let path = path.with_offset((position.x, position.y));

                builder.add_path(&path, None);
            }
        }
    }

    weighted_path(&builder.detach(), shape.thickness)
}

pub(crate) fn weighted_path(path: &skia_safe::Path, thickness: f32) -> skia_safe::Path {
    if thickness <= 0.0 {
        return path.clone();
    }

    let mut paint = skia_safe::Paint::default();

    paint
        .set_style(skia_safe::PaintStyle::Stroke)
        .set_stroke_join(skia_safe::PaintJoin::Round)
        .set_stroke_width(thickness);

    let mut stroke = skia_safe::PathBuilder::new();

    skia_safe::path_utils::fill_path_with_paint(path, &paint, &mut stroke, None, None);

    let stroke = stroke.detach();
    let mut combined = skia_safe::PathBuilder::new();

    combined.add_path(path, None).add_path(&stroke, None);

    let combined = combined.detach();

    path.op(&stroke, skia_safe::PathOp::Union)
        .or_else(|| combined.simplify())
        .unwrap_or(combined)
}

fn draw_text_object(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) {
    let shape = world.get::<&TextShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();

    let fade = world.get::<&TextFade>(entity).unwrap();

    if let Some(incoming) = fade.incoming_shape(&shape) {
        draw_text_2d(
            &shape,
            &style,
            transform.scale,
            opacity * (1.0 - fade.progress),
            canvas,
        );
        draw_text_2d(
            &incoming,
            &style,
            transform.scale,
            opacity * fade.progress,
            canvas,
        );
    } else {
        draw_text_2d(&shape, &style, transform.scale, opacity, canvas);
    }
}

impl Default for Text2D {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: draw_text_object,
                box_size: |world, entity| {
                    let shape = world.get::<&TextShape>(entity).unwrap();
                    let fade = world.get::<&TextFade>(entity).unwrap();
                    let size = text_box(&shape);

                    fade.incoming_shape(&shape).map_or(size, |incoming| {
                        let incoming = text_box(&incoming);

                        Vector2::new(size.x.max(incoming.x), size.y.max(incoming.y))
                    })
                },
                visual_bounds: |world, entity| {
                    let shape = world.get::<&TextShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();

                    let fade = world.get::<&TextFade>(entity).unwrap();
                    let mut bounds = *text_path(&shape).bounds();

                    if let Some(incoming) = fade.incoming_shape(&shape) {
                        bounds.join(text_path(&incoming).bounds());
                    }

                    crate::core::components::styled_bounds(bounds, &style, transform.scale)
                },
                ..Default::default()
            },
            filter: Default::default(),
            fade: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Scene, objects::ObjectHandler};

    #[test]
    fn builder_accepts_bundled_and_user_fonts_with_trackable_thickness() {
        let mut scene = Scene::new();
        let bundled = text_2d()
            .font(fonts::HACK_MONO)
            .thickness(3.0)
            .build(&mut scene);
        let custom_path = std::path::PathBuf::from(fonts::CASKAYDIA_MONO);
        let custom = text_2d().font(custom_path.clone()).build(&mut scene);
        let world = scene.world();

        assert_eq!(
            world.get::<&TextShape>(bundled.entity()).unwrap().font,
            Font::new(fonts::HACK_MONO)
        );

        assert_eq!(
            world.get::<&TextShape>(custom.entity()).unwrap().font,
            Font::new(custom_path)
        );

        assert_eq!(
            world.get::<&TextShape>(bundled.entity()).unwrap().thickness,
            3.0
        );

        assert_eq!(TextShape::thickness_property().info().name, "thickness");
    }

    #[test]
    fn thickness_expands_a_filled_path_without_creating_a_gap() {
        let path = skia_safe::Path::rect(skia_safe::Rect::from_xywh(0.0, 0.0, 10.0, 10.0), None);
        let weighted = weighted_path(&path, 4.0);
        let bounds = weighted.compute_tight_bounds();

        assert!(weighted.contains((5.0, 5.0)));
        assert!(!path.contains((-1.0, 5.0)));
        assert!(weighted.contains((-1.0, 5.0)));
        assert!((bounds.width() - 14.0).abs() < 0.01);
        assert!((bounds.height() - 14.0).abs() < 0.01);
    }

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
