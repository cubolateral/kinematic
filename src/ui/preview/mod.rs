mod fullscreen;
mod image;
mod state;

use crate::editor::Editor;

use super::{controls, icons, widgets::hide_single_window_tab};
use state::Mode;

pub(super) const WINDOW_NAME: &str = "Preview";

pub(super) use state::State;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    let is_exporting = editor.is_exporting();
    hide_single_window_tab(ui);

    ui.window(WINDOW_NAME).build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let plain_keyboard_input = !is_exporting
            && !ui.io().want_text_input()
            && !ui.is_any_item_active()
            && !ui.io().key_ctrl()
            && !ui.io().key_shift()
            && !ui.io().key_alt()
            && !ui.io().key_super();
        let requested =
            shortcut_mode(ui, plain_keyboard_input).or_else(|| state.take_requested_mode());
        if let Some(mode) = requested {
            state.set_mode(mode);
        }
        let reset = plain_keyboard_input
            && (ui.is_key_pressed(dear_imgui_rs::Key::Key0)
                || ui.is_key_pressed(dear_imgui_rs::Key::Keypad0));
        if reset {
            match state.mode() {
                Mode::Two => editor.toggle_editor_2d_camera_view(),
                Mode::Three => editor.toggle_editor_3d_camera_view(),
                Mode::Preview => {}
            }
        }
        let reset_camera = plain_keyboard_input && ui.is_key_pressed(dear_imgui_rs::Key::R);
        if reset_camera {
            match state.mode() {
                Mode::Two => editor.reset_editor_2d_camera_transform(),
                Mode::Three => editor.reset_editor_3d_camera_transform(),
                Mode::Preview => {}
            }
        }
        let Some(_tabs) = ui.tab_bar("Preview modes") else {
            return;
        };

        let preview_tab =
            ui.tab_item_with_flags("PREVIEW", None, tab_flags(requested == Some(Mode::Preview)));
        if ui.is_item_hovered() {
            ui.tooltip_text("Show the final exported view [1]");
        }
        if let Some(_tab) = preview_tab
            && requested.is_none_or(|mode| mode == Mode::Preview)
        {
            state.set_mode(Mode::Preview);
            image::draw(ui, image::preview(editor), ui.content_region_avail());
        }

        let editor_2d_tab =
            ui.tab_item_with_flags("2D", None, tab_flags(requested == Some(Mode::Two)));
        if ui.is_item_hovered() {
            ui.tooltip_text("Edit the selected Canvas2D [2]");
        }
        if let Some(_tab) = editor_2d_tab
            && requested.is_none_or(|mode| mode == Mode::Two)
        {
            state.set_mode(Mode::Two);
            let canvas = editor.editor_2d_canvas();
            if state.sync_canvas_2d(editor.active_scene_index(), canvas) {
                editor.reset_editor_2d_view();
            }
            draw_editor_2d(editor, ui);
        }

        let editor_3d_tab =
            ui.tab_item_with_flags("3D", None, tab_flags(requested == Some(Mode::Three)));
        if ui.is_item_hovered() {
            ui.tooltip_text("Edit the selected Canvas3D [3]");
        }
        if let Some(_tab) = editor_3d_tab
            && requested.is_none_or(|mode| mode == Mode::Three)
        {
            state.set_mode(Mode::Three);
            let canvas = editor.editor_3d_canvas();
            if state.sync_canvas_3d(editor.active_scene_index(), canvas) {
                editor.reset_editor_3d_view();
            }
            draw_editor_3d(editor, ui, plain_keyboard_input);
        }
    });
}

