mod entities;
#[allow(dead_code)]
mod tracks;

use crate::editor::{Editor, Timeline};

const BUTTON_SIZE: f32 = 25.0;
const KEYFRAME_HALF_SIZE: f32 = 4.0;
const KEYFRAME_HITBOX_SIZE: f32 = 16.0;
const KEYFRAME_HOVER_SCALE: f32 = 2.0;
const LABEL_PADDING: [f32; 2] = [8.0, 4.0];
const TRANSPORT_HEIGHT: f32 = 40.0;
const SCRUBBER_HEIGHT: f32 = 82.0;
const SEGMENT_THICKNESS: f32 = 4.0;
const TRACK_HEIGHT: f32 = 16.0;
const TRACK_SPACING: f32 = 4.0;
const TRACK_TIMELINE_PADDING: f32 = 8.0;
const GRID_TARGET_SPACING: f32 = 100.0;
const DRAG_DIRECTION_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Interaction {
    #[default]
    None,
    Scrubber,
    Tracks,
    Zoom,
    Pan,
}

#[derive(Default)]
pub(super) struct State {
    duration: f32,
    view_start: f32,
    view_end: f32,
    interaction: Interaction,
}

impl State {
    fn sync_duration(&mut self, duration: f32) {
        if self.duration != duration {
            self.duration = duration;
            self.view_start = 0.0;
            self.view_end = duration;
        }
    }

    fn view_range(&self) -> (f32, f32) {
        (self.view_start, self.view_end)
    }

    fn press(&mut self, timeline_hovered: bool, right_pressed: bool) {
        self.interaction = if timeline_hovered {
            if right_pressed {
                Interaction::Scrubber
            } else {
                Interaction::Tracks
            }
        } else {
            Interaction::None
        };
    }

    fn resolve_tracks(&mut self, delta: [f32; 2]) {
        if self.interaction != Interaction::Tracks {
            return;
        }

        let horizontal = delta[0].abs();
        let vertical = delta[1].abs();
        if horizontal.max(vertical) >= DRAG_DIRECTION_THRESHOLD {
            self.interaction = if vertical > horizontal {
                Interaction::Zoom
            } else {
                Interaction::Pan
            };
        }
    }

    fn zoom(&mut self, delta_y: f32, anchor: f32) {
        if self.duration <= 0.0 || delta_y == 0.0 {
            return;
        }

        let old_span = (self.view_end - self.view_start).max(f32::EPSILON);
        let min_span = (self.duration * 0.01).max(0.05).min(self.duration);
        let new_span = (old_span * (delta_y * 0.01).exp()).clamp(min_span, self.duration);
        let anchor = anchor.clamp(0.0, 1.0);
        let anchor_time = self.view_start + old_span * anchor;
        let new_start = anchor_time - new_span * anchor;

        self.set_view(new_start, new_start + new_span);
    }

    fn pan(&mut self, delta_x: f32, width: f32) {
        if self.duration <= 0.0 || width <= 0.0 {
            return;
        }

        let span = self.view_end - self.view_start;
        self.set_view(
            self.view_start - delta_x / width * span,
            self.view_end - delta_x / width * span,
        );
    }

    fn set_view(&mut self, start: f32, end: f32) {
        let span = (end - start).clamp(0.0, self.duration);
        self.view_start = start.clamp(0.0, self.duration - span);
        self.view_end = self.view_start + span;
    }
}

#[derive(Clone, Copy)]
struct TimeRange {
    current: f32,
    start: f32,
    end: f32,
}

