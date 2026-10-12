use super::browser;
use crate::editor::{Editor, documents::Documents};
use dear_imgui_cte::{CteUiExt, Language, TextEditor};
use std::{collections::BTreeMap, path::PathBuf};

pub(super) const WINDOW_NAME: &str = "Code";
pub(super) const FILES_WINDOW: &str = "Files";
pub(super) const DIAGNOSTICS_WINDOW: &str = "Diagnostics";
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
struct Tab {
    editor: TextEditor,
    version: u64,
}
#[derive(Default)]
pub(super) struct State {
    tabs: BTreeMap<PathBuf, Tab>,
    pub(super) projects_open: bool,
    root: String,
    browser: Option<browser::State>,
    name: String,
    three_d: bool,
    file_path: String,
    selected: Option<PathBuf>,
    directory: bool,
    pending: Option<Pending>,
    ready: Option<ProjectAction>,
    completions: Vec<(PathBuf, u64, serde_json::Value)>,
    assistance: String,
    displayed_active: Option<PathBuf>,
    jump: Option<(PathBuf, usize, usize, bool)>,
    pub focused: bool,
    pub error: Option<String>,
}
impl State {
    pub fn prepare(&mut self, context: &dear_imgui_rs::Context, documents: Option<&Documents>) {
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
                    editor.set_language(Some(Language::Lua));
                    editor.set_show_line_numbers(true);
                    editor.set_auto_indent_enabled(true);
                    editor.set_text(&doc.text)?;
                    self.tabs.insert(
                        path.clone(),
                        Tab {
                            editor,
                            version: doc.version,
                        },
                    );
                }
                let tab = self.tabs.get_mut(path).unwrap();
                if tab.version != doc.version {
                    let cursor = tab.editor.main_cursor_position();
                    tab.editor.set_text(&doc.text)?;
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
        self.selected = None;
        self.completions.clear();
        self.assistance.clear();
        self.displayed_active = None;
        self.jump = None;
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
            ui.window("Projects")
                .opened(&mut open)
                .flags(dear_imgui_rs::WindowFlags::NO_DOCKING)
                .size([440.0, 250.0], dear_imgui_rs::Condition::FirstUseEver)
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
            if let Some(_modal) = ui.begin_modal_popup("Unsaved changes") {
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
    pub fn draw(&mut self, documents: &mut Documents, ui: &dear_imgui_rs::Ui) {
        self.focused = false;
        for reply in std::mem::take(&mut documents.replies) {
            if documents
                .open
                .get(&reply.path)
                .is_none_or(|doc| doc.version != reply.version)
            {
                continue;
            }
            match reply.method.as_str() {
                "textDocument/completion" => {
                    self.completions = reply
                        .value
                        .as_array()
                        .or_else(|| {
                            reply
                                .value
                                .get("items")
                                .and_then(serde_json::Value::as_array)
                        })
                        .map(|items| {
                            items
                                .iter()
                                .take(100)
                                .map(|item| (reply.path.clone(), reply.version, item.clone()))
                                .collect()
                        })
                        .unwrap_or_default();
                }
                "textDocument/definition" => {
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
                                let _ = documents.open(path);
                            }
                        }
                    }
                }
                "textDocument/publishDiagnostics" => {
                    let prefix = format!("LuaLS {}:", reply.path.display());
                    documents
                        .diagnostics
                        .retain(|message| !message.starts_with(&prefix));
                    if let Some(items) = reply.value.as_array() {
                        for item in items {
                            documents.diagnostics.push(format!(
                                "{}{}:{}: {}",
                                prefix,
                                item["range"]["start"]["line"].as_u64().unwrap_or(0) + 1,
                                item["range"]["start"]["character"].as_u64().unwrap_or(0) + 1,
                                item["message"].as_str().unwrap_or("Diagnostic.")
                            ));
                        }
                    }
                }
                _ => self.assistance = assistance_text(&reply.value),
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
            ui.input_text("Path", &mut self.file_path).build();
            ui.checkbox("Folder", &mut self.directory);
            if ui.button("Create") {
                if let Err(error) =
                    documents.create(&PathBuf::from(&self.file_path), self.directory)
                {
                    self.error = Some(error);
                }
            }
            ui.same_line();
            if ui.button("Rename selected") {
                if let Some(from) = &self.selected {
                    if let Err(error) = documents.rename(from, &PathBuf::from(&self.file_path)) {
                        self.error = Some(error);
                    }
                }
            }
            ui.text_wrapped("Renaming does not rewrite imports.");
            match documents.files() {
                Ok(files) => {
                    self.file_tree(documents, ui, &files, std::path::Path::new(""));
                }
                Err(error) => self.error = Some(error),
            }
        });
        ui.window(WINDOW_NAME).build(|| {
            self.focused = ui
                .is_window_focused_with_flags(dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS);
            ui.text(&documents.status);
            if documents.replaying && ui.button("Cancel replay") {
                documents.cancel_replay();
            }
            if let Some(_bar) = ui.tab_bar("Documents") {
                let paths = documents.open.keys().cloned().collect::<Vec<_>>();
                for path in paths {
                    let dirty = documents.open[&path].dirty();
                    let label = format!("{}{}", path.display(), if dirty { " *" } else { "" });
                    let mut open = true;
                    let mut flags = dear_imgui_rs::TabItemFlags::NONE;
                    if documents.active.as_ref() == Some(&path)
                        && self.displayed_active != documents.active
                    {
                        flags |= dear_imgui_rs::TabItemFlags::SET_SELECTED;
                    }
                    if let Some(_tab) = ui.tab_item_with_flags(&label, Some(&mut open), flags) {
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
                        if ui.button("Save")
                            || (self.focused
                                && ui.io().key_ctrl()
                                && ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::S, false))
                        {
                            if let Err(error) = documents.save(&path) {
                                self.error = Some(error);
                            }
                        }
                        if let Some(tab) = self.tabs.get_mut(&path) {
                            ui.same_line();
                            if ui.button("Find / replace") {
                                tab.editor.open_find_replace();
                            }
                            ui.same_line();
                            if ui.button("Undo") {
                                tab.editor.undo();
                            }
                            ui.same_line();
                            if ui.button("Redo") {
                                tab.editor.redo();
                            }
                            let cursor = tab.editor.main_cursor_position();
                            for (label, method, shortcut) in [
                                (
                                    "Complete",
                                    "textDocument/completion",
                                    self.focused
                                        && ui.io().key_ctrl()
                                        && ui.is_key_pressed_with_repeat(
                                            dear_imgui_rs::Key::Space,
                                            false,
                                        ),
                                ),
                                (
                                    "Documentation",
                                    "textDocument/hover",
                                    self.focused
                                        && ui.is_key_pressed_with_repeat(
                                            dear_imgui_rs::Key::F1,
                                            false,
                                        ),
                                ),
                                ("Signature", "textDocument/signatureHelp", false),
                                (
                                    "Definition",
                                    "textDocument/definition",
                                    self.focused
                                        && ui.is_key_pressed_with_repeat(
                                            dear_imgui_rs::Key::F12,
                                            false,
                                        ),
                                ),
                            ] {
                                ui.same_line();
                                if ui.button(label) || shortcut {
                                    if let Some(lsp) = &mut documents.lsp {
                                        let doc = &documents.open[&path];
                                        lsp.request(
                                            method,
                                            &path,
                                            &doc.text,
                                            doc.version,
                                            cursor.line,
                                            cursor.column,
                                        );
                                    }
                                }
                            }
                            if !self.assistance.is_empty() {
                                ui.text_wrapped(&self.assistance);
                            }
                            let mut accepted = None;
                            for (index, (file, version, item)) in
                                self.completions.iter().enumerate()
                            {
                                if file != &path || *version != documents.open[&path].version {
                                    continue;
                                }
                                if ui.selectable(item["label"].as_str().unwrap_or("Completion")) {
                                    accepted = Some(index);
                                }
                            }
                            if let Some(index) = accepted {
                                let item = &self.completions[index].2;
                                let text = item
                                    .pointer("/textEdit/newText")
                                    .or_else(|| item.get("insertText"))
                                    .or_else(|| item.get("label"))
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or("");
                                let result = if let Some(range) = item.pointer("/textEdit/range") {
                                    let position = |value: &serde_json::Value| {
                                        let line = value["line"].as_u64().unwrap_or(0) as usize;
                                        let column = crate::editor::lsp::glyph_column(
                                            &documents.open[&path].text,
                                            line,
                                            value["character"].as_u64().unwrap_or(0) as usize,
                                            documents.lsp.as_ref().is_some_and(|lsp| lsp.utf8),
                                        );
                                        dear_imgui_cte::Position::new(line, column)
                                    };
                                    tab.editor.replace_section(
                                        dear_imgui_cte::Selection::new(
                                            position(&range["start"]),
                                            position(&range["end"]),
                                        ),
                                        text,
                                    )
                                } else {
                                    tab.editor.replace_current_selection(text)
                                };
                                if let Err(error) = result {
                                    self.error = Some(error.to_string());
                                }
                                self.completions.clear();
                            }
                            if let Err(error) = ui
                                .text_editor(&mut tab.editor, "Source")
                                .size(ui.content_region_avail())
                                .build()
                            {
                                self.error = Some(error.to_string());
                            }
                            match tab.editor.text() {
                                Ok(text) => {
                                    documents.edit(&path, text);
                                    tab.version = documents.open[&path].version;
                                }
                                Err(error) => self.error = Some(error.to_string()),
                            }
                        }
                    }
                    if !open {
                        self.pending = Some(Pending::Document(path));
                    }
                }
            }
        });
        self.displayed_active = documents.active.clone();
        ui.window(DIAGNOSTICS_WINDOW).build(|| {
            for error in documents.diagnostics.clone() {
                if ui.selectable(&error) {
                    if let Some((path, line, column)) = diagnostic_location(&error) {
                        let path = PathBuf::from(path);
                        if documents.open(&path).is_ok() {
                            let utf8 = !error.starts_with("LuaLS ")
                                || documents.lsp.as_ref().is_some_and(|lsp| lsp.utf8);
                            self.jump = Some((path, line, column, utf8));
                        }
                    }
                }
            }
            if let Some(error) = &self.error {
                ui.text_wrapped(error);
            }
        });
        ui.window(CONSOLE_WINDOW).build(|| {
            for line in &documents.console {
                ui.text_wrapped(line);
            }
        });
    }
    fn file_tree(
        &mut self,
        documents: &mut Documents,
        ui: &dear_imgui_rs::Ui,
        files: &[PathBuf],
        parent: &std::path::Path,
    ) {
        for path in files.iter().filter(|path| path.parent() == Some(parent)) {
            let label = path.file_name().unwrap().to_string_lossy();
            let _id = ui.push_id(path.to_string_lossy().as_ref());
            if documents.context.root().join(path).is_dir() {
                let node = ui
                    .tree_node_config(label.as_ref())
                    .selected(self.selected.as_ref() == Some(path))
                    .push();
                if ui.is_item_clicked() {
                    self.selected = Some(path.clone());
                }
                if let Some(_node) = node {
                    self.file_tree(documents, ui, files, path);
                }
            } else if ui
                .selectable_config(label.as_ref())
                .selected(self.selected.as_ref() == Some(path))
                .build()
            {
                self.selected = Some(path.clone());
                if let Err(error) = documents.open(path) {
                    self.error = Some(error);
                }
            }
        }
    }
}
fn diagnostic_location(error: &str) -> Option<(&str, usize, usize)> {
    for line in error.lines() {
        let line = line.trim().trim_start_matches("LuaLS ");
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
    use super::{State, diagnostic_location};

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
                format!(
                    "{}\n{}",
                    v["label"].as_str().unwrap_or(""),
                    assistance_text(&v["documentation"])
                )
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
