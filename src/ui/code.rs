use super::browser;
use crate::editor::{Editor, documents::Documents};
use crate::ui::inputs::{self, Action as InputAction};
use dear_imgui_cte::{
    AutocompleteConfig, CteUiExt, Language, Palette, PaletteColor, Position, TextEditor,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, rc::Rc};

mod completion;
mod diagnostics;
mod documentation;
mod files;
mod popup;
mod shortcuts;

pub(super) const WINDOW_NAME: &str = "Code";
pub(super) const FILES_WINDOW: &str = "Files";
pub(super) const CONSOLE_WINDOW: &str = "Console";

#[derive(Clone)]
pub(crate) enum ProjectAction {
    Open(PathBuf),
    Create(PathBuf, String, bool),
    Close,
    Exit,
}
#[derive(Clone)]
enum Pending {
    Document(PathBuf),
    Project(ProjectAction),
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Settings {
    pub font_size: f32,
    pub line_height: f32,
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub rulers: String,
    pub line_numbers: bool,
    pub autocomplete: bool,
    pub show_spaces: bool,
    pub hover_delay_ms: usize,
    pub format_on_save: bool,
    pub trim_trailing_whitespace: bool,
    pub insert_final_newline: bool,
    pub auto_save: bool,
    pub auto_save_delay_ms: usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_height: 1.5,
            tab_size: 4,
            insert_spaces: true,
            word_wrap: false,
            rulers: "80".into(),
            line_numbers: true,
            autocomplete: true,
            show_spaces: false,
            hover_delay_ms: 500,
            format_on_save: true,
            trim_trailing_whitespace: true,
            insert_final_newline: true,
            auto_save: true,
            auto_save_delay_ms: 1000,
        }
    }
}
#[derive(Default)]
struct Assistance {
    completion: Option<Position>,
    completion_position: Option<Position>,
    hover: Option<Position>,
    hover_position: Option<Position>,
    changed: bool,
    available: bool,
    error: Option<String>,
}
struct DocumentationPopup {
    position: [f32; 2],
    size: [f32; 2],
    last_hover: f64,
    focused: bool,
    bounds: [[f32; 2]; 2],
}

impl DocumentationPopup {
    fn contains(&self, mouse: [f32; 2]) -> bool {
        (0..2)
            .all(|axis| mouse[axis] >= self.bounds[0][axis] && mouse[axis] <= self.bounds[1][axis])
    }
}

struct Tab {
    editor: TextEditor,
    chord_started: Option<f64>,
    version: u64,
    completions: completion::Completion,
    settings: Option<Settings>,
    assistance: Rc<RefCell<Assistance>>,
    signature: String,
    signature_position: Option<Position>,
    documentation: TextEditor,
    documentation_view: documentation::Documentation,
    documentation_has_content: bool,
    hover_candidate: Option<(Position, f64)>,
    documentation_popup: Option<DocumentationPopup>,
    popup_content: popup::Content,
}
impl Tab {
    fn configure(&mut self, settings: &Settings) -> dear_imgui_cte::CteResult<()> {
        if self.settings.as_ref() == Some(settings) {
            return Ok(());
        }

        self.completions.clear();

        self.assistance.borrow_mut().completion_position = None;

        self.editor.set_tab_size(settings.tab_size.clamp(1, 8))?;
        self.editor
            .set_line_spacing(if settings.line_height.is_finite() {
                settings.line_height.clamp(1.0, 2.0)
            } else {
                1.5
            })?;

        self.editor
            .set_insert_spaces_on_tabs(settings.insert_spaces);

        self.editor.set_word_wrap_enabled(settings.word_wrap);
        self.editor.set_show_line_numbers(settings.line_numbers);
        self.editor.set_show_minimap(false);
        self.editor.set_show_whitespaces(settings.show_spaces);
        self.editor.set_show_spaces(settings.show_spaces);
        self.editor.set_show_tabs(settings.show_spaces);

        let assistance = self.assistance.clone();

        self.editor.set_autocomplete(
            &AutocompleteConfig::new()
                .shortcut(inputs::map().chord(InputAction::RequestSuggestions))
                .trigger_on_typing(settings.autocomplete && self.assistance.borrow().available)
                .trigger_delay(std::time::Duration::from_millis(100))
                .no_suggestions_label(""),
            move |request| {
                if let Ok(range) = request.range() {
                    let mut assistance = assistance.borrow_mut();

                    if let Err(error) = request
                        .clear_suggestions()
                        .and_then(|_| request.set_promised(false))
                    {
                        assistance.error = Some(error.to_string());
                    }

                    assistance.completion = Some(range.end);
                    assistance.completion_position = Some(range.end);
                }
            },
        )?;

        self.settings = Some(settings.clone());

        Ok(())
    }

    fn cursor_popup_position(&self, origin: [f32; 2], cursor: Position) -> [f32; 2] {
        let visual = self.editor.document_to_visual(cursor).unwrap_or_default();
        let gutter = if self.editor.shows_line_numbers() {
            self.editor.line_number_left_margin()
                + (self.editor.line_count() + 1).ilog10() as usize
                + 1
        } else {
            0
        } + usize::from(self.editor.is_line_folding_enabled())
            + self.editor.text_left_margin();

        [
            origin[0]
                + (visual
                    .column
                    .saturating_sub(self.editor.first_visible_column())
                    + gutter) as f32
                    * self.editor.glyph_width().unwrap_or(8.0),
            origin[1]
                + (visual.row.saturating_sub(self.editor.first_visible_row()) + 1) as f32
                    * self.editor.line_height().unwrap_or(16.0),
        ]
    }

    fn save(&mut self, documents: &mut Documents, path: &std::path::Path) -> Result<(), String> {
        documents.save(path)?;

        let document = &documents.open[path];

        if self.version != document.version {
            let cursor = self.editor.main_cursor_position();
            let last_line = self.editor.line_count().saturating_sub(1);
            let end = Position::new(
                last_line,
                self.editor
                    .line_text(last_line)
                    .map_err(|e| e.to_string())?
                    .chars()
                    .count(),
            );

            self.editor
                .replace_section(
                    dear_imgui_cte::Selection::new(Position::new(0, 0), end),
                    &document.text,
                )
                .map_err(|e| e.to_string())?;

            let line = cursor.line.min(self.editor.line_count().saturating_sub(1));
            let column = cursor.column.min(
                self.editor
                    .line_text(line)
                    .map_err(|e| e.to_string())?
                    .chars()
                    .count(),
            );

            self.editor
                .set_cursor(Position::new(line, column))
                .map_err(|e| e.to_string())?;

            self.version = document.version;
        }

        Ok(())
    }

