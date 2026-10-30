pub fn text_size(ui: &dear_imgui_rs::Ui, text: &str) -> [f32; 2] {
    ui.calc_text_size(text)
}

pub fn hide_single_window_tab(ui: &dear_imgui_rs::Ui) {
    ui.set_next_window_class(
        &dear_imgui_rs::WindowClass::default().dock_node_flags_override_set(
            dear_imgui_rs::WindowClassDockNodeFlags::AUTO_HIDE_TAB_BAR,
        ),
    );
}

pub fn text(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    ui.text(text);
}

pub fn muted(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    ui.text_disabled(text);
}

pub fn tooltip(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    ui.tooltip_text(text.as_ref().trim_end().trim_end_matches('.'));
}

pub fn message(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    ui.text_wrapped(text);
}

pub fn separator(ui: &dear_imgui_rs::Ui) {
    ui.separator();
}

pub fn horizontal_border(ui: &dear_imgui_rs::Ui, y: f32) {
    let x = ui.window_pos()[0];
    ui.get_window_draw_list()
        .add_line(
            [x, y],
            [x + ui.window_size()[0], y],
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .thickness(crate::theme::BORDER_WIDTH)
        .build();
}

pub fn error(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    let _color = ui.push_style_color(dear_imgui_rs::StyleColor::Text, crate::theme::ERROR);
    ui.text_wrapped(text);
}
