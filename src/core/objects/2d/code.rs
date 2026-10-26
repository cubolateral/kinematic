use crate::core::{
    components::{Draw2D, Filter, Style, Transform2D, draw_complete_styled_path},
    objects::{
        Font, TextFade, fonts,
        text_morph::{MorphPart, morph_parts},
    },
    types::{CodePosition, CodeRanges, Color, Vector2, code_interval},
};
use kinematic_macros::{Object, Trackable};

pub use super::code_syntax::CodeLanguage;
use super::code_syntax::{self, TokenKind};

/// Source, typography, syntax colors, and emphasis of a code object.
#[derive(Clone, Trackable)]
pub struct CodeShape {
    /// Code displayed by the object, preserving indentation.
    #[track]
    pub code: String,
    /// Font size in logical canvas units.
    #[track(min = 0.0)]
    pub size: f32,
    /// Line spacing as a multiple of the font's natural spacing.
    #[track(min = 0.1)]
    pub line_spacing: f32,
    /// Number of visual columns between tab stops.
    #[track(min = 1, max = 16)]
    pub tab_size: u32,
    /// Syntax language, defaulting to plain.
    /// Web: javascript, typescript, jsx, tsx, html, css, scss.
    /// General: python, c, cpp, csharp, java, rust, go, php, ruby, swift, kotlin, dart.
    /// Scripts and data: bash, powershell, sql, r.
    /// Configuration: json, jsonc, yaml, toml, xml, ini.
    /// Documents: markdown, latex. Build: dockerfile, makefile, cmake.
    /// Shaders: glsl, hlsl, wgsl.
    #[track]
    pub language: CodeLanguage,
    /// Color of keywords, types, and language-specific declarations.
    #[track]
    pub keyword_color: Color,
    /// Color of string literals and quoted values.
    #[track]
    pub string_color: Color,
    /// Color of numeric literals.
    #[track]
    pub number_color: Color,
    /// Color of line and long bracket comments.
    #[track]
    pub comment_color: Color,
    /// Color of operators and punctuation.
    #[track]
    pub operator_color: Color,
    /// Background color of highlighted code ranges.
    #[track]
    pub highlight_color: Color,
    /// Opacity multiplier for text outside the selection.
    #[track(min = 0.0, max = 1.0)]
    pub dim_opacity: f32,
    /// Selected half-open ranges, using one-based Unicode line and column positions.
    #[track]
    pub selection: CodeRanges,
    /// Ranges marked with a colored background, independent of selection.
    #[track]
    pub highlights: CodeRanges,

    /// Font used to draw the code, defaulting to Hack Nerd Font Mono.
    pub font: Font,
}

impl Default for CodeShape {
    fn default() -> Self {
        Self {
            code: String::new(),
            size: 36.0,
            line_spacing: 1.0,
            tab_size: 4,
            language: Default::default(),
            keyword_color: Color::new(0.78, 0.47, 0.87, 1.0),
            string_color: Color::new(0.60, 0.76, 0.47, 1.0),
            number_color: Color::new(0.82, 0.60, 0.40, 1.0),
            comment_color: Color::new(0.45, 0.51, 0.56, 1.0),
            operator_color: Color::new(0.34, 0.71, 0.76, 1.0),
            highlight_color: Color::new(0.30, 0.55, 1.0, 0.3),
            dim_opacity: 0.25,
            selection: Default::default(),
            highlights: Default::default(),
            font: Font::new(fonts::HACK_MONO),
        }
    }
}

/// Source edit boundaries used to preserve intact tokens during code transitions.
#[derive(Clone, Default, Trackable)]
pub struct CodeEditState {
    /// Whether this transition uses token edits instead of a block fade.
    #[track]
    pub active: bool,
    /// Edited intervals in the source code before this transition.
    #[track]
    pub ranges: CodeRanges,
    /// Text inserted at each edited interval.
    #[track]
    pub replacement: String,
}

