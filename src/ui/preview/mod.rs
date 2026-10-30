use crate::ui::inputs::{self, Action as InputAction};
use kinematic_ui::widgets;
mod fullscreen;
mod image;
mod playback;
mod state;

use crate::editor::Editor;

use kinematic_ui::canvas::{
    camera_frame, camera_mask as draw_camera_mask, project_outline as draw_project_outline,
};
use state::Mode;

pub(super) const WINDOW_NAME: &str = "Preview";

pub(in crate::ui) use playback::shortcuts;
pub(super) use state::State;

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    keyboard_enabled: bool,
    volume: &mut i32,
) -> bool {
    let mut toggle_fullscreen = false;
    let is_exporting = editor.is_exporting();

    kinematic_ui::windows::viewport_panel(ui, WINDOW_NAME, || {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let plain_keyboard_input = keyboard_enabled
            && !is_exporting
            && !ui.io().want_text_input()
            && !ui.is_any_item_active();
        let requested =
            shortcut_mode(ui, plain_keyboard_input).or_else(|| state.take_requested_mode());

        if let Some(mode) = requested {
            state.set_mode(mode);
        }

        if plain_keyboard_input && inputs::active(ui, InputAction::ToggleMargin) {
            editor.toggle_margin();
        }

        let reset = plain_keyboard_input && inputs::active(ui, InputAction::ToggleCamera);

        if reset {
            match state.mode() {
                Mode::Two => editor.toggle_editor_2d_camera_view(),
                Mode::Three => editor.toggle_editor_3d_camera_view(),
                Mode::Preview => state.view = kinematic_ui::canvas::View::default(),
            }
        }

        let reset_camera = plain_keyboard_input && inputs::active(ui, InputAction::ResetCamera);

        if reset_camera {
            match state.mode() {
                Mode::Two => editor.reset_editor_2d_camera_transform(),
                Mode::Three => editor.reset_editor_3d_camera_transform(),
                Mode::Preview => {}
            }
        }

        kinematic_ui::windows::viewport_body(ui, || {
            let Some(_tabs) = ui.tab_bar("Preview modes") else {
                return;
            };

            let preview_tab = ui.tab_item_with_flags(
                "PREVIEW",
                None,
                tab_flags(requested == Some(Mode::Preview)),
            );

            if ui.is_item_hovered() {
                inputs::tooltip(ui, InputAction::Preview, "Show the final exported view");
            }

            if let Some(_tab) = preview_tab
                && requested.is_none_or(|mode| mode == Mode::Preview)
            {
                state.set_mode(Mode::Preview);

                let min = ui.cursor_screen_pos();
                let available = ui.content_region_avail();
                let margin = editor.margin_visible().then(|| editor.margin_size());

                if image::draw_navigable(
                    ui,
                    image::preview(editor),
                    available,
                    margin,
                    &mut state.view,
                ) {
                    editor.clear_selection();
                }
                if kinematic_ui::canvas::preview_tools(
                    ui,
                    min,
                    &mut state.view,
                    editor.margin_visible(),
                    [
                        inputs::map()
                            .binding(InputAction::ToggleCamera)
                            .display
                            .as_str(),
                        inputs::map()
                            .binding(InputAction::ToggleMargin)
                            .display
                            .as_str(),
                    ],
                ) {
                    editor.toggle_margin();
                }
                if let Some(error) = editor.render_error() {
                    ui.set_cursor_screen_pos([min[0] + 8.0, min[1] + ui.frame_height() + 16.0]);
                    widgets::message(ui, error);
                }
            }

            let editor_2d_tab =
                ui.tab_item_with_flags("2D", None, tab_flags(requested == Some(Mode::Two)));

            if ui.is_item_hovered() {
                inputs::tooltip(ui, InputAction::Edit2d, "Edit the selected Canvas2D");
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
                inputs::tooltip(ui, InputAction::Edit3d, "Edit the selected Canvas3D");
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
        kinematic_ui::timeline::ruler::duration_bar(
            ui,
            editor.absolute_time(),
            editor.project_duration(),
        );
        let fps = editor.preview_fps();
        toggle_fullscreen =
            playback::draw(editor.timeline_mut(), ui, fps, !is_exporting, false, volume);
    });
    toggle_fullscreen
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

        if editor.margin_visible() {
            let [a, b] = camera_frame(min, available, canvas_aspect);
            let size = editor.editor_3d_canvas_camera().1;

            kinematic_ui::canvas::margin(
                ui,
                [a, b],
                [size.0.max(1) as f32, size.1.max(1) as f32],
                editor.margin_size(),
            );
        }
    }

    let right = dear_imgui_rs::MouseButton::Right;
    let looking = viewport_active && ui.is_mouse_down(right);

    if looking {
        let axis = |positive: InputAction, negative: InputAction| {
            f32::from(inputs::active(ui, positive)) - f32::from(inputs::active(ui, negative))
        };
        let mouse_delta = if ui.is_mouse_clicked(right) {
            [0.0; 2]
        } else {
            editor.editor_mouse_delta(ui.io().mouse_delta())
        };

        editor.control_editor_3d(
            mouse_delta,
            [
                axis(InputAction::MoveRight, InputAction::MoveLeft),
                axis(InputAction::MoveUp, InputAction::MoveDown),
                axis(InputAction::MoveForward, InputAction::MoveBackward),
            ],
            ui.io().delta_time(),
            inputs::active(ui, InputAction::CameraFast),
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

    let tools = kinematic_ui::canvas::tools(
        ui,
        min,
        available,
        editor.editor_3d_camera_view(),
        editor.margin_visible(),
        &editor.editor_3d_axes(),
        camera_shortcuts(),
    );
    if tools.camera {
        editor.toggle_editor_3d_camera_view();
    }
    if tools.reset {
        editor.reset_editor_3d_camera_transform();
    }
    if tools.margin {
        editor.toggle_margin();
    }
    if let Some(axis) = tools.axis {
        editor.toggle_editor_3d_axis(axis);
    }
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

        draw_project_outline(ui, min, available, canvas_aspect);
    }

    let tools = kinematic_ui::canvas::tools(
        ui,
        min,
        available,
        editor.editor_2d_camera_view(),
        editor.margin_visible(),
        &editor.editor_2d_axes(),
        camera_shortcuts(),
    );
    if tools.camera {
        editor.toggle_editor_2d_camera_view();
    }
    if tools.reset {
        editor.reset_editor_2d_camera_transform();
    }
    if tools.margin {
        editor.toggle_margin();
    }
    if let Some(axis) = tools.axis {
        editor.toggle_editor_2d_axis(axis);
    }
    if let Some(error) = editor.render_error() {
        widgets::message(ui, error);
    }
}

fn camera_shortcuts() -> [&'static str; 3] {
    [
        InputAction::ToggleCamera,
        InputAction::ResetCamera,
        InputAction::ToggleMargin,
    ]
    .map(|action| inputs::map().binding(action).display.as_str())
}

fn shortcut_mode(ui: &dear_imgui_rs::Ui, enabled: bool) -> Option<Mode> {
    if !enabled {
        return None;
    }

    if inputs::active(ui, InputAction::Preview) {
        Some(Mode::Preview)
    } else if inputs::active(ui, InputAction::Edit2d) {
        Some(Mode::Two)
    } else if inputs::active(ui, InputAction::Edit3d) {
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

pub(super) fn draw_fullscreen(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    volume: &mut i32,
) -> bool {
    fullscreen::draw(editor, ui, volume)
}