impl TimeRange {
    fn ratio(self, time: f32) -> f32 {
        if self.end > self.start {
            ((time - self.start) / (self.end - self.start)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn x(self, layout: Layout, time: f32) -> f32 {
        layout.timeline_left + layout.timeline_width * self.ratio(time)
    }
}

#[derive(Clone, Copy)]
struct Layout {
    content_left: f32,
    top: f32,
    viewport_top: f32,
    bottom: f32,
    divider_x: f32,
    timeline_left: f32,
    timeline_width: f32,
}

impl Layout {
    fn new(ui: &dear_imgui_rs::Ui) -> Self {
        let [content_left, top] = ui.cursor_screen_pos();
        let available = ui.content_region_avail_width().max(1.0);
        let [_, window_top] = ui.window_pos();

        let [_, window_height] = ui.window_size();

        Self {
            content_left,
            top,
            viewport_top: top + ui.scroll_y(),
            bottom: window_top + window_height,
            divider_x: content_left,
            timeline_left: content_left,
            timeline_width: available,
        }
    }

    fn timeline_right(self) -> f32 {
        self.timeline_left + self.timeline_width
    }

    fn contains_timeline_x(self, x: f32) -> bool {
        (self.timeline_left..=self.timeline_right()).contains(&x)
    }
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    ui.window("Timeline").build(|| {
        let time = {
            let timeline = editor.get_timeline();
            state.sync_duration(timeline.get_duration());
            let (start, end) = state.view_range();
            TimeRange {
                current: timeline.get_time(),
                start,
                end,
            }
        };
        let layout = Layout::new(ui);

        controls(editor.get_timeline(), ui, layout);

        let mouse = ui.io().mouse_pos();
        let timeline_hovered = ui.is_window_hovered_with_flags(
            dear_imgui_rs::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
        ) && layout.contains_timeline_x(mouse[0])
            && (layout.viewport_top..=layout.bottom).contains(&mouse[1]);
        let draw_list = ui.get_window_draw_list();
        let playhead_x = time.x(layout, time.current);

        draw_scrubber(ui, &draw_list, layout, time, playhead_x);
        draw_time_grid(ui, &draw_list, layout, time);
        entities::draw(editor, ui, &draw_list, layout, time);
        draw_mouse_indicator(ui, &draw_list, layout);
        draw_overlay(ui, &draw_list, layout, time, playhead_x);
        update_interaction(editor.get_timeline(), ui, layout, state, timeline_hovered);
    });
}

fn controls(timeline: &mut Timeline, ui: &dear_imgui_rs::Ui, layout: Layout) {
    let spacing = unsafe { ui.style().item_spacing() }[0];
    let width = BUTTON_SIZE * 3.0 + spacing * 2.0;
    ui.set_cursor_screen_pos([
        layout.timeline_left + ((layout.timeline_width - width) * 0.5).max(0.0),
        layout.top + (TRANSPORT_HEIGHT - BUTTON_SIZE) * 0.5,
    ]);

    if transport_button(
        ui,
        "<<",
        "Go to left [ArrowLeft]",
        dear_imgui_rs::Key::LeftArrow,
    ) {
        timeline.go_to_start();
    }
    ui.same_line();

    let is_playing = timeline.is_playing();
    if transport_button(
        ui,
        if is_playing { "||" } else { "|>" },
        if is_playing {
            "Pause [Space]"
        } else {
            "Play [Space]"
        },
        dear_imgui_rs::Key::Space,
    ) {
        timeline.toggle();
    }
    ui.same_line();

    if transport_button(
        ui,
        ">>",
        "Go to right [ArrowRight]",
        dear_imgui_rs::Key::RightArrow,
    ) {
        timeline.go_to_end();
    }
}

fn transport_button(
    ui: &dear_imgui_rs::Ui,
    label: &str,
    tooltip: &str,
    key: dear_imgui_rs::Key,
) -> bool {
    let clicked = ui.button_config(label).size([BUTTON_SIZE; 2]).build();
    if ui.is_item_hovered() {
        ui.tooltip_text(tooltip);
    }
    clicked || ui.is_key_pressed(key)
}

fn draw_scrubber(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
) {
    let text_y = layout.top + TRANSPORT_HEIGHT + 8.0;
    let line_y = layout.top + SCRUBBER_HEIGHT - 2.0;
    let end_text = format!("{:.2}s", time.end);
    let end_width = text_size(ui, &end_text)[0];

    draw_list.add_text(
        [layout.timeline_right() - end_width, text_y],
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        end_text,
    );
    draw_list.add_line_h(
        playhead_x,
        layout.timeline_right(),
        line_y,
        ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg),
        2.0,
    );
    draw_list.add_line_h(
        layout.timeline_left,
        playhead_x,
        line_y,
        ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
        2.0,
    );
}

fn grid_step(time: TimeRange, width: f32) -> f32 {
    let span = time.end - time.start;
    if span <= 0.0 || width <= 0.0 {
        return 0.0;
    }

    let raw_step = span / (width / GRID_TARGET_SPACING);
    let magnitude = 10.0_f32.powf(raw_step.log10().floor());
    let normalized = raw_step / magnitude;
    let multiple = if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    };

    multiple * magnitude
}

fn draw_time_grid(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
) {
    let step = grid_step(time, layout.timeline_width);
    if step <= 0.0 {
        return;
    }

    let first_tick = (time.start / step).ceil() * step;
    let color = ui.get_color_u32(dear_imgui_rs::StyleColor::Separator);
    let mut tick = first_tick;

    while tick <= time.end + step * 0.001 {
        let x = time.x(layout, tick);
        draw_list.add_line_v(x, layout.top + SCRUBBER_HEIGHT, layout.bottom, color, 1.0);
        let label = format_grid_time(tick, step);
        draw_list.add_text(
            [x + 3.0, layout.top + TRANSPORT_HEIGHT + 8.0],
            ui.get_color_u32(dear_imgui_rs::StyleColor::TextDisabled),
            label,
        );
        tick += step;
    }
}

fn format_grid_time(time: f32, step: f32) -> String {
    let mut decimals = 0;
    let mut value = step.abs();
    while value < 1.0 && decimals < 6 {
        value *= 10.0;
        decimals += 1;
    }

    format!("{:.*}s", decimals, time)
}

fn draw_overlay(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
) {
    let text_color = ui.get_color_u32(dear_imgui_rs::StyleColor::Text);
    draw_list.add_line_v(
        playhead_x,
        layout.top + SCRUBBER_HEIGHT - 10.0,
        layout.bottom,
        text_color,
        2.0,
    );

    let text = format!("{:.2}s", time.current);
    let size = text_size(ui, &text);
    let position = [
        playhead_x - size[0] * 0.5,
        layout.top + TRANSPORT_HEIGHT + 8.0,
    ];
    let min = [
        position[0] - LABEL_PADDING[0],
        position[1] - LABEL_PADDING[1],
    ];
    let max = [
        position[0] + size[0] + LABEL_PADDING[0],
        position[1] + size[1] + LABEL_PADDING[1],
    ];

    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::PopupBg),
        )
        .filled(true)
        .rounding(3.0)
        .build();
    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .rounding(3.0)
        .build();
    draw_list.add_text(position, text_color, text);
}