/// Built-in 2D code object with syntax coloring, animated edits, and emphasis.
///
/// ```lua
/// local code = s:code {language = "python", code = "print(42)"}
/// s:world_2d():add(code)
/// code:append("print(100)"):play(1)
/// ```
#[derive(Object)]
#[object(spatial = "2d", builder = "code")]
pub struct Code {
    #[trackable]
    pub shape: CodeShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
    #[trackable]
    pub filter: Filter,

    pub(crate) fade: TextFade,
    pub(crate) edit: CodeEditState,
}

struct Cell {
    position: CodePosition,
    byte: usize,
    token: std::ops::Range<usize>,
    rect: skia_safe::Rect,
    part: MorphPart,
}

struct Layout {
    cells: Vec<Cell>,
    size: Vector2,
}

fn layout(shape: &CodeShape, source: &str, fill: Color) -> Layout {
    let font = shape.font.skia_font(shape.size);
    let (_, metrics) = font.metrics();
    let spacing = font.spacing() * shape.line_spacing;
    let height = metrics.descent - metrics.ascent;
    let tab = shape.tab_size.max(1) as usize;
    let mut cells = Vec::new();
    let mut max_width = 0.0_f32;
    let mut byte_offset = 0;
    let tokens = syntax_tokens(source, shape, fill);
    let mut token_index = 0;
    let (space, _) = font.measure_str(" ", None);
    let mut count = 0_usize;
    for (line_index, line) in source.split('\n').enumerate() {
        let raw_length = line.len();
        count += 1;
        let line = line.strip_suffix('\r').unwrap_or(line);
        let mut x = 0.0;
        let mut visual_column = 0;
        for (column, (byte, ch)) in line.char_indices().enumerate() {
            let text = ch.to_string();
            let width = if ch == '\t' {
                (tab - visual_column % tab) as f32 * space
            } else {
                font.measure_str(&text, None).0
            };
            let y = line_index as f32 * spacing;
            let mut builder = skia_safe::PathBuilder::new();
            let glyphs = font.text_to_glyphs_vec(&text);
            let mut positions = vec![skia_safe::Point::default(); glyphs.len()];
            font.get_pos(
                &glyphs,
                &mut positions,
                Some((x, y - metrics.ascent).into()),
            );
            if ch != '\t' {
                for (glyph, position) in glyphs.into_iter().zip(positions) {
                    if let Some(path) = font.get_path(glyph) {
                        builder.add_path(&path.with_offset(position), None);
                    }
                }
            }
            while tokens[token_index].0.end <= byte_offset + byte {
                token_index += 1;
            }
            cells.push(Cell {
                byte: byte_offset + byte,
                token: tokens[token_index].0.clone(),
                position: [line_index + 1, column + 1],
                rect: skia_safe::Rect::from_xywh(x, y, width, height),
                part: MorphPart {
                    key: text,
                    path: builder.detach(),
                    color: Some(tokens[token_index].1),
                },
            });
            x += width;
            visual_column += if ch == '\t' {
                tab - visual_column % tab
            } else {
                1
            };
        }
        max_width = max_width.max(x);
        byte_offset += raw_length + 1;
    }
    let size = Vector2::new(max_width, height + spacing * count.saturating_sub(1) as f32);
    let offset = (-size.x * 0.5, -size.y * 0.5);
    for cell in &mut cells {
        cell.rect.offset(offset);
        cell.part.path = cell.part.path.with_offset(offset);
    }
    Layout { cells, size }
}

fn emphasized_parts(layout: &Layout, shape: &CodeShape) -> Vec<MorphPart> {
    layout
        .cells
        .iter()
        .filter(|cell| !cell.part.path.is_empty())
        .map(|cell| {
            let mut rgba = cell.part.color.unwrap().rgba();
            rgba[3] *= shape.dim_opacity
                + (1.0 - shape.dim_opacity) * shape.selection.coverage(cell.position, true);
            MorphPart {
                key: cell.part.key.clone(),
                path: cell.part.path.clone(),
                color: Some(rgba.into()),
            }
        })
        .collect()
}

