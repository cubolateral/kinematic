mod objects;
mod tracks;

use crate::editor::{Editor, Timeline};

const BUTTON_SIZE: f32 = 25.0;
const FULLSCREEN_SCRUBBER_HEIGHT: f32 = 12.0;
const FULLSCREEN_SCRUBBER_THICKNESS: f32 = 3.0;
const KEYFRAME_RADIUS: f32 = 3.0;
const KEYFRAME_HITBOX_SIZE: f32 = 16.0;
const KEYFRAME_HOVER_SCALE: f32 = 1.5;
const LABEL_PADDING: [f32; 2] = [8.0, 4.0];
const SCRUBBER_HEIGHT: f32 = 40.0;
const SEGMENT_THICKNESS: f32 = 4.0;
const TRACK_HEIGHT: f32 = 24.0;
const TRACK_SPACING: f32 = 4.0;
const PANEL_TEXT_PADDING: f32 = 10.0;
const TRACK_TEXT_INDENT: f32 = 16.0;
const OBJECT_PANEL_MAX_WIDTH: f32 = 280.0;
const OBJECT_PANEL_MIN_WIDTH: f32 = 160.0;
const OBJECT_PANEL_RATIO: f32 = 0.3;
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
    pressed_entity: Option<Option<hecs::Entity>>,
    pressed_toggle: bool,
    expanded_objects: std::collections::HashSet<hecs::Entity>,
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

    fn press_entity(&mut self, entity: Option<hecs::Entity>, toggle: bool) {
        self.pressed_entity = Some(entity);
        self.pressed_toggle = toggle;
    }

    fn release_entity(
        &mut self,
        entity: Option<hecs::Entity>,
        toggle: bool,
        over_view: bool,
        drag_delta: [f32; 2],
    ) -> Option<(Option<hecs::Entity>, bool)> {
        let pressed = self.pressed_entity.take()?;
        let pressed_toggle = std::mem::take(&mut self.pressed_toggle);
        let moved = drag_delta[0].abs().max(drag_delta[1].abs()) >= DRAG_DIRECTION_THRESHOLD;

        if over_view
            && matches!(self.interaction, Interaction::None | Interaction::Tracks)
            && !moved
            && pressed == entity
            && (!pressed_toggle || toggle)
        {
            Some((entity, pressed_toggle))
        } else {
            None
        }
    }

    fn toggle_object(&mut self, entity: hecs::Entity) {
        if !self.expanded_objects.remove(&entity) {
            self.expanded_objects.insert(entity);
        }
    }

    fn is_object_expanded(&self, entity: hecs::Entity) -> bool {
        self.expanded_objects.contains(&entity)
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
    fn new(ui: &dear_imgui_rs::Ui, top: f32) -> Self {
        let [content_left, viewport_top] = ui.cursor_screen_pos();
        let available = ui.content_region_avail_width().max(1.0);
        let [_, window_top] = ui.window_pos();
        let [_, window_height] = ui.window_size();
        let panel_width = (available * OBJECT_PANEL_RATIO)
            .clamp(OBJECT_PANEL_MIN_WIDTH, OBJECT_PANEL_MAX_WIDTH)
            .min((available - 1.0).max(0.0));
        let divider_x = content_left + panel_width;
        let timeline_left = (divider_x + TRACK_SPACING).min(content_left + available);

        Self {
            content_left,
            top,
            viewport_top: viewport_top + ui.scroll_y(),
            bottom: window_top + window_height,
            divider_x,
            timeline_left,
            timeline_width: (content_left + available - timeline_left).max(1.0),
        }
    }

    fn timeline_right(self) -> f32 {
        self.timeline_left + self.timeline_width
    }

    fn contains_timeline_x(self, x: f32) -> bool {
        (self.timeline_left..=self.timeline_right()).contains(&x)
    }
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) -> bool {
    let is_exporting = editor.is_exporting();
    let mut toggle_fullscreen = false;

    shortcuts(editor.get_timeline(), ui, !is_exporting);

    ui.set_next_window_class(
        &dear_imgui_rs::WindowClass::default()
            .dock_node_flags_override_set(dear_imgui_rs::DockFlags::AUTO_HIDE_TAB_BAR),
    );

    ui.window("Timeline").build(|| {
        let fps = editor.get_preview_fps();
        let _disabled = ui.begin_disabled_with_cond(is_exporting);

        toggle_fullscreen = controls(editor.get_timeline(), ui, fps, !is_exporting, false);
        ui.spacing();
        ui.separator();

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
        let [_, top] = ui.cursor_screen_pos();
        let content_left = ui.cursor_screen_pos()[0];
        let available_height = ui.content_region_avail_height();
        let child_height = (available_height - SCRUBBER_HEIGHT).max(1.0);
        let header_hovered = ui.is_window_hovered_with_flags(
            dear_imgui_rs::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
        );
        let mut view = None;
        let mut objects_hovered = false;

        ui.set_cursor_screen_pos([content_left, top + SCRUBBER_HEIGHT]);
        ui.child_window("Timeline Objects")
            .size([0.0, child_height])
            .flags(dear_imgui_rs::WindowFlags::ALWAYS_VERTICAL_SCROLLBAR)
            .build(ui, || {
                let layout = Layout::new(ui, top);
                let mouse = ui.io().mouse_pos();
                objects_hovered = ui.is_window_hovered_with_flags(
                    dear_imgui_rs::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
                );
                let draw_list = ui.get_window_draw_list();
                let playhead_x = time.x(layout, time.current);

                draw_time_grid_lines(ui, &draw_list, layout, time);
                objects::draw(editor, ui, &draw_list, layout, time, state);
                draw_panel_divider(ui, &draw_list, layout, layout.viewport_top, layout.bottom);
                draw_mouse_indicator(ui, &draw_list, layout);
                draw_playhead(ui, &draw_list, layout, playhead_x);
                view = Some((layout, mouse));
            });

        let Some((layout, mouse)) = view else { return };
        let timeline_hovered = (header_hovered || objects_hovered)
            && layout.contains_timeline_x(mouse[0])
            && (layout.top..=layout.bottom).contains(&mouse[1]);
        let draw_list = ui.get_window_draw_list();
        let playhead_x = time.x(layout, time.current);

        draw_scrubber(ui, &draw_list, layout, time, playhead_x);
        draw_panel_divider(ui, &draw_list, layout, layout.top, layout.viewport_top);
        draw_time_grid_labels(ui, &draw_list, layout, time);
        draw_time_panel(ui, &draw_list, layout, time, playhead_x);

        if is_exporting {
            state.interaction = Interaction::None;
            editor.get_timeline().is_controlling = false;
        } else {
            update_interaction(editor.get_timeline(), ui, layout, state, timeline_hovered);
        }
    });

    toggle_fullscreen
}

