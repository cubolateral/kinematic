use dear_imgui_cte::{Position, Selection, TextEditor, VisualPosition};
use dear_imgui_rs::{StyleColor, Ui};
use std::ops::Range;

pub const SELECTION_COLOR: u32 = 0xff784f26;

pub struct Layout {
    origin: [f32; 2],
    min: [f32; 2],
    max: [f32; 2],
    width: f32,
    height: f32,
    padding: f32,
}

impl Layout {
    pub fn capture(editor: &TextEditor, ui: &Ui) -> Option<Self> {
        let width = editor.glyph_width()?;
        let height = editor.line_height()?;
        let gutter = if editor.shows_line_numbers() {
            editor.line_number_left_margin() + (editor.line_count() + 1).ilog10() as usize + 1
        } else {
            0
        } + usize::from(editor.is_line_folding_enabled())
            + editor.text_left_margin();
        let window = ui.window_pos();
        let start = ui.cursor_start_pos();
        let draw = ui.get_window_draw_list();
        let mut min = draw.clip_rect_min();
        min[0] = min[0].max(window[0] + gutter as f32 * width);
        Some(Self {
            origin: [
                window[0] + start[0] + gutter as f32 * width,
                window[1] + start[1],
            ],
            min,
            max: draw.clip_rect_max(),
            width,
            height,
            padding: (height - ui.text_line_height()).max(0.0) * 0.5,
        })
    }

    pub fn contains(&self, mouse: [f32; 2]) -> bool {
        (0..2).all(|axis| mouse[axis] >= self.min[axis] && mouse[axis] < self.max[axis])
    }
}