fn key_edit_parts(
    layout: &Layout,
    parts: &mut [MorphPart],
    intervals: &[(std::ops::Range<usize>, std::ops::Range<usize>)],
    incoming: bool,
) {
    for (cell, part) in layout
        .cells
        .iter()
        .filter(|cell| !cell.part.path.is_empty())
        .zip(parts)
    {
        let touched = intervals.iter().any(|(old, new)| {
            let range = if incoming { new } else { old };
            if range.is_empty() {
                cell.token.start < range.start && range.start < cell.token.end
            } else {
                cell.token.start < range.end && range.start < cell.token.end
            }
        });
        if touched {
            part.key = format!("{incoming}:{}", cell.byte);
        } else {
            let shift: isize = if incoming {
                intervals
                    .iter()
                    .filter(|(_, new)| new.end <= cell.byte)
                    .map(|(old, new)| old.len() as isize - new.len() as isize)
                    .sum()
            } else {
                0
            };
            part.key = format!("Keep:{}", cell.byte as isize + shift);
        }
    }
}

fn draw_highlights(layout: &Layout, shape: &CodeShape, opacity: f32, canvas: &skia_safe::Canvas) {
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    for cell in &layout.cells {
        let mut color = shape.highlight_color.rgba();
        color[3] *= opacity * shape.highlights.coverage(cell.position, false);
        if color[3] > 0.0 {
            paint.set_color4f(
                skia_safe::Color4f::new(color[0], color[1], color[2], color[3]),
                None,
            );
            canvas.draw_rect(cell.rect, &paint);
        }
    }
}

fn draw_code(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas, opacity: f32) {
    let shape = world.get::<&CodeShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();
    let fade = world.get::<&TextFade>(entity).unwrap();
    let from = layout(&shape, &shape.code, style.fill);
    let mut part_style = (*style).clone();
    let mut parts = emphasized_parts(&from, &shape);
    if let Some(text) = fade.incoming_text(&shape.code) {
        let to = layout(&shape, text, style.fill);
        let mut incoming = emphasized_parts(&to, &shape);
        let edit = world.get::<&CodeEditState>(entity).unwrap();
        let mut token_edit = false;
        if edit.active {
            let mut shift = 0_isize;
            let intervals = edit
                .ranges
                .0
                .iter()
                .flat_map(|(ranges, _)| ranges)
                .map(|range| {
                    let old = code_interval(&shape.code, *range)?;
                    let start = (old.start as isize + shift) as usize;
                    let new = start..start + edit.replacement.len();
                    shift += new.len() as isize - old.len() as isize;
                    Ok((old, new))
                })
                .collect::<Result<Vec<_>, String>>();
            if let Ok(intervals) = intervals {
                key_edit_parts(&from, &mut parts, &intervals, false);
                key_edit_parts(&to, &mut incoming, &intervals, true);
                token_edit = true;
            }
        }
        draw_highlights(&from, &shape, opacity * (1.0 - fade.progress), canvas);
        draw_highlights(&to, &shape, opacity * fade.progress, canvas);
        if fade.morph || token_edit {
            for part in morph_parts(&parts, &incoming, fade.progress) {
                part_style.fill = part.color(style.fill);
                part_style.stroke.a = style.stroke.a * part_style.fill.a;
                draw_complete_styled_path(
                    &part.path(),
                    &part_style,
                    transform.scale,
                    opacity * part.opacity,
                    canvas,
                );
            }
            return;
        }
        for (parts, alpha) in [(&parts, 1.0 - fade.progress), (&incoming, fade.progress)] {
            for part in parts {
                part_style.fill = part.color.unwrap();
                part_style.stroke.a = style.stroke.a * part_style.fill.a;
                draw_complete_styled_path(
                    &part.path,
                    &part_style,
                    transform.scale,
                    opacity * alpha,
                    canvas,
                );
            }
        }
    } else {
        draw_highlights(&from, &shape, opacity, canvas);
        for part in parts {
            part_style.fill = part.color.unwrap();
            part_style.stroke.a = style.stroke.a * part_style.fill.a;
            draw_complete_styled_path(&part.path, &part_style, transform.scale, opacity, canvas);
        }
    }
}