fn draw_panel_divider(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    top: f32,
    bottom: f32,
) {
    draw_list.add_line_v(
        layout.divider_x,
        top,
        bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::Separator),
        1.0,
    );
}

fn controls(
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

fn transport_width(ui: &dear_imgui_rs::Ui) -> f32 {
    let spacing = unsafe { ui.style().item_spacing() }[0];

    BUTTON_SIZE * 5.0 + spacing * 4.0
}

pub(super) fn shortcuts(timeline: &mut Timeline, ui: &dear_imgui_rs::Ui, interactive: bool) {
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

pub(super) fn fullscreen_controls(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    fps: f32,
    interactive: bool,
) -> bool {
    let scrubber_min = ui.cursor_screen_pos();
    let scrubber_width = ui.content_region_avail_width().max(1.0);
    let buttons_y = scrubber_min[1] + FULLSCREEN_SCRUBBER_HEIGHT + 7.0;

    let toggle_fullscreen = {
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
        controls(timeline, ui, fps, interactive, true)
    };

    toggle_fullscreen
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

fn draw_scrubber(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
) {
    let text_y = layout.top + 8.0;
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

fn draw_time_grid_lines(
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
        draw_list.add_line_v(x, layout.viewport_top, layout.bottom, color, 1.0);
        tick += step;
    }
}

fn draw_time_grid_labels(
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
    let color = ui.get_color_u32(dear_imgui_rs::StyleColor::TextDisabled);
    let mut tick = first_tick;

    while tick <= time.end + step * 0.001 {
        draw_list.add_text(
            [time.x(layout, tick) + 3.0, layout.top + 8.0],
            color,
            format_grid_time(tick, step),
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

fn draw_playhead(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    playhead_x: f32,
) {
    draw_list.add_line_v(
        playhead_x,
        layout.viewport_top,
        layout.bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        2.0,
    );
}

fn draw_time_panel(
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
        layout.viewport_top,
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

fn draw_mouse_indicator(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
) {
    let mouse = ui.io().mouse_pos();
    if !layout.contains_timeline_x(mouse[0])
        || mouse[1] < layout.viewport_top
        || mouse[1] > layout.bottom
    {
        return;
    }

    draw_list.add_line_v(
        mouse[0],
        layout.viewport_top,
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
