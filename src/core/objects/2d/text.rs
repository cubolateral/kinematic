use kinematic_macros::{Object, Trackable};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::{
    Easing, Task, Tween,
    components::PARTICLE_COUNT,
    components::{Draw2D, Morph, Style, Transform2D, stroke_width_for_scale},
    objects::{
        CreationDraw, ObjectHandler,
        particle::{ParticleTransform, Silhouette, morph_opacities},
        particle_visual_key,
        string_morph::{
            ContentMorph, ContentMorphTransition, GlyphLayer, MovingGlyphLayer, TextMorphPlan,
            morph_text,
        },
    },
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
#[object(spatial = "2d", builder = "text")]
#[morph]
pub struct Text {
    #[trackable]
    pub shape: TextShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
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

struct LayoutCluster {
    text: String,
    glyphs: Vec<skia_safe::GlyphId>,
    positions: Vec<skia_safe::Point>,
    origin: Vector2,
}

fn text_clusters(shape: &TextShape) -> Vec<LayoutCluster> {
    let font = shape.font.skia_font(shape.size);
    let mut clusters = Vec::new();

    for line in text_lines(shape, &font) {
        let mut advance = 0.0;
        for text in line.text.graphemes(true) {
            let glyphs = font.text_to_glyphs_vec(text);
            let origin = Vector2::new(line.origin.0 + advance, line.origin.1);
            let mut glyph_positions = vec![skia_safe::Point::default(); glyphs.len()];
            font.get_pos(
                &glyphs,
                &mut glyph_positions,
                Some(skia_safe::Point::new(origin.x, origin.y)),
            );
            clusters.push(LayoutCluster {
                text: text.to_owned(),
                glyphs,
                positions: glyph_positions,
                origin,
            });
            advance += font.measure_str(text, None).0;
        }
    }

    clusters
}

fn match_clusters(from: &[LayoutCluster], to: &[LayoutCluster]) -> Vec<(usize, usize)> {
    let columns = to.len() + 1;
    let mut lengths = vec![0usize; (from.len() + 1) * columns];

    for from_index in (0..from.len()).rev() {
        for to_index in (0..to.len()).rev() {
            let index = from_index * columns + to_index;
            lengths[index] = if from[from_index].text == to[to_index].text {
                1 + lengths[(from_index + 1) * columns + to_index + 1]
            } else {
                lengths[(from_index + 1) * columns + to_index]
                    .max(lengths[from_index * columns + to_index + 1])
            };
        }
    }

    let mut matches = Vec::new();
    let (mut from_index, mut to_index) = (0, 0);
    while from_index < from.len() && to_index < to.len() {
        if from[from_index].text == to[to_index].text {
            matches.push((from_index, to_index));
            from_index += 1;
            to_index += 1;
        } else if lengths[(from_index + 1) * columns + to_index]
            >= lengths[from_index * columns + to_index + 1]
        {
            from_index += 1;
        } else {
            to_index += 1;
        }
    }

    let mut matched_from = vec![false; from.len()];
    let mut matched_to = vec![false; to.len()];
    for &(from_index, to_index) in &matches {
        matched_from[from_index] = true;
        matched_to[to_index] = true;
    }

    for source in 0..from.len() {
        if matched_from[source] {
            continue;
        }
        let target = (0..to.len())
            .filter(|&target| !matched_to[target] && from[source].text == to[target].text)
            .min_by(|&left, &right| {
                from[source]
                    .origin
                    .distance_squared(to[left].origin)
                    .total_cmp(&from[source].origin.distance_squared(to[right].origin))
                    .then_with(|| left.cmp(&right))
            });
        if let Some(target) = target {
            matched_from[source] = true;
            matched_to[target] = true;
            matches.push((source, target));
        }
    }

    matches.sort_unstable();
    matches
}

fn glyph_layer(clusters: &[LayoutCluster], included: impl Fn(usize) -> bool) -> GlyphLayer {
    let mut glyphs = Vec::new();
    let mut positions = Vec::new();
    for (index, cluster) in clusters.iter().enumerate() {
        if included(index) {
            glyphs.extend_from_slice(&cluster.glyphs);
            positions.extend_from_slice(&cluster.positions);
        }
    }

    GlyphLayer { glyphs, positions }
}

fn draw_glyphs(
    glyphs: &[skia_safe::GlyphId],
    positions: &[skia_safe::Point],
    font: &skia_safe::Font,
    style: &Style,
    opacity: f32,
    scale: Vector2,
    canvas: &skia_safe::Canvas,
) {
    if glyphs.is_empty() {
        return;
    }
    let mut paint = text_paint(style.fill, opacity);
    canvas.draw_glyphs_at(glyphs, positions, (0.0, 0.0), font, &paint);

    if style.stroke_width <= 0.0 {
        return;
    }
    paint.set_color4f(text_paint(style.stroke, opacity).color4f(), None);
    paint.set_style(skia_safe::PaintStyle::Stroke);
    paint.set_stroke_width(stroke_width_for_scale(style.stroke_width, scale));
    canvas.draw_glyphs_at(glyphs, positions, (0.0, 0.0), font, &paint);
}

fn capture_text_morph_silhouette(
    shape: &TextShape,
    style: &Style,
    transform: &Transform2D,
    glyphs: &[skia_safe::GlyphId],
    positions: &[skia_safe::Point],
    particle_count: usize,
) -> Silhouette {
    let size = text_box(shape);
    let padding = stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5 + 2.0;
    let bounds = skia_safe::Rect::new(
        -size.x * 0.5 - padding,
        -size.y * 0.5 - padding,
        size.x * 0.5 + padding,
        size.y * 0.5 + padding,
    );
    Silhouette::capture(bounds, particle_count, |canvas| {
        let font = shape.font.skia_font(shape.size);
        draw_glyphs(
            glyphs,
            positions,
            &font,
            style,
            1.0,
            transform.scale,
            canvas,
        );
    })
}

fn text_morph_silhouette(
    shape: &TextShape,
    style: &Style,
    transform: &Transform2D,
    layer: &GlyphLayer,
    total_glyphs: usize,
) -> Silhouette {
    let particle_count = if layer.glyphs.is_empty() {
        0
    } else {
        ((PARTICLE_COUNT as usize * layer.glyphs.len()).div_ceil(total_glyphs.max(1)))
            .clamp(128, PARTICLE_COUNT as usize)
    };

    capture_text_morph_silhouette(
        shape,
        style,
        transform,
        &layer.glyphs,
        &layer.positions,
        particle_count,
    )
}

fn prepare_text_morph(
    from_shape: &TextShape,
    from_style: &Style,
    from_transform: &Transform2D,
    to_shape: &TextShape,
    to_style: &Style,
    to_transform: &Transform2D,
) -> TextMorphPlan {
    let from = text_clusters(from_shape);
    let to = text_clusters(to_shape);
    let matches = match_clusters(&from, &to);
    let last_from_match = matches
        .iter()
        .max_by_key(|(from_index, _)| from_index)
        .map(|(from_index, _)| *from_index);
    let last_to_match = matches
        .iter()
        .max_by_key(|(_, to_index)| to_index)
        .map(|(_, to_index)| *to_index);
    let mut matched_from = vec![false; from.len()];
    let mut matched_to = vec![false; to.len()];
    let mut stable = MovingGlyphLayer {
        glyphs: Vec::new(),
        from: Vec::new(),
        to: Vec::new(),
    };
    let mut from_anchors = Vec::new();
    let mut to_anchors = Vec::new();

    for (from_index, to_index) in matches {
        let source = &from[from_index];
        let target = &to[to_index];
        assert_eq!(
            source.glyphs.len(),
            target.glyphs.len(),
            "Equal text clusters must contain the same glyph count."
        );
        matched_from[from_index] = true;
        matched_to[to_index] = true;
        stable.glyphs.extend_from_slice(&source.glyphs);
        stable.from.extend_from_slice(&source.positions);
        stable.to.extend_from_slice(&target.positions);
        from_anchors.push(source.origin);
        to_anchors.push(target.origin);
    }

    let source = glyph_layer(&from, |index| !matched_from[index]);
    let target = glyph_layer(&to, |index| !matched_to[index]);
    let from_glyph_count = from.iter().map(|cluster| cluster.glyphs.len()).sum();
    let to_glyph_count = to.iter().map(|cluster| cluster.glyphs.len()).sum();
    let mut from_silhouette = text_morph_silhouette(
        from_shape,
        from_style,
        from_transform,
        &source,
        from_glyph_count,
    );
    let mut to_silhouette =
        text_morph_silhouette(to_shape, to_style, to_transform, &target, to_glyph_count);

    if from_silhouette.is_empty() && !to_silhouette.is_empty() {
        if let Some(cluster) = last_from_match.map(|index| &from[index]) {
            from_silhouette = capture_text_morph_silhouette(
                from_shape,
                from_style,
                from_transform,
                &cluster.glyphs,
                &cluster.positions,
                to_silhouette.sample_count(),
            );
        }
        if from_silhouette.is_empty() {
            from_silhouette = to_silhouette.collapsed_at(&from_anchors);
        }
    } else if to_silhouette.is_empty() && !from_silhouette.is_empty() {
        if let Some(cluster) = last_to_match.map(|index| &to[index]) {
            to_silhouette = capture_text_morph_silhouette(
                to_shape,
                to_style,
                to_transform,
                &cluster.glyphs,
                &cluster.positions,
                from_silhouette.sample_count(),
            );
        }
        if to_silhouette.is_empty() {
            to_silhouette = from_silhouette.collapsed_at(&to_anchors);
        }
    }

    TextMorphPlan {
        stable,
        source,
        target,
        particles: ParticleTransform::sampled(
            from_silhouette,
            to_silhouette,
            crate::core::Easing::Linear,
        ),
    }
}

fn draw_text_morph(
    transition: &ContentMorphTransition,
    shape: &TextShape,
    style: &Style,
    transform: &Transform2D,
    progress: f32,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let plan = transition.text_plan();
    let font = shape.font.skia_font(shape.size);
    let (source_opacity, target_opacity) = morph_opacities(progress);
    draw_glyphs(
        &plan.source.glyphs,
        &plan.source.positions,
        &font,
        style,
        opacity * source_opacity,
        transform.scale,
        canvas,
    );
    draw_glyphs(
        &plan.target.glyphs,
        &plan.target.positions,
        &font,
        style,
        opacity * target_opacity,
        transform.scale,
        canvas,
    );
    plan.particles.draw(canvas, progress, opacity);

    let movement = progress * progress * (3.0 - 2.0 * progress);
    let positions: Vec<_> = plan
        .stable
        .from
        .iter()
        .zip(&plan.stable.to)
        .map(|(from, to)| {
            skia_safe::Point::new(
                from.x + (to.x - from.x) * movement,
                from.y + (to.y - from.y) * movement,
            )
        })
        .collect();
    draw_glyphs(
        &plan.stable.glyphs,
        &positions,
        &font,
        style,
        opacity,
        transform.scale,
        canvas,
    );
}

struct WriteStep {
    target: GlyphLayer,
    bounds: skia_safe::Rect,
    visual_key: u64,
}

struct WritePlan {
    steps: Vec<WriteStep>,
    character_duration: f32,
    interval: f32,
    particle_count: usize,
    easing: Easing,
    reverse: bool,
}

const WRITE_INTERVAL_RATIO: f32 = 0.1;

#[derive(Default, Trackable)]
struct WriteState {
    #[track]
    progress: f32,
    #[track]
    transition: u32,
    #[track]
    active: bool,

    plans: Vec<WritePlan>,
}

fn prepare_write_plan(
    shape: &TextShape,
    style: &Style,
    transform: &Transform2D,
    duration: f32,
    easing: Easing,
    reverse: bool,
) -> WritePlan {
    let clusters = text_clusters(shape);
    let count = clusters.len();
    let character_duration =
        duration / (1.0 + count.saturating_sub(1) as f32 * WRITE_INTERVAL_RATIO);
    let interval = character_duration * WRITE_INTERVAL_RATIO;
    let particle_count = if count == 0 {
        0
    } else {
        (PARTICLE_COUNT as usize)
            .div_ceil(count)
            .clamp(128, PARTICLE_COUNT as usize)
    };
    let mut steps = Vec::with_capacity(clusters.len());
    let font_path = shape.font.path().to_string_lossy();

    for (index, cluster) in clusters.iter().enumerate() {
        let target = glyph_layer(&clusters, |candidate| candidate == index);
        let bounds = glyph_layer_bounds(shape, style, transform, &target);
        let visual_key = particle_visual_key(
            "Write",
            style,
            &[
                shape.size,
                transform.scale.x,
                transform.scale.y,
                index as f32,
            ],
            &[&cluster.text, &font_path],
        );
        steps.push(WriteStep {
            target,
            bounds,
            visual_key,
        });
    }

    WritePlan {
        steps,
        character_duration,
        interval,
        particle_count,
        easing,
        reverse,
    }
}

fn glyph_layer_bounds(
    shape: &TextShape,
    style: &Style,
    transform: &Transform2D,
    layer: &GlyphLayer,
) -> skia_safe::Rect {
    let font = shape.font.skia_font(shape.size);
    let mut glyph_bounds = vec![skia_safe::Rect::default(); layer.glyphs.len()];
    font.get_bounds(&layer.glyphs, &mut glyph_bounds, None);
    let mut bounds: Option<skia_safe::Rect> = None;

    for (mut glyph, position) in glyph_bounds.into_iter().zip(&layer.positions) {
        if glyph.is_empty() {
            continue;
        }
        glyph.offset((position.x, position.y));
        match &mut bounds {
            Some(bounds) => bounds.join(glyph),
            None => bounds = Some(glyph),
        }
    }

    let Some(mut bounds) = bounds else {
        return skia_safe::Rect::default();
    };
    let padding = stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5 + 2.0;
    bounds.outset((padding, padding));
    bounds
}

fn draw_write(
    entity: hecs::Entity,
    state: &WriteState,
    shape: &TextShape,
    style: &Style,
    transform: &Transform2D,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let plan = &state.plans[state.transition as usize];
    let font = shape.font.skia_font(shape.size);

    for (index, step) in plan.steps.iter().enumerate() {
        let order = if plan.reverse {
            plan.steps.len() - index - 1
        } else {
            index
        };
        let elapsed = state.progress - order as f32 * plan.interval;
        if elapsed < 0.0 {
            if plan.reverse {
                draw_glyphs(
                    &step.target.glyphs,
                    &step.target.positions,
                    &font,
                    style,
                    opacity,
                    transform.scale,
                    canvas,
                );
            }
            continue;
        }
        let local = plan
            .easing
            .evaluate((elapsed / plan.character_duration).clamp(0.0, 1.0));
        let progress = if plan.reverse { 1.0 - local } else { local };
        let morph = Morph {
            progress,
            particles_enabled: true,
        };
        if !(CreationDraw {
            entity,
            cache_slot: index as u64 + 1,
            bounds: step.bounds,
            visual_key: step.visual_key,
            particle_count: plan.particle_count,
            style,
            morph: &morph,
            opacity,
            canvas,
        })
        .render(|target, target_opacity| {
            draw_glyphs(
                &step.target.glyphs,
                &step.target.positions,
                &font,
                style,
                target_opacity,
                transform.scale,
                target,
            );
        }) {
            draw_glyphs(
                &step.target.glyphs,
                &step.target.positions,
                &font,
                style,
                opacity,
                transform.scale,
                canvas,
            );
        }
    }
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
    let morph_state = world.get::<&Morph>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();

    if let Ok(write) = world.get::<&WriteState>(entity)
        && write.active
    {
        draw_write(entity, &write, &shape, &style, &transform, opacity, canvas);
        return;
    }

    if let Ok(morph) = world.get::<&ContentMorph>(entity)
        && morph.active
        && morph.progress > 0.0
    {
        draw_text_morph(
            &morph.transitions[morph.transition as usize],
            &shape,
            &style,
            &transform,
            morph.progress.clamp(0.0, 1.0),
            opacity,
            canvas,
        );
        return;
    }

    if morph_state.particles_enabled && morph_state.progress < 1.0 {
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
            cache_slot: 0,
            bounds,
            visual_key,
            particle_count: PARTICLE_COUNT as usize,
            style: &style,
            morph: &morph_state,
            opacity,
            canvas,
        })
        .render(|target, target_opacity| {
            draw_complete_text(&shape, &style, target_opacity, transform.scale, target);
        }) {
            return;
        }
    }

    draw_complete_text(&shape, &style, opacity, transform.scale, canvas);
}

