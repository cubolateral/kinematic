//! Project status and application information at the bottom of the viewport.

use crate::{controls, icons, widgets};
use dear_imgui_rs::{Condition, StyleVar, Ui, WindowFlags};

pub fn height(ui: &Ui) -> f32 {
    ui.frame_height() + 8.0
}

pub fn draw(ui: &Ui, status: &str, version: &str) -> bool {
    let viewport = ui.main_viewport();
    let position = viewport.pos();
    let size = viewport.size();
    let height = height(ui);
    let _padding = ui.push_style_var(StyleVar::WindowPadding([8.0, 4.0]));
    let _rounding = ui.push_style_var(StyleVar::WindowRounding(0.0));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
    let _minimum = ui.push_style_var(StyleVar::WindowMinSize([0.0; 2]));
    let mut repository_clicked = false;

    ui.window("Project status")
        .position(
            [position[0], position[1] + size[1] - height],
            Condition::Always,
        )
        .size([size[0], height], Condition::Always)
        .flags(
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                | WindowFlags::NO_DOCKING
                | WindowFlags::NO_SAVED_SETTINGS
                | WindowFlags::NO_FOCUS_ON_APPEARING,
        )
        .build(|| {
            widgets::horizontal_border(ui, ui.window_pos()[1]);
            let version = format!("VERSION {version}");
            let spacing = ui.clone_style().item_spacing()[0];
            let button_width = ui.frame_height();
            let right_width = button_width + spacing + ui.calc_text_size(&version)[0];
            let origin = ui.cursor_pos();
            let right = origin[0] + ui.content_region_avail()[0] - right_width;

            ui.child_window("Status message")
                .size([(right - origin[0] - spacing).max(1.0), ui.frame_height()])
                .flags(WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE)
                .build(ui, || {
                    ui.align_text_to_frame_padding();
                    widgets::muted(ui, status);
                });

            ui.set_cursor_pos([right.max(origin[0]), origin[1]]);
            let _id = ui.push_id("Repository");
            repository_clicked =
                controls::text_button(ui, icons::REPOSITORY, [button_width, ui.frame_height()]);
            if ui.is_item_hovered() {
                widgets::tooltip(ui, "Open the Kinematic repository");
            }
            ui.same_line();
            ui.align_text_to_frame_padding();
            widgets::muted(ui, version);
        });

    repository_clicked
}
