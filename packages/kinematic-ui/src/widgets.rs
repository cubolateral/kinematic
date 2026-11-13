pub fn text_size(ui: &dear_imgui_rs::Ui, text: &str) -> [f32; 2] {
    ui.calc_text_size(text)
}

pub fn hide_window_tab_bar(ui: &dear_imgui_rs::Ui) {
    ui.set_next_window_class(
        &dear_imgui_rs::WindowClass::default().dock_node_flags_override_set(
            dear_imgui_rs::WindowClassDockNodeFlags::NO_TAB_BAR
                | dear_imgui_rs::WindowClassDockNodeFlags::NO_UNDOCKING
                | dear_imgui_rs::WindowClassDockNodeFlags::NO_DOCKING_SPLIT
                | dear_imgui_rs::WindowClassDockNodeFlags::NO_WINDOW_MENU_BUTTON
                | dear_imgui_rs::WindowClassDockNodeFlags::NO_CLOSE_BUTTON,
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
    let position = ui.cursor_screen_pos();
    let size = [
        ui.content_region_avail_width(),
        ui.clone_style().docking_separator_size(),
    ];

    ui.get_window_draw_list()
        .add_rect(
            position,
            [position[0] + size[0], position[1] + size[1]],
            ui.style_color(dear_imgui_rs::StyleColor::Separator),
        )
        .filled(true)
        .build();
    ui.dummy(size);
}

pub fn horizontal_divider(ui: &dear_imgui_rs::Ui, y: f32) {
    let position = ui.window_pos();
    let size = ui.window_size();
    let x = position[0];
    let _clip = ui.push_clip_rect(
        position,
        [position[0] + size[0], position[1] + size[1]],
        false,
    );
    let height = ui
        .clone_style()
        .docking_separator_size()
        .min(crate::theme::FRAME_PADDING[1]);

    ui.get_window_draw_list()
        .add_rect(
            [x, y],
            [x + ui.window_size()[0], y + height],
            ui.style_color(dear_imgui_rs::StyleColor::DockingEmptyBg),
        )
        .filled(true)
        .build();
}

pub fn error(ui: &dear_imgui_rs::Ui, text: impl AsRef<str>) {
    let _color = ui.push_style_color(dear_imgui_rs::StyleColor::Text, crate::theme::ERROR);

    ui.text_wrapped(text);
}
