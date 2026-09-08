use dear_imgui_rs::{StyleColor, StyleVar, Ui};

pub(super) fn text_button(ui: &Ui, label: &str, size: [f32; 2]) -> bool {
    let transparent = [0.0, 0.0, 0.0, 0.0];
    let _background = ui.push_style_color(StyleColor::Button, transparent);
    let _hovered = ui.push_style_color(StyleColor::ButtonHovered, transparent);
    let _active = ui.push_style_color(StyleColor::ButtonActive, transparent);
    let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));

    let clicked = ui.invisible_button(label, size);
    let (min, max) = ui.item_rect();
    let text_size =
        ui.current_font()
            .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, label);
    let text_position = [
        min[0] + (max[0] - min[0] - text_size[0]) * 0.5,
        min[1] + (max[1] - min[1] - text_size[1]) * 0.5,
    ];
    let text_color = if ui.is_item_hovered() {
        StyleColor::TextLink
    } else {
        StyleColor::Text
    };
    ui.get_window_draw_list()
        .add_text(text_position, ui.get_color_u32(text_color), label);

    clicked
}
