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

pub fn message(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    ui.text_wrapped(text);
}

pub fn separator(ui: &dear_imgui_rs::Ui) {
    ui.separator();
}

pub fn error(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    let _color = ui.push_style_color(dear_imgui_rs::StyleColor::Text, crate::theme::ERROR);
    ui.text_wrapped(text);
}