fn draw_editor_3d(editor: &mut Editor, ui: &dear_imgui_rs::Ui, keyboard: bool) {
    let min = ui.cursor_screen_pos();
    let available = ui.content_region_avail();
    let editor_image = image::editor_3d(editor);
    let (viewport_active, clicked) = image::draw_editor_3d(ui, editor_image, available, editor);
    if let Some(point) = clicked {
        editor.select_at_editor_3d(point);
    }
    if editor.editor_3d_camera_view() {
        let (_, canvas_size) = editor.editor_3d_canvas_camera();
        let canvas_aspect = canvas_size.0.max(1) as f32 / canvas_size.1.max(1) as f32;
        draw_camera_mask(ui, min, available, canvas_aspect);
    }
    let right = dear_imgui_rs::MouseButton::Right;
    let looking = viewport_active && ui.is_mouse_down(right) && !editor.editor_3d_camera_view();
    if looking {
        let axis = |positive: dear_imgui_rs::Key, negative: dear_imgui_rs::Key| {
            f32::from(ui.is_key_down(positive)) - f32::from(ui.is_key_down(negative))
        };
        let mouse_delta = editor.editor_mouse_delta(ui.io().mouse_delta());
        editor.control_editor_3d(
            mouse_delta,
            [
                axis(dear_imgui_rs::Key::D, dear_imgui_rs::Key::A),
                axis(dear_imgui_rs::Key::E, dear_imgui_rs::Key::Q),
                axis(dear_imgui_rs::Key::W, dear_imgui_rs::Key::S),
            ],
            ui.io().delta_time(),
            ui.io().key_shift(),
        );
        image::wrap_pointer(
            ui,
            editor,
            min,
            [min[0] + available[0], min[1] + available[1]],
        );
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    } else if keyboard {
        editor.control_editor_3d([0.0; 2], [0.0; 3], 0.0, false);
    }

    ui.set_cursor_screen_pos([min[0] + 8.0, min[1] + 8.0]);
    if camera_button(ui, editor.editor_3d_camera_view()) {
        editor.toggle_editor_3d_camera_view();
    }
    ui.same_line();
    if reset_camera_button(ui) {
        editor.reset_editor_3d_camera_transform();
    }

    ui.set_cursor_screen_pos([min[0] + available[0] - 112.0, min[1] + 8.0]);
    for (axis, label, color) in [
        (0, "X", [1.0, 0.3, 0.3, 1.0]),
        (1, "Y", [0.3, 1.0, 0.3, 1.0]),
        (2, "Z", [0.35, 0.55, 1.0, 1.0]),
    ] {
        if axis > 0 {
            ui.same_line();
        }
        ui.set_next_item_allow_overlap();
        let enabled = editor.editor_3d_axes()[axis];
        let _color = ui.push_style_color(
            dear_imgui_rs::StyleColor::Text,
            if enabled {
                color
            } else {
                [0.45, 0.45, 0.45, 1.0]
            },
        );
        if ui.button_with_size(label, [32.0, ui.frame_height()]) {
            editor.toggle_editor_3d_axis(axis);
        }
    }
}

fn draw_camera_mask(ui: &dear_imgui_rs::Ui, min: [f32; 2], size: [f32; 2], camera_aspect: f32) {
    let size = [size[0].max(1.0), size[1].max(1.0)];
    let max = [min[0] + size[0], min[1] + size[1]];
    let draw = ui.get_window_draw_list();
    let _clip = draw.push_clip_rect(min, max, true);
    let [rect_min, rect_max] = camera_frame(min, size, camera_aspect);
    for [shade_min, shade_max] in [
        [min, [max[0], rect_min[1]]],
        [[min[0], rect_max[1]], max],
        [[min[0], rect_min[1]], [rect_min[0], rect_max[1]]],
        [[rect_max[0], rect_min[1]], [max[0], rect_max[1]]],
    ] {
        draw.add_rect(shade_min, shade_max, [0.0, 0.0, 0.0, 0.25])
            .filled(true)
            .build();
    }
    draw.add_rect(rect_min, rect_max, [0.7, 0.7, 0.7, 1.0])
        .thickness(1.0)
        .build();
}

