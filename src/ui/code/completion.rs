use dear_imgui_cte::{Position, Selection, TextEditor};
use serde_json::Value;

#[derive(Default)]
pub(super) struct Completion {
    pub items: Vec<Value>,
    pub selected: usize,
    pub position: Option<Position>,
    pub resolve: bool,
    pub focused: bool,
    scroll_selected: bool,
}

impl Completion {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn receive(&mut self, value: &Value, position: Position) {
        self.items = value
            .as_array()
            .or_else(|| value.get("items").and_then(Value::as_array))
            .into_iter()
            .flatten()
            .filter(|item| item["label"].is_string() && item["insertTextFormat"] != 2)
            .take(500)
            .cloned()
            .collect();
        self.selected = 0;
        self.position = Some(position);
        self.resolve = !self.items.is_empty();
        self.scroll_selected = true;
    }

    pub fn resolve(&mut self, item: Value) {
        if let Some(selected) = self.items.get_mut(self.selected) {
            if selected["label"] == item["label"] {
                if let (Some(selected), Some(item)) = (selected.as_object_mut(), item.as_object()) {
                    selected.extend(item.iter().map(|(key, value)| (key.clone(), value.clone())));
                }
            }
        }
    }

    pub fn keyboard(&mut self, ui: &dear_imgui_rs::Ui) -> Option<bool> {
        use dear_imgui_rs::Key;
        if self.items.is_empty() || ui.io().key_ctrl() || ui.io().key_alt() || ui.io().key_super() {
            return None;
        }
        if ui.is_key_pressed(Key::UpArrow) {
            self.selected = (self.selected + self.items.len() - 1) % self.items.len();
        } else if ui.is_key_pressed(Key::DownArrow) {
            self.selected = (self.selected + 1) % self.items.len();
        } else if ui.is_key_pressed(Key::Enter)
            || ui.is_key_pressed(Key::KeypadEnter)
            || ui.is_key_pressed(Key::Tab)
        {
            return Some(true);
        } else if ui.is_key_pressed(Key::Escape) {
            self.clear();
            return Some(false);
        } else {
            return None;
        }
        self.resolve = true;
        self.scroll_selected = true;
        Some(false)
    }

    pub fn accept(
        &mut self,
        editor: &mut TextEditor,
        text: &str,
        utf8: bool,
    ) -> Result<(), String> {
        let cursor = editor.main_cursor_position();
        if self.position != Some(cursor) {
            self.clear();
            return Ok(());
        }
        let Some(item) = self.items.get(self.selected) else {
            return Ok(());
        };
        let (range, replacement) = edit(item, text, cursor, utf8)?;
        editor.select_region(range).map_err(|e| e.to_string())?;
        editor
            .replace_current_selection(&replacement)
            .map_err(|e| e.to_string())?;
        self.clear();
        Ok(())
    }

    pub fn contains_mouse(&self, ui: &dear_imgui_rs::Ui, position: [f32; 2]) -> bool {
        let (position, size) = bounds(ui, position);
        let mouse = ui.io().mouse_pos();
        (0..2)
            .all(|axis| mouse[axis] >= position[axis] && mouse[axis] <= position[axis] + size[axis])
    }

    pub fn draw(
        &mut self,
        ui: &dear_imgui_rs::Ui,
        position: [f32; 2],
        content: &mut super::popup::Content,
        font_size: f32,
    ) -> Result<bool, String> {
        if self.items.is_empty() {
            return Ok(false);
        }
        let (position, size) = bounds(ui, position);
        let mut accept = false;
        let mut error = None;
        super::popup::window(
            ui,
            "Suggestions##Code",
            position,
            Some(size),
            dear_imgui_rs::WindowFlags::empty(),
            || {
                ui.child_window("Candidates")
                    .size([0.0, 190.0_f32.min(size[1] * 0.6)])
                    .build(ui, || {
                        for (index, item) in self.items.iter().enumerate() {
                            let label = item["label"].as_str().unwrap_or_default();
                            let kind = kind(item);
                            let row = format!("{label}##{index}");
                            let selected = index == self.selected;
                            let _white = ui.push_style_color(
                                dear_imgui_rs::StyleColor::Text,
                                [1.0, 1.0, 1.0, 1.0],
                            );
                            ui.selectable_config(&row).selected(selected).build();
                            // The source editor keeps keyboard focus behind this overlay.
                            let pressed = ui.is_item_hovered_with_flags(
                                dear_imgui_rs::ItemHoveredFlags::ALLOW_WHEN_OVERLAPPED_BY_WINDOW,
                            ) && ui
                                .is_mouse_clicked(dear_imgui_rs::MouseButton::Left);
                            if pressed {
                                accept = selected;
                                self.selected = index;
                                self.resolve = !selected;
                            }
                            if !kind.is_empty() {
                                let origin = ui.item_rect_min();
                                ui.get_window_draw_list().add_text(
                                    [origin[0] + ui.calc_text_size(label)[0] + 28.0, origin[1]],
                                    content.packed_color(color(item)),
                                    kind,
                                );
                            }
                            if index == self.selected && self.scroll_selected {
                                ui.set_scroll_here_y(0.5);
                            }
                        }
                    });
                ui.separator();
                if let Some(item) = self.items.get(self.selected) {
                    if let Err(failure) = content.set(&documentation(item)) {
                        error = Some(failure);
                    }
                }
                ui.child_window("Details").build(ui, || {
                    if error.is_none() {
                        if let Err(failure) = content.draw(ui, font_size) {
                            error = Some(failure);
                        }
                    }
                });
                self.focused = ui.is_window_focused_with_flags(
                    dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS,
                );
            },
        );
        self.scroll_selected = false;
        error.map_or(Ok(accept), Err)
    }
}

