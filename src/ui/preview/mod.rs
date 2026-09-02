mod fullscreen;
mod image;

use crate::editor::Editor;

use super::widgets::hide_single_window_tab;

pub(super) const WINDOW_NAME: &str = "Preview";

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
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
        ui.separator();

        clicked = image::draw(ui, preview, ui.content_region_avail(), true);
    });

    if let Some(point) = clicked {
        editor.select_at(point);
    }
}

pub(super) fn draw_fullscreen(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    fullscreen::draw(editor, ui)
}
