use crate::editor::{Editor, Timeline};
use crate::ui::inputs::{self, Action as InputAction};
use kinematic_ui::timeline::transport::{FULLSCREEN_SCRUBBER_HEIGHT, time_at_position};
use kinematic_ui::timeline::{ruler::format_timecode, transport};

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    interactive: bool,
    is_fullscreen: bool,
    volume: &mut i32,
) -> bool {
    use kinematic_ui::timeline::transport::{self, Action};

    let fps_code = editor.project_info().1.fps;
    let time_labels = [
        format_timecode(editor.absolute_time(), fps_code),
        format_timecode(editor.project_duration(), fps_code),
    ];
    let action = {
        let timeline = editor.timeline_mut();
        let playing = timeline.is_playing();
        let bindings = inputs::map();
        let mut speed = timeline.playback_speed();
        let action = transport::draw(
            ui,
            transport::State {
                playing,
                looping: timeline.loop_enabled(),
                fullscreen: is_fullscreen,
                interactive,
                time_labels: [&time_labels[0], &time_labels[1]],
                shortcuts: [
                    InputAction::GoToStart,
                    InputAction::PreviousFrame,
                    InputAction::PlayPause,
                    InputAction::NextFrame,
                    InputAction::GoToEnd,
                    InputAction::ToggleLoop,
                    InputAction::ToggleFullscreen,
                ]
                .map(|action| bindings.binding(action).display.as_str()),
            },
            volume,
            &mut speed,
        );

        timeline.set_playback_speed(speed);

        match action {
            Some(Action::Start) => timeline.go_to_start(),
            Some(Action::Previous) => timeline.previous_frame(),
            Some(Action::Toggle) => timeline.toggle(),
            Some(Action::Next) => timeline.next_frame(),
            Some(Action::End) => timeline.go_to_end(),
            Some(Action::Loop) => timeline.toggle_loop(),
            _ => (),
        }

        action
    };

    if action == Some(Action::Screenshot) {
        editor.request_screenshot();
    }

    action == Some(Action::Fullscreen)
}

pub(in crate::ui) fn shortcuts(timeline: &mut Timeline, ui: &dear_imgui_rs::Ui, interactive: bool) {
    let io = ui.io();

    if !interactive || io.want_text_input() || ui.is_any_item_active() {
        return;
    }

    if inputs::active(ui, InputAction::GoToStart) {
        timeline.go_to_start();
    } else if inputs::active(ui, InputAction::PreviousFrame) {
        timeline.previous_frame();
    }

    if inputs::active(ui, InputAction::ToggleLoop) {
        timeline.toggle_loop();
    }

    if inputs::active(ui, InputAction::PlayPause) {
        timeline.toggle();
    }

    if inputs::active(ui, InputAction::GoToEnd) {
        timeline.go_to_end();
    } else if inputs::active(ui, InputAction::NextFrame) {
        timeline.next_frame();
    }
}

pub(in crate::ui) fn fullscreen_controls(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    interactive: bool,
    volume: &mut i32,
) -> bool {
    let scrubber_min = ui.cursor_screen_pos();
    let scrubber_width = ui.content_region_avail_width().max(1.0);
    let available_height = ui.content_region_avail_height();
    let group_height = FULLSCREEN_SCRUBBER_HEIGHT + transport::BUTTON_SIZE;
    let group_top = scrubber_min[1] + ((available_height - group_height) * 0.5).max(0.0);
    let buttons_y = group_top + FULLSCREEN_SCRUBBER_HEIGHT;
    let panel_bottom = scrubber_min[1] + ui.content_region_avail_height();
    let display_time = editor.absolute_time();
    let display_duration = editor.project_duration();

    {
        let _disabled = ui.begin_disabled_with_cond(!interactive);

        ui.set_cursor_screen_pos([scrubber_min[0], buttons_y]);

        let response = draw(editor, ui, interactive, true, volume);
        let timeline = editor.timeline_mut();
        let mouse = ui.io().mouse_pos();
        let panel_hovered = ui.is_mouse_hovering_rect(
            scrubber_min,
            [scrubber_min[0] + scrubber_width, panel_bottom],
        );
        let blocked_by_button = ui.is_any_item_hovered()
            || ui.is_popup_open_with_flags("", dear_imgui_rs::PopupQueryFlags::ANY_POPUP);
        let left = dear_imgui_rs::MouseButton::Left;

        if !interactive {
            timeline.is_controlling = false;
        } else if timeline.is_controlling {
            if ui.is_mouse_down(left) {
                timeline.go_to(time_at_position(
                    mouse[0],
                    scrubber_min[0],
                    scrubber_width,
                    timeline.duration(),
                ));
            } else {
                timeline.is_controlling = false;
            }
        } else if panel_hovered && !blocked_by_button && ui.is_mouse_clicked(left) {
            timeline.is_controlling = true;

            timeline.go_to(time_at_position(
                mouse[0],
                scrubber_min[0],
                scrubber_width,
                timeline.duration(),
            ));
        }

        transport::scrubber(
            display_time,
            display_duration,
            ui,
            [scrubber_min[0], group_top],
            scrubber_width,
        );

        response
    }
}
