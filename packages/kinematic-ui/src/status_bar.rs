//! Version information shared by the workspace bottom bar.

use crate::widgets;
use dear_imgui_rs::Ui;

pub fn width(ui: &Ui, version: &str) -> f32 {
    ui.calc_text_size(format!("v{version}"))[0]
}

pub fn draw(ui: &Ui, version: &str) {
    let origin = ui.cursor_pos();
    let right = origin[0] + ui.content_region_avail_width() - width(ui, version);

    ui.set_cursor_pos([right.max(origin[0]), origin[1]]);

    ui.align_text_to_frame_padding();
    widgets::muted(ui, format!("v{version}"));
}

pub fn height(ui: &Ui) -> f32 {
    ui.frame_height() + 8.0
}

pub fn home(ui: &Ui, version: &str) {
    let viewport = ui.main_viewport();
    let position = viewport.pos();
    let size = viewport.size();
    let height = height(ui);
    let _padding = ui.push_style_var(dear_imgui_rs::StyleVar::WindowPadding([8.0, 4.0]));
    let _rounding = ui.push_style_var(dear_imgui_rs::StyleVar::WindowRounding(0.0));
    let _border = ui.push_style_var(dear_imgui_rs::StyleVar::WindowBorderSize(0.0));
    let _minimum = ui.push_style_var(dear_imgui_rs::StyleVar::WindowMinSize([0.0; 2]));

    ui.window("Home status")
        .position(
            [position[0], position[1] + size[1] - height],
            dear_imgui_rs::Condition::Always,
        )
        .size([size[0], height], dear_imgui_rs::Condition::Always)
        .flags(
            dear_imgui_rs::WindowFlags::NO_DECORATION
                | dear_imgui_rs::WindowFlags::NO_MOVE
                | dear_imgui_rs::WindowFlags::NO_DOCKING
                | dear_imgui_rs::WindowFlags::NO_SAVED_SETTINGS
                | dear_imgui_rs::WindowFlags::NO_NAV_FOCUS
                | dear_imgui_rs::WindowFlags::NO_FOCUS_ON_APPEARING,
        )
        .build(|| draw(ui, version));
}
