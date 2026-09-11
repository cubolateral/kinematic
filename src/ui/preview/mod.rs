mod fullscreen;
mod image;
mod state;

use crate::editor::Editor;

use super::{controls, icons, widgets::hide_single_window_tab};

pub(super) const WINDOW_NAME: &str = "Preview";

pub(super) use state::State;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    let is_exporting = editor.is_exporting();
    let mut clicked = None;
    let preview = image::preview(editor);

    hide_single_window_tab(ui);

    ui.window(WINDOW_NAME).build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let plain_keyboard_input = !ui.io().want_text_input()
            && !ui.is_any_item_active()
            && !ui.io().key_ctrl()
            && !ui.io().key_shift()
            && !ui.io().key_alt()
            && !ui.io().key_super();
        let reset_shortcut = plain_keyboard_input
            && (ui.is_key_pressed(dear_imgui_rs::Key::Key0)
                || ui.is_key_pressed(dear_imgui_rs::Key::Keypad0));
        if controls::text_button(ui, icons::RESET, [ui.frame_height(); 2]) || reset_shortcut {
            state.reset();
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Reset zoom and center the preview [0]");
        }
        ui.same_line();
        let mouse_position_cursor = ui.cursor_screen_pos();
        ui.new_line();
        ui.separator();
        if let Some(error) = editor.get_render_error() {
            ui.text_wrapped(error);
        }

        let interaction =
            image::draw_interactive(ui, preview, ui.content_region_avail(), state, editor);
        clicked = interaction.clicked;
        if let Some(position) = interaction.mouse_position {
            ui.set_cursor_screen_pos(mouse_position_cursor);
            ui.text(format!("Mouse: ({:.2}, {:.2})", position.x, position.y));
        }
    });

    if let Some(point) = clicked {
        editor.select_at(point);
    }
}

pub(super) fn draw_fullscreen(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    fullscreen::draw(editor, ui)
}
