use crate::editor::Editor;

use super::{controls, icons};

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    if controls::text_button(ui, icons::SHARE, [ui.frame_height(); 2]) {
        ui.open_popup("Export");
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Export");
    }

    let (_, anchor) = ui.item_rect();
    unsafe {
        dear_imgui_rs::sys::igSetNextWindowPos(
            dear_imgui_rs::sys::ImVec2 {
                x: anchor[0],
                y: anchor[1],
            },
            dear_imgui_rs::Condition::Always as i32,
            dear_imgui_rs::sys::ImVec2 { x: 1.0, y: 0.0 },
        );
        dear_imgui_rs::sys::igSetNextWindowSize(
            dear_imgui_rs::sys::ImVec2 {
                x: ui.frame_height() * 10.0,
                y: 0.0,
            },
            dear_imgui_rs::Condition::Always as i32,
        );
    }

    let Some(_popup) = ui.begin_popup("Export") else {
        return false;
    };

    let is_exporting = editor.is_exporting();
    let mut started = false;
    {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        if ui.button_with_size("Save screenshot", [ui.content_region_avail_width(), 0.0]) {
            editor.request_screenshot();
        }
        if ui.button_with_size("Export scene", [ui.content_region_avail_width(), 0.0]) {
            editor.export_scene(true);
            started = editor.is_exporting();
        }
    }

    let label = if is_exporting { "Cancel" } else { "Export" };
    if ui.button_with_size(label, [ui.content_region_avail_width(), 0.0]) {
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
