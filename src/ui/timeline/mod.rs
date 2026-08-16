mod tracks;

use crate::editor::{Editor, Timeline};

const BUTTON_SIZE: f32 = 25.0;
const KEYFRAME_HALF_SIZE: f32 = 4.0;
const KEYFRAME_HITBOX_SIZE: f32 = 16.0;
const KEYFRAME_HOVER_SCALE: f32 = 2.0;
const LABEL_PADDING: [f32; 2] = [8.0, 4.0];
const SCRUBBER_HEIGHT: f32 = 50.0;
const SEGMENT_THICKNESS: f32 = 4.0;
const TRACK_HEIGHT: f32 = 16.0;
const TRACK_LABEL_WIDTH: f32 = 128.0;
const TRACK_SPACING: f32 = 4.0;
const TRACK_TIMELINE_PADDING: f32 = 8.0;

#[derive(Clone, Copy)]
struct TimeRange {
    current: f32,
    duration: f32,
}

impl TimeRange {
    fn ratio(self, time: f32) -> f32 {
        if self.duration > 0.0 {
            (time / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn x(self, layout: Layout, time: f32) -> f32 {
        layout.timeline_left + layout.timeline_width * self.ratio(time)
    }

    fn time_at(self, layout: Layout, x: f32) -> f32 {
        ((x - layout.timeline_left) / layout.timeline_width).clamp(0.0, 1.0) * self.duration
    }
}

#[derive(Clone, Copy)]
struct Layout {
    content_left: f32,
    top: f32,
    window_top: f32,
    bottom: f32,
    divider_x: f32,
    timeline_left: f32,
    timeline_width: f32,
}

impl Layout {
    fn new(ui: &dear_imgui_rs::Ui) -> Self {
        let [content_left, top] = ui.cursor_screen_pos();
        let available = ui.content_region_avail_width().max(1.0);
        let label_width = TRACK_LABEL_WIDTH.min(available * 0.4);
        let divider_x = content_left + label_width;
        let timeline_left = divider_x + TRACK_TIMELINE_PADDING;
        let [_, window_top] = ui.window_pos();
        let [_, window_height] = ui.window_size();

        Self {
            content_left,
            top,
            window_top,
            bottom: window_top + window_height,
            divider_x,
            timeline_left,
            timeline_width: (available - label_width - TRACK_TIMELINE_PADDING).max(1.0),
        }
    }

    fn timeline_right(self) -> f32 {
        self.timeline_left + self.timeline_width
    }

    fn contains_timeline_x(self, x: f32) -> bool {
        (self.timeline_left..=self.timeline_right()).contains(&x)
    }
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    ui.window("Timeline").build(|| {
        let (time, was_controlling) = {
            let timeline = editor.get_timeline();
            (
                TimeRange {
                    current: timeline.get_current_time(),
                    duration: timeline.get_max_time(),
                },
                timeline.is_controlling,
            )
        };
        let layout = Layout::new(ui);

        controls(editor.get_timeline(), ui, layout);

        ui.set_cursor_screen_pos([layout.timeline_left, layout.top]);
        ui.invisible_button("scrubber_area", [layout.timeline_width, SCRUBBER_HEIGHT]);

        let mouse_x = ui.io().mouse_pos()[0];
        let scrubber_active = ui.is_item_active() && layout.contains_timeline_x(mouse_x);
        let draw_list = ui.get_window_draw_list();
        let playhead_x = time.x(layout, time.current);

        draw_scrubber(ui, &draw_list, layout, time, playhead_x);
        tracks::draw(editor, ui, &draw_list, layout, time);
        draw_overlay(ui, &draw_list, layout, time, playhead_x);
        update_interaction(
            editor.get_timeline(),
            ui,
            layout,
            time,
            scrubber_active,
            was_controlling,
        );
    });
}

fn controls(timeline: &mut Timeline, ui: &dear_imgui_rs::Ui, layout: Layout) {
    let spacing = unsafe { ui.style().item_spacing() }[0];
    let width = BUTTON_SIZE * 3.0 + spacing * 2.0;
    ui.set_cursor_screen_pos([
        layout.content_left + ((layout.divider_x - layout.content_left - width) * 0.5).max(0.0),
        layout.top + (SCRUBBER_HEIGHT - BUTTON_SIZE) * 0.5,
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
    let text_y = layout.top + 8.0;
    let line_y = text_y + 32.0;
    let end_text = format!("{:.2}s", time.duration);
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

fn draw_overlay(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
) {
    let text_color = ui.get_color_u32(dear_imgui_rs::StyleColor::Text);
    draw_list.add_line_v(
        layout.divider_x,
        layout.top,
        layout.bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::Separator),
        1.0,
    );
    draw_list.add_line_v(
        playhead_x,
        layout.top + 40.0,
        layout.bottom,
        text_color,
        2.0,
    );

    let text = format!("{:.2}s", time.current);
    let size = text_size(ui, &text);
    let position = [playhead_x - size[0] * 0.5, layout.top + 8.0];
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
        .build();
    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .build();
    draw_list.add_text(position, text_color, text);
}

fn update_interaction(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    layout: Layout,
    time: TimeRange,
    scrubber_active: bool,
    was_controlling: bool,
) {
    let mouse = ui.io().mouse_pos();
    let window_hovered = ui.is_window_hovered_with_flags(
        dear_imgui_rs::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
    );
    let tracks_hovered = window_hovered
        && ui.is_mouse_hovering_rect_with_clip(
            [layout.timeline_left, layout.top],
            [layout.timeline_right(), layout.bottom],
            false,
        );
    let dragging =
        ui.is_mouse_down(dear_imgui_rs::MouseButton::Left) && (tracks_hovered || was_controlling);

    timeline.is_controlling = scrubber_active || dragging;
    if timeline.is_controlling {
        timeline.go_to(time.time_at(layout, mouse[0]));
    }
}

fn text_size(ui: &dear_imgui_rs::Ui, text: &str) -> [f32; 2] {
    ui.current_font()
        .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, text)
}
