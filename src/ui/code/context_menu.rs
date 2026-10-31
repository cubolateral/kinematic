use super::{Position, Tab, shortcuts};
use dear_imgui_cte::{CteResult, TextChange, TextChangeKind};
use dear_imgui_rs::{MouseButton, Ui};
use kinematic_ui::shortcut;
use std::collections::BTreeSet;

impl Tab {
    pub(super) fn context_menu(&mut self, ui: &Ui, hovered: bool) -> CteResult<()> {
        if hovered && ui.is_mouse_clicked(MouseButton::Right) {
            self.completions.clear();
            self.documentation_popup = None;
            ui.open_popup("Editor context menu");
        }
        let Some(_popup) = ui.begin_popup("Editor context menu") else {
            return Ok(());
        };

        if shortcut::menu_item_enabled_selected(ui, "Undo", "Ctrl+Z", false, self.editor.can_undo())
        {
            self.editor.undo();
        }
        if shortcut::menu_item_enabled_selected(
            ui,
            "Redo",
            "Ctrl+Shift+Z",
            false,
            self.editor.can_redo(),
        ) {
            self.editor.redo();
        }
        ui.separator();
        if shortcut::menu_item(ui, "Cut", "Ctrl+X") {
            shortcuts::cut(&mut self.editor)?;
        }
        if shortcut::menu_item(ui, "Copy", "Ctrl+C") {
            self.editor.copy();
        }
        if shortcut::menu_item(ui, "Paste", "Ctrl+V") {
            self.editor.paste();
        }
        ui.separator();
        if shortcut::menu_item(ui, "Select All", "Ctrl+A") {
            self.editor.select_all();
        }
        ui.separator();
        if shortcut::menu_item(ui, "Indent", "Tab") {
            self.editor.indent_lines();
        }
        if shortcut::menu_item(ui, "Unindent", "Shift+Tab") {
            self.editor.deindent_lines();
        }
        if shortcut::menu_item(ui, "Toggle Comment", "Ctrl+/") {
            self.editor.toggle_comments();
        }
        let line = self.editor.main_cursor_position().line;
        let bookmarked = self.assistance.borrow().bookmarks.contains(&line);
        if shortcut::menu_item_enabled_selected(
            ui,
            "Toggle Bookmark",
            "Ctrl+Alt+B",
            bookmarked,
            true,
        ) {
            self.toggle_bookmark();
        }
        Ok(())
    }

    pub(super) fn toggle_bookmark(&mut self) {
        let line = self.editor.main_cursor_position().line;
        let mut assistance = self.assistance.borrow_mut();
        if !assistance.bookmarks.remove(&line) {
            assistance.bookmarks.insert(line);
        }
    }

    pub(super) fn draw_bookmarks(&self, ui: &Ui) {
        let Some(height) = self.editor.line_height() else {
            return;
        };
        let window = ui.window_pos();
        let start = ui.cursor_start_pos();
        for &line in &self.assistance.borrow().bookmarks {
            if line >= self.editor.line_count()
                || !self.editor.is_line_visible(line).unwrap_or(false)
            {
                continue;
            }
            if let Ok(visual) = self.editor.document_to_visual(Position::new(line, 0)) {
                ui.get_window_draw_list()
                    .add_circle(
                        [
                            window[0] + 4.0,
                            window[1] + start[1] + (visual.row as f32 + 0.5) * height,
                        ],
                        3.0,
                        self.editor
                            .palette()
                            .get(dear_imgui_cte::PaletteColor::Keyword),
                    )
                    .filled(true)
                    .build();
            }
        }
    }
}

pub(super) fn move_bookmarks(bookmarks: &mut BTreeSet<usize>, change: TextChange<'_>) {
    let start = change.range.start.line;
    let end = change.range.end.line;
    let count = end.saturating_sub(start);
    if count == 0 {
        return;
    }
    *bookmarks = bookmarks
        .iter()
        .map(|&line| match change.kind {
            TextChangeKind::Insert
                if line > start || (line == start && change.range.start.column == 0) =>
            {
                line + count
            }
            TextChangeKind::Delete if line > start => line.saturating_sub(count).max(start),
            _ => line,
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::code::{Settings, State};
    use dear_imgui_cte::Selection;

    #[test]
    fn bookmarks_follow_inserted_and_deleted_lines() {
        let mut bookmarks = BTreeSet::from([0, 2, 5]);
        let range = Selection::new(Position::new(2, 0), Position::new(4, 0));
        move_bookmarks(
            &mut bookmarks,
            TextChange {
                kind: TextChangeKind::Insert,
                range,
                text: "\n\n",
            },
        );
        assert_eq!(bookmarks, BTreeSet::from([0, 4, 7]));
        move_bookmarks(
            &mut bookmarks,
            TextChange {
                kind: TextChangeKind::Delete,
                range,
                text: "\n\n",
            },
        );
        assert_eq!(bookmarks, BTreeSet::from([0, 2, 5]));
    }

    #[test]
    fn context_menu_stays_at_the_click_position_when_the_mouse_moves() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([800.0, 600.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let root =
            std::env::temp_dir().join(format!("kinematic-context-menu-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.lua"), "local value = 1").unwrap();
        let mut documents =
            super::super::Documents::new(crate::core::ProjectContext::new(&root).unwrap(), "")
                .unwrap();
        documents.lsp = None;
        let path = std::path::Path::new("main.lua");
        documents.open(path).unwrap();
        let mut state = State::default();
        state.prepare(&context, Some(&documents), &Settings::default());
        let tab = state.tabs.get_mut(path).unwrap();
        let frame = |context: &mut dear_imgui_rs::Context, tab: &mut Tab| {
            let ui = context.frame();
            let mut popup_position = None;
            ui.window("Menu test")
                .position([0.0, 0.0], dear_imgui_rs::Condition::Always)
                .size([780.0, 580.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    tab.context_menu(ui, true).unwrap();
                    if let Some(_popup) = ui.begin_popup("Editor context menu") {
                        popup_position = Some(ui.window_pos());
                    }
                });
            context.end_frame();
            popup_position
        };
        frame(&mut context, tab);
        context.io_mut().add_mouse_pos_event([200.0, 200.0]);
        frame(&mut context, tab);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Right, true);
        assert!(frame(&mut context, tab).is_some());
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Right, false);
        let position = frame(&mut context, tab).unwrap();
        context.io_mut().add_mouse_pos_event([700.0, 500.0]);
        for _ in 0..3 {
            assert_eq!(frame(&mut context, tab), Some(position));
        }
        tab.toggle_bookmark();
        assert!(tab.assistance.borrow().bookmarks.contains(&0));
        tab.toggle_bookmark();
        assert!(tab.assistance.borrow().bookmarks.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