fn code_box(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    let shape = world.get::<&CodeShape>(entity).unwrap();
    let fade = world.get::<&TextFade>(entity).unwrap();
    let mut size = layout(&shape, &shape.code, Color::default()).size;
    if let Some(text) = fade.incoming_text(&shape.code) {
        size = size.max(layout(&shape, text, Color::default()).size);
    }
    size
}

impl Default for Code {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            filter: Default::default(),
            fade: Default::default(),
            edit: Default::default(),
            draw: Draw2D {
                on_draw: draw_code,
                box_size: code_box,
                visual_bounds: |world, entity| {
                    let size = code_box(world, entity);
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    crate::core::components::styled_bounds(
                        skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y),
                        &style,
                        transform.scale,
                    )
                },
                ..Default::default()
            },
        }
    }
}

fn syntax_tokens(
    source: &str,
    shape: &CodeShape,
    fill: Color,
) -> Vec<(std::ops::Range<usize>, Color)> {
    code_syntax::tokens(source, shape.language)
        .iter()
        .map(|token| {
            let color = match token.kind {
                TokenKind::Plain => fill,
                TokenKind::Keyword => shape.keyword_color,
                TokenKind::String => shape.string_color,
                TokenKind::Number => shape.number_color,
                TokenKind::Comment => shape.comment_color,
                TokenKind::Operator => shape.operator_color,
            };
            (token.range.clone(), color)
        })
        .collect()
}

