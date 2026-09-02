use crate::editor::Timeline;

use super::metrics::{BUTTON_SIZE, FULLSCREEN_SCRUBBER_HEIGHT, FULLSCREEN_SCRUBBER_THICKNESS};

pub(super) fn draw(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    fps: f32,
    interactive: bool,
    is_fullscreen: bool,
) -> bool {
    let is_playing = timeline.is_playing();
    let spacing = unsafe { ui.style().item_spacing() }[0];
    let width = transport_width(ui) + spacing + BUTTON_SIZE;

    ui.set_cursor_pos_x(
        ui.cursor_pos_x() + ((ui.content_region_avail_width() - width) * 0.5).max(0.0),
    );

    transport_controls(timeline, ui, interactive);

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
    if !interactive {
        return;
    }

    let shift = ui.io().key_shift();

    if ui.is_key_pressed(dear_imgui_rs::Key::LeftArrow) {
        if shift {
            timeline.go_to_start();
        } else {
            timeline.previous_frame();
        }
    }

    if ui.is_key_pressed(dear_imgui_rs::Key::Space) {
        timeline.toggle();
    }

    if ui.is_key_pressed(dear_imgui_rs::Key::RightArrow) {
        if shift {
            timeline.go_to_end();
        } else {
            timeline.next_frame();
        }
    }
}

pub(in crate::ui) fn fullscreen_controls(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    fps: f32,
    interactive: bool,
) -> bool {
    let scrubber_min = ui.cursor_screen_pos();
    let scrubber_width = ui.content_region_avail_width().max(1.0);
    let buttons_y = scrubber_min[1] + FULLSCREEN_SCRUBBER_HEIGHT + 7.0;

    {
        let _disabled = ui.begin_disabled_with_cond(!interactive);

        ui.invisible_button(
            "Fullscreen scrubber.",
            [scrubber_width, FULLSCREEN_SCRUBBER_HEIGHT],
        );

        timeline.is_controlling = interactive && ui.is_item_active();
        if timeline.is_controlling {
            timeline.go_to(time_at_position(
                ui.io().mouse_pos()[0],
                scrubber_min[0],
                scrubber_width,
                timeline.get_duration(),
            ));
        }

        draw_fullscreen_scrubber(timeline, ui, scrubber_min, scrubber_width);

        ui.set_cursor_screen_pos([scrubber_min[0], buttons_y]);
        draw(timeline, ui, fps, interactive, true)
    }
}

fn transport_controls(timeline: &mut Timeline, ui: &dear_imgui_rs::Ui, interactive: bool) {
    let is_playing = timeline.is_playing();

    if transport_button(ui, "<<", "Go to start [Shift + LeftArrow]") && interactive {
        timeline.go_to_start();
    }

    ui.same_line();

    if transport_button(ui, "<", "Previous frame [LeftArrow]") && interactive {
        timeline.previous_frame();
    }

    ui.same_line();

    if transport_button(
        ui,
        if is_playing { "||" } else { "|>" },
        if is_playing {
            "Pause [Space]"
        } else {
            "Play [Space]"
        },
    ) && interactive
    {
        timeline.toggle();
    }

    ui.same_line();

    if transport_button(ui, ">", "Next frame [RightArrow]") && interactive {
        timeline.next_frame();
    }

    ui.same_line();

    if transport_button(ui, ">>", "Go to end [Shift + RightArrow]") && interactive {
        timeline.go_to_end();
    }
}

fn transport_width(ui: &dear_imgui_rs::Ui) -> f32 {
    let spacing = unsafe { ui.style().item_spacing() }[0];

    BUTTON_SIZE * 5.0 + spacing * 4.0
}

fn fullscreen_button(ui: &dear_imgui_rs::Ui, is_fullscreen: bool) -> bool {
    transport_button(
        ui,
        "[]",
        if is_fullscreen {
            "Exit fullscreen [F]"
        } else {
            "Enter fullscreen [F]"
        },
    )
}

fn draw_fullscreen_scrubber(
    timeline: &Timeline,
    ui: &dear_imgui_rs::Ui,
    min: [f32; 2],
    width: f32,
) {
    let duration = timeline.get_duration();
    let ratio = if duration > 0.0 {
        (timeline.get_time() / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let progress_x = min[0] + width * ratio;
    let line_y = min[1] + FULLSCREEN_SCRUBBER_HEIGHT * 0.5;
    let draw_list = ui.get_window_draw_list();

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
}

fn time_at_position(position: f32, start: f32, width: f32, duration: f32) -> f32 {
    if width <= 0.0 || duration <= 0.0 {
        return 0.0;
    }

    ((position - start) / width).clamp(0.0, 1.0) * duration
}

fn transport_button(ui: &dear_imgui_rs::Ui, label: &str, tooltip: &str) -> bool {
    let clicked = ui.button_config(label).size([BUTTON_SIZE; 2]).build();

    if ui.is_item_hovered() {
        ui.tooltip_text(tooltip);
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
