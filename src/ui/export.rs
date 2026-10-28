use crate::editor::Editor;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    let Some(_menu) = ui.begin_menu("Export") else {
        return false;
    };

    let is_exporting = editor.is_exporting();
    let mut started = false;

    {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);

        if ui.menu_item("Save screenshot") {
            editor.request_screenshot();
        }

        if ui.menu_item("Export scene") {
            editor.export_scene(true);

            started = editor.is_exporting();
        }
    }

    let label = if is_exporting { "Cancel" } else { "Export" };

    if ui.menu_item(label) {
        editor.toggle_export(true);

        started = !is_exporting && editor.is_exporting();
    }

    if is_exporting {
        let progress = editor.export_progress();
        let percentage = format!("{:.0}%", progress * 100.0);

        ui.progress_bar_with_overlay(progress, &percentage)
            .size([ui.content_region_avail_width(), 0.0])
            .build();
    }

    if let Some(message) = editor.export_message() {
        ui.text_wrapped(message);
    }

    started
}
