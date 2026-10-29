use dear_imgui_rs::{StyleColor, StyleVar, Ui};

pub fn text_button(ui: &Ui, label: &str, size: [f32; 2]) -> bool {
    text_button_colored(ui, label, size, StyleColor::Text)
}

pub fn text_button_colored(ui: &Ui, label: &str, size: [f32; 2], color: StyleColor) -> bool {
    let transparent = [0.0, 0.0, 0.0, 0.0];
    let _background = ui.push_style_color(StyleColor::Button, transparent);
    let _hovered = ui.push_style_color(StyleColor::ButtonHovered, transparent);
    let _active = ui.push_style_color(StyleColor::ButtonActive, transparent);
    let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));

    let clicked = ui.invisible_button(label, size);
    let (min, max) = ui.item_rect();
    let text_size = ui.calc_text_size(label);
    let text_position = [
        min[0] + (max[0] - min[0] - text_size[0]) * 0.5,
        min[1] + (max[1] - min[1] - text_size[1]) * 0.5,
    ];
    let text_color = if ui.is_item_hovered() {
        StyleColor::TextLink
    } else {
        color
    };

    ui.get_window_draw_list()
        .add_text(text_position, ui.get_color_u32(text_color), label);

    clicked
}

pub fn button(ui: &Ui, label: &str) -> bool {
    ui.button(label)
}

pub fn button_sized(ui: &Ui, label: &str, size: [f32; 2]) -> bool {
    ui.button_with_size(label, size)
}

pub fn checkbox(ui: &Ui, label: &str, value: &mut bool) -> bool {
    ui.checkbox(label, value)
}

pub fn text_input(ui: &Ui, label: &str, value: &mut String) -> bool {
    ui.input_text(label, value).build()
}

pub fn text_area(ui: &Ui, label: &str, value: &mut String) -> bool {
    ui.input_text_multiline(label, value, [0.0, ui.frame_height() * 3.0])
        .flags(
            dear_imgui_rs::InputTextMultilineFlags::CTRL_ENTER_FOR_NEW_LINE
                | dear_imgui_rs::InputTextMultilineFlags::WORD_WRAP,
        )
        .build()
}

pub fn color(ui: &Ui, label: &str, value: &mut [f32; 4]) -> bool {
    ui.color_edit4(label, value)
}

pub fn choice<S: AsRef<str>>(ui: &Ui, label: &str, current: &mut usize, options: &[S]) -> bool {
    let preview = options.get(*current).map_or("Select...", AsRef::as_ref);
    let Some(_combo) = ui.begin_combo(label, preview) else {
        return false;
    };
    let mut changed = false;
    for (index, option) in options.iter().enumerate() {
        let _id = ui.push_id(index);
        if ui
            .selectable_config(option.as_ref())
            .selected(*current == index)
            .build()
        {
            changed = *current != index;
            *current = index;
        }
        if *current == index {
            ui.set_item_default_focus();
        }
    }
    changed
}

pub fn text_submit(ui: &Ui, label: &str, value: &mut String) -> bool {
    ui.input_text(label, value).enter_returns_true(true).build()
}

pub fn slider<T: dear_imgui_rs::internal::DataTypeKind>(
    ui: &Ui,
    label: &str,
    value: &mut T,
    range: std::ops::RangeInclusive<T>,
    format: &str,
) -> bool {
    ui.slider_config(label, *range.start(), *range.end())
        .display_format(dear_imgui_rs::NumericFormat::new(format).expect("Valid slider format."))
        .build(value)
}

pub fn radio_button(ui: &Ui, label: &str, selected: bool) -> bool {
    ui.radio_button(label, selected)
}

pub fn menu<'ui>(ui: &'ui Ui, label: &str) -> Option<dear_imgui_rs::MenuToken<'ui>> {
    ui.begin_menu(label)
}

pub fn menu_item(ui: &Ui, label: impl AsRef<str>) -> bool {
    ui.menu_item(label)
}

pub fn progress(ui: &Ui, progress: f32, label: &str) {
    ui.progress_bar_with_overlay(progress, label)
        .size([ui.content_region_avail_width(), 0.0])
        .build();
}

pub fn icon_toggle(ui: &Ui, label: &str, enabled: bool) -> bool {
    ui.set_next_item_allow_overlap();
    text_button_colored(
        ui,
        label,
        [ui.frame_height(); 2],
        if enabled {
            StyleColor::Text
        } else {
            StyleColor::TextDisabled
        },
    )
}