    fn draw(
        &mut self,
        documents: &mut Documents,
        path: &std::path::Path,
        settings: &Settings,
        font_size: f32,
        bold_font: Option<dear_imgui_rs::FontId>,
        focused: bool,
        ui: &dear_imgui_rs::Ui,
        footer_height: f32,
        output_error: &mut Option<String>,
    ) {
        if focused && inputs::active(ui, InputAction::Find) {
            self.editor.open_find_replace();
        }

        if focused
            && !self
                .documentation_popup
                .as_ref()
                .is_some_and(|popup| popup.focused)
        {
            if let Err(error) = self.shortcuts(ui) {
                *output_error = Some(error.to_string());
            }
        }

        let _font = ui.push_font_with_size(bold_font, font_size);
        let available = ui.content_region_avail();
        let height = (available[1] - footer_height).max(1.0);
        let origin = ui.cursor_screen_pos();
        let signature_active = self.signature_position.is_some();
        let before = self.editor.main_cursor_position();
        let selection = self.editor.main_cursor_selection();

        if focused
            && !ui.is_any_item_active()
            && self.editor.cursor_count() == 1
            && selection.start == selection.end
            && inputs::active(ui, InputAction::IndentBackspace)
        {
            let prefix = line_prefix(&documents.open[path].text, before);

            if let Some(column) = indentation_backspace(&prefix, settings.tab_size) {
                if let Err(error) = self.editor.select_region(dear_imgui_cte::Selection::new(
                    Position::new(before.line, column),
                    before,
                )) {
                    *output_error = Some(error.to_string());
                }
            }
        }

        let completion_popup_position = self
            .completions
            .position
            .filter(|_| !self.completions.items.is_empty())
            .map(|position| self.cursor_popup_position(origin, position));
        let mouse_on_completion = completion_popup_position
            .is_some_and(|position| self.completions.contains_mouse(ui, position));
        let completion_focused = focused || self.completions.focused || mouse_on_completion;

        if !completion_focused
            || self.completions.position != Some(before)
            || self.editor.cursor_count() != 1
            || selection.start != selection.end
        {
            self.completions.clear();
        }

        let completion_key = if completion_focused {
            self.completions.keyboard(ui)
        } else {
            None
        };

        if let Some(accept) = completion_key {
            if accept {
                let utf8 = documents.lsp.as_ref().is_some_and(|lsp| lsp.utf8);

                if let Err(error) =
                    self.completions
                        .accept(&mut self.editor, &documents.open[path].text, utf8)
                {
                    *output_error = Some(error);
                }

                self.assistance.borrow_mut().completion_position = None;
            }
        }

        // Completion keys belong to the suggestion list for this frame.
        let completion_cursor = self.editor.main_cursor_position();

        self.editor.set_read_only(completion_key.is_some());

        let rendered = ui
            .text_editor(&mut self.editor, "Source")
            .size([available[0], height])
            .build();

        self.editor.set_read_only(false);

        if let Some(glyph_width) = self.editor.glyph_width().filter(|_| rendered.is_ok()) {
            let gutter = if self.editor.shows_line_numbers() {
                self.editor.line_number_left_margin()
                    + (self.editor.line_count() + 1).ilog10() as usize
                    + 1
            } else {
                0
            } + usize::from(self.editor.is_line_folding_enabled())
                + self.editor.text_left_margin();
            let min = ui.item_rect_min();
            let max = ui.item_rect_max();
            let padding = ui.clone_style().window_padding();
            let draw = ui.get_window_draw_list();
            let _clip = draw.push_clip_rect(min, max, true);
            let color = ui.get_color_u32(dear_imgui_rs::StyleColor::Header);

            // shortcut: CTE exposes scroll in whole columns, upgrade if it exposes pixel scroll.
            for column in ruler_columns(&settings.rulers) {
                let x = min[0] + padding[0] + (gutter + column) as f32 * glyph_width
                    - self.editor.first_visible_column() as f32 * glyph_width;

                if x >= min[0] && x < max[0] {
                    draw.add_line([x, min[1]], [x, max[1]], color)
                        .thickness(1.0)
                        .build();
                }
            }
        }

        let popup_cursor = if completion_popup_position.is_some() || completion_key.is_some() {
            completion_cursor
        } else {
            self.editor.main_cursor_position()
        };
        let popup_position = completion_popup_position
            .unwrap_or_else(|| self.cursor_popup_position(origin, popup_cursor));

        if (completion_key.is_some() || mouse_on_completion)
            && self.editor.main_cursor_position() != completion_cursor
        {
            let _ = self.editor.set_cursor(completion_cursor);
        }

        match rendered {
            Ok(changed) => {
                let callback_changed = std::mem::take(&mut self.assistance.borrow_mut().changed);
                let changed = changed || callback_changed;

                if changed {
                    match self.editor.text() {
                        Ok(text) => {
                            documents.edit(path, text);

                            self.version = documents.open[path].version;

                            let mut assistance = self.assistance.borrow_mut();

                            self.documentation_popup = None;
                            assistance.hover_position = None;
                            self.hover_candidate = None;
                            self.documentation_has_content = false;
                        }

                        Err(error) => *output_error = Some(error.to_string()),
                    }
                }

                let cursor = self.editor.main_cursor_position();

                if changed || cursor != before {
                    self.completions.clear();

                    let mut assistance = self.assistance.borrow_mut();

                    if assistance.completion_position != Some(cursor)
                        || assistance.completion.is_none()
                    {
                        assistance.completion_position = None;
                    }
                }

                if focused && inputs::active(ui, InputAction::DismissAssistance) {
                    self.completions.clear();

                    self.assistance.borrow_mut().completion_position = None;
                }

                if cursor != before && !changed {
                    self.signature.clear();

                    self.signature_position = None;
                }

                let text = &documents.open[path].text;
                let prefix = line_prefix(text, cursor);
                let signature = (changed && (prefix.ends_with(['(', ',']) || signature_active))
                    || (focused
                        && !ui.is_any_item_active()
                        && inputs::active(ui, InputAction::SignatureHelp));

                if prefix.ends_with(')') {
                    self.signature.clear();
                }

                if signature {
                    self.signature_position = Some(cursor);
                }

                let definition = focused && inputs::active(ui, InputAction::Definition);
                let click_definition = inputs::active(ui, InputAction::ClickDefinition)
                    && self
                        .editor
                        .is_mouse_over_glyph(ui.io().mouse_pos())
                        .unwrap_or(false);
                let clicked = self
                    .editor
                    .position_at_mouse(ui.io().mouse_pos())
                    .unwrap_or(cursor);
                let hovered = if ui.is_item_hovered()
                    && !ui.is_mouse_down(dear_imgui_rs::MouseButton::Left)
                    && self
                        .editor
                        .is_mouse_over_glyph(ui.io().mouse_pos())
                        .unwrap_or(false)
                {
                    self.editor
                        .position_at_mouse(ui.io().mouse_pos())
                        .ok()
                        .and_then(|position| self.editor.word_start(position).ok())
                } else {
                    None
                };
                let mut assistance = self.assistance.borrow_mut();

                if let Some(popup) = &mut self.documentation_popup {
                    let over_popup = popup.contains(ui.io().mouse_pos());

                    if over_popup || hovered == self.hover_candidate.map(|(position, _)| position) {
                        popup.last_hover = ui.time();
                    } else if ui.time() - popup.last_hover > 0.25 {
                        if popup.focused {
                            ui.set_window_focus(Some(WINDOW_NAME));
                            self.editor.focus();
                        }

                        self.documentation_popup = None;
                        self.hover_candidate = None;
                        assistance.hover_position = None;
                        self.documentation_has_content = false;
                    }
                } else {
                    if self.hover_candidate.map(|(position, _)| position) != hovered {
                        self.hover_candidate = hovered.map(|position| (position, ui.time()));
                        assistance.hover_position = None;
                        self.documentation_has_content = false;
                    }

                    if let Some((position, since)) = self.hover_candidate {
                        if ui.time() - since
                            >= settings.hover_delay_ms.clamp(200, 2000) as f64 / 1000.0
                            && assistance.hover_position != Some(position)
                        {
                            assistance.hover_position = Some(position);
                            assistance.hover = Some(position);
                        }
                    }
                }

                if let Some(error) = assistance.error.take() {
                    *output_error = Some(error);
                }

                if changed
                    && settings.autocomplete
                    && assistance.available
                    && prefix.ends_with(['.', ':', '{', ','])
                {
                    assistance.completion = Some(cursor);
                    assistance.completion_position = Some(cursor);
                }

                let completion = assistance.completion.take();
                let hover = assistance.hover.take();

                if let Some(lsp) = &mut documents.lsp {
                    if std::mem::take(&mut self.completions.resolve) {
                        if let Some(item) = self.completions.items.get(self.completions.selected) {
                            lsp.resolve_completion(
                                path,
                                &documents.open[path].text,
                                documents.open[path].version,
                                cursor.line,
                                cursor.column,
                                item.clone(),
                            );
                        }
                    }

                    let doc = &documents.open[path];

                    for (method, position) in [
                        ("textDocument/completion", completion),
                        ("textDocument/hover", hover),
                        ("textDocument/signatureHelp", signature.then_some(cursor)),
                        (
                            "textDocument/definition",
                            if click_definition {
                                Some(clicked)
                            } else {
                                definition.then_some(cursor)
                            },
                        ),
                    ] {
                        if let Some(position) = position {
                            lsp.request(
                                method,
                                path,
                                &doc.text,
                                doc.version,
                                position.line,
                                position.column,
                            );
                        }
                    }
                }

                if self.documentation_has_content && self.completions.items.is_empty() {
                    let popup = self.documentation_popup.get_or_insert_with(|| {
                        let viewport = ui.window_viewport();
                        let work_position = viewport.work_pos();
                        let work_size = viewport.work_size();
                        let width = 560.0_f32.min(work_size[0]).max(1.0);
                        let text = self.documentation.text().unwrap_or_default();
                        let height =
                            ui.calc_text_size_with_opts(&text, false, (width - 32.0).max(1.0))[1]
                                + 32.0;
                        let size = [width, height.min(420.0).min(work_size[1]).max(1.0)];
                        let mouse = ui.io().mouse_pos();
                        let position = std::array::from_fn(|axis| {
                            (mouse[axis] + 12.0).clamp(
                                work_position[axis],
                                work_position[axis]
                                    + (work_size[axis]
                                        - [width, 420.0_f32.min(work_size[1])][axis])
                                        .max(0.0),
                            )
                        });

                        DocumentationPopup {
                            position,
                            size,
                            last_hover: ui.time(),
                            focused: false,
                            bounds: [position, [position[0] + size[0], position[1] + size[1]]],
                        }
                    });

                    popup::window(
                        ui,
                        "Documentation##Code",
                        popup.position,
                        Some(popup.size),
                        dear_imgui_rs::WindowFlags::empty(),
                        || {
                            let position = ui.window_pos();
                            let size = ui.window_size();

                            popup.bounds =
                                [position, [position[0] + size[0], position[1] + size[1]]];

                            if let Err(error) =
                                self.documentation_view
                                    .draw(ui, &mut self.documentation, font_size)
                            {
                                *output_error = Some(error);
                            }

                            popup.focused = ui.is_window_focused_with_flags(
                                dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS,
                            );

                            popup.size[1] = (self.documentation_view.height + 24.0)
                                .min(420.0)
                                .min(ui.window_viewport().work_size()[1])
                                .max(1.0);
                        },
                    );
                }

                drop(assistance);

                if focused && inputs::active(ui, InputAction::DismissAssistance) {
                    self.signature.clear();

                    self.signature_position = None;
                }

                if completion_focused && !self.completions.items.is_empty() {
                    let accepted = match self.completions.draw(
                        ui,
                        popup_position,
                        &mut self.popup_content,
                        font_size,
                    ) {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            *output_error = Some(error);

                            false
                        }
                    };

                    if accepted {
                        let utf8 = documents.lsp.as_ref().is_some_and(|lsp| lsp.utf8);

                        if let Err(error) = self.completions.accept(
                            &mut self.editor,
                            &documents.open[path].text,
                            utf8,
                        ) {
                            *output_error = Some(error);
                        }

                        if let Ok(text) = self.editor.text() {
                            documents.edit(path, text);

                            self.version = documents.open[path].version;
                        }

                        self.assistance.borrow_mut().completion_position = None;

                        ui.set_window_focus(Some(WINDOW_NAME));
                        self.editor.focus();
                    }
                }

                if focused && !self.signature.is_empty() && self.completions.items.is_empty() {
                    let (label, description) = self
                        .signature
                        .split_once('\n')
                        .unwrap_or((&self.signature, ""));
                    let markdown = format!("```lua\n{label}\n```\n\n{description}");

                    if let Err(error) = self.popup_content.set(&markdown) {
                        *output_error = Some(error);
                    }

                    let viewport = ui.window_viewport();
                    let size = [
                        500.0_f32.min(viewport.work_size()[0]).max(1.0),
                        (self.popup_content.height() + 24.0)
                            .max(font_size * 3.0)
                            .min(340.0)
                            .min(viewport.work_size()[1])
                            .max(1.0),
                    ];

                    popup::window(
                        ui,
                        "Signature##Code",
                        popup_position,
                        Some(size),
                        dear_imgui_rs::WindowFlags::NO_INPUTS,
                        || {
                            if let Err(error) = self.popup_content.draw(ui, font_size) {
                                *output_error = Some(error);
                            }
                        },
                    );
                }
            }

            Err(error) => *output_error = Some(error.to_string()),
        }
    }
}
#[derive(Default)]
pub(super) struct State {
    tabs: BTreeMap<PathBuf, Tab>,
    pub(super) bold_font: Option<dear_imgui_rs::FontId>,
    zoom: f32,
    pub(super) projects_open: bool,
    root: String,
    browser: Option<browser::State>,
    name: String,
    three_d: bool,
    files: files::State,
    pending: Option<Pending>,
    ready: Option<ProjectAction>,
    focus_requested: bool,
    displayed_active: Option<PathBuf>,
    jump: Option<(PathBuf, usize, usize, bool)>,
    navigation: Option<(bool, String)>,
    navigation_opening: bool,
    navigation_selected: usize,
    diagnostics: diagnostics::Footer,
    pub focused: bool,
    pub error: Option<String>,
}
impl State {
    pub fn prepare(
        &mut self,
        context: &dear_imgui_rs::Context,
        documents: Option<&Documents>,
        settings: &Settings,
    ) {
        let Some(documents) = documents else {
            self.tabs.clear();

            return;
        };

        self.tabs
            .retain(|path, _| documents.open.contains_key(path));

        for (path, doc) in &documents.open {
            let result = (|| {
                if !self.tabs.contains_key(path) {
                    let mut editor = TextEditor::try_create(context)?;
                    let lua = crate::editor::lsp::analyzed_path(&documents.context, path);

                    editor.set_language(lua.then_some(Language::Lua));
                    editor.set_show_line_numbers(true);
                    editor.set_auto_indent_enabled(true);
                    editor.set_text(&doc.text)?;

                    let mut palette = Palette::dark();

                    for (slot, color) in [
                        (PaletteColor::Text, 0xffd4d4d4),
                        (PaletteColor::Identifier, 0xffd4d4d4),
                        (PaletteColor::Keyword, 0xffd69c56),
                        (PaletteColor::Declaration, 0xffb0c94e),
                        (PaletteColor::Number, 0xffa8ceb5),
                        (PaletteColor::String, 0xff7891ce),
                        (PaletteColor::Comment, 0xff55996a),
                        (PaletteColor::Background, 0x00000000),
                        (PaletteColor::LineNumber, 0xff858585),
                        (PaletteColor::Selection, 0xff784f26),
                    ] {
                        palette.set(slot, color);
                    }

                    editor.set_palette(&palette)?;

                    if let Some(value) = documents.lsp_diagnostics.get(path) {
                        diagnostic_markers(&mut editor, value)?;
                    }

                    let assistance = Rc::new(RefCell::new(Assistance {
                        available: lua && documents.lsp.is_some(),
                        ..Assistance::default()
                    }));
                    let changes = assistance.clone();

                    editor.set_transaction_callback(move |_| {
                        changes.borrow_mut().changed = true;
                    })?;

                    let documentation = popup::editor(context, &editor.palette())?;
                    let popup_content = popup::Content::new(context, &editor.palette())?;

                    self.tabs.insert(
                        path.clone(),
                        Tab {
                            editor,
                            chord_started: None,
                            version: doc.version,
                            completions: completion::Completion::default(),
                            settings: None,
                            assistance,
                            signature: String::new(),
                            signature_position: None,
                            documentation,
                            documentation_view: documentation::Documentation::default(),
                            documentation_has_content: false,
                            hover_candidate: None,
                            documentation_popup: None,
                            popup_content,
                        },
                    );
                }

                let tab = self.tabs.get_mut(path).unwrap();

                tab.configure(settings)?;

                if tab.version != doc.version {
                    tab.completions.clear();

                    tab.assistance.borrow_mut().completion_position = None;

                    let cursor = tab.editor.main_cursor_position();

                    tab.editor.set_text(&doc.text)?;

                    if let Some(value) = documents.lsp_diagnostics.get(path) {
                        diagnostic_markers(&mut tab.editor, value)?;
                    }

                    let _ = tab.editor.set_cursor(cursor);

                    tab.version = doc.version;
                }

                Ok::<_, dear_imgui_cte::CteError>(())
            })();

            if let Err(error) = result {
                self.error = Some(error.to_string());
            }
        }
    }

