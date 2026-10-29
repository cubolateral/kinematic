use crate::editor::Editor;
use kinematic_ui::controls;
use kinematic_ui::widgets;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    let Some(_menu) = controls::menu(ui, "Export") else {
        return false;
    };

    let is_exporting = editor.is_exporting();
    let mut started = false;

    {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);

        if controls::menu_item(ui, "Save screenshot") {
            editor.request_screenshot();
        }

        if controls::menu_item(ui, "Export scene") {
            editor.export_scene(true);

            started = editor.is_exporting();
        }
    }

    let label = if is_exporting { "Cancel" } else { "Export" };

    if controls::menu_item(ui, label) {
        editor.toggle_export(true);

        started = !is_exporting && editor.is_exporting();
    }

    if is_exporting {
        let progress = editor.export_progress();
        let percentage = format!("{:.0}%", progress * 100.0);

        controls::progress(ui, progress, &percentage);
    }

    if let Some(message) = editor.export_message() {
        widgets::message(ui, message);
    }

    started
}
