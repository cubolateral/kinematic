use crate::ui::code::{
    Documents, InputAction, Pending, Position, Settings, State, Tab, TextEditor, inputs,
};
use dear_imgui_cte::{CteResult, ScrollAlignment, Selection};

impl Tab {
    pub(super) fn shortcuts(&mut self, ui: &dear_imgui_rs::Ui) -> CteResult<()> {
        use InputAction::*;

        let text_input = ui.is_any_item_active();

        if !text_input && inputs::map().routed(ui, ChordPrefix) {
            self.chord_started = Some(ui.time());
        }

        if self
            .chord_started
            .is_some_and(|time| ui.time() - time > 3.0)
            || ui.is_key_pressed(dear_imgui_rs::Key::Escape)
        {
            self.chord_started = None;
        }

        if !text_input && self.chord_started.is_some() {
            for action in [CommentLines, UncommentLines, FoldAll, UnfoldAll] {
                if !inputs::map().routed(ui, action) {
                    continue;
                }

                match action {
                    CommentLines | UncommentLines => {
                        comment_lines(&mut self.editor, action == CommentLines)?
                    }
                    FoldAll => {
                        for line in (0..self.editor.line_count()).rev() {
                            self.editor.fold_around_line(line)?;
                        }
                    }
                    UnfoldAll => self.editor.unfold_all(),
                    _ => unreachable!(),
                }

                self.chord_started = None;
            }
        }

        // Register overrides before CTE so its built-in bindings cannot run the same key twice.
        for action in [
            SelectLine,
            Cut,
            NextOccurrence,
            AllOccurrences,
            MoveLineUp,
            MoveLineDown,
            DuplicateUp,
            DuplicateDown,
            DeleteWordLeft,
            DeleteWordRight,
            InsertAbove,
            SoftNewline,
            InsertBelow,
            ScrollUp,
            ScrollDown,
            FindNext,
            FindPrevious,
            ToggleWordWrap,
            JumpBracket,
            Fold,
            Unfold,
            Replace,
        ] {
            if text_input && !matches!(action, FindNext | FindPrevious) {
                continue;
            }

            if !inputs::map().routed(ui, action) {
                continue;
            }

            let cursor = self.editor.main_cursor_position();

            match action {
                Cut => cut(&mut self.editor)?,
                SelectLine => {
                    let selection = self.editor.main_cursor_selection();
                    let line =
                        if selection.end.column == 0 && selection.end.line > selection.start.line {
                            selection.end.line
                        } else {
                            selection.start.line
                        };
                    let end = line_end(&self.editor, line)?;

                    self.editor.select_region(Selection::new(
                        Position::new(selection.start.line, 0),
                        end,
                    ))?;
                }
                NextOccurrence | AllOccurrences => {
                    if !self.editor.main_cursor_has_selection() {
                        self.editor
                            .select_region(word_selection(&self.editor, cursor)?)?;

                        if action == NextOccurrence {
                            continue;
                        }
                    }

                    if self.editor.main_cursor_has_selection() {
                        if action == NextOccurrence {
                            self.editor.add_next_occurrence();
                        } else {
                            self.editor.select_all_occurrences();
                        }
                    }
                }
                MoveLineUp | MoveLineDown => move_lines(&mut self.editor, action == MoveLineDown)?,
                DuplicateUp | DuplicateDown => {
                    duplicate_lines(&mut self.editor, action == DuplicateDown)?
                }
                DeleteWordLeft | DeleteWordRight => {
                    delete_word(&mut self.editor, action == DeleteWordLeft)?
                }
                SoftNewline => {
                    let line = self.editor.line_text(cursor.line)?;
                    let indentation: String = line
                        .chars()
                        .take(cursor.column)
                        .take_while(|c| matches!(c, ' ' | '\t'))
                        .collect();

                    self.editor
                        .replace_all_selections(&format!("\n{indentation}"))?;
                }
                InsertAbove | InsertBelow => {
                    let line = self.editor.line_text(cursor.line)?;
                    let indentation: String = line
                        .chars()
                        .take_while(|c| matches!(c, ' ' | '\t'))
                        .collect();
                    let (at, text, target) = if action == InsertAbove {
                        (
                            Position::new(cursor.line, 0),
                            format!("{indentation}\n"),
                            cursor.line,
                        )
                    } else {
                        (
                            Position::new(cursor.line, line.chars().count()),
                            format!("\n{indentation}"),
                            cursor.line + 1,
                        )
                    };

                    self.editor.replace_section(Selection::new(at, at), &text)?;

                    self.editor
                        .set_cursor(Position::new(target, indentation.chars().count()))?;
                }
                ScrollUp | ScrollDown => {
                    let row = self.editor.first_visible_row();
                    let row = if action == ScrollUp {
                        row.saturating_sub(1)
                    } else {
                        row + 1
                    };
                    let position = self
                        .editor
                        .visual_to_document(dear_imgui_cte::VisualPosition::new(row, 0));

                    self.editor
                        .scroll_to_line(position.line, ScrollAlignment::Top)?;
                }
                FindNext => self.editor.find_next(),
                FindPrevious => find_previous(&mut self.editor)?,
                ToggleWordWrap => self
                    .editor
                    .set_word_wrap_enabled(!self.editor.is_word_wrap_enabled()),
                JumpBracket => jump_bracket(&mut self.editor)?,
                Fold => self.editor.fold_around_line(cursor.line)?,
                Unfold => self.editor.unfold_around_line(cursor.line)?,
                Replace => self.editor.open_find_replace(),
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}

fn cut(editor: &mut TextEditor) -> CteResult<()> {
    if editor.any_cursor_has_selection() {
        editor.cut();

        return Ok(());
    }

    editor.copy();

    let mut lines = (0..editor.cursor_count())
        .map(|i| editor.cursor_position(i).map(|p| p.line))
        .collect::<CteResult<Vec<_>>>()?;

    lines.sort_unstable();
    lines.dedup();

    for line in lines.into_iter().rev() {
        let start = if line + 1 == editor.line_count() && line > 0 {
            Position::new(line - 1, editor.line_text(line - 1)?.chars().count())
        } else {
            Position::new(line, 0)
        };
        let end = line_end(editor, line)?;

        editor.replace_section(Selection::new(start, end), "")?;
    }

    Ok(())
}

fn jump_bracket(editor: &mut TextEditor) -> CteResult<()> {
    let cursor = editor.main_cursor_position();
    let original = editor.main_cursor_selection();
    let chars: Vec<_> = editor.line_text(cursor.line)?.chars().collect();
    let mut probe = cursor;

    if chars
        .get(cursor.column)
        .is_some_and(|c| matches!(c, '(' | '[' | '{'))
    {
        probe.column += 1;
    } else if cursor.column > 0
        && !chars
            .get(cursor.column)
            .is_some_and(|c| matches!(c, ')' | ']' | '}'))
        && matches!(chars[cursor.column - 1], ')' | ']' | '}')
    {
        probe.column -= 1;
    }

    editor.select_region(Selection::new(probe, probe))?;
    editor.select_to_brackets(true);

    let selection = editor.main_cursor_selection();
    let text = editor.section_text(selection)?;

    if !matches!(
        (text.chars().next(), text.chars().last()),
        (Some('('), Some(')')) | (Some('['), Some(']')) | (Some('{'), Some('}'))
    ) {
        editor.select_region(original)?;

        return Ok(());
    }

    let target = if cursor == selection.start {
        Position::new(selection.end.line, selection.end.column - 1)
    } else {
        selection.start
    };

    editor.set_cursor(target)
}

fn word_selection(editor: &TextEditor, cursor: Position) -> CteResult<Selection> {
    let chars: Vec<_> = editor.line_text(cursor.line)?.chars().collect();
    let mut start = cursor.column;
    let mut end = cursor.column;
    let word = |c: char| c.is_alphanumeric() || c == '_';

    if chars.get(start).is_some_and(|&c| word(c)) {
        while start > 0 && word(chars[start - 1]) {
            start -= 1;
        }

        while end < chars.len() && word(chars[end]) {
            end += 1;
        }
    } else if start > 0 && word(chars[start - 1]) {
        while start > 0 && word(chars[start - 1]) {
            start -= 1;
        }
    } else if end < chars.len() {
        end += 1;
    }

    Ok(Selection::new(
        Position::new(cursor.line, start),
        Position::new(cursor.line, end),
    ))
}

fn comment_lines(editor: &mut TextEditor, add: bool) -> CteResult<()> {
    let selections = (0..editor.cursor_count())
        .map(|i| editor.cursor_selection(i))
        .collect::<CteResult<Vec<_>>>()?;
    let mut line = 0;

    editor.filter_lines(|text| {
        let selected = selections.iter().any(|s| {
            let last = if s.end.column == 0 && s.end.line > s.start.line {
                s.end.line - 1
            } else {
                s.end.line
            };

            (s.start.line..=last).contains(&line)
        });

        line += 1;

        if !selected {
            return text.to_owned();
        }

        let trimmed = text.trim_start_matches([' ', '\t']);
        let indentation = text.len() - trimmed.len();

        if add && !trimmed.starts_with("--") && !trimmed.is_empty() {
            format!("{}-- {}", &text[..indentation], trimmed)
        } else if !add && trimmed.starts_with("--") {
            format!(
                "{}{}",
                &text[..indentation],
                trimmed[2..].strip_prefix(' ').unwrap_or(&trimmed[2..])
            )
        } else {
            text.to_owned()
        }
    })
}

fn line_end(editor: &TextEditor, line: usize) -> CteResult<Position> {
    Ok(if line + 1 < editor.line_count() {
        Position::new(line + 1, 0)
    } else {
        Position::new(line, editor.line_text(line)?.chars().count())
    })
}

fn move_lines(editor: &mut TextEditor, down: bool) -> CteResult<()> {
    if editor.cursor_count() > 1 {
        // shortcut: Multiple cursors use CTE until its cursor API supports restoring selections.
        if down {
            editor.move_lines_down();
        } else {
            editor.move_lines_up();
        }
        return Ok(());
    }

    let selection = editor.main_cursor_selection();
    let first = selection.start.line;
    let last = if selection.end.column == 0 && selection.end.line > first {
        selection.end.line - 1
    } else {
        selection.end.line
    };

    if (!down && first == 0) || (down && last + 1 == editor.line_count()) {
        return Ok(());
    }

    let start = if down { first } else { first - 1 };
    let end = if down { last + 1 } else { last };
    let mut lines = (start..=end)
        .map(|line| editor.line_text(line))
        .collect::<CteResult<Vec<_>>>()?;
    let range = Selection::new(
        Position::new(start, 0),
        Position::new(end, lines.last().unwrap().chars().count()),
    );

    if down {
        lines.rotate_right(1);
    } else {
        lines.rotate_left(1);
    }

    editor.replace_section(range, &lines.join("\n"))?;

    let shifted = |position: Position| {
        Position::new(
            if down {
                position.line + 1
            } else {
                position.line - 1
            },
            position.column,
        )
    };
    let last_line = editor.line_count() - 1;
    let end = Position::new(last_line, editor.line_text(last_line)?.chars().count());
    editor.select_region(Selection::new(
        shifted(selection.start).min(end),
        shifted(selection.end).min(end),
    ))
}

fn duplicate_lines(editor: &mut TextEditor, down: bool) -> CteResult<()> {
    let selection = editor.main_cursor_selection();
    let first = selection.start.line;
    let last = if selection.end.column == 0 && selection.end.line > first {
        selection.end.line - 1
    } else {
        selection.end.line
    };
    let text = (first..=last)
        .map(|line| editor.line_text(line))
        .collect::<CteResult<Vec<_>>>()?
        .join("\n");
    let at = Position::new(last, editor.line_text(last)?.chars().count());

    editor.replace_section(Selection::new(at, at), &format!("\n{text}"))?;

    let offset = if down { last - first + 1 } else { 0 };

    if selection.start == selection.end {
        editor.set_cursor(Position::new(first + offset, selection.start.column))?;
    } else {
        editor.select_region(Selection::new(
            Position::new(selection.start.line + offset, selection.start.column),
            Position::new(selection.end.line + offset, selection.end.column),
        ))?;
    }

    Ok(())
}

fn delete_word(editor: &mut TextEditor, left: bool) -> CteResult<()> {
    // Work backwards so editing one cursor does not shift the preceding selections.
    let mut selections = (0..editor.cursor_count())
        .map(|i| editor.cursor_selection(i))
        .collect::<CteResult<Vec<_>>>()?;

    selections.sort_by_key(|s| (s.start.line, s.start.column));

    for selection in selections.into_iter().rev() {
        let mut start = selection.start;
        let mut end = selection.end;

        if start == end {
            let line = editor.line_text(start.line)?;
            let chars: Vec<_> = line.chars().collect();

            if left {
                if start.column == 0 && start.line > 0 {
                    start = Position::new(
                        start.line - 1,
                        editor.line_text(start.line - 1)?.chars().count(),
                    );
                } else {
                    while start.column > 0 && chars[start.column - 1].is_whitespace() {
                        start.column -= 1;
                    }

                    if start.column > 0 {
                        let word = chars[start.column - 1].is_alphanumeric()
                            || chars[start.column - 1] == '_';

                        while start.column > 0
                            && !chars[start.column - 1].is_whitespace()
                            && (chars[start.column - 1].is_alphanumeric()
                                || chars[start.column - 1] == '_')
                                == word
                        {
                            start.column -= 1;
                        }
                    }
                }
            } else if end.column == chars.len() && end.line + 1 < editor.line_count() {
                end = Position::new(end.line + 1, 0);
            } else {
                while end.column < chars.len() && chars[end.column].is_whitespace() {
                    end.column += 1;
                }

                if end.column < chars.len() {
                    let word = chars[end.column].is_alphanumeric() || chars[end.column] == '_';

                    while end.column < chars.len()
                        && !chars[end.column].is_whitespace()
                        && (chars[end.column].is_alphanumeric() || chars[end.column] == '_') == word
                    {
                        end.column += 1;
                    }
                }
            }
        }

        editor.replace_section(Selection::new(start, end), "")?;
    }

    Ok(())
}

fn find_previous(editor: &mut TextEditor) -> CteResult<()> {
    if !editor.has_find_string() {
        return Ok(());
    }

    let original = editor.main_cursor_selection();

    editor.find_next();

    let first = editor.main_cursor_selection();

    if first.start == first.end {
        return Ok(());
    }

    let mut previous = None;
    let mut last = first;

    loop {
        let found = editor.main_cursor_selection();

        if (found.start.line, found.start.column) < (original.start.line, original.start.column) {
            if previous.is_none_or(|p: Selection| {
                (found.start.line, found.start.column) > (p.start.line, p.start.column)
            }) {
                previous = Some(found);
            }
        }

        last = if (found.start.line, found.start.column) > (last.start.line, last.start.column) {
            found
        } else {
            last
        };

        editor.find_next();

        if editor.main_cursor_selection() == first {
            break;
        }
    }

    editor.select_region(previous.unwrap_or(last))?;

    Ok(())
}

impl State {
    pub(super) fn document_shortcuts(
        &mut self,
        documents: &mut Documents,
        settings: &Settings,
        ui: &dear_imgui_rs::Ui,
    ) {
        if !self.focused || ui.is_any_item_active() || self.navigation.is_some() {
            return;
        }

        for action in [
            InputAction::GoToLine,
            InputAction::QuickOpen,
            InputAction::CloseTab,
            InputAction::NextTab,
            InputAction::PreviousTab,
            InputAction::SaveAll,
            InputAction::FormatDocument,
        ] {
            if !inputs::map().routed(ui, action) {
                continue;
            }

            match action {
                InputAction::GoToLine | InputAction::QuickOpen => {
                    self.navigation = Some((action == InputAction::QuickOpen, String::new()));
                    self.navigation_opening = true;
                    self.navigation_selected = 0;
                }
                InputAction::CloseTab => {
                    if let Some(path) = &documents.active {
                        self.pending = Some(Pending::Document(path.clone()));
                    }
                }
                InputAction::NextTab | InputAction::PreviousTab => {
                    let paths: Vec<_> = documents.open.keys().cloned().collect();

                    if let Some(index) = paths
                        .iter()
                        .position(|p| Some(p) == documents.active.as_ref())
                    {
                        let offset = if action == InputAction::NextTab {
                            1
                        } else {
                            paths.len() - 1
                        };

                        documents.active = Some(paths[(index + offset) % paths.len()].clone());
                        self.focus_requested = true;
                    }
                }
                InputAction::SaveAll => {
                    for (path, tab) in &mut self.tabs {
                        if !documents.open[path].dirty() {
                            continue;
                        }

                        if let Err(error) = tab.save(documents, path) {
                            self.error = Some(error);
                        }
                    }
                }
                InputAction::FormatDocument => {
                    if let Some(path) = documents.active.clone() {
                        let result = (|| {
                            let document = &documents.open[&path];
                            let lsp = documents.lsp.as_mut().ok_or("LuaLS is unavailable.")?;
                            let text = lsp.format(&path, &document.text, document.version,
                                &serde_json::json!({"tabSize": settings.tab_size.clamp(1, 8), "insertSpaces": settings.insert_spaces}), &mut documents.replies)?;

                            if let Some(tab) =
                                self.tabs.get_mut(&path).filter(|_| text != document.text)
                            {
                                let cursor = tab.editor.main_cursor_position();
                                let last = tab.editor.line_count().saturating_sub(1);
                                let end = Position::new(
                                    last,
                                    tab.editor
                                        .line_text(last)
                                        .map_err(|e| e.to_string())?
                                        .chars()
                                        .count(),
                                );

                                tab.editor
                                    .replace_section(
                                        Selection::new(Position::new(0, 0), end),
                                        &text,
                                    )
                                    .map_err(|e| e.to_string())?;

                                let line =
                                    cursor.line.min(tab.editor.line_count().saturating_sub(1));
                                let column = cursor.column.min(
                                    tab.editor
                                        .line_text(line)
                                        .map_err(|e| e.to_string())?
                                        .chars()
                                        .count(),
                                );

                                tab.editor
                                    .set_cursor(Position::new(line, column))
                                    .map_err(|e| e.to_string())?;
                            }

                            documents.edit(&path, text);

                            if let Some(tab) = self.tabs.get_mut(&path) {
                                tab.version = documents.open[&path].version;
                            }

                            Ok::<_, String>(())
                        })();

                        if let Err(error) = result {
                            self.error = Some(error);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub(super) fn navigation_popup(&mut self, documents: &mut Documents, ui: &dear_imgui_rs::Ui) {
        let Some((files, query)) = &mut self.navigation else {
            return;
        };

        if self.navigation_opening {
            ui.open_popup("Navigate code");
        }

        let mut close = false;
        let mut open_path = None;
        let mut position = None;

        if let Some(_popup) = ui.begin_popup("Navigate code") {
            ui.text(if *files {
                "Go to file"
            } else {
                "Go to line:column"
            });

            if self.navigation_opening {
                ui.set_keyboard_focus_here();

                self.navigation_opening = false;
            }

            let enter = ui
                .input_text("##Navigation", query)
                .enter_returns_true(true)
                .build();

            if *files {
                match documents.files() {
                    Ok(entries) => {
                        let query = query.to_lowercase();
                        let entries: Vec<_> = entries
                            .iter()
                            .filter(|e| {
                                !e.directory
                                    && e.path.to_string_lossy().to_lowercase().contains(&query)
                            })
                            .collect();
                        let moved = ui.is_key_pressed(dear_imgui_rs::Key::DownArrow)
                            || ui.is_key_pressed(dear_imgui_rs::Key::UpArrow);

                        if ui.is_key_pressed(dear_imgui_rs::Key::DownArrow) {
                            self.navigation_selected = self.navigation_selected.saturating_add(1);
                        }

                        if ui.is_key_pressed(dear_imgui_rs::Key::UpArrow) {
                            self.navigation_selected = self.navigation_selected.saturating_sub(1);
                        }

                        self.navigation_selected = self
                            .navigation_selected
                            .min(entries.len().saturating_sub(1));

                        for (index, entry) in entries.iter().enumerate() {
                            let selected = index == self.navigation_selected;

                            if (enter && selected)
                                || ui
                                    .selectable_config(entry.path.to_string_lossy())
                                    .selected(selected)
                                    .build()
                            {
                                open_path = Some(entry.path.clone());
                                close = true;
                            }

                            if moved && selected {
                                ui.set_scroll_here_y(0.5);
                            }
                        }
                    }

                    Err(error) => ui.text_wrapped(error),
                }
            } else if enter {
                let mut parts = query.split(':');

                if let Some(line) = parts.next().and_then(|p| p.parse::<usize>().ok()) {
                    let column = parts
                        .next()
                        .and_then(|p| p.parse::<usize>().ok())
                        .unwrap_or(1);

                    position = Some(Position::new(
                        line.saturating_sub(1),
                        column.saturating_sub(1),
                    ));

                    close = true;
                }
            }

            close |= ui.is_key_pressed(dear_imgui_rs::Key::Escape);

            if close {
                ui.close_current_popup();
            }
        } else {
            close = true;
        }

        if let Some(path) = open_path {
            if let Err(error) = documents.open(&path) {
                self.error = Some(error);
            }
        }

        if let Some(position) = position {
            if let Some(tab) = documents.active.as_ref().and_then(|p| self.tabs.get_mut(p)) {
                let line = position.line.min(tab.editor.line_count().saturating_sub(1));
                let column = position.column.min(
                    tab.editor
                        .line_text(line)
                        .unwrap_or_default()
                        .chars()
                        .count(),
                );

                if let Err(error) = tab.editor.set_cursor(Position::new(line, column)) {
                    self.error = Some(error.to_string());
                }
            }
        }

        if close {
            self.navigation = None;
            self.focus_requested = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::code::{Documents, Position, Settings, State};
    use dear_imgui_cte::Selection;
    use dear_imgui_rs::Key;

    #[test]
    fn moving_lines_preserves_zero_columns_blank_lines_selections_and_undo() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let context = dear_imgui_rs::Context::create();
        let mut editor = dear_imgui_cte::TextEditor::create(&context);

        for (text, start, end, down, expected, target) in [
            ("alpha\nbeta", (0, 0), (0, 0), true, "beta\nalpha", (1, 0)),
            ("alpha\nbeta", (1, 0), (1, 0), false, "beta\nalpha", (0, 0)),
            ("alpha\nbeta", (1, 2), (1, 2), false, "beta\nalpha", (0, 2)),
            (
                "alpha\n\nbeta",
                (1, 0),
                (1, 0),
                true,
                "alpha\nbeta\n",
                (2, 0),
            ),
            ("alpha\nbeta", (0, 0), (1, 0), true, "beta\nalpha", (1, 0)),
            ("a\nb\nc", (0, 0), (1, 1), true, "c\na\nb", (1, 0)),
            ("a\nb\nc", (1, 0), (2, 1), false, "b\nc\na", (0, 0)),
            ("ação\nbeta\n", (0, 4), (0, 4), true, "beta\nação\n", (1, 4)),
        ] {
            editor.set_text(text).unwrap();
            editor
                .select_region(Selection::new(
                    Position::new(start.0, start.1),
                    Position::new(end.0, end.1),
                ))
                .unwrap();
            super::move_lines(&mut editor, down).unwrap();
            assert_eq!(editor.text().unwrap(), expected);
            assert_eq!(
                editor.main_cursor_selection().start,
                Position::new(target.0, target.1)
            );
            editor.undo();
            assert_eq!(editor.text().unwrap(), text);
            editor.redo();
            assert_eq!(editor.text().unwrap(), expected);
        }
        for (line, down) in [(0, false), (1, true)] {
            editor.set_text("alpha\nbeta").unwrap();
            editor.set_cursor(Position::new(line, 0)).unwrap();
            super::move_lines(&mut editor, down).unwrap();
            assert_eq!(editor.text().unwrap(), "alpha\nbeta");
            assert!(!editor.can_undo());
        }
    }

    #[test]
    fn vscode_overrides_edit_once_and_preserve_undo_and_focus() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
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

        let root = std::env::temp_dir().join(format!("kinematic-shortcuts-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("a.lua"), "local café = 1\nprint(café)\n").unwrap();

        let mut documents =
            Documents::new(crate::core::ProjectContext::new(&root).unwrap(), "").unwrap();

        documents.lsp = None;

        let path = std::path::Path::new("a.lua");

        documents.open(path).unwrap();

        let settings = Settings {
            autocomplete: false,
            ..Settings::default()
        };
        let mut state = State {
            focus_requested: true,
            ..State::default()
        };
        let frame =
            |context: &mut dear_imgui_rs::Context, state: &mut State, documents: &mut Documents| {
                state.prepare(context, Some(documents), &settings);
                state.draw(documents, &settings, context.frame());
                context.end_frame();
            };

        for _ in 0..5 {
            frame(&mut context, &mut state, &mut documents);
        }

        let press = |context: &mut dear_imgui_rs::Context,
                     state: &mut State,
                     documents: &mut Documents,
                     key: Key,
                     shift: bool,
                     alt: bool| {
            for (key, held) in [
                (Key::ModCtrl, true),
                (Key::ModShift, shift),
                (Key::ModAlt, alt),
                (key, true),
            ] {
                context.io_mut().add_key_event(key, held);
            }

            frame(context, state, documents);

            for key in [key, Key::ModCtrl, Key::ModShift, Key::ModAlt] {
                context.io_mut().add_key_event(key, false);
            }

            frame(context, state, documents);
        };

        for (key, shift, alt, text, position, expected) in [
            (Key::X, false, false, "alpha\nbeta", (0, 2), "beta"),
            (Key::X, false, false, "alpha\nbeta", (1, 2), "alpha"),
            (Key::X, false, false, "alpha", (0, 2), ""),
            (
                Key::KeypadEnter,
                true,
                false,
                "alpha\nbeta",
                (0, 2),
                "\nalpha\nbeta",
            ),
            (
                Key::KeypadEnter,
                false,
                false,
                "alpha\nbeta",
                (0, 2),
                "alpha\n\nbeta",
            ),
            (Key::Delete, false, false, "alpha beta", (0, 0), " beta"),
            (Key::K, true, false, "alpha\nbeta", (0, 2), "beta"),
            (
                Key::RightBracket,
                false,
                false,
                "alpha\nbeta",
                (0, 0),
                "    alpha\nbeta",
            ),
            (
                Key::LeftBracket,
                false,
                false,
                "    alpha\nbeta",
                (0, 4),
                "alpha\nbeta",
            ),
            (
                Key::UpArrow,
                true,
                true,
                "alpha\nbeta",
                (0, 2),
                "alpha\nalpha\nbeta",
            ),
        ] {
            documents.edit(path, text.to_owned());

            let tab = state.tabs.get_mut(path).unwrap();

            tab.editor.set_text(text).unwrap();

            tab.version = documents.open[path].version;

            tab.editor
                .set_cursor(Position::new(position.0, position.1))
                .unwrap();

            for _ in 0..3 {
                frame(&mut context, &mut state, &mut documents);
            }

            press(&mut context, &mut state, &mut documents, key, shift, alt);
            assert_eq!(documents.open[path].text, expected, "{key:?}");
            state.tabs.get_mut(path).unwrap().editor.undo();

            frame(&mut context, &mut state, &mut documents);
            assert_eq!(documents.open[path].text, text, "Undo {key:?}");
        }

        for (column, target) in [(4, 6), (6, 4), (5, 4), (7, 4), (0, 0)] {
            documents.edit(path, "call(a)".to_owned());

            let tab = state.tabs.get_mut(path).unwrap();

            tab.editor.set_text("call(a)").unwrap();

            tab.version = documents.open[path].version;

            tab.editor.set_cursor(Position::new(0, column)).unwrap();

            for _ in 0..3 {
                frame(&mut context, &mut state, &mut documents);
            }

            press(
                &mut context,
                &mut state,
                &mut documents,
                Key::Backslash,
                true,
                false,
            );

            assert_eq!(
                state.tabs[path].editor.main_cursor_position(),
                Position::new(0, target)
            );
        }

        documents.edit(path, "local café = 1\nprint(café)\n".to_owned());

        let tab = state.tabs.get_mut(path).unwrap();

        tab.editor.set_text(&documents.open[path].text).unwrap();

        tab.version = documents.open[path].version;

        tab.editor.set_cursor(Position::new(0, 0)).unwrap();

        for _ in 0..3 {
            frame(&mut context, &mut state, &mut documents);
        }

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::Slash,
            false,
            false,
        );

        assert_eq!(
            documents.open[path].text,
            "-- local café = 1\nprint(café)\n"
        );

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::Slash,
            false,
            false,
        );

        assert_eq!(documents.open[path].text, "local café = 1\nprint(café)\n");

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::L,
            false,
            false,
        );

        assert_eq!(
            state.tabs[path].editor.main_cursor_selection(),
            Selection::new(Position::new(0, 0), Position::new(1, 0))
        );

        assert_eq!(documents.open[path].text, "local café = 1\nprint(café)\n");

        state
            .tabs
            .get_mut(path)
            .unwrap()
            .editor
            .set_cursor(Position::new(0, 10))
            .unwrap();

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::Backspace,
            false,
            false,
        );

        assert_eq!(documents.open[path].text, "local  = 1\nprint(café)\n");
        state.tabs.get_mut(path).unwrap().editor.undo();

        frame(&mut context, &mut state, &mut documents);
        assert_eq!(documents.open[path].text, "local café = 1\nprint(café)\n");

        state
            .tabs
            .get_mut(path)
            .unwrap()
            .editor
            .set_cursor(Position::new(0, 0))
            .unwrap();

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::DownArrow,
            true,
            true,
        );

        assert_eq!(
            documents.open[path].text,
            "local café = 1\nlocal café = 1\nprint(café)\n"
        );

        assert_eq!(
            state.tabs[path].editor.main_cursor_position(),
            Position::new(1, 0)
        );

        state
            .tabs
            .get_mut(path)
            .unwrap()
            .editor
            .set_cursor(Position::new(0, 6))
            .unwrap();

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::D,
            false,
            false,
        );

        assert_eq!(
            state.tabs[path]
                .editor
                .section_text(state.tabs[path].editor.main_cursor_selection())
                .unwrap(),
            "café"
        );

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::L,
            true,
            false,
        );

        assert_eq!(state.tabs[path].editor.cursor_count(), 3);

        state
            .tabs
            .get_mut(path)
            .unwrap()
            .editor
            .set_cursor(Position::new(0, 0))
            .unwrap();

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::K,
            false,
            false,
        );

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::C,
            false,
            false,
        );

        assert!(documents.open[path].text.starts_with("-- local café"));

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::K,
            false,
            false,
        );

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::U,
            false,
            false,
        );

        assert!(documents.open[path].text.starts_with("local café"));

        press(
            &mut context,
            &mut state,
            &mut documents,
            Key::G,
            false,
            false,
        );

        assert!(state.navigation.is_some());

        for character in "3:4".chars() {
            context.io_mut().add_input_character(character);
        }

        frame(&mut context, &mut state, &mut documents);
        context.io_mut().add_key_event(Key::Enter, true);

        frame(&mut context, &mut state, &mut documents);
        context.io_mut().add_key_event(Key::Enter, false);

        frame(&mut context, &mut state, &mut documents);
        assert!(state.navigation.is_none());

        assert_eq!(
            state.tabs[path].editor.main_cursor_position(),
            Position::new(2, 3)
        );

        assert!(state.error.is_none(), "{:?}", state.error);

        drop(documents);

        std::fs::remove_dir_all(root).unwrap();
    }
}