fn draw_mouse_indicator(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
) {
    let mouse = ui.io().mouse_pos();
    if !layout.contains_timeline_x(mouse[0]) || mouse[1] < layout.top || mouse[1] > layout.bottom {
        return;
    }

    draw_list.add_line_v(
        mouse[0],
        layout.top + SCRUBBER_HEIGHT,
        layout.bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::TextDisabled),
        1.0,
    );
}

fn update_interaction(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    layout: Layout,
    state: &mut State,
    timeline_hovered: bool,
) {
    let mouse = ui.io().mouse_pos();
    let mouse_delta = ui.io().mouse_delta();
    let left = dear_imgui_rs::MouseButton::Left;
    let right = dear_imgui_rs::MouseButton::Right;
    let left_down = ui.is_mouse_down(left);
    let right_down = ui.is_mouse_down(right);
    let left_pressed = ui.is_mouse_clicked(left);
    let right_pressed = ui.is_mouse_clicked(right);

    if !left_down && !right_down {
        state.interaction = Interaction::None;
    } else if left_pressed || right_pressed {
        state.press(timeline_hovered, right_pressed || right_down);
    } else {
        state.resolve_tracks(ui.mouse_drag_delta_with_threshold(left, DRAG_DIRECTION_THRESHOLD));
    }

    if state.interaction == Interaction::Zoom && left_down {
        let anchor = ((mouse[0] - layout.timeline_left) / layout.timeline_width).clamp(0.0, 1.0);
        state.zoom(mouse_delta[1], anchor);
    }

    if state.interaction == Interaction::Pan && left_down {
        state.pan(mouse_delta[0], layout.timeline_width);
    }

    timeline.is_controlling = state.interaction == Interaction::Scrubber && right_down;
    if timeline.is_controlling {
        let (start, end) = state.view_range();
        let visible_time = start
            + ((mouse[0] - layout.timeline_left) / layout.timeline_width).clamp(0.0, 1.0)
                * (end - start);
        timeline.go_to(visible_time);
    }
}

fn text_size(ui: &dear_imgui_rs::Ui, text: &str) -> [f32; 2] {
    ui.current_font()
        .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_uses_seconds_for_wide_ranges() {
        let time = TimeRange {
            current: 0.0,
            start: 0.0,
            end: 40.0,
        };

        assert_eq!(grid_step(time, 800.0), 5.0);
    }

    #[test]
    fn grid_uses_small_intervals_when_zoomed_in() {
        let time = TimeRange {
            current: 0.0,
            start: 0.0,
            end: 0.4,
        };

        assert!((grid_step(time, 800.0) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn scrubber_keeps_the_gesture_until_the_mouse_is_released() {
        let mut state = State::default();

        state.press(true, true);
        state.resolve_tracks([20.0, 0.0]);

        assert_eq!(state.interaction, Interaction::Scrubber);
    }

    #[test]
    fn track_gesture_waits_for_a_clear_accumulated_direction() {
        let mut state = State::default();

        state.press(true, false);
        state.resolve_tracks([3.0, 1.0]);
        assert_eq!(state.interaction, Interaction::Tracks);

        state.resolve_tracks([3.0, 9.0]);
        assert_eq!(state.interaction, Interaction::Zoom);
    }

    #[test]
    fn zooming_up_reduces_the_visible_time_span() {
        let mut state = State::default();
        state.sync_duration(10.0);
        let initial = state.view_range();

        state.zoom(-20.0, 0.5);

        let zoomed = state.view_range();
        assert!(zoomed.1 - zoomed.0 < initial.1 - initial.0);
    }

    #[test]
    fn panning_stays_inside_the_timeline_duration() {
        let mut state = State::default();
        state.sync_duration(10.0);

        state.pan(-1000.0, 100.0);
        assert_eq!(state.view_range().1, 10.0);

        state.pan(1000.0, 100.0);
        assert_eq!(state.view_range().0, 0.0);
    }
}
