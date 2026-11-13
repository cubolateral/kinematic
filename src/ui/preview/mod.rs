use crate::ui::inputs::{self, Action as InputAction};
use kinematic_ui::widgets;
mod fullscreen;
mod image;
mod navigation;
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
    workspace: &mut super::workspace::State,
) -> bool {
    let mut toggle_fullscreen = false;
    let is_exporting = editor.is_exporting();

    kinematic_ui::windows::viewport_panel(ui, WINDOW_NAME, || {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let plain_keyboard_input = keyboard_enabled
            && !is_exporting
            && !ui.io().want_text_input()
            && !ui.is_any_item_active()
            && !ui.is_popup_open_with_flags("", dear_imgui_rs::PopupQueryFlags::ANY_POPUP);
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
                Mode::Preview => state.view = kinematic_ui::canvas::View::default(),
            }
        }

        let modes = [Mode::Preview, Mode::Two, Mode::Three];
        let labels = [
            format!("{} Preview", kinematic_ui::icons::FILM),
            format!("{} 2D", kinematic_ui::icons::SQUARE),
            format!("{} 3D", kinematic_ui::icons::CUBE),
        ];
        let labels = labels.each_ref().map(String::as_str);
        let selected = modes.iter().position(|mode| *mode == state.mode()).unwrap();

        let tooltips = [
            ("Show the final exported view", "F1"),
            ("Edit the selected Canvas2D", "F2"),
            ("Edit the selected Canvas3D", "F3"),
        ];
        let (selected, toggle) = kinematic_ui::docking::centered_header(
            ui,
            selected,
            &labels,
            kinematic_ui::icons::LAYOUT_SIDEBAR_LEFT,
            &tooltips,
        );

        if let Some(index) = selected {
            state.set_mode(modes[index]);
        }
        if toggle {
            workspace.toggle();
        }
        if ui.is_item_hovered() {
            inputs::tooltip(ui, InputAction::TogglePanels, "Toggle sidebars");
        }

        kinematic_ui::windows::viewport_body(ui, || match state.mode() {
            Mode::Preview => {
                let min = ui.cursor_screen_pos();
                let available = ui.content_region_avail();
                let margin = editor.margin_visible().then(|| editor.margin_size());

                let delta = editor.editor_mouse_delta(ui.io().mouse_delta());
                let (clicked, dragging) = image::draw_navigable(
                    ui,
                    image::preview(editor),
                    available,
                    margin,
                    &mut state.view,
                    &mut state.displayed,
                    delta,
                );

                if clicked {
                    editor.clear_selection();
                }

                if dragging {
                    image::wrap_pointer(
                        ui,
                        editor,
                        min,
                        [
                            min[0] + available[0].max(1.0),
                            min[1] + available[1].max(1.0),
                        ],
                    );
                }

                let mut qualities = editor.viewport_qualities();
                let mut quality = quality_index(qualities.preview);

                if kinematic_ui::canvas::preview_tools(
                    ui,
                    min,
                    &mut state.view,
                    editor.margin_visible(),
                    [
                        inputs::map()
                            .binding(InputAction::ResetCamera)
                            .display
                            .as_str(),
                        inputs::map()
                            .binding(InputAction::ToggleMargin)
                            .display
                            .as_str(),
                    ],
                    &mut quality,
                    &crate::editor::PreviewQuality::LABELS,
                ) {
                    editor.toggle_margin();
                }

                qualities.preview = crate::editor::PreviewQuality::ALL[quality];
                editor.set_viewport_qualities(qualities);

                if let Some(error) = editor.render_error() {
                    ui.set_cursor_screen_pos([min[0] + 8.0, min[1] + ui.frame_height() + 16.0]);

                    widgets::message(ui, error);
                }
            }

            Mode::Two => {
                let canvas = editor.editor_2d_canvas();

                if state.sync_canvas_2d(editor.active_scene_index(), canvas) {
                    editor.reset_editor_2d_view();
                }

                draw_editor_2d(editor, ui);
            }

            Mode::Three => {
                let canvas = editor.editor_3d_canvas();

                if state.sync_canvas_3d(editor.active_scene_index(), canvas) {
                    editor.reset_editor_3d_view();
                }

                draw_editor_3d(editor, ui, plain_keyboard_input, &mut state.orientation);
            }
        });
        kinematic_ui::timeline::ruler::duration_bar(
            ui,
            editor.absolute_time(),
            editor.project_duration(),
        );

        ui.set_cursor_screen_pos([ui.cursor_screen_pos()[0], ui.item_rect_max()[1]]);

        toggle_fullscreen = playback::draw(editor, ui, !is_exporting, false, volume);
    });

    toggle_fullscreen
}

fn draw_editor_3d(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    keyboard: bool,
    orientation: &mut kinematic_ui::canvas::orientation::State,
) {
    let min = ui.cursor_screen_pos();
    let available = ui.content_region_avail();
    let editor_image = image::editor_3d(editor);
    let (viewport_active, hovered, clicked) =
        image::draw_editor_3d(ui, editor_image, available, editor);

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

    navigation::control(
        editor,
        ui,
        keyboard,
        viewport_active,
        hovered,
        min,
        available,
    );
    navigation::tools(editor, ui, min);
    navigation::gizmo(editor, ui, min, available, orientation);
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

    let mut qualities = editor.viewport_qualities();
    let mut quality = quality_index(qualities.editor_2d);
    let tools = kinematic_ui::canvas::tools(
        ui,
        min,
        editor.editor_2d_camera_view(),
        editor.margin_visible(),
        &editor.editor_2d_axes(),
        camera_shortcuts(),
        &[],
        &mut quality,
        &crate::editor::PreviewQuality::LABELS,
    );

    qualities.editor_2d = crate::editor::PreviewQuality::ALL[quality];
    editor.set_viewport_qualities(qualities);

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

fn quality_index(quality: crate::editor::PreviewQuality) -> usize {
    crate::editor::PreviewQuality::ALL
        .iter()
        .position(|value| *value == quality)
        .unwrap()
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

pub(super) fn draw_fullscreen(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    volume: &mut i32,
) -> bool {
    fullscreen::draw(editor, ui, volume)
}