    pub fn reset(&mut self) {
        self.tabs.clear();

        self.files = files::State::default();
        self.focus_requested = false;
        self.displayed_active = None;
        self.jump = None;
        self.navigation = None;
        self.diagnostics = diagnostics::Footer::default();
        self.error = None;
        self.focused = false;
        self.projects_open = true;
        self.browser = None;
    }

    pub fn request_exit(&mut self) {
        self.pending = Some(Pending::Project(ProjectAction::Exit));
    }

    pub fn take_action(&mut self) -> Option<ProjectAction> {
        self.ready.take()
    }

    pub fn projects(&mut self, mut editor: Option<&mut Editor>, ui: &dear_imgui_rs::Ui) {
        if self.projects_open {
            let mut open = true;

            super::windows::window(ui, "Projects", [440.0, 250.0])
                .opened(&mut open)
                .build(|| {
                    ui.text("Directory");

                    let style = ui.clone_style();
                    let browse_width =
                        ui.calc_text_size("Browse")[0] + style.frame_padding()[0] * 2.0;

                    ui.set_next_item_width(
                        (ui.content_region_avail_width() - browse_width - style.item_spacing()[0])
                            .max(1.0),
                    );

                    ui.input_text("##ProjectDirectory", &mut self.root).build();
                    ui.same_line();

                    if ui.button_with_size("Browse", [browse_width, 0.0]) {
                        self.browse();
                    }

                    if ui.button("Open project") {
                        self.pending = Some(Pending::Project(ProjectAction::Open(
                            self.root.clone().into(),
                        )));
                    }

                    ui.input_text("Name", &mut self.name).build();
                    ui.checkbox("3D template", &mut self.three_d);

                    if ui.button("Create project") {
                        self.pending = Some(Pending::Project(ProjectAction::Create(
                            self.root.clone().into(),
                            self.name.clone(),
                            self.three_d,
                        )));
                    }

                    if editor.is_some() {
                        ui.same_line();

                        if ui.button("Close project") {
                            self.pending = Some(Pending::Project(ProjectAction::Close));
                        }
                    }

                    if let Some(error) = &self.error {
                        ui.text_wrapped(error);
                    }
                });

            self.projects_open = open;
        }

        if let Some(browser) = &mut self.browser {
            if let Some(result) = browser.draw(ui) {
                self.finish_browse(result);
            }
        }

        if let Some(pending) = self.pending.clone() {
            let dirty = editor
                .as_ref()
                .and_then(|e| e.documents.as_ref())
                .is_some_and(|documents| match &pending {
                    Pending::Document(path) => documents.open.get(path).is_some_and(|d| d.dirty()),
                    Pending::Project(_) => documents.dirty(),
                });
            let exporting = editor.as_ref().is_some_and(|e| e.is_exporting());

            if exporting {
                self.error = Some("Finish or cancel export before closing the project.".into());
                self.pending = None;

                return;
            }

            if !dirty {
                self.finish(pending, editor.as_deref_mut());

                return;
            }

            ui.open_popup("Unsaved changes");

            let mut open = true;

            if let Some(_modal) = super::windows::modal(ui, "Unsaved changes", &mut open) {
                ui.text("Save changes before continuing?");

                if ui.button("Save") {
                    let result = editor
                        .as_deref_mut()
                        .and_then(|e| e.documents.as_mut())
                        .map_or(Ok(()), |d| match &pending {
                            Pending::Document(path) => d.save(path),
                            Pending::Project(_) => d.save_all(),
                        });

                    match result {
                        Ok(()) => {
                            self.finish(pending.clone(), editor.as_deref_mut());
                            ui.close_current_popup();
                        }

                        Err(error) => self.error = Some(error),
                    }
                }

                ui.same_line();

                if ui.button("Discard") {
                    self.finish(pending, editor.as_deref_mut());
                    ui.close_current_popup();
                }

                ui.same_line();

                if ui.button("Cancel") {
                    self.pending = None;

                    ui.close_current_popup();
                }

                if let Some(error) = &self.error {
                    ui.text_wrapped(error);
                }
            }

            if !open {
                self.pending = None;
            }
        }
    }