fn bounds(ui: &dear_imgui_rs::Ui, position: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let viewport = ui.window_viewport();
    let work = viewport.work_pos();
    let available = viewport.work_size();
    let size = [560.0_f32.min(available[0]), 340.0_f32.min(available[1])];
    let position = std::array::from_fn(|axis| {
        position[axis].clamp(
            work[axis],
            work[axis] + (available[axis] - size[axis]).max(0.0),
        )
    });
    (position, size)
}

fn documentation(item: &Value) -> String {
    let markdown = item
        .get("documentation")
        .and_then(|doc| doc.as_str().or_else(|| doc["value"].as_str()))
        .unwrap_or_default();
    let detail = detail(item);
    if detail.eq_ignore_ascii_case(kind(item)) {
        markdown.to_owned()
    } else {
        format!("```lua\n{detail}\n```\n\n{markdown}")
    }
}

fn color(item: &Value) -> dear_imgui_cte::PaletteColor {
    use dear_imgui_cte::PaletteColor;
    match item["kind"].as_u64() {
        Some(2 | 3) => PaletteColor::Declaration,
        Some(5 | 10) => PaletteColor::KnownIdentifier,
        Some(7) => PaletteColor::Keyword,
        Some(14) => PaletteColor::Keyword,
        Some(21) => PaletteColor::Number,
        _ => PaletteColor::Text,
    }
}

fn kind(item: &Value) -> &'static str {
    match item["kind"].as_u64() {
        Some(2) => "Method",
        Some(3) => "Function",
        Some(5) => "Field",
        Some(6) => "Variable",
        Some(7) => "Class",
        Some(10) => "Property",
        Some(14) => "Keyword",
        Some(21) => "Constant",
        _ => "",
    }
}

fn detail(item: &Value) -> &str {
    item["detail"]
        .as_str()
        .or_else(|| item.pointer("/labelDetails/detail").and_then(Value::as_str))
        .or_else(|| item["label"].as_str())
        .unwrap_or_default()
}

fn edit(
    item: &Value,
    text: &str,
    cursor: Position,
    utf8: bool,
) -> Result<(Selection, String), String> {
    let replacement = item
        .pointer("/textEdit/newText")
        .or_else(|| item.get("insertText"))
        .or_else(|| item.get("label"))
        .and_then(Value::as_str)
        .ok_or("Completion has no insertion text.")?;
    let range = item
        .pointer("/textEdit/range")
        .or_else(|| item.pointer("/textEdit/replace"));
    let range = if let Some(range) = range {
        let position = |name: &str| -> Result<Position, String> {
            let line = range[name]["line"]
                .as_u64()
                .ok_or("Completion has no line.")? as usize;
            let units = range[name]["character"]
                .as_u64()
                .ok_or("Completion has no column.")? as usize;
            let column = crate::editor::lsp::glyph_column(text, line, units, utf8);
            let source = text
                .split('\n')
                .nth(line)
                .ok_or("Completion line is outside the document.")?;
            if crate::editor::lsp::position_units(source, 0, column, utf8) != units {
                return Err(
                    "Completion column splits a character or is outside the document.".into(),
                );
            }
            Ok(Position::new(line, column))
        };
        Selection::new(position("start")?, position("end")?)
    } else {
        let prefix = super::line_prefix(text, cursor);
        let count = prefix
            .chars()
            .rev()
            .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
            .count();
        Selection::new(Position::new(cursor.line, cursor.column - count), cursor)
    };
    if range.start > range.end
        || cursor < range.start
        || cursor > range.end
        || replacement.contains('\0')
    {
        return Err("Completion edit does not contain the cursor or has invalid text.".into());
    }
    Ok((range, replacement.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_keep_metadata_and_accept_both_server_response_shapes() {
        let mut completion = Completion::default();
        let item = serde_json::json!({"label":"length", "kind":2, "insertText":"length", "detail":"length(): number"});
        for value in [
            serde_json::json!([item]),
            serde_json::json!({"items":[item]}),
        ] {
            completion.receive(&value, Position::new(0, 3));
            assert_eq!(completion.items, vec![item.clone()]);
            assert_eq!(detail(&completion.items[0]), "length(): number");
            completion.resolve(
                serde_json::json!({"label":"length", "documentation":"Returns the magnitude."}),
            );
            assert_eq!(completion.items[0]["insertText"], "length");
        }
        completion.receive(&Value::Null, Position::new(0, 3));
        assert!(completion.items.is_empty());
    }

    #[test]
    fn insertion_uses_server_ranges_and_preserves_receivers_and_unicode() {
        let item = serde_json::json!({"label":"normalized()", "textEdit":{"newText":"normalized", "range":{"start":{"line":0,"character":5},"end":{"line":0,"character":8}}}});
        let (range, text) = edit(&item, "ação:nor", Position::new(0, 8), false).unwrap();
        assert_eq!(
            range,
            Selection::new(Position::new(0, 5), Position::new(0, 8))
        );
        assert_eq!(text, "normalized");
    }

    #[test]
    fn generic_detail_is_hidden() {
        let item = serde_json::json!({"label":"OnSetText", "kind":3, "detail":"function", "documentation":"```lua\nfunction OnSetText(uri, text)\n```"});
        assert_eq!(
            documentation(&item),
            item["documentation"].as_str().unwrap()
        );
    }
}
