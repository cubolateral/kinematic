//! Shared file rows and suggestion rows, independent of their data sources.

use dear_imgui_rs::{StyleColor, Ui};

fn file_icon(ui: &Ui, label: &str, directory: bool) -> (&'static str, [f32; 4]) {
    let icon = if directory {
        crate::icons::FOLDER
    } else {
        match label
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "lua" => crate::icons::SCRIPT,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tga" | "tif" | "tiff" | "svg"
            | "ico" | "avif" => crate::icons::IMAGE,
            "msc" | "wav" | "mp3" | "ogg" | "flac" | "m4a" | "aac" => crate::icons::MUSIC,
            _ => crate::icons::FILE,
        }
    };

    (
        icon,
        ui.style_color(if directory {
            StyleColor::CheckMark
        } else {
            StyleColor::Text
        }),
    )
}

pub fn file_label(ui: &Ui, position: [f32; 2], label: &str, directory: bool, selected: bool) {
    let (icon, _) = file_icon(ui, label, directory);
    let color = ui.get_color_u32(if selected {
        StyleColor::CheckMark
    } else {
        StyleColor::Text
    });
    let x = position[0] + ui.tree_node_to_label_spacing();
    let y = position[1] + ui.clone_style().frame_padding()[1];
    let draw = ui.get_window_draw_list();

    draw.add_text([x, y], color, icon);
    draw.add_text(
        [
            x + ui.calc_text_size(icon)[0] + ui.clone_style().item_spacing()[0].max(10.0),
            y,
        ],
        color,
        label,
    );
}

pub fn file_branch(ui: &Ui, parent: [f32; 2], child: [f32; 2], directory: bool) {
    let middle = ui.frame_height() * 0.5;
    let x = parent[0] + ui.tree_node_to_label_spacing() * 0.5;
    let color = ui.get_color_u32(StyleColor::TextDisabled);
    let draw = ui.get_window_draw_list();

    let gap = 4.0;
    let arrow = ui.calc_text_size(crate::icons::CHEVRON_DOWN);
    let end = child[0] + ui.tree_node_to_label_spacing()
        - if directory {
            (ui.tree_node_to_label_spacing() + arrow[0]) * 0.5 + gap
        } else {
            gap
        };

    draw.add_line(
        [x, parent[1] + middle + arrow[1] * 0.5 + gap],
        [x, child[1] + middle],
        color,
    )
    .thickness(1.0)
    .build();
    draw.add_line([x, child[1] + middle], [end, child[1] + middle], color)
        .thickness(1.0)
        .build();
}

pub fn file_row(ui: &Ui, label: &str, directory: bool, selected: bool) -> bool {
    let (icon, color) = file_icon(ui, label, directory);

    ui.text_colored(color, icon);
    ui.same_line_with_spacing(0.0, ui.clone_style().item_spacing()[0].max(10.0));

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

                crate::widgets::muted(ui, crate::shortcut::compact(binding));
            }
        }
    }
}

pub fn project_row(
    ui: &Ui,
    name: &str,
    path: &str,
    opened_at: &str,
    version: &str,
    available: bool,
) -> bool {
    let style = ui.clone_style();
    let padding = [
        style.frame_padding()[0].max(12.0),
        style.frame_padding()[1].max(10.0),
    ];
    let spacing = style.item_spacing()[1];
    let position = ui.cursor_screen_pos();
    let size = [
        ui.content_region_avail_width(),
        ui.text_line_height() * 2.0 + spacing + padding[1] * 2.0,
    ];
    let _background = ui.push_style_color(StyleColor::Button, [0.0; 4]);
    let clicked = crate::controls::button_sized(ui, "##Project", size);
    let draw = ui.get_window_draw_list();
    let min = [position[0] + padding[0], position[1] + padding[1]];
    let max_x = position[0] + size[0] - padding[0];
    let color = ui.get_color_u32(if available {
        StyleColor::Text
    } else {
        StyleColor::TextDisabled
    });

    {
        let _clip = draw.push_clip_rect(min, [max_x, position[1] + size[1]], true);

        draw.add_text(min, color, name);

        if !available {
            let y = min[1] + ui.text_line_height() * 0.5;

            draw.add_line(
                [min[0], y],
                [(min[0] + ui.calc_text_size(name)[0]).min(max_x), y],
                color,
            )
            .build();
        }
    }

    let y = min[1] + ui.text_line_height() + spacing;
    let metadata = format!("{opened_at} {version}");
    let metadata_x = (max_x - ui.calc_text_size(&metadata)[0]).max(min[0]);
    let color = ui.get_color_u32(StyleColor::TextDisabled);

    {
        let _clip = draw.push_clip_rect(
            [min[0], y],
            [
                (metadata_x - padding[0]).max(min[0]),
                y + ui.text_line_height(),
            ],
            true,
        );

        draw.add_text(
            [min[0], y],
            color,
            format!("{}  {path}", crate::icons::FOLDER),
        );
    }

    {
        let _clip = draw.push_clip_rect([metadata_x, y], [max_x, y + ui.text_line_height()], true);

        draw.add_text([metadata_x, y], color, opened_at);
        draw.add_text(
            [max_x - ui.calc_text_size(version)[0], y],
            ui.get_color_u32(if available {
                StyleColor::Text
            } else {
                StyleColor::TextDisabled
            }),
            version,
        );
    }

    clicked
}
