//! Shared file rows and suggestion rows, independent of their data sources.

use dear_imgui_rs::{StyleColor, Ui};

fn file_icon(directory: bool) -> (&'static str, [f32; 4]) {
    if directory {
        (crate::icons::FOLDER, [0.95, 0.72, 0.3, 1.0])
    } else {
        (crate::icons::FILE, [0.4, 0.7, 0.95, 1.0])
    }
}

pub fn file_label(ui: &Ui, position: [f32; 2], label: &str, directory: bool) {
    let (icon, color) = file_icon(directory);
    let x = position[0] + ui.tree_node_to_label_spacing();
    let y = position[1] + ui.clone_style().frame_padding()[1];
    let draw = ui.get_window_draw_list();
    draw.add_text([x, y], color, icon);
    draw.add_text(
        [x + ui.calc_text_size(icon)[0] + 6.0, y],
        ui.get_color_u32(StyleColor::Text),
        label,
    );
}

pub fn file_row(ui: &Ui, label: &str, directory: bool, selected: bool) -> bool {
    let (icon, color) = file_icon(directory);
    ui.text_colored(color, icon);
    ui.same_line();
    let position = ui.cursor_screen_pos();
    let clicked = ui
        .selectable_config("##Entry")
        .size([ui.content_region_avail_width(), ui.text_line_height()])
        .flags(
            dear_imgui_rs::SelectableFlags::NO_AUTO_CLOSE_POPUPS
                | dear_imgui_rs::SelectableFlags::ALLOW_DOUBLE_CLICK,
        )
        .selected(selected)
        .build();
    ui.get_window_draw_list()
        .add_text(position, ui.get_color_u32(StyleColor::Text), label);
    clicked
}

pub fn suggestion(
    ui: &Ui,
    id: &str,
    label: &str,
    detail: &str,
    detail_color: u32,
    selected: bool,
) -> bool {
    ui.selectable_config(id).selected(selected).build();
    // The source editor keeps keyboard focus behind this overlay.
    let pressed = ui.is_item_hovered_with_flags(
        dear_imgui_rs::ItemHoveredFlags::ALLOW_WHEN_OVERLAPPED_BY_WINDOW,
    ) && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left);
    if !detail.is_empty() {
        let origin = ui.item_rect_min();
        ui.get_window_draw_list().add_text(
            [origin[0] + ui.calc_text_size(label)[0] + 28.0, origin[1]],
            detail_color,
            detail,
        );
    }
    pressed
}

pub fn bindings<'a>(ui: &Ui, groups: impl IntoIterator<Item = (&'a str, Vec<(&'a str, &'a str)>)>) {
    let flags = dear_imgui_rs::TableFlags::SCROLL_Y;

    let Some(_table) = ui.begin_table_with_sizing(
        "Shortcut list",
        2,
        dear_imgui_rs::TableOptions::from(flags),
        [0.0, ui.content_region_avail()[1]],
        0.0,
    ) else {
        return;
    };

    ui.table_setup_scroll_freeze(0, 1);
    ui.table_setup_column("Action", dear_imgui_rs::TableColumnFlags::empty(), None);
    ui.table_setup_column("Binding", dear_imgui_rs::TableColumnFlags::empty(), None);
    ui.table_headers_row();

    for (title, bindings) in groups {
        ui.table_next_row();
        ui.table_next_column();

        let group = ui
            .tree_node_config(title)
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .no_tree_push_on_open(true)
            .push();

        ui.table_next_column();

        if group.is_some() {
            for (name, binding) in bindings {
                ui.table_next_row();
                ui.table_next_column();
                ui.set_cursor_pos_x(ui.cursor_pos_x() + ui.tree_node_to_label_spacing());
                crate::widgets::text(ui, name);
                ui.table_next_column();
                crate::widgets::text(ui, binding);
            }
        }
    }
}
