//! Playback presentation. The caller applies returned transport actions.

use super::format_time;
use crate::{
    controls, icons,
    widgets::{self, text_size},
};

pub const BUTTON_SIZE: f32 = 25.0;
pub const FULLSCREEN_SCRUBBER_HEIGHT: f32 = 12.0;
const FULLSCREEN_SCRUBBER_THICKNESS: f32 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start,
    Previous,
    Toggle,
    Next,
    End,
    Loop,
    Fullscreen,
}

pub struct State<'a> {
    pub playing: bool,
    pub looping: bool,
    pub fullscreen: bool,
    pub interactive: bool,
    pub status: &'a str,
    pub shortcuts: [&'a str; 7],
}

pub fn draw(ui: &dear_imgui_rs::Ui, state: State<'_>, volume: &mut i32) -> Option<Action> {
    let _disabled = ui.begin_disabled_with_cond(!state.interactive);
    let spacing = ui.clone_style().item_spacing()[0];
    let status_width = text_size(ui, "FPS: 99999.99")[0];
    let width = BUTTON_SIZE * 8.0 + spacing * 8.0 + status_width;
    ui.set_cursor_pos_x(
        ui.cursor_pos_x() + ((ui.content_region_avail_width() - width) * 0.5).max(0.0),
    );
    let mut clicked = None;
    for (index, (action, icon, label)) in [
        (Action::Start, icons::BACKWARD_FAST, "Go to start"),
        (Action::Previous, icons::BACKWARD, "Previous frame"),
        (
            Action::Toggle,
            if state.playing {
                icons::PAUSE
            } else {
                icons::PLAY
            },
            if state.playing { "Pause" } else { "Play" },
        ),
        (Action::Next, icons::FORWARD, "Next frame"),
        (Action::End, icons::FORWARD_FAST, "Go to end"),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            ui.same_line();
        }
        if controls::text_button(ui, icon, [BUTTON_SIZE; 2]) {
            clicked = Some(action);
        }
        crate::shortcut::tooltip(ui, label, state.shortcuts[index]);
    }
    ui.same_line();
    volume_button(ui, volume);
    ui.same_line();
    if controls::text_button_colored(
        ui,
        icons::RETWEET,
        [BUTTON_SIZE; 2],
        if state.looping {
            dear_imgui_rs::StyleColor::Text
        } else {
            dear_imgui_rs::StyleColor::TextDisabled
        },
    ) {
        clicked = Some(Action::Loop);
    }
    crate::shortcut::tooltip(ui, "Toggle loop", state.shortcuts[5]);
    ui.same_line();
    if controls::text_button(
        ui,
        if state.fullscreen {
            icons::COMPRESS
        } else {
            icons::EXPAND
        },
        [BUTTON_SIZE; 2],
    ) {
        clicked = Some(Action::Fullscreen);
    }
    crate::shortcut::tooltip(
        ui,
        if state.fullscreen {
            "Exit fullscreen"
        } else {
            "Enter fullscreen"
        },
        state.shortcuts[6],
    );
    ui.same_line();
    widgets::text(ui, state.status);
    clicked.filter(|_| state.interactive)
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
        controls::slider(ui, "##Volume", volume, 0..=100, "%d%%");
    }
}

pub fn scrubber(
    current_time: f32,
    duration: f32,
    ui: &dear_imgui_rs::Ui,
    min: [f32; 2],
    width: f32,
) {
    let ratio = if duration > 0.0 {
        (current_time / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let progress_x = min[0] + width * ratio;
    let line_y = min[1] + FULLSCREEN_SCRUBBER_HEIGHT * 0.5;
    let draw_list = ui.get_window_draw_list();
    let current = format_time(current_time, 0);
    let duration = format_time(duration, 0);
    let current_size = text_size(ui, &current);
    let duration_size = text_size(ui, &duration);
    let current_x = (progress_x - current_size[0] * 0.5)
        .clamp(min[0], min[0] + (width - current_size[0]).max(0.0));
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

pub fn time_at_position(position: f32, start: f32, width: f32, duration: f32) -> f32 {
    if width <= 0.0 || duration <= 0.0 {
        return 0.0;
    }

    ((position - start) / width).clamp(0.0, 1.0) * duration
}

#[cfg(test)]
mod tests {
    use super::time_at_position;

    #[test]
    fn playback_status_does_not_move_controls() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let mut positions = Vec::new();
        for status in ["PAUSED", "FPS: 9.00", "FPS: 144.00", "EXPORTING"] {
            let ui = context.frame();
            ui.window("Transport")
                .size([800.0, 100.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    super::draw(
                        ui,
                        super::State {
                            playing: status.starts_with("FPS"),
                            looping: false,
                            fullscreen: false,
                            interactive: true,
                            status,
                            shortcuts: [""; 7],
                        },
                        &mut 100,
                    );
                    positions.push(ui.item_rect_min());
                });
            context.end_frame();
        }
        assert!(positions.windows(2).all(|pair| pair[0] == pair[1]));
    }

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