    fn browse(&mut self) {
        match browser::State::new(&self.root) {
            Ok(browser) => self.browser = Some(browser),
            Err(error) => self.error = Some(format!("Could not browse directories: {error}.")),
        }
    }

    fn finish_browse(&mut self, path: Option<PathBuf>) {
        self.browser = None;

        if let Some(path) = path {
            if let Some(path) = path.to_str() {
                self.root = path.to_owned();
                self.error = None;
            } else {
                self.error = Some("Directory path must be valid UTF-8.".into());
            }
        }
    }

    fn finish(&mut self, pending: Pending, editor: Option<&mut Editor>) {
        match pending {
            Pending::Project(action) => self.ready = Some(action),
            Pending::Document(path) => {
                if let Some(documents) = editor.and_then(|e| e.documents.as_mut()) {
                    if let Some(lsp) = &mut documents.lsp {
                        lsp.close(&path);
                    }

                    documents.open.remove(&path);

                    if documents.active.as_ref() == Some(&path) {
                        documents.active = documents.open.keys().next().cloned();
                    }
                }

                self.tabs.remove(&path);
            }
        }

        self.pending = None;
    }

    pub fn draw(&mut self, documents: &mut Documents, settings: &Settings, ui: &dear_imgui_rs::Ui) {
        self.focused = false;

        let formatting = settings.format_on_save.then(|| {
            serde_json::json!({
                "tabSize": settings.tab_size.clamp(1, 8), "insertSpaces": settings.insert_spaces,
                "trimTrailingWhitespace": settings.trim_trailing_whitespace,
                "insertFinalNewline": settings.insert_final_newline,
            })
        });
        if documents.formatting != formatting {
            documents.clear_formatting_previews();
        }
        documents.formatting = formatting;

        for reply in std::mem::take(&mut documents.replies) {
            let document = documents.open.get(&reply.path);

            if document.is_some_and(|doc| doc.version != reply.version)
                || (document.is_none() && reply.method != "textDocument/publishDiagnostics")
            {
                continue;
            }

            match reply.method.as_str() {
                "textDocument/completion" => {
                    if let Some(tab) = self.tabs.get_mut(&reply.path) {
                        let assistance = tab.assistance.borrow();

                        if reply.position
                            == assistance.completion_position.map(|p| (p.line, p.column))
                            && reply.position
                                == Some((
                                    tab.editor.main_cursor_position().line,
                                    tab.editor.main_cursor_position().column,
                                ))
                        {
                            tab.completions
                                .receive(&reply.value, tab.editor.main_cursor_position());
                        }
                    }
                }
                "completionItem/resolve" => {
                    if let Some(tab) = self.tabs.get_mut(&reply.path) {
                        if reply.position == tab.completions.position.map(|p| (p.line, p.column)) {
                            tab.completions.resolve(reply.value);
                        }
                    }
                }
                "textDocument/definition" => {
                    if documents.active.as_ref() != Some(&reply.path) {
                        continue;
                    }

                    let location = reply
                        .value
                        .as_array()
                        .and_then(|items| items.first())
                        .unwrap_or(&reply.value);

                    if let Some(path) = location
                        .get("uri")
                        .or_else(|| location.get("targetUri"))
                        .and_then(serde_json::Value::as_str)
                        .and_then(crate::editor::lsp::uri_path)
                    {
                        if let Ok(path) = path.strip_prefix(documents.context.root()) {
                            let start = location
                                .pointer("/range/start")
                                .or_else(|| location.pointer("/targetSelectionRange/start"));

                            if let Some(start) = start {
                                self.jump = Some((
                                    path.to_owned(),
                                    start["line"].as_u64().unwrap_or(0) as usize,
                                    start["character"].as_u64().unwrap_or(0) as usize,
                                    documents.lsp.as_ref().is_some_and(|lsp| lsp.utf8),
                                ));

                                if documents.open(path).is_ok() {
                                    self.focus_requested = true;
                                }
                            }
                        }
                    }
                }
                "textDocument/publishDiagnostics" => {
                    if !crate::editor::lsp::analyzed_path(&documents.context, &reply.path)
                        || !reply.value.is_array()
                    {
                        continue;
                    }

                    if let Some(tab) = self.tabs.get_mut(&reply.path) {
                        if let Err(error) = diagnostic_markers(&mut tab.editor, &reply.value) {
                            self.error = Some(error.to_string());
                        }
                    }

                    documents.lsp_diagnostics.insert(reply.path, reply.value);
                }
                "textDocument/hover" => {
                    if let Some(tab) = self.tabs.get_mut(&reply.path) {
                        let assistance = tab.assistance.borrow();

                        if reply.position == assistance.hover_position.map(|p| (p.line, p.column)) {
                            tab.documentation_view =
                                documentation::Documentation::parse(&assistance_text(&reply.value));

                            let text = &tab.documentation_view.text;

                            tab.documentation_has_content = !text.is_empty();

                            let result = tab
                                .documentation
                                .set_text(&text)
                                .and_then(|_| tab.documentation.set_cursor(Position::new(0, 0)))
                                .and_then(|_| {
                                    tab.documentation
                                        .scroll_to_line(0, dear_imgui_cte::ScrollAlignment::Top)
                                });

                            if let Err(error) = result {
                                self.error = Some(error.to_string());
                            }
                        }
                    }
                }
                "textDocument/signatureHelp" => {
                    if let Some(tab) = self.tabs.get_mut(&reply.path) {
                        if reply.position == tab.signature_position.map(|p| (p.line, p.column)) {
                            tab.signature = assistance_text(&reply.value);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some((path, line, units, utf8)) = self.jump.clone() {
            if let Some(tab) = self.tabs.get_mut(&path) {
                let column = crate::editor::lsp::glyph_column(
                    &documents.open[&path].text,
                    line,
                    units,
                    utf8,
                );
                let _ = tab
                    .editor
                    .set_cursor(dear_imgui_cte::Position::new(line, column));

                self.jump = None;
            }
        }

        ui.window(FILES_WINDOW).build(|| {
            self.focus_requested |= self.files.draw(documents, ui, &mut self.error);
        });

        if self.focus_requested {
            ui.set_window_focus(Some(WINDOW_NAME));
        }

        ui.window(WINDOW_NAME).build(|| {
            let _tabs_style = ui.push_style_var(dear_imgui_rs::StyleVar::TabRounding(0.0));

            self.focused = ui
                .is_window_focused_with_flags(dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS);

            self.document_shortcuts(documents, settings, ui);

            let requested_active = documents.active.clone();
            let select_active = self.displayed_active != requested_active || self.focus_requested;
            let base_size = if settings.font_size.is_finite() {
                settings.font_size.clamp(10.0, 28.0)
            } else {
                16.0
            };

            if self.focused {
                if inputs::active(ui, InputAction::ZoomIn) {
                    self.zoom += 1.0;
                }

                if inputs::active(ui, InputAction::ZoomOut) {
                    self.zoom -= 1.0;
                }
            }

            self.zoom = self.zoom.clamp(10.0 - base_size, 28.0 - base_size);

            if documents.replaying && ui.button("Cancel replay") {
                documents.cancel_replay();
            }

            if let Some(_bar) = ui.tab_bar_with_flags(
                "Documents",
                dear_imgui_rs::TabBarFlags::REORDERABLE
                    | dear_imgui_rs::TabBarFlags::AUTO_SELECT_NEW_TABS,
            ) {
                let paths = documents.open.keys().cloned().collect::<Vec<_>>();

                for path in paths {
                    let dirty = documents.open[&path].dirty();
                    let label = tab_label(&path);
                    let mut flags = if dirty {
                        dear_imgui_rs::TabItemFlags::UNSAVED_DOCUMENT
                    } else {
                        dear_imgui_rs::TabItemFlags::NONE
                    };
                    let mut open = true;

                    if requested_active.as_ref() == Some(&path) && select_active {
                        flags |= dear_imgui_rs::TabItemFlags::SET_SELECTED;
                    }

                    if let Some(_tab) = ui.tab_item_with_flags(&label, Some(&mut open), flags) {
                        if select_active && requested_active.as_ref() != Some(&path) {
                            continue;
                        }

                        documents.active = Some(path.clone());

                        if documents.open[&path].conflict {
                            ui.text_wrapped("This file changed outside the editor.");

                            if ui.button("Keep local buffer") {
                                if let Err(error) = documents.resolve_conflict(&path, true) {
                                    self.error = Some(error);
                                }
                            }

                            ui.same_line();

                            if ui.button("Load disk version") {
                                if let Err(error) = documents.resolve_conflict(&path, false) {
                                    self.error = Some(error);
                                }
                            }
                        }

                        let save_requested = self.focused && inputs::active(ui, InputAction::Save);

                        if let Some(tab) = self.tabs.get_mut(&path) {
                            let mut messages = documents.diagnostic_messages();

                            messages.extend(self.error.iter().cloned());

                            let footer_height = self.diagnostics.height(ui, &messages);

                            if self.focus_requested || self.displayed_active.as_ref() != Some(&path)
                            {
                                tab.editor.focus();

                                self.focus_requested = false;
                                self.focused = true;
                            }

                            tab.draw(
                                documents,
                                &path,
                                settings,
                                base_size + self.zoom,
                                self.bold_font,
                                self.focused,
                                ui,
                                footer_height,
                                &mut self.error,
                            );

                            self.focused |= ui.is_window_focused_with_flags(
                                dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS,
                            );

                            self.focused |= tab
                                .documentation_popup
                                .as_ref()
                                .is_some_and(|popup| popup.focused);

                            if save_requested {
                                if let Err(error) = tab.save(documents, &path) {
                                    self.error = Some(error);
                                }
                            }

                            let cursor = tab.editor.main_cursor_position();
                            let position = format!(
                                "Ln {}, Col {}    {}: {}",
                                cursor.line + 1,
                                cursor.column + 1,
                                if settings.insert_spaces {
                                    "Spaces"
                                } else {
                                    "Tabs"
                                },
                                settings.tab_size.clamp(1, 8),
                            );

                            if let Some(message) = self.diagnostics.draw(
                                ui,
                                &messages,
                                &documents.status,
                                &position,
                                footer_height,
                            ) {
                                self.navigate_diagnostic(documents, &message);
                            }
                        }
                    }

                    if !open {
                        self.pending = Some(Pending::Document(path));
                    }
                }
            }

            self.navigation_popup(documents, ui);

            if documents.open.is_empty() {
                let mut messages = documents.diagnostic_messages();

                messages.extend(self.error.iter().cloned());

                let height = self.diagnostics.height(ui, &messages);
                let position = ui.cursor_pos();

                ui.set_cursor_pos([
                    position[0],
                    position[1] + (ui.content_region_avail()[1] - height).max(0.0),
                ]);

                if let Some(message) =
                    self.diagnostics
                        .draw(ui, &messages, &documents.status, "Lua", height)
                {
                    self.navigate_diagnostic(documents, &message);
                }
            }
        });

        self.displayed_active = documents.active.clone();

        if settings.auto_save && !documents.replaying {
            for path in documents.auto_save_paths(std::time::Duration::from_millis(
                settings.auto_save_delay_ms.clamp(100, 60_000) as u64,
            )) {
                let result = if let Some(tab) = self.tabs.get_mut(&path) {
                    tab.save(documents, &path)
                } else {
                    documents.save(&path)
                };
                if let Err(error) = result {
                    self.error = Some(error);
                }
            }
        }

        ui.window(CONSOLE_WINDOW).build(|| {
            for line in &documents.console {
                ui.text_wrapped(line);
            }
        });
    }

    fn navigate_diagnostic(&mut self, documents: &mut Documents, message: &str) {
        if let Some((path, line, column)) = diagnostic_location(message) {
            let path = PathBuf::from(path);

            match documents.open(&path) {
                Ok(()) => {
                    let utf8 = !message
                        .trim_start_matches("Warning: ")
                        .starts_with("LuaLS ")
                        || documents.lsp.as_ref().is_some_and(|lsp| lsp.utf8);

                    self.jump = Some((path, line, column, utf8));
                    self.focus_requested = true;
                }

                Err(error) => self.error = Some(error),
            }
        }
    }
}
fn ruler_columns(rulers: &str) -> impl Iterator<Item = usize> + '_ {
    rulers
        .split(',')
        .filter_map(|value| value.trim().parse::<u16>().ok())
        .filter(|column| *column > 0)
        .map(usize::from)
}
fn indentation_backspace(prefix: &str, tab_size: usize) -> Option<usize> {
    if prefix.is_empty() || !prefix.bytes().all(|byte| byte == b' ') {
        return None;
    }

    let size = tab_size.clamp(1, 8);

    Some((prefix.len() - 1) / size * size)
}
fn diagnostic_markers(
    editor: &mut TextEditor,
    value: &serde_json::Value,
) -> Result<(), dear_imgui_cte::CteError> {
    editor.clear_markers();

    for (line, error) in line_diagnostics(value) {
        let color = if error { 0xff5555f1 } else { 0xff55cde2 };
        let background = if error { 0x445555f1 } else { 0x4455cde2 };

        editor.add_marker(line, color, background, "", "")?;
    }

    Ok(())
}
fn line_diagnostics(value: &serde_json::Value) -> Vec<(usize, bool)> {
    let mut lines = BTreeMap::<usize, bool>::new();

    if let Some(items) = value.as_array() {
        for item in items {
            let severity = item["severity"].as_u64().unwrap_or(1);

            if severity > 2 {
                continue;
            }

            let line = item["range"]["start"]["line"].as_u64().unwrap_or(0) as usize;

            *lines.entry(line).or_default() |= severity == 1;
        }
    }

    lines.into_iter().collect()
}
#[cfg(test)]
fn documentation_text(markdown: &str) -> String {
    documentation::Documentation::parse(markdown).text
}

fn tab_label(path: &std::path::Path) -> String {
    format!(
        "{}###{}",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy(),
        path.display()
    )
}
fn line_prefix(text: &str, position: Position) -> String {
    text.lines()
        .nth(position.line)
        .unwrap_or("")
        .chars()
        .take(position.column)
        .collect()
}

fn diagnostic_location(error: &str) -> Option<(&str, usize, usize)> {
    for line in error.lines() {
        let line = line
            .trim()
            .trim_start_matches("Warning: ")
            .trim_start_matches("LuaLS ");

        let Some(end) = line.find(".lua:") else {
            continue;
        };

        let path = line[..end + 4].rsplit(": ").next()?.trim_start_matches('@');
        let mut parts = line[end + 5..].split(':');

        if let Some(number) = parts.next().and_then(|n| n.parse::<usize>().ok()) {
            let column = parts
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(1);

            return Some((path, number.saturating_sub(1), column.saturating_sub(1)));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{Position, Settings, State, diagnostic_location, ruler_columns};

    #[test]
    fn rulers_accept_comma_separated_positive_columns() {
        assert_eq!(ruler_columns("80, 100").collect::<Vec<_>>(), [80, 100]);
        assert_eq!(ruler_columns("0, invalid, 80").collect::<Vec<_>>(), [80]);
    }

    #[test]
    fn typing_and_switching_files_keep_the_requested_editor_focused() {
        let _context_lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        #[derive(Default)]
        struct Clipboard(String);

        impl dear_imgui_rs::ClipboardBackend for Clipboard {
            fn get(&mut self) -> Option<String> {
                Some(self.0.clone())
            }

            fn set(&mut self, value: &str) {
                self.0 = value.to_owned();
            }
        }

        context.set_clipboard_backend(Clipboard::default());
        context.set_ini_filename(None::<&str>).unwrap();

        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context.load_ini_settings("[Window][Files]\nPos=0,0\nSize=240,600\n[Window][Code]\nPos=250,0\nSize=700,600\n[Window][Console]\nPos=250,610\nSize=700,100\n");

        let root =
            std::env::temp_dir().join(format!("kinematic-code-focus-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("a.lua"), "local a = 1").unwrap();

        std::fs::write(root.join("b.lua"), "local b = 2").unwrap();

        let mut documents = crate::editor::documents::Documents::new(
            crate::core::ProjectContext::new(&root).unwrap(),
            "",
        )
        .unwrap();

        documents.lsp = None;

        let a = std::path::Path::new("a.lua");
        let b = std::path::Path::new("b.lua");

        documents.open(b).unwrap();
        documents.open(a).unwrap();

        let settings = Settings {
            autocomplete: false,
            tab_size: 2,
            ..Settings::default()
        };
        let mut state = State::default();

        state.focus_requested = true;

        for _ in 0..4 {
            state.prepare(&context, Some(&documents), &settings);
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        assert_eq!(state.tabs[a].editor.tab_size(), 2);

        let original_text = documents.open[a].text.clone();

        for (ctrl, shift, key, zoom) in [
            (false, false, dear_imgui_rs::Key::Equal, 0.0),
            (true, true, dear_imgui_rs::Key::Equal, 1.0),
            (true, false, dear_imgui_rs::Key::Minus, 0.0),
            (true, true, dear_imgui_rs::Key::Minus, -1.0),
            (true, false, dear_imgui_rs::Key::KeypadAdd, 0.0),
        ] {
            context
                .io_mut()
                .add_key_event(dear_imgui_rs::Key::ModCtrl, ctrl);

            context
                .io_mut()
                .add_key_event(dear_imgui_rs::Key::ModShift, shift);

            context.io_mut().add_key_event(key, true);
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
            assert_eq!(state.zoom, zoom);
            assert_eq!(documents.open[a].text, original_text);
            context.io_mut().add_key_event(key, false);

            context
                .io_mut()
                .add_key_event(dear_imgui_rs::Key::ModCtrl, false);

            context
                .io_mut()
                .add_key_event(dear_imgui_rs::Key::ModShift, false);

            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        for character in ['x', 'y'] {
            context.io_mut().add_input_character(character);
            state.prepare(&context, Some(&documents), &settings);
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
            assert_eq!(documents.active.as_deref(), Some(a));
            assert!(state.focused);
            assert!(state.error.is_none(), "{:?}", state.error);
        }

        assert!(documents.open[a].text.contains("xy"));
        assert_eq!(documents.open[b].text, "local b = 2");
        documents.open(b).unwrap();

        state.focus_requested = true;

        for _ in 0..3 {
            state.prepare(&context, Some(&documents), &settings);
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        context.io_mut().add_input_character('z');
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.active.as_deref(), Some(b));
        assert!(documents.open[b].text.contains('z'));
        assert!(state.focused);
        assert!(state.error.is_none(), "{:?}", state.error);

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/publishDiagnostics".into(),
            path: b.to_owned(), version: documents.open[b].version, position: None,
            value: serde_json::json!([
                { "range": { "start": { "line": 0, "character": 0 } }, "severity": 1, "message": "Test error." },
                { "range": { "start": { "line": 0, "character": 0 } }, "severity": 2, "message": "Test warning." },
                { "range": { "start": { "line": 0, "character": 0 } }, "severity": 3, "message": "Test information." }
            ]),
        });

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert!(state.tabs[b].editor.has_markers());

        assert!(
            documents
                .diagnostic_messages()
                .iter()
                .any(|message| message == "LuaLS b.lua:1:1: Test error.")
        );

        assert!(
            documents
                .diagnostic_messages()
                .iter()
                .any(|message| message == "Warning: LuaLS b.lua:1:1: Test warning.")
        );

        assert!(
            !documents
                .diagnostic_messages()
                .iter()
                .any(|message| message.contains("Test information."))
        );

        documents.diagnostics.clear();
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.diagnostic_messages().len(), 2);

        documents.diagnostics = vec!["Preview failed.".into()];

        assert_eq!(documents.diagnostic_messages().len(), 3);
        documents.diagnostics.clear();

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/publishDiagnostics".into(),
            path: b.to_owned(), version: documents.open[b].version, position: None,
            value: serde_json::json!([{ "range": { "start": { "line": 0, "character": 0 } }, "severity": 2, "message": "Remaining warning." }]),
        });

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        assert_eq!(
            documents.diagnostic_messages(),
            ["Warning: LuaLS b.lua:1:1: Remaining warning."]
        );

        documents.edit(b, "local b = 3".into());

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/publishDiagnostics".into(),
            path: b.to_owned(),
            version: documents.open[b].version - 1,
            position: None,
            value: serde_json::json!([]),
        });

        state.prepare(&context, Some(&documents), &settings);
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        assert_eq!(
            documents.diagnostic_messages(),
            ["Warning: LuaLS b.lua:1:1: Remaining warning."]
        );

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/publishDiagnostics".into(),
            path: b.to_owned(),
            version: documents.open[b].version,
            position: None,
            value: serde_json::json!([]),
        });

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert!(!state.tabs[b].editor.has_markers());

        assert!(
            !documents
                .diagnostic_messages()
                .iter()
                .any(|message| message.contains("LuaLS b.lua:"))
        );

        documents.edit(b, "cir".into());
        state.prepare(&context, Some(&documents), &settings);

        state
            .tabs
            .get_mut(b)
            .unwrap()
            .editor
            .set_cursor(dear_imgui_cte::Position::new(0, 3))
            .unwrap();

        state.focus_requested = true;

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::ModCtrl, true);

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Space, true);

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Space, false);

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::ModCtrl, false);

        std::thread::sleep(std::time::Duration::from_millis(220));
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        assert_eq!(
            state.tabs[b].assistance.borrow().completion_position,
            Some(dear_imgui_cte::Position::new(0, 3))
        );

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/completion".into(),
            path: b.to_owned(),
            version: documents.open[b].version,
            position: Some((0, 3)),
            value: serde_json::json!([{ "label": "circle" }]),
        });

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Enter, true);

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.open[b].text, "circle");

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Enter, false);

        context.io_mut().add_input_character('x');
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.open[b].text, "circlex");
        assert!(state.focused);
        assert!(!state.tabs[b].editor.shows_spaces());
        documents.edit(b, "        value".into());
        state.prepare(&context, Some(&documents), &settings);

        state
            .tabs
            .get_mut(b)
            .unwrap()
            .editor
            .set_cursor(dear_imgui_cte::Position::new(0, 8))
            .unwrap();

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Backspace, true);

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.open[b].text, "      value");

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Backspace, false);

        state.tabs.get_mut(b).unwrap().editor.undo();
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.open[b].text, "        value");
        documents.edit(b, "circle()".into());
        state.prepare(&context, Some(&documents), &settings);
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        let point = (30..160)
            .flat_map(|y| (260..450).map(move |x| [x as f32, y as f32]))
            .find(|point| {
                state.tabs[b]
                    .editor
                    .is_mouse_over_glyph(*point)
                    .unwrap_or(false)
                    && state.tabs[b]
                        .editor
                        .position_at_mouse(*point)
                        .unwrap()
                        .column
                        < 6
            })
            .unwrap();

        context.io_mut().add_mouse_pos_event(point);

        for _ in 0..4 {
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        assert!(state.tabs[b].hover_candidate.is_some());
        assert!(state.tabs[b].assistance.borrow().hover_position.is_none());
        context.io_mut().set_delta_time(0.7);
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        assert_eq!(
            state.tabs[b].assistance.borrow().hover_position,
            Some(dear_imgui_cte::Position::new(0, 0))
        );

        assert!(!state.tabs[b].documentation_has_content);

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/hover".into(), path: b.to_owned(), version: documents.open[b].version,
            position: Some((0, 0)), value: serde_json::json!({"contents":{"kind":"markdown","value":"```lua\nfunction circle() end\n```\nDraw a circle."}}),
        });

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert!(state.tabs[b].documentation_has_content);

        assert_eq!(
            state.tabs[b].documentation.text().unwrap(),
            "function circle() end\n\nDraw a circle."
        );

        assert!(state.focused);

        let popup_position = state.tabs[b].documentation_popup.as_ref().unwrap().position;

        context.io_mut().set_delta_time(0.01);

        context
            .io_mut()
            .add_mouse_pos_event([popup_position[0] + 30.0, popup_position[1] + 65.0]);

        for _ in 0..3 {
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        assert_eq!(
            state.tabs[b].documentation_popup.as_ref().unwrap().position,
            popup_position
        );

        assert!(state.tabs[b].documentation_has_content);

        let documentation = "function circle() end\n\nDraw a circle.";

        for pressed in [true, false] {
            context
                .io_mut()
                .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, pressed);

            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        for (key, pressed) in [
            (dear_imgui_rs::Key::ModCtrl, true),
            (dear_imgui_rs::Key::A, true),
            (dear_imgui_rs::Key::A, false),
            (dear_imgui_rs::Key::C, true),
            (dear_imgui_rs::Key::C, false),
            (dear_imgui_rs::Key::ModCtrl, false),
        ] {
            context.io_mut().add_key_event(key, pressed);
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        assert!(state.focused);
        assert_eq!(context.clipboard_text().as_deref(), Some(documentation));

        let tab = state.tabs.get_mut(b).unwrap();

        assert!(tab.documentation.main_cursor_has_selection());
        tab.documentation.cut();
        assert_eq!(tab.documentation.text().unwrap(), documentation);

        let long_documentation =
            format!("{documentation}\n{}", "Detailed explanation.\n".repeat(80));

        tab.documentation_view = super::documentation::Documentation::parse(&long_documentation);

        tab.documentation
            .set_text(&tab.documentation_view.text)
            .unwrap();

        for _ in 0..3 {
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        context.io_mut().add_mouse_wheel_event([0.0, -8.0]);

        for _ in 0..3 {
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        assert!(state.tabs[b].documentation_view.scroll_y > 0.0);

        assert_eq!(
            state.tabs[b].documentation_popup.as_ref().unwrap().position,
            popup_position
        );

        assert_eq!(documents.open[b].text, "circle()");

        let popup = state
            .tabs
            .get_mut(b)
            .unwrap()
            .documentation_popup
            .as_mut()
            .unwrap();

        popup.position = [850.0, 280.0];
        popup.size[0] = 400.0;

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        let bounds = state.tabs[b].documentation_popup.as_ref().unwrap().bounds;

        assert!(bounds[1][0] > 950.0 && bounds[1][1] > 600.0);

        for point in [
            [bounds[0][0] + 1.0, bounds[0][1] + 1.0],
            [bounds[1][0] - 1.0, bounds[1][1] - 1.0],
            [bounds[1][0] - 8.0, (bounds[0][1] + bounds[1][1]) * 0.5],
            [(bounds[0][0] + bounds[1][0]) * 0.5, bounds[1][1] - 12.0],
        ] {
            context.io_mut().set_delta_time(0.3);
            context.io_mut().add_mouse_pos_event(point);

            for _ in 0..2 {
                state.draw(&mut documents, &settings, context.frame());
                context.end_frame();
            }

            assert!(state.tabs[b].documentation_popup.is_some(), "{point:?}");
        }

        context.io_mut().set_delta_time(0.01);
        context.io_mut().add_mouse_pos_event([-100.0, -100.0]);
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert!(state.tabs[b].documentation_has_content);
        context.io_mut().set_delta_time(0.3);
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert!(!state.tabs[b].documentation_has_content);

        let settings = Settings {
            autocomplete: true,
            ..settings
        };

        documents.edit(b, "ball".into());
        state.prepare(&context, Some(&documents), &settings);

        {
            let tab = state.tabs.get_mut(b).unwrap();

            tab.assistance.borrow_mut().available = true;
            tab.settings = None;

            tab.editor.set_cursor(Position::new(0, 4)).unwrap();
        }

        state.focus_requested = true;

        for _ in 0..3 {
            state.prepare(&context, Some(&documents), &settings);
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        context.io_mut().add_input_character('.');
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        std::thread::sleep(std::time::Duration::from_millis(120));
        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        assert_eq!(
            state.tabs[b].assistance.borrow().completion_position,
            Some(Position::new(0, 5))
        );

        documents.replies.push(crate::editor::lsp::Reply {
            method: "textDocument/completion".into(), path: b.to_owned(),
            version: documents.open[b].version, position: Some((0, 5)),
            value: serde_json::json!([{ "label": "radius", "kind": 5 }, { "label": "position", "kind": 5 }]),
        });

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::DownArrow, true);

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(state.tabs[b].completions.selected, 1);

        assert_eq!(
            state.tabs[b].editor.main_cursor_position(),
            Position::new(0, 5)
        );

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::DownArrow, false);

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Tab, true);

        state.draw(&mut documents, &settings, context.frame());
        context.end_frame();
        assert_eq!(documents.open[b].text, "ball.position");

        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::Tab, false);

        documents.edit(b, "ball.".into());
        state.prepare(&context, Some(&documents), &settings);

        state
            .tabs
            .get_mut(b)
            .unwrap()
            .editor
            .set_cursor(Position::new(0, 5))
            .unwrap();

        state.focus_requested = true;

        for _ in 0..3 {
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        state.tabs.get_mut(b).unwrap().completions.receive(
            &serde_json::json!([{"label":"radius", "kind":5}, {"label":"position", "kind":5}]),
            Position::new(0, 5),
        );

        let mut suggestion_position = [0.0, 0.0];

        for _ in 0..3 {
            let ui = context.frame();

            state.draw(&mut documents, &settings, ui);

            ui.window("Suggestions##Code")
                .build(|| suggestion_position = ui.window_pos());

            context.end_frame();
        }

        context
            .io_mut()
            .add_mouse_pos_event([suggestion_position[0] + 30.0, suggestion_position[1] + 38.0]);

        for _ in 0..3 {
            state.draw(&mut documents, &settings, context.frame());
            context.end_frame();
        }

        for expected in ["ball.", "ball.position"] {
            for pressed in [true, false] {
                context
                    .io_mut()
                    .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, pressed);

                state.draw(&mut documents, &settings, context.frame());
                context.end_frame();
            }

            assert_eq!(documents.open[b].text, expected);
        }

        drop(state);

        drop(documents);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn diagnostics_prioritize_errors_and_documentation_preserves_order() {
        assert_eq!(
            super::line_diagnostics(&serde_json::json!([
                {"severity": 1, "range": {"start": {"line": 0}}},
                {"severity": 2, "range": {"start": {"line": 0}}},
                {"severity": 2, "range": {"start": {"line": 2}}},
                {"severity": 3, "range": {"start": {"line": 3}}}
            ])),
            vec![(0, true), (2, false)]
        );

        assert_eq!(
            super::documentation_text(
                "```lua\nfunction circle() end\n```\n---\nDraw&#x20;a circle."
            ),
            "function circle() end\n\nDraw a circle."
        );

        assert_eq!(
            super::documentation_text("Intro.\n\n```lua\nexample()\n```\n\nDetails.\n\n"),
            "Intro.\n\nexample()\n\nDetails."
        );

        assert_eq!(
            super::assistance_text(
                &serde_json::json!({"signatures":[{"label":"function example()"}]})
            ),
            "function example()"
        );

        assert_eq!(super::indentation_backspace("        ", 4), Some(4));
        assert_eq!(super::indentation_backspace("      ", 4), Some(4));
        assert_eq!(super::indentation_backspace("  ", 4), Some(0));
        assert_eq!(super::indentation_backspace("a   ", 4), None);
        assert_eq!(super::indentation_backspace("", 4), None);
    }

    #[test]
    fn browsing_selects_a_directory_without_opening_and_cancel_preserves_it() {
        let mut state = State::default();

        state.root = std::env::temp_dir().to_str().unwrap().to_owned();

        state.browse();
        assert!(state.browser.is_some());

        let selected = std::env::temp_dir().join("Project with spaces");

        state.finish_browse(Some(selected.clone()));
        assert_eq!(state.root, selected.to_str().unwrap());
        assert!(state.browser.is_none());
        assert!(state.pending.is_none());
        assert!(state.take_action().is_none());
        state.browse();
        state.finish_browse(None);
        assert_eq!(state.root, selected.to_str().unwrap());
        assert!(state.browser.is_none());
        assert!(state.error.is_none());
    }

    #[test]
    fn diagnostic_navigation_preserves_paths_and_columns() {
        assert_eq!(
            diagnostic_location("LuaLS scenes/my scene.lua:12:4: Message."),
            Some(("scenes/my scene.lua", 11, 3))
        );

        assert_eq!(
            diagnostic_location("Warning: LuaLS scenes/my scene.lua:12:4: Message."),
            Some(("scenes/my scene.lua", 11, 3))
        );

        assert_eq!(
            diagnostic_location("Update: runtime error: scenes/main.lua:9: Failed."),
            Some(("scenes/main.lua", 8, 0))
        );

        assert_eq!(diagnostic_location("No location."), None);
    }
}

fn assistance_text(value: &serde_json::Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_owned();
    }

    if let Some(text) = value.get("value").and_then(|v| v.as_str()) {
        return text.to_owned();
    }

    if let Some(contents) = value.get("contents") {
        return assistance_text(contents);
    }

    if let Some(signatures) = value.get("signatures").and_then(|v| v.as_array()) {
        return signatures
            .iter()
            .map(|v| {
                let label = v["label"].as_str().unwrap_or("");
                let documentation = assistance_text(&v["documentation"]);

                if documentation.trim().is_empty() {
                    label.trim().to_owned()
                } else {
                    format!("{label}\n{}", documentation.trim())
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    value
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(assistance_text)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}
