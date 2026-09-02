use crate::editor::Editor;

use super::widgets::{hide_single_window_tab, text_size};

pub(super) const WINDOW_NAME: &str = "Export";

pub(super) struct State {
    silent: bool,
}

impl Default for State {
    fn default() -> Self {
        Self { silent: true }
    }
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    hide_single_window_tab(ui);

    ui.window(WINDOW_NAME).build(|| {
        let is_exporting = editor.is_exporting();
        let label = if is_exporting { "Cancel" } else { "Export" };

        let available_width = ui.content_region_avail_width();
        let item_spacing = unsafe { ui.style().item_spacing() }[0];
        let item_inner_spacing = unsafe { ui.style().item_inner_spacing() }[0];
        let silent_label_width = text_size(ui, "Silent")[0];
        let checkbox_width = ui.frame_height() + item_inner_spacing + silent_label_width;
        let button_width = (available_width - checkbox_width - item_spacing).max(0.0);

        if ui.button_with_size(label, [button_width, 0.0]) {
            editor.toggle_export(state.silent);
        }

        ui.same_line();

        let _ = ui.begin_disabled_with_cond(is_exporting);
        ui.checkbox("Silent", &mut state.silent);

        ui.spacing();

        if is_exporting {
            let progress = editor.get_export_progress();
            let percentage = format!("{:.0}%", progress * 100.0);

            ui.progress_bar_with_overlay(progress, &percentage)
                .size([ui.content_region_avail_width(), 0.0])
                .build();
        }

        if let Some(message) = editor.get_export_message() {
            ui.text_wrapped(message);
        }
    });
}
