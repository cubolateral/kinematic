use dear_imgui_cte::{PaletteColor, Position, Selection, TextEditor};
use dear_imgui_rs::{FocusedFlags, Key, MouseButton, StyleColor, Ui};
use std::{collections::HashMap, ops::Range};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Code,
    Heading,
    Paragraph,
}

struct Block {
    range: Range<usize>,
    kind: Kind,
    colors: Vec<(Range<usize>, PaletteColor)>,
}

#[derive(Default)]
pub(super) struct Documentation {
    pub text: String,
    blocks: Vec<Block>,
    selection: Option<(usize, usize)>,
    dragging: bool,
    reset_scroll: bool,
    pub height: f32,
    pub scroll_y: f32,
}

struct Glyph {
    range: Range<usize>,
    position: [f32; 2],
    width: f32,
    height: f32,
    color: u32,
}

impl Documentation {
    pub fn parse(markdown: &str) -> Self {
        let mut result = Self {
            reset_scroll: true,
            ..Self::default()
        };
        let mut code = false;
        let mut pending = String::new();
        let mut parameters = false;
        let mut returns = false;
        for original in markdown.lines() {
            let line = original.trim();
            if line.starts_with("```") {
                result.append(&pending, if code { Kind::Code } else { Kind::Paragraph });
                pending.clear();
                code = !code;
            } else if code {
                if !pending.is_empty() {
                    pending.push('\n');
                }
                pending.push_str(original.trim_end());
            } else if line.is_empty() || line == "---" {
                result.append(&pending, Kind::Paragraph);
                pending.clear();
            } else if let Some(parameter) = line
                .strip_prefix("@*param*")
                .or_else(|| line.strip_prefix("@param"))
            {
                result.append(&pending, Kind::Paragraph);
                pending.clear();
                if !parameters {
                    result.append("Parameters", Kind::Heading);
                    parameters = true;
                }
                result.append(parameter.trim(), Kind::Paragraph);
            } else if let Some(returned) = line
                .strip_prefix("@*return*")
                .or_else(|| line.strip_prefix("@return"))
            {
                result.append(&pending, Kind::Paragraph);
                pending.clear();
                if !returns {
                    result.append("Returns", Kind::Heading);
                    returns = true;
                }
                result.append(returned.trim(), Kind::Paragraph);
            } else if line.starts_with('#') {
                result.append(&pending, Kind::Paragraph);
                pending.clear();
                result.append(line.trim_start_matches('#').trim(), Kind::Heading);
            } else if line.starts_with("- ") || line.starts_with("* ") {
                result.append(&pending, Kind::Paragraph);
                pending.clear();
                result.append(&format!("• {}", &line[2..]), Kind::Paragraph);
            } else {
                if !pending.is_empty() {
                    pending.push(' ');
                }
                pending.push_str(line);
            }
        }
        result.append(&pending, if code { Kind::Code } else { Kind::Paragraph });
        result
    }

    fn append(&mut self, source: &str, kind: Kind) {
        if source.trim().is_empty() {
            return;
        }
        let mut source = source.trim().to_owned();
        if kind == Kind::Code {
            for prefix in ["(method) ", "(function) ", "(field) "] {
                if let Some(rest) = source.strip_prefix(prefix) {
                    source = rest.to_owned();
                    break;
                }
            }
        }
        if !self.text.is_empty() {
            self.text.push_str("\n\n");
        }
        let start = self.text.len();
        let mut colors = Vec::new();
        if kind == Kind::Code {
            self.text.push_str(&source);
            colors = code_colors(&source, start);
        } else {
            let source = source.replace("&#x20;", " ").replace("&nbsp;", " ");
            let mut remaining = source.as_str();
            while !remaining.is_empty() {
                if let Some(rest) = remaining.strip_prefix('`') {
                    if let Some(end) = rest.find('`') {
                        let begin = self.text.len();
                        self.text.push_str(&rest[..end]);
                        colors.push((begin..self.text.len(), PaletteColor::KnownIdentifier));
                        remaining = &rest[end + 1..];
                        continue;
                    }
                }
                let marker = if remaining.starts_with("**") {
                    "**"
                } else {
                    "*"
                };
                if let Some(rest) = remaining.strip_prefix(marker) {
                    if rest.chars().next().is_some_and(|ch| !ch.is_whitespace()) {
                        if let Some(end) = rest.find(marker) {
                            let begin = self.text.len();
                            self.text.push_str(&rest[..end].replace('`', ""));
                            colors.push((begin..self.text.len(), PaletteColor::Declaration));
                            remaining = &rest[end + marker.len()..];
                            continue;
                        }
                    }
                }
                if let Some(rest) = remaining.strip_prefix('[') {
                    if let Some(end) = rest.find("](") {
                        if let Some(close) = rest[end + 2..].find(')') {
                            let begin = self.text.len();
                            self.text.push_str(&rest[..end]);
                            colors.push((begin..self.text.len(), PaletteColor::KnownIdentifier));
                            remaining = &rest[end + 2 + close + 1..];
                            continue;
                        }
                    }
                }
                let character = remaining.chars().next().unwrap();
                self.text.push(character);
                remaining = &remaining[character.len_utf8()..];
            }
        }
        self.blocks.push(Block {
            range: start..self.text.len(),
            kind,
            colors,
        });
    }