fn is_word(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

fn word(line: &str, column: usize, trailing: bool) -> Option<Range<usize>> {
    let characters: Vec<_> = line.chars().collect();
    let column = if trailing && !characters.get(column).copied().is_some_and(is_word) && column > 0
    {
        column - 1
    } else {
        column
    };

    if !characters.get(column).copied().is_some_and(is_word) {
        return None;
    }

    let mut start = column;
    let mut end = column + 1;
    while start > 0 && is_word(characters[start - 1]) {
        start -= 1;
    }
    while end < characters.len() && is_word(characters[end]) {
        end += 1;
    }
    Some(start..end)
}

pub fn word_at_mouse(editor: &TextEditor, mouse: [f32; 2]) -> Option<Selection> {
    // CTE's text-area check rejects scrolled document coordinates.
    if !editor.is_mouse_over_glyph(mouse).ok()? {
        return None;
    }
    let position = editor.position_at_mouse(mouse).ok()?;
    let line = editor.line_text(position.line).ok()?;
    let range = word(&line, position.column, false)?;
    Some(Selection::new(
        Position::new(position.line, range.start),
        Position::new(position.line, range.end),
    ))
}

pub fn draw(editor: &TextEditor, ui: &Ui, layout: &Layout, rulers: &str) {
    let Layout {
        origin,
        min,
        max,
        width,
        height,
        padding,
    } = *layout;
    let draw = ui.get_window_draw_list();
    let _clip = draw.push_clip_rect(min, max, true);

    let cursor = editor.main_cursor_position();
    let current = editor.line_text(cursor.line).ok().and_then(|line| {
        let range = word(&line, cursor.column, true)?;
        let selection = editor.main_cursor_selection();
        if !selection.is_empty()
            && selection
                != Selection::new(
                    Position::new(cursor.line, range.start),
                    Position::new(cursor.line, range.end),
                )
        {
            return None;
        }
        Some(
            line.chars()
                .skip(range.start)
                .take(range.len())
                .collect::<String>(),
        )
    });
    let selections: Vec<_> = (0..editor.cursor_count())
        .filter_map(|index| editor.cursor_selection(index).ok())
        .filter(|selection| !selection.is_empty())
        .filter_map(|selection| {
            Some((
                editor.document_to_visual(selection.start).ok()?,
                editor.document_to_visual(selection.end).ok()?,
            ))
        })
        .collect();

    let mut previous_line = None;
    for row in editor.first_visible_row()..=editor.last_visible_row() {
        let position = editor.visual_to_document(VisualPosition::new(row, 0));
        let Ok(line) = editor.line_text(position.line) else {
            continue;
        };
        let end = Position::new(position.line, line.chars().count());
        let Ok(line_end) = editor.document_to_visual(end) else {
            continue;
        };
        let indent = editor
            .document_to_visual(position)
            .map_or(0, |visual| visual.column);
        for &(start, end) in &selections {
            if row < start.row || row > end.row {
                continue;
            }
            let left = if row == start.row {
                start.column
            } else {
                indent
            };
            let right = if row == end.row {
                end.column
            } else if row == line_end.row {
                line_end.column + 1
            } else {
                let section_end = editor.visual_to_document(VisualPosition::new(row, usize::MAX));
                let column = section_end.column.saturating_sub(1);
                let last = editor
                    .document_to_visual(Position::new(position.line, column))
                    .unwrap_or_default();
                last.column
                    + if line.chars().nth(column) == Some('\t') {
                        editor.tab_size() - last.column % editor.tab_size()
                    } else {
                        1
                    }
            };
            if right > left {
                let point = [
                    origin[0] + left as f32 * width,
                    origin[1] + row as f32 * height - padding,
                ];
                draw.add_rect(
                    point,
                    [origin[0] + right as f32 * width, point[1] + height],
                    SELECTION_COLOR,
                )
                .filled(true)
                .build();
            }
        }
        if previous_line == Some(position.line) {
            continue;
        }
        previous_line = Some(position.line);

        if let Some(current) = &current {
            let mut column = 0;
            let mut previous_byte = 0;
            for (byte, matched) in line.match_indices(current) {
                column += line[previous_byte..byte].chars().count();
                previous_byte = byte;
                if line[..byte].chars().next_back().is_some_and(is_word)
                    || line[byte + matched.len()..]
                        .chars()
                        .next()
                        .is_some_and(is_word)
                {
                    continue;
                }
                for column in column..column + matched.chars().count() {
                    if let Ok(visual) =
                        editor.document_to_visual(Position::new(position.line, column))
                    {
                        let point = [
                            origin[0] + visual.column as f32 * width,
                            origin[1] + visual.row as f32 * height - padding,
                        ];
                        draw.add_rect(point, [point[0] + width, point[1] + height], 0x20ff_ffff)
                            .filled(true)
                            .build();
                    }
                }
            }
        }
    }

    let color = ui.get_color_u32(StyleColor::Header);
    for column in ruler_columns(rulers) {
        let x = origin[0] + column as f32 * width;
        draw.add_line([x, min[1]], [x, max[1]], color).build();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dear_imgui_cte::CteUiExt;

    #[test]
    fn words_use_glyph_columns_and_respect_mouse_boundaries() {
        assert_eq!(word("left right()", 5, false), Some(5..10));
        assert_eq!(word("left.right()", 5, false), Some(5..10));
        assert_eq!(word("left.right()", 4, false), None);
        assert_eq!(word("left right()", 4, false), None);
        assert_eq!(word("left right()", 10, true), Some(5..10));
        assert_eq!(word("left right()", 11, true), None);
        assert_eq!(word("é value_1", 2, false), Some(2..9));
        assert_eq!(word("", 0, true), None);
    }

    #[test]
    fn backgrounds_and_hover_follow_native_text_after_scrolling() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let mut editor = TextEditor::create(&context);
        editor.set_show_minimap(false);
        editor.set_line_spacing(1.5).unwrap();
        editor.set_word_wrap_enabled(true);
        let mut palette = editor.palette();
        palette.set(dear_imgui_cte::PaletteColor::Background, 0);
        palette.set(dear_imgui_cte::PaletteColor::Selection, 0);
        palette.set(dear_imgui_cte::PaletteColor::Text, 0xff123456);
        editor.set_palette(&palette).unwrap();
        editor
            .set_text(&format!(
                "{}left.right rightward right\n\nright",
                "local value = 1\n".repeat(40)
            ))
            .unwrap();
        let caret = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = caret.clone();
        editor
            .set_custom_caret_callback(move |_, event| captured.set(Some(event)))
            .unwrap();
        editor.focus();

        for phase in 0..5 {
            if phase == 2 {
                editor.set_cursor(Position::new(40, 5)).unwrap();
                editor
                    .scroll_to_line(40, dear_imgui_cte::ScrollAlignment::Top)
                    .unwrap();
            } else if phase == 4 {
                editor
                    .select_region(Selection::new(Position::new(40, 0), Position::new(42, 5)))
                    .unwrap();
            }
            let selected = phase == 4;
            let mut expected = [0.0; 2];
            let ui = context.frame();
            ui.window("Text test")
                .position([0.0, 0.0], dear_imgui_rs::Condition::Always)
                .size([600.0, 400.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    let origin = ui.cursor_screen_pos();
                    ui.text_editor(&mut editor, "Source")
                        .size([180.0, 300.0])
                        .build()
                        .unwrap();
                    let next = ui.cursor_screen_pos();
                    let mut layout = None;
                    ui.set_cursor_screen_pos(origin);
                    ui.child_window("Source")
                        .size([180.0, 300.0])
                        .build(ui, || {
                            layout = Layout::capture(&editor, ui);
                        });
                    ui.set_cursor_screen_pos(next);
                    let layout = layout.unwrap();
                    draw(&editor, ui, &layout, "");
                    if phase < 3 {
                        return;
                    }
                    let native = caret.get().unwrap();
                    let visual = editor
                        .document_to_visual(editor.main_cursor_position())
                        .unwrap();
                    let native_origin = [
                        native.glyph_position[0] - visual.column as f32 * layout.width,
                        native.glyph_position[1] - visual.row as f32 * layout.height,
                    ];
                    for axis in 0..2 {
                        assert!((native_origin[axis] - layout.origin[axis]).abs() < 0.1);
                    }
                    let word_visual = editor.document_to_visual(Position::new(40, 5)).unwrap();
                    let word_point = [
                        native_origin[0] + word_visual.column as f32 * layout.width + 0.1,
                        native_origin[1]
                            + word_visual.row as f32 * layout.height
                            + ui.text_line_height() * 0.5,
                    ];
                    assert!(layout.contains(word_point));
                    assert_eq!(
                        word_at_mouse(&editor, word_point),
                        Some(Selection::new(Position::new(40, 5), Position::new(40, 10)))
                    );
                    assert!(editor.first_visible_row() > 0);
                    expected = if selected {
                        let blank_visual = editor.document_to_visual(Position::new(41, 0)).unwrap();
                        [
                            native_origin[0],
                            native_origin[1] + blank_visual.row as f32 * layout.height
                                - layout.padding,
                        ]
                    } else {
                        [
                            native_origin[0] + word_visual.column as f32 * layout.width,
                            native_origin[1] + word_visual.row as f32 * layout.height
                                - layout.padding,
                        ]
                    };
                    assert!(layout.padding > 0.0);
                });
            let frame = context.render_legacy();
            if phase < 3 {
                continue;
            }
            let data = frame.draw_data();
            let color = if selected {
                SELECTION_COLOR
            } else {
                0x20ff_ffff
            };
            let background = data
                .draw_lists()
                .enumerate()
                .find_map(|(index, list)| {
                    list.vtx_buffer()
                        .iter()
                        .any(|vertex| {
                            vertex.col == color
                                && (vertex.pos[0] - expected[0]).abs() < 0.1
                                && (vertex.pos[1] - expected[1]).abs() < 0.1
                        })
                        .then_some(index)
                })
                .expect("Background must follow the native text position.");
            let foreground = data
                .draw_lists()
                .enumerate()
                .find_map(|(index, list)| {
                    list.vtx_buffer()
                        .iter()
                        .any(|vertex| vertex.col == 0xff123456)
                        .then_some(index)
                })
                .unwrap();
            assert!(
                background < foreground,
                "Backgrounds must render behind the text."
            );
        }
    }
}

pub fn ruler_columns(rulers: &str) -> impl Iterator<Item = usize> + '_ {
    rulers
        .split(',')
        .filter_map(|value| value.trim().parse::<u16>().ok())
        .filter(|column| *column > 0)
        .map(usize::from)
}
