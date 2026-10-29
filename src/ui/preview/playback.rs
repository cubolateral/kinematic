use crate::editor::Timeline;
use crate::ui::inputs::{self, Action as InputAction};
use kinematic_ui::timeline::transport::{self, FULLSCREEN_SCRUBBER_HEIGHT, time_at_position};

pub(super) fn draw(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    fps: f32,
    interactive: bool,
    is_fullscreen: bool,
    volume: &mut i32,
) -> bool {
    use kinematic_ui::timeline::transport::{self, Action};
    let playing = timeline.is_playing();
    let status = if !interactive {
        "EXPORTING".to_owned()
    } else if playing {
        format!("FPS: {fps:.2}")
    } else {
        "PAUSED".to_owned()
    };
    let bindings = inputs::map();
    let action = transport::draw(
        ui,
        transport::State {
            playing,
            looping: timeline.loop_enabled(),
            fullscreen: is_fullscreen,
            interactive,
            status: &status,
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
    );
    match action {
        Some(Action::Start) => timeline.go_to_start(),
        Some(Action::Previous) => timeline.previous_frame(),
        Some(Action::Toggle) => timeline.toggle(),
        Some(Action::Next) => timeline.next_frame(),
        Some(Action::End) => timeline.go_to_end(),
        Some(Action::Loop) => timeline.toggle_loop(),
        _ => (),
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
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    fps: f32,
    interactive: bool,
    volume: &mut i32,
) -> bool {
    let scrubber_min = ui.cursor_screen_pos();
    let scrubber_width = ui.content_region_avail_width().max(1.0);
    let buttons_y = scrubber_min[1] + FULLSCREEN_SCRUBBER_HEIGHT + 18.0;
    let panel_bottom = scrubber_min[1] + ui.content_region_avail_height();

    {
        let _disabled = ui.begin_disabled_with_cond(!interactive);

        ui.set_cursor_screen_pos([scrubber_min[0], buttons_y]);

        let response = draw(timeline, ui, fps, interactive, true, volume);

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
            timeline.time(),
            timeline.duration(),
            ui,
            scrubber_min,
            scrubber_width,
        );

        response
    }
}