    pub fn draw(&mut self, ui: &Ui, buffer: &mut TextEditor, font_size: f32) -> Result<(), String> {
        let palette = buffer.palette();
        let mut error = None;
        ui.child_window("Documentation content").build(ui, || {
            if self.reset_scroll {
                ui.set_scroll_y(0.0);
                self.reset_scroll = false;
            }
            self.scroll_y = ui.scroll_y();
            let origin = ui.cursor_screen_pos();
            let width = ui.content_region_avail()[0].max(1.0);
            let mut glyphs = Vec::new();
            let mut backgrounds = Vec::new();
            let mut y = origin[1];
            for (block_index, block) in self.blocks.iter().enumerate() {
                let size = if block.kind == Kind::Heading {
                    font_size * 1.05
                } else {
                    font_size
                };
                let font = ui.push_font_with_size(None, size);
                let line_height = ui.text_line_height() * 1.35;
                let inset = if block.kind == Kind::Code { 12.0 } else { 0.0 };
                let top = y;
                if block.kind == Kind::Code {
                    y += 12.0;
                }
                let mut x = origin[0] + inset;
                let limit = origin[0] + width - inset;
                let mut color_index = 0;
                let mut widths = HashMap::new();
                for (offset, character) in self.text[block.range.clone()].char_indices() {
                    let index = block.range.start + offset;
                    if character == '\n' {
                        x = origin[0] + inset;
                        y += line_height;
                        continue;
                    }
                    let glyph_width = *widths
                        .entry(character)
                        .or_insert_with(|| ui.calc_text_size(character.to_string())[0]);
                    // Wrap at word boundaries; very long identifiers still wrap by character.
                    if x > origin[0] + inset
                        && !character.is_whitespace()
                        && (offset == 0
                            || self.text[..index]
                                .chars()
                                .next_back()
                                .is_some_and(char::is_whitespace))
                    {
                        let word = self.text[index..block.range.end]
                            .split_whitespace()
                            .next()
                            .unwrap_or("");
                        if x + ui.calc_text_size(word)[0] > limit {
                            x = origin[0] + inset;
                            y += line_height;
                        }
                    }
                    if x + glyph_width > limit && x > origin[0] + inset {
                        x = origin[0] + inset;
                        y += line_height;
                    }
                    while block
                        .colors
                        .get(color_index)
                        .is_some_and(|(range, _)| range.end <= index)
                    {
                        color_index += 1;
                    }
                    let color = block
                        .colors
                        .get(color_index)
                        .filter(|(range, _)| range.contains(&index))
                        .map(|(_, color)| *color)
                        .unwrap_or(if block.kind == Kind::Heading {
                            PaletteColor::Keyword
                        } else {
                            PaletteColor::Text
                        });
                    glyphs.push(Glyph {
                        range: index..index + character.len_utf8(),
                        position: [x, y],
                        width: glyph_width,
                        height: line_height,
                        color: palette.get(color),
                    });
                    x += glyph_width;
                }
                y += ui.text_line_height();
                if block.kind == Kind::Code {
                    y += 12.0;
                    backgrounds.push(([origin[0], top], [origin[0] + width, y]));
                }
                if block_index + 1 < self.blocks.len() {
                    y += if block.kind == Kind::Heading {
                        5.0
                    } else {
                        12.0
                    };
                }
                drop(font);
            }
            self.height = y - origin[1];
            ui.invisible_button("Documentation text", [width, self.height.max(1.0)]);
            let hovered = ui.is_item_hovered();
            if hovered {
                ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::TextInput));
            }
            let mouse = ui.io().mouse_pos();
            if hovered && ui.is_mouse_clicked(MouseButton::Left) {
                let offset = hit(&glyphs, mouse, self.text.len());
                self.selection = Some((offset, offset));
                self.dragging = true;
            }
            if self.dragging {
                if let Some((start, _)) = self.selection {
                    self.selection = Some((start, hit(&glyphs, mouse, self.text.len())));
                }
                if !ui.is_mouse_down(MouseButton::Left) {
                    self.dragging = false;
                }
            }
            let focused = ui.is_window_focused_with_flags(FocusedFlags::ROOT_AND_CHILD_WINDOWS);
            if focused && ui.io().key_ctrl() && ui.is_key_pressed(Key::A) {
                self.selection = Some((0, self.text.len()));
            }
            if let Some((a, b)) = self.selection {
                if let Err(failure) = buffer.select_region(Selection::new(
                    position(&self.text, a.min(b)),
                    position(&self.text, a.max(b)),
                )) {
                    error = Some(failure.to_string());
                }
                if focused && ui.io().key_ctrl() && ui.is_key_pressed(Key::C) {
                    buffer.copy();
                }
            }
            let draw = ui.get_window_draw_list();
            let surface = ui.clone_style().color(StyleColor::Header);
            for (min, max) in backgrounds {
                draw.add_rect(min, max, surface)
                    .filled(true)
                    .rounding(5.0)
                    .build();
            }
            let selected = self.selection.map(|(a, b)| a.min(b)..a.max(b));
            for block in &self.blocks {
                let font = ui.push_font_with_size(
                    None,
                    if block.kind == Kind::Heading {
                        font_size * 1.05
                    } else {
                        font_size
                    },
                );
                for glyph in glyphs
                    .iter()
                    .filter(|glyph| block.range.contains(&glyph.range.start))
                {
                    if glyph.position[1] + glyph.height < draw.clip_rect_min()[1]
                        || glyph.position[1] > draw.clip_rect_max()[1]
                    {
                        continue;
                    }
                    if selected
                        .as_ref()
                        .is_some_and(|range| range.contains(&glyph.range.start))
                    {
                        draw.add_rect(
                            glyph.position,
                            [
                                glyph.position[0] + glyph.width,
                                glyph.position[1] + glyph.height,
                            ],
                            palette.get(PaletteColor::Selection),
                        )
                        .filled(true)
                        .build();
                    }
                    draw.add_text(glyph.position, glyph.color, &self.text[glyph.range.clone()]);
                }
                drop(font);
            }
        });
        error.map_or(Ok(()), Err)
    }
}