fn camera_frame(min: [f32; 2], size: [f32; 2], camera_aspect: f32) -> [[f32; 2]; 2] {
    let editor_aspect = size[0].max(1.0) / size[1].max(1.0);
    let (width, height) = if editor_aspect >= camera_aspect {
        (size[0] * camera_aspect / editor_aspect, size[1])
    } else {
        (size[0], size[1] * editor_aspect / camera_aspect)
    };
    let left = min[0] + (size[0] - width) * 0.5;
    let top = min[1] + (size[1] - height) * 0.5;
    [[left, top], [left + width, top + height]]
}

fn draw_editor_2d(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    let min = ui.cursor_screen_pos();
    let available = ui.content_region_avail();
    let background = editor.editor_2d_background();
    let editor_image = image::editor(editor);
    if let Some(point) = image::draw_editor(ui, editor_image, available, editor, background) {
        editor.select_at_editor_2d(point);
    }
    if editor.editor_2d_camera_view() {
        let canvas_size = editor.editor_2d_canvas_size();
        let canvas_aspect = canvas_size.0.max(1) as f32 / canvas_size.1.max(1) as f32;
        draw_camera_mask(ui, min, available, canvas_aspect);
    }
    ui.set_cursor_screen_pos([min[0] + 8.0, min[1] + 8.0]);
    if camera_button(ui, editor.editor_2d_camera_view()) {
        editor.toggle_editor_2d_camera_view();
    }
    ui.same_line();
    if reset_camera_button(ui) {
        editor.reset_editor_2d_camera_transform();
    }
    if let Some(error) = editor.render_error() {
        ui.text_wrapped(error);
    }
}

fn camera_button(ui: &dear_imgui_rs::Ui, camera_view: bool) -> bool {
    ui.set_next_item_allow_overlap();
    let clicked = controls::text_button_colored(
        ui,
        icons::VIDEO_CAMERA,
        [ui.frame_height(); 2],
        if camera_view {
            dear_imgui_rs::StyleColor::Text
        } else {
            dear_imgui_rs::StyleColor::TextDisabled
        },
    );
    if ui.is_item_hovered() {
        ui.tooltip_text(if camera_view {
            "Use the free editor camera [0]"
        } else {
            "Use the canvas camera [0]"
        });
    }
    clicked
}

fn reset_camera_button(ui: &dear_imgui_rs::Ui) -> bool {
    let clicked = controls::text_button(ui, icons::RESET, [ui.frame_height(); 2]);
    if ui.is_item_hovered() {
        ui.tooltip_text("Reset camera transform [R]");
    }
    clicked
}

fn shortcut_mode(ui: &dear_imgui_rs::Ui, enabled: bool) -> Option<Mode> {
    if !enabled {
        return None;
    }
    if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::Key1, false)
        || ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::Keypad1, false)
    {
        Some(Mode::Preview)
    } else if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::Key2, false)
        || ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::Keypad2, false)
    {
        Some(Mode::Two)
    } else if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::Key3, false)
        || ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::Keypad3, false)
    {
        Some(Mode::Three)
    } else {
        None
    }
}

fn tab_flags(selected: bool) -> dear_imgui_rs::TabItemFlags {
    if selected {
        dear_imgui_rs::TabItemFlags::SET_SELECTED
    } else {
        dear_imgui_rs::TabItemFlags::NONE
    }
}

pub(super) fn draw_fullscreen(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    fullscreen::draw(editor, ui)
}

#[cfg(test)]
mod tests {
    use super::camera_frame;

    #[test]
    fn camera_frame_keeps_the_canvas_aspect_inside_a_dynamic_view() {
        let wide_camera = camera_frame([10.0, 20.0], [1600.0, 1000.0], 16.0 / 9.0);
        assert_eq!([wide_camera[0][0], wide_camera[1][0]], [10.0, 1610.0]);
        assert!((wide_camera[0][1] - 70.0).abs() < 0.001);
        assert!((wide_camera[1][1] - 970.0).abs() < 0.001);
        assert_eq!(
            camera_frame([0.0, 0.0], [2000.0, 1000.0], 1.0),
            [[500.0, 0.0], [1500.0, 1000.0]]
        );
    }
}
