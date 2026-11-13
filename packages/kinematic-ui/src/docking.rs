//! Presentation for fixed dock positions and resizable workspace regions.

use dear_imgui_rs::{Condition, MouseCursor, StyleColor, StyleVar, Ui, WindowFlags};

const SPLITTER_HIT_TARGET: f32 = 12.0;

pub fn splitter(ui: &Ui, id: &str, position: [f32; 2], size: [f32; 2], axis: usize) -> f32 {
    let mut hit_position = position;
    let mut hit_size = size;
    hit_size[axis] = hit_size[axis].max(SPLITTER_HIT_TARGET);
    hit_position[axis] -= (hit_size[axis] - size[axis]) * 0.5;

    let _padding = ui.push_style_var(StyleVar::WindowPadding([0.0; 2]));
    let _minimum = ui.push_style_var(StyleVar::WindowMinSize([0.0; 2]));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));

    ui.window(format!("Workspace splitter##{id}"))
        .position(hit_position, Condition::Always)
        .size(hit_size.map(|value| value.max(1.0)), Condition::Always)
        .flags(
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                | WindowFlags::NO_DOCKING
                | WindowFlags::NO_BACKGROUND
                | WindowFlags::NO_SAVED_SETTINGS
                | WindowFlags::NO_FOCUS_ON_APPEARING,
        )
        .build(|| {
            ui.set_cursor_screen_pos(hit_position);
            ui.invisible_button(id, hit_size.map(|value| value.max(1.0)));

            let active = ui.is_item_active();
            let hovered = ui.is_item_hovered();
            if active || hovered {
                ui.set_mouse_cursor(Some(if axis == 0 {
                    MouseCursor::ResizeEW
                } else {
                    MouseCursor::ResizeNS
                }));
            }

            if active {
                ui.io().mouse_delta()[axis]
            } else {
                0.0
            }
        })
        .unwrap_or(0.0)
}

pub fn background(ui: &Ui) {
    let viewport = ui.main_viewport();
    let position = viewport.pos();
    let size = viewport.size();

    ui.get_background_draw_list()
        .add_rect(
            position,
            [position[0] + size[0], position[1] + size[1]],
            ui.style_color(StyleColor::DockingEmptyBg),
        )
        .filled(true)
        .build();
}

pub fn footer(ui: &Ui, selected: Option<usize>, titles: &[&str], version: &str) -> Option<usize> {
    let origin = ui.cursor_screen_pos();
    let available = ui.content_region_avail_width()
        - crate::status_bar::width(ui, version)
        - ui.clone_style().item_spacing()[0];
    let activated = header_with_width(
        ui,
        selected.map_or("", |index| titles[index]),
        titles,
        available,
    );

    ui.same_line();
    ui.set_cursor_screen_pos([ui.cursor_screen_pos()[0], origin[1]]);

    crate::status_bar::draw(ui, version);
    activated
}

pub fn centered_header(
    ui: &Ui,
    selected: usize,
    titles: &[&str],
    action: &str,
    tooltips: &[(&str, &str)],
) -> (Option<usize>, bool) {
    let origin = ui.cursor_screen_pos();
    let available = ui.content_region_avail_width();
    let width = titles
        .iter()
        .map(|title| ui.calc_text_size(title)[0])
        .sum::<f32>()
        + ui.clone_style().item_spacing()[0] * titles.len().saturating_sub(1) as f32;
    ui.set_cursor_pos_x(
        ui.cursor_pos_x() + ((ui.content_region_avail_width() - width) * 0.5).max(0.0),
    );

    let selected = header_with_width_and_hover(
        ui,
        titles[selected],
        titles,
        (available - ui.frame_height() * 2.0).max(1.0),
        Some(tooltips),
        true,
    );

    ui.same_line();
    ui.set_cursor_screen_pos([origin[0], origin[1]]);

    let clicked = crate::controls::text_button(ui, action, [ui.frame_height(); 2]);

    (selected, clicked)
}

pub fn header(ui: &Ui, title: &str, titles: &[&str]) -> Option<usize> {
    header_with_width(ui, title, titles, ui.content_region_avail_width())
}

fn header_with_width(ui: &Ui, title: &str, titles: &[&str], available: f32) -> Option<usize> {
    header_with_width_and_hover(ui, title, titles, available, None, false)
}