#[cfg(test)]
fn syntax_colors(source: &str, shape: &CodeShape, fill: Color) -> Vec<Color> {
    let mut colors = vec![fill; source.len()];
    for (range, color) in syntax_tokens(source, shape, fill) {
        colors[range].fill(color);
    }
    colors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene,
        objects::{Object2DHandler, ObjectHandler},
    };

    #[test]
    fn code_draws_syntax_and_highlights_inside_bounds_without_changing_state() {
        let mut scene = Scene::new();
        let code = code()
            .code("local x = 42\n-- Note.")
            .keyword_color(Color::RED)
            .highlight_color(Color::BLUE)
            .highlights(CodeRanges::new(vec![[[1, 1], [1, 6]]]))
            .selection(CodeRanges::new(vec![[[1, 1], [1, 6]]]))
            .build(&mut scene);
        let mut surface = skia_safe::surfaces::raster_n32_premul((640, 320)).unwrap();
        surface.canvas().translate((320.0, 160.0));
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        scene.draw(surface.canvas());
        assert_eq!(surface.peek_pixels().unwrap().get_color((320, 160)).a(), 0);
        scene.world_2d().add(&code);
        let before = code.get(CodeShape::code_property());
        let selection = code.get(CodeShape::selection_property());
        let bounds = code.box_size();
        scene.draw(surface.canvas());
        let pixels = surface.peek_pixels().unwrap();
        let mut red = false;
        let mut blue = false;
        let mut dimmed = false;
        for y in 0..320 {
            for x in 0..640 {
                let color = pixels.get_color((x, y));
                if color.a() == 0 {
                    continue;
                }
                red |= color.r() > 200 && color.b() < 20;
                blue |= color.b() > 200 && color.r() < 20;
                dimmed |= color.a() > 30 && color.a() <= 64;
                assert!((x as f32 - 320.0).abs() <= bounds.x * 0.5 + 1.0);
                assert!((y as f32 - 160.0).abs() <= bounds.y * 0.5 + 1.0);
            }
        }
        assert!(red && blue && dimmed);
        assert_eq!(code.get(CodeShape::code_property()), before);
        assert_eq!(code.get(CodeShape::selection_property()), selection);
    }

    #[test]
    fn lua_coloring_handles_multiline_literals_comments_and_numeric_operators() {
        let shape = CodeShape::default();
        let source = "local é = 0x1.fp+2 .. 2e-3 --[=[ Comment.\n]=]\n[==[String.]==] 'é\\\"'";
        let colors = syntax_colors(source, &shape, Color::WHITE);
        for (text, expected) in [
            ("local", shape.keyword_color),
            ("0x1.fp+2", shape.number_color),
            ("..", shape.operator_color),
            ("2e-3", shape.number_color),
            ("Comment.", shape.comment_color),
            ("String.", shape.string_color),
            ("é", Color::WHITE),
        ] {
            let index = source.find(text).unwrap();
            assert!(
                colors[index..index + text.len()]
                    .iter()
                    .all(|c| *c == expected),
                "{text}"
            );
        }
        let mut plain = shape.clone();
        plain.language = CodeLanguage::Plain;
        assert!(
            syntax_colors(source, &plain, Color::WHITE)
                .iter()
                .all(|c| *c == Color::WHITE)
        );
    }

    #[test]
    fn defaults_to_plain_and_hack_nerd_font() {
        let shape = CodeShape::default();
        assert_eq!(shape.language, CodeLanguage::Plain);
        assert_eq!(shape.font, Font::new(fonts::HACK_MONO));
    }

    #[test]
    fn token_edits_fade_whole_words_and_new_lines_without_matching_repeated_letters() {
        let shape = CodeShape::default();
        for (source, target, old, new, changed_line, changed_columns) in [
            (
                "local value = 1",
                "local values = 1",
                6..11,
                6..12,
                1,
                7..13,
            ),
            (
                "local value = 1",
                "local value = 1\nlocal value = 1",
                15..15,
                15..31,
                2,
                1..16,
            ),
            ("-- Note.", "-- Note.\n-- Note.", 8..8, 8..17, 2, 1..9),
        ] {
            let from = layout(&shape, source, Color::WHITE);
            let to = layout(&shape, target, Color::WHITE);
            let mut outgoing = emphasized_parts(&from, &shape);
            let mut incoming = emphasized_parts(&to, &shape);
            let edits = [(old, new)];
            key_edit_parts(&from, &mut outgoing, &edits, false);
            key_edit_parts(&to, &mut incoming, &edits, true);
            for (cell, part) in to
                .cells
                .iter()
                .filter(|cell| !cell.part.path.is_empty())
                .zip(&incoming)
            {
                if cell.position[0] == changed_line && changed_columns.contains(&cell.position[1]) {
                    assert!(part.key.starts_with("true:"), "{}", part.key);
                    assert!(!outgoing.iter().any(|old| old.key == part.key));
                }
            }
            let draws = morph_parts(&outgoing, &incoming, 0.5);
            assert!(
                draws
                    .iter()
                    .filter(|part| part.incoming)
                    .all(|part| part.opacity == 0.5)
            );
            assert!(draws.iter().any(|part| part.opacity == 1.0));
        }
    }

    #[test]
    fn tabs_use_visual_stops_and_preserve_unicode_source_positions() {
        let shape = CodeShape::default();
        let block = layout(&shape, "é\tx\r\nz", Color::WHITE);
        assert_eq!(
            block.cells.iter().map(|c| c.position).collect::<Vec<_>>(),
            vec![[1, 1], [1, 2], [1, 3], [2, 1]]
        );
        let space = shape.font.skia_font(shape.size).measure_str(" ", None).0;
        assert!((block.cells[2].rect.left - block.cells[0].rect.left - space * 4.0).abs() < 0.01);
    }
}