impl Default for Text {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: draw_text,
                get_box: |world, entity| text_box(&world.get::<&TextShape>(entity).unwrap()),
                ..Default::default()
            },
        }
    }
}

impl TextHandler {
    /// Morphs this text into `text` through particle silhouettes.
    ///
    /// Unlike [`crate::core::effects::morph`], this keeps the same text object and
    /// changes its discrete string value when the returned tween completes.
    pub fn morph(&self, text: impl Into<String>) -> Tween<Text> {
        let text = text.into();
        let from_text = self.get(TextShape::text_property());
        let tween = self.text(text.clone());
        morph_text(tween, self.get_id(), from_text, text, prepare_text_morph)
    }

    pub(crate) fn play_write(&self, duration: f32, easing: Easing, reverse: bool) {
        let opacity = self.get(Draw2D::opacity_property());
        let anchor = self.animate(Draw2D::opacity_property(), opacity);
        let (world, animator) = anchor.context();
        let plan = {
            let world = world.borrow();
            prepare_write_plan(
                &world.get::<&TextShape>(self.get_id()).unwrap(),
                &world.get::<&Style>(self.get_id()).unwrap(),
                &world.get::<&Transform2D>(self.get_id()).unwrap(),
                duration,
                easing,
                reverse,
            )
        };
        if plan.steps.is_empty() {
            return;
        }
        let total_duration = duration;
        let transition = {
            let mut world = world.borrow_mut();
            if world.get::<&WriteState>(self.get_id()).is_err() {
                world
                    .insert_one(self.get_id(), WriteState::default())
                    .unwrap();
            }
            let mut state = world.get::<&mut WriteState>(self.get_id()).unwrap();
            let transition = state.plans.len() as u32;
            state.plans.push(plan);
            transition
        };
        let progress = WriteState::progress_property()
            .handle(world.clone(), self.get_id(), animator.clone())
            .animate_from::<Text>(0.0, total_duration)
            .duration(total_duration)
            .easing(Easing::Linear)
            .task();
        let transition = WriteState::transition_property()
            .handle(world.clone(), self.get_id(), animator.clone())
            .animate_from::<Text>(transition, transition)
            .duration(total_duration)
            .easing(Easing::Linear)
            .task();
        let activate = WriteState::active_property()
            .handle(world.clone(), self.get_id(), animator.clone())
            .animate_from::<Text>(false, true)
            .duration(0.0)
            .easing(Easing::Linear)
            .task();
        let active = WriteState::active_property()
            .handle(world.clone(), self.get_id(), animator.clone())
            .animate_from::<Text>(true, false)
            .duration(total_duration)
            .easing(Easing::Linear)
            .task();
        let animation = Task::All(vec![activate, progress, transition, active]);
        if reverse {
            let hide = Draw2D::opacity_property()
                .handle(world, self.get_id(), animator.clone())
                .animate_from::<Text>(opacity, 0.0)
                .duration(0.0)
                .task();
            animator.play(Task::Chain(vec![animation, hide]));
        } else {
            animator.play(animation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene, SceneBuilder,
        effects::{Effect, unwrite, write},
    };

    fn pixels(scene: &Scene) -> Vec<skia_safe::Color> {
        let mut surface = skia_safe::surfaces::raster_n32_premul((640, 240)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((320.0, 120.0));
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();
        let mut colors = Vec::with_capacity(640 * 240);

        for y in 0..240 {
            for x in 0..640 {
                colors.push(pixels.get_color((x, y)));
            }
        }

        colors
    }

    #[test]
    fn consecutive_text_morphs_share_their_boundary_without_interrupting_the_first() {
        struct ConsecutiveMorphs;

        impl SceneBuilder for ConsecutiveMorphs {
            fn build(&mut self, scene: &mut Scene) {
                let text = text().text("Kinematic".to_owned()).build(scene);
                scene.get_world_2d().add(&text);
                text.morph("Is").play();
                text.morph("Awesome.").play();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut ConsecutiveMorphs), 2.0);

        assert_eq!(scene.get_world().len(), 4);
        scene.update(0.5);
        let first_morph = pixels(&scene);
        assert!(first_morph.iter().any(|color| color.a() > 0));
        {
            let world = scene.get_world();
            let mut query = world.query::<(&TextShape, &ContentMorph)>();
            let (shape, morph) = query.iter().next().unwrap();

            assert_eq!(shape.text, "Kinematic");
            assert_eq!(morph.progress, 0.5);
            assert_eq!(morph.transition, 0);
            assert_eq!(morph.transitions.len(), 2);
        }

        scene.update(0.9999);
        let before_first_handoff = pixels(&scene);
        scene.update(1.0);
        assert_eq!(before_first_handoff, pixels(&scene));

        scene.update(1.5);
        {
            let world = scene.get_world();
            let mut query = world.query::<(&TextShape, &ContentMorph)>();
            let (shape, morph) = query.iter().next().unwrap();

            assert_eq!(shape.text, "Is");
            assert_eq!(morph.progress, 0.5);
            assert_eq!(morph.transition, 1);
        }

        scene.update(2.0);
        scene.update(0.5);
        assert_eq!(pixels(&scene), first_morph);
        scene.update(2.0);
        let world = scene.get_world();
        let mut query = world.query::<&TextShape>();
        assert_eq!(query.iter().next().unwrap().text, "Awesome.");
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

    #[test]
    fn morph_matches_reordered_and_repeated_graphemes() {
        let from = text_clusters(&TextShape {
            text: "ABBA".to_owned(),
            ..Default::default()
        });
        let to = text_clusters(&TextShape {
            text: "BABA".to_owned(),
            ..Default::default()
        });
        let matches = match_clusters(&from, &to);

        assert_eq!(matches.len(), 4);
        assert!(
            matches
                .iter()
                .all(|&(source, target)| from[source].text == to[target].text)
        );
    }

    #[test]
    fn morph_treats_joined_emoji_as_one_character() {
        let from = text_clusters(&TextShape {
            text: "A👨‍👩‍👧B".to_owned(),
            ..Default::default()
        });
        let to = text_clusters(&TextShape {
            text: "B👨‍👩‍👧A".to_owned(),
            ..Default::default()
        });

        assert_eq!(from.len(), 3);
        assert_eq!(to.len(), 3);
        assert_eq!(match_clusters(&from, &to).len(), 3);
    }

    #[test]
    fn morph_keeps_common_glyphs_out_of_particle_silhouettes() {
        let from = TextShape {
            text: "KEEP".to_owned(),
            ..Default::default()
        };
        let to = TextShape {
            text: "SKEEP!".to_owned(),
            ..Default::default()
        };
        let plan = prepare_text_morph(
            &from,
            &Style::default(),
            &Transform2D::default(),
            &to,
            &Style::default(),
            &Transform2D::default(),
        );

        assert_eq!(plan.stable.glyphs.len(), 4);
        assert!(plan.source.glyphs.is_empty());
        assert_eq!(plan.target.glyphs.len(), 2);
        assert!(plan.particles.from.sample_y_span() > 1.0);
        assert!(plan.particles.from.sample_x_span() < text_box(&from).x * 0.5);
    }

    #[test]
    fn write_staggers_character_morphs_without_waiting_for_each_one() {
        struct WrittenText;

        impl SceneBuilder for WrittenText {
            fn build(&mut self, scene: &mut Scene) {
                let label = text().text("ABC".to_owned()).build(scene);
                scene.get_world_2d().add(&label);
                write().duration(1.0).play(&label);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut WrittenText), 1.0);

        scene.update(0.5);
        let world = scene.get_world();
        let mut query = world.query::<(&TextShape, &WriteState)>();
        let (shape, state) = query.iter().next().unwrap();
        assert_eq!(shape.text, "ABC");
        assert_eq!(state.progress, 0.5);
        assert!(state.active);
        let plan = &state.plans[state.transition as usize];
        assert_eq!(plan.steps.len(), 3);
        assert!(plan.interval < plan.character_duration);
        assert!(approximately_equal(
            plan.character_duration + plan.interval * 2.0,
            1.0,
        ));

        drop(query);
        drop(world);
        scene.update(1.0);
        let world = scene.get_world();
        assert!(!world.query::<&WriteState>().iter().next().unwrap().active);
    }

    #[test]
    fn write_activates_only_at_its_scheduled_time() {
        struct DelayedWrite;

        impl SceneBuilder for DelayedWrite {
            fn build(&mut self, scene: &mut Scene) {
                let label = text().text("AB".to_owned()).build(scene);
                scene.get_world_2d().add(&label);
                scene.wait(1.0);
                write().duration(1.0).play(&label);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut DelayedWrite), 2.0);
        scene.update(0.5);
        assert!(
            !scene
                .get_world()
                .query::<&WriteState>()
                .iter()
                .next()
                .unwrap()
                .active
        );
        scene.update(1.0);
        assert!(
            scene
                .get_world()
                .query::<&WriteState>()
                .iter()
                .next()
                .unwrap()
                .active
        );
    }

    #[test]
    fn unwrite_runs_in_reverse_and_keeps_the_text_hidden() {
        struct UnwrittenText;

        impl SceneBuilder for UnwrittenText {
            fn build(&mut self, scene: &mut Scene) {
                let label = text().text("ABC".to_owned()).build(scene);
                scene.get_world_2d().add(&label);
                unwrite().duration(1.0).play(&label);
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut UnwrittenText), 1.0);
        scene.update(0.0);
        {
            let world = scene.get_world();
            let mut query = world.query::<&WriteState>();
            let state = query.iter().next().unwrap();
            assert!(state.plans[state.transition as usize].reverse);
            assert!(state.active);
        }

        scene.update(1.0);
        let world = scene.get_world();
        assert!(!world.query::<&WriteState>().iter().next().unwrap().active);
        assert_eq!(
            world
                .query::<(&TextShape, &Draw2D)>()
                .iter()
                .next()
                .unwrap()
                .1
                .opacity,
            0.0,
        );
    }
}