fn position(text: &str, offset: usize) -> Position {
    let prefix = &text[..offset];
    Position::new(
        prefix.bytes().filter(|byte| *byte == b'\n').count(),
        prefix.rsplit('\n').next().unwrap_or("").chars().count(),
    )
}

fn hit(glyphs: &[Glyph], mouse: [f32; 2], end: usize) -> usize {
    let Some(first) = glyphs.first() else {
        return 0;
    };
    if mouse[1] < first.position[1] {
        return 0;
    }
    let mut last = 0;
    for glyph in glyphs {
        if mouse[1] < glyph.position[1] {
            return last;
        }
        last = glyph.range.end;
        if mouse[1] < glyph.position[1] + glyph.height
            && mouse[0] < glyph.position[0] + glyph.width * 0.5
        {
            return glyph.range.start;
        }
    }
    end
}

fn code_colors(text: &str, base: usize) -> Vec<(Range<usize>, PaletteColor)> {
    let mut colors = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        let character = rest.chars().next().unwrap();
        let (length, color) = if rest.starts_with("--") {
            (rest.find('\n').unwrap_or(rest.len()), PaletteColor::Comment)
        } else if character == '\'' || character == '"' {
            (
                rest[1..].find(character).map_or(rest.len(), |end| end + 2),
                PaletteColor::String,
            )
        } else if character.is_alphanumeric() || character == '_' {
            let length = rest
                .char_indices()
                .find(|(_, ch)| !ch.is_alphanumeric() && *ch != '_')
                .map_or(rest.len(), |(end, _)| end);
            let word = &rest[..length];
            let color = if character.is_numeric() {
                PaletteColor::Number
            } else if matches!(
                word,
                "function"
                    | "end"
                    | "local"
                    | "return"
                    | "nil"
                    | "true"
                    | "false"
                    | "number"
                    | "string"
                    | "boolean"
                    | "integer"
                    | "table"
                    | "fun"
            ) {
                PaletteColor::Keyword
            } else if character.is_uppercase() {
                PaletteColor::Declaration
            } else if rest[length..].starts_with('(') {
                PaletteColor::KnownIdentifier
            } else {
                PaletteColor::Identifier
            };
            (length, color)
        } else {
            (character.len_utf8(), PaletteColor::Punctuation)
        };
        colors.push((base + offset..base + offset + length, color));
        offset += length;
    }
    colors
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn luals_hover_has_readable_sections_and_preserves_unicode_selection() {
        let doc = Documentation::parse(
            "```lua\n(method) Kinematic.Scene:text_2d(options: Kinematic.Text2DOptions)\n  -> object: Kinematic.Text2D\n```\n---\nBuilt-in **text** scene object.\n\n@*param* `options` — Initial properties.\n\n@*return* `object` — Handle exposing properties.\n\n## Example\n```lua\nlocal label = s:text_2d {text='Olá'}\n```\n",
        );
        assert!(!doc.text.contains("(method)"));
        assert!(
            doc.text
                .contains("Parameters\n\noptions — Initial properties.")
        );
        assert!(
            doc.text
                .contains("Returns\n\nobject — Handle exposing properties.")
        );
        assert!(!doc.text.contains('@') && !doc.text.contains('*'));
        assert!(doc.text.contains("Built-in text scene object."));
        let formula = "Falloff: linear * distance + quadratic * distance squared.";
        assert_eq!(Documentation::parse(formula).text, formula);
        let offset = doc.text.find("Olá").unwrap() + "Olá".len();
        assert_eq!(
            position(&doc.text, offset).column,
            "local label = s:text_2d {text='Olá".chars().count()
        );
        assert!(
            doc.blocks
                .iter()
                .any(|block| block.kind == Kind::Code && !block.colors.is_empty())
        );
    }
}
