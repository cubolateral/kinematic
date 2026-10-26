use crate::editor::Timeline;
use crate::ui::inputs::{self, Action as InputAction};
use crate::ui::widgets::text_size;
use crate::ui::{controls, icons};

use super::{
    metrics::{BUTTON_SIZE, FULLSCREEN_SCRUBBER_HEIGHT, FULLSCREEN_SCRUBBER_THICKNESS},
    ruler::format_time,
};

pub(super) fn draw(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    fps: f32,
    interactive: bool,
    is_fullscreen: bool,
    volume: &mut i32,
) -> bool {
    let is_playing = timeline.is_playing();
    let spacing = unsafe { ui.style().item_spacing() }[0];
    let width = transport_width(ui) + spacing + BUTTON_SIZE;

    ui.set_cursor_pos_x(
        ui.cursor_pos_x() + ((ui.content_region_avail_width() - width) * 0.5).max(0.0),
    );

    transport_controls(timeline, ui, interactive, volume);

    ui.same_line();

    let toggle_fullscreen = fullscreen_button(ui, is_fullscreen);

    ui.same_line();

    ui.text(if !interactive {
        "EXPORTING".to_owned()
    } else if is_playing {
        format!("FPS: {fps:.2}")
    } else {
        "PAUSED".to_owned()
    });

    toggle_fullscreen
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

        draw_fullscreen_scrubber(timeline, ui, scrubber_min, scrubber_width);

        response
    }
}

fn transport_controls(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    interactive: bool,
    volume: &mut i32,
) {
    let is_playing = timeline.is_playing();

    if transport_button(
        ui,
        icons::BACKWARD_FAST,
        "Go to start",
        InputAction::GoToStart,
    ) && interactive
    {
        timeline.go_to_start();
    }

    ui.same_line();

    if transport_button(
        ui,
        icons::BACKWARD,
        "Previous frame",
        InputAction::PreviousFrame,
    ) && interactive
    {
        timeline.previous_frame();
    }

    ui.same_line();

    if transport_button(
        ui,
        if is_playing {
            icons::PAUSE
        } else {
            icons::PLAY
        },
        if is_playing { "Pause" } else { "Play" },
        InputAction::PlayPause,
    ) && interactive
    {
        timeline.toggle();
    }

    ui.same_line();

    if transport_button(ui, icons::FORWARD, "Next frame", InputAction::NextFrame) && interactive {
        timeline.next_frame();
    }

    ui.same_line();

    if transport_button(ui, icons::FORWARD_FAST, "Go to end", InputAction::GoToEnd) && interactive {
        timeline.go_to_end();
    }

    ui.same_line();

    volume_button(ui, volume);

    ui.same_line();

    let clicked = controls::text_button_colored(
        ui,
        icons::RETWEET,
        [BUTTON_SIZE; 2],
        if timeline.loop_enabled() {
            dear_imgui_rs::StyleColor::Text
        } else {
            dear_imgui_rs::StyleColor::TextDisabled
        },
    );

    if ui.is_item_hovered() {
        inputs::tooltip(ui, InputAction::ToggleLoop, "Toggle loop");
    }

    if clicked && interactive {
        timeline.toggle_loop();
    }
}

fn volume_button(ui: &dear_imgui_rs::Ui, volume: &mut i32) {
    if controls::text_button(ui, icons::VOLUME_HIGH, [BUTTON_SIZE; 2]) {
        ui.open_popup("Preview volume");
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("Volume: {volume}%."));
    }
    if let Some(_popup) = ui.begin_popup("Preview volume") {
        ui.set_next_item_width(140.0);
        ui.slider_config("##Volume", 0, 100)
            .display_format(
                dear_imgui_rs::NumericFormat::new("%d%%").expect("Valid volume format."),
            )
            .build(volume);
    }
}

fn transport_width(ui: &dear_imgui_rs::Ui) -> f32 {
    let spacing = unsafe { ui.style().item_spacing() }[0];

    BUTTON_SIZE * 7.0 + spacing * 6.0
}

fn fullscreen_button(ui: &dear_imgui_rs::Ui, is_fullscreen: bool) -> bool {
    transport_button(
        ui,
        if is_fullscreen {
            icons::COMPRESS
        } else {
            icons::EXPAND
        },
        if is_fullscreen {
            "Exit fullscreen"
        } else {
            "Enter fullscreen"
        },
        InputAction::ToggleFullscreen,
    )
}

fn draw_fullscreen_scrubber(
    timeline: &Timeline,
    ui: &dear_imgui_rs::Ui,
    min: [f32; 2],
    width: f32,
) {
    let duration = timeline.duration();
    let ratio = if duration > 0.0 {
        (timeline.time() / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let progress_x = min[0] + width * ratio;
    let line_y = min[1] + FULLSCREEN_SCRUBBER_HEIGHT * 0.5;
    let draw_list = ui.get_window_draw_list();
    let current = format_time(timeline.time(), 0);
    let duration = format_time(duration, 0);
    let current_size = text_size(ui, &current);
    let duration_size = text_size(ui, &duration);
    let current_x =
        (progress_x - current_size[0] * 0.5).clamp(min[0], min[0] + width - current_size[0]);
    let duration_x = min[0] + width - duration_size[0];

    draw_list.add_text(
        [current_x, line_y + 4.0],
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        &current,
    );

    draw_list.add_line_h(
        min[0],
        min[0] + width,
        line_y,
        ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg),
        FULLSCREEN_SCRUBBER_THICKNESS,
    );

    draw_list.add_line_h(
        min[0],
        progress_x,
        line_y,
        ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
        FULLSCREEN_SCRUBBER_THICKNESS,
    );

    draw_list
        .add_circle(
            [progress_x, line_y],
            FULLSCREEN_SCRUBBER_THICKNESS,
            ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
        )
        .filled(true)
        .build();

    draw_list.add_text(
        [duration_x, line_y + 4.0],
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        &duration,
    );
}

fn time_at_position(position: f32, start: f32, width: f32, duration: f32) -> f32 {
    if width <= 0.0 || duration <= 0.0 {
        return 0.0;
    }

    ((position - start) / width).clamp(0.0, 1.0) * duration
}

fn transport_button(
    ui: &dear_imgui_rs::Ui,
    label: &str,
    tooltip: &str,
    action: InputAction,
) -> bool {
    let clicked = controls::text_button(ui, label, [BUTTON_SIZE; 2]);

    if ui.is_item_hovered() {
        inputs::tooltip(ui, action, tooltip);
    }

    clicked
}

#[cfg(test)]
mod tests {
    use super::time_at_position;

    #[test]
    fn time_position_is_clamped_to_the_duration() {
        assert_eq!(time_at_position(-10.0, 0.0, 100.0, 5.0), 0.0);
        assert_eq!(time_at_position(50.0, 0.0, 100.0, 5.0), 2.5);
        assert_eq!(time_at_position(110.0, 0.0, 100.0, 5.0), 5.0);
    }

    #[test]
    fn invalid_time_ranges_resolve_to_the_start() {
        assert_eq!(time_at_position(10.0, 0.0, 0.0, 5.0), 0.0);
        assert_eq!(time_at_position(10.0, 0.0, 100.0, 0.0), 0.0);
    }
}