fn header_with_width_and_hover(
    ui: &Ui,
    title: &str,
    titles: &[&str],
    available: f32,
    tooltips: Option<&[(&str, &str)]>,
    white_inactive: bool,
) -> Option<usize> {
    let spacing = ui.clone_style().item_spacing()[0];
    let width = titles
        .iter()
        .map(|name| ui.calc_text_size(name)[0])
        .sum::<f32>();
    let scale = ((available - spacing * titles.len().saturating_sub(1) as f32) / width.max(1.0))
        .clamp(0.0, 1.0);

    let mut activated = None;

    for (index, &name) in titles.iter().enumerate() {
        if index > 0 {
            ui.same_line_with_spacing(0.0, spacing);
        }

        let color = if name == title {
            if white_inactive {
                StyleColor::TextLink
            } else {
                StyleColor::Text
            }
        } else if white_inactive {
            StyleColor::Text
        } else {
            StyleColor::TextDisabled
        };
        let position = ui.cursor_screen_pos();
        let size = [
            (ui.calc_text_size(name)[0] * scale).max(1.0),
            ui.frame_height(),
        ];
        let _clip = ui.push_clip_rect(
            position,
            [position[0] + size[0], position[1] + size[1]],
            true,
        );

        if crate::controls::text_button_colored(ui, name, size, color) {
            activated = Some(index);
        }
        if let Some((action, shortcut)) = tooltips.and_then(|items| items.get(index)) {
            crate::shortcut::tooltip(ui, action, shortcut);
        }
    }

    activated
}

/// Select a panel before submitting docked windows for this frame.
pub fn activate(ui: &Ui, title: &str, titles: &[&str]) {
    let Ok(target_name) = std::ffi::CString::new(title) else {
        return;
    };

    // ImGui shows the first docked window when its native tab bar is disabled.
    ui.with_bound_context(|| unsafe {
        let target = dear_imgui_rs::sys::igFindWindowByName(target_name.as_ptr());
        if target.is_null() || (*target).DockId == 0 {
            return;
        }
        let dock = (*target).DockId;

        for name in titles {
            let Ok(name) = std::ffi::CString::new(*name) else {
                continue;
            };
            let window = dear_imgui_rs::sys::igFindWindowByName(name.as_ptr());
            if window.is_null() || window == target || (*window).DockId != dock {
                continue;
            }

            // Keep the target attached so the node and its split sizes survive.
            dear_imgui_rs::sys::igSetWindowDock(window, 0, Condition::Always as i32);
            dear_imgui_rs::sys::igSetWindowDock(window, dock, Condition::Always as i32);
        }
    });
    ui.set_window_focus(Some(title));
}

pub fn position_menu(
    ui: &Ui,
    title: &str,
    titles: &[&str],
    selected: usize,
) -> (Option<usize>, Option<usize>) {
    let width =
        ui.content_region_avail_width() - ui.frame_height() - ui.clone_style().item_spacing()[0];

    let activated = header_with_width(ui, title, titles, width);
    ui.same_line();
    ui.set_cursor_pos_x(
        ui.cursor_pos_x() + (ui.content_region_avail_width() - ui.frame_height()).max(0.0),
    );

    if crate::controls::text_button(ui, crate::icons::MORE_VERTICAL, [ui.frame_height(); 2]) {
        ui.open_popup("Dock position");
    }

    if !ui.is_popup_open("Dock position") {
        return (activated, None);
    }

    let Some(_popup) = ui.begin_popup_with_flags("Dock position", WindowFlags::NO_MOVE) else {
        return (activated, None);
    };

    crate::widgets::text(ui, "Dock position");

    let origin = ui.cursor_screen_pos();
    let cell = ui.frame_height();
    let gap = 3.0;
    let mut clicked = None;

    for side in 0..2 {
        for column in 0..2 {
            for row in 0..2 {
                let index = side * 4 + column * 2 + row;
                let x = origin[0] + (side * 4 + column) as f32 * (cell + gap);
                let y = origin[1] + row as f32 * (cell * 1.5 + gap);

                ui.set_cursor_screen_pos([x, y]);

                let _id = ui.push_id(index as i32);
                let pressed = ui.invisible_button("Position", [cell, cell * 1.5]);
                let color = if selected == index {
                    StyleColor::ButtonActive
                } else if ui.is_item_hovered() {
                    StyleColor::ButtonHovered
                } else {
                    StyleColor::Button
                };

                ui.get_window_draw_list()
                    .add_rect([x, y], [x + cell, y + cell * 1.5], ui.style_color(color))
                    .filled(true)
                    .build();

                if pressed {
                    clicked = Some(index);
                    ui.close_current_popup();
                }
            }
        }
    }

    let center = [origin[0] + 2.0 * (cell + gap), origin[1]];
    let right = origin[0] + 4.0 * (cell + gap) - gap;
    let bottom = origin[1] + 3.0 * cell + gap;
    let draw = ui.get_window_draw_list();
    let fill = ui.style_color(StyleColor::Button);
    let gap = ui.clone_style().docking_separator_size();
    let timeline = bottom - cell * 0.6;

    draw.add_rect(center, [right, timeline - gap], fill)
        .rounding(ui.clone_style().frame_rounding())
        .filled(true)
        .build();
    draw.add_rect([center[0], timeline], [right, bottom], fill)
        .rounding(ui.clone_style().frame_rounding())
        .filled(true)
        .build();

    ui.set_cursor_screen_pos(origin);
    ui.dummy([6.0 * (cell + gap), 3.0 * cell + gap]);

    (activated, clicked)
}
