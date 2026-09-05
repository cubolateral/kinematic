mod fullscreen;
mod image;
mod state;

use crate::editor::Editor;

use super::widgets::hide_single_window_tab;

pub(super) const WINDOW_NAME: &str = "Preview";

pub(super) use state::State;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    let is_exporting = editor.is_exporting();
    let mut clicked = None;
    let (name, resolution, fps) = {
        let project = editor.get_project();
        (project.name, project.resolution, project.fps)
    };
    let preview = image::preview(editor);

    hide_single_window_tab(ui);

    ui.window(WINDOW_NAME).build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        ui.text(format!(
            "[PROJECT INFO] Name: {name} / Resolution: {}x{} / FPS: {fps}",
            resolution.0, resolution.1,
        ));
        ui.same_line();
        let plain_keyboard_input = !ui.io().want_text_input()
            && !ui.io().key_ctrl()
            && !ui.io().key_shift()
            && !ui.io().key_alt()
            && !ui.io().key_super();
        let reset_shortcut = plain_keyboard_input
            && (ui.is_key_pressed(dear_imgui_rs::Key::Key0)
                || ui.is_key_pressed(dear_imgui_rs::Key::Keypad0));
        if ui.button("Reset View") || reset_shortcut {
            state.reset();
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Reset zoom and center the preview [0]");
        }
        ui.same_line();
        ui.text(format!("Zoom: {:.0}%", state.zoom() * 100.0));
        ui.separator();

        clicked = image::draw_interactive(ui, preview, ui.content_region_avail(), state, editor);
    });

    if let Some(point) = clicked {
        editor.select_at(point);
    }
}

pub(super) fn draw_fullscreen(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    fullscreen::draw(editor, ui)
}
