use super::{
    layout::{Layout, TimeRange},
    metrics::{PANEL_TEXT_PADDING, TRACK_SPACING},
    state::State,
};
use crate::{
    editor::Editor,
    ui::widgets::{draw_panel_rect, text_size},
};

const SCENE_HEIGHT: f32 = 48.0;
const EVENT_HEIGHT: f32 = SCENE_HEIGHT * 0.5;

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    state: &mut State,
) {
    ui.dummy([0.0, TRACK_SPACING]);

    let top = ui.cursor_screen_pos()[1];
    ui.dummy([layout.timeline_right() - layout.content_left, SCENE_HEIGHT]);

    let clip = draw_list.push_clip_rect(
        [layout.timeline_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );
    let active_scene = editor.get_active_scene_index();
    let scenes = editor
        .get_scenes()
        .map(|(name, range, events)| {
            (
                name,
                range,
                events
                    .iter()
                    .map(|event| {
                        (
                            event.file_index,
                            event.name.clone(),
                            event.creation_time,
                            event.duration,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let mut hovered_event = None;

    for (index, (name, range, _)) in scenes.iter().enumerate() {
        let start = range[0].max(time.start);
        let end = range[1].min(time.end);
        if end <= start {
            continue;
        }

        let min = [time.x(layout, start), top];
        let max = [time.x(layout, end), top + SCENE_HEIGHT];
        let fill = if index == active_scene {
            active_scene_color(ui)
        } else {
            ui.get_color_u32(dear_imgui_rs::StyleColor::WindowBg)
        };
        let is_active = index == active_scene;
        let border = ui.get_color_u32(if is_active {
            dear_imgui_rs::StyleColor::CheckMark
        } else {
            dear_imgui_rs::StyleColor::Border
        });

        draw_panel_rect(
            draw_list,
            min,
            max,
            Some(fill),
            border,
            if is_active { 2.0 } else { 1.0 },
        );

        let text_height = text_size(ui, name)[1];
        let text_clip = draw_list.push_clip_rect(min, max, true);
        draw_list.add_text(
            [
                min[0] + PANEL_TEXT_PADDING,
                top + (EVENT_HEIGHT - text_height) * 0.5,
            ],
            ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            name,
        );
        drop(text_clip);
    }

    for (index, (_, range, events)) in scenes.iter().enumerate() {
        for (event_index, (file_event, event_name, creation_time, stored_duration)) in
            events.iter().enumerate()
        {
            let drag = state
                .event_drag()
                .filter(|drag| drag.scene == index && drag.event == event_index);
            let duration = drag.map_or(*stored_duration, |drag| drag.duration);
            let global_start = range[0] + creation_time;
            let global_end = global_start + duration;
            if global_end < time.start || global_start > time.end {
                continue;
            }

            let duration_min = [
                time.x(layout, global_start.max(time.start)),
                top + EVENT_HEIGHT,
            ];
            let duration_max = [time.x(layout, global_end.min(time.end)), top + SCENE_HEIGHT];
            if duration_max[0] > duration_min[0] {
                draw_list
                    .add_rect(
                        duration_min,
                        duration_max,
                        ui.get_color_u32_with_alpha(dear_imgui_rs::StyleColor::CheckMark, 0.5),
                    )
                    .filled(true)
                    .build();
            }

            let text_size = text_size(ui, event_name);
            let handle_min = [time.x(layout, global_end), top + EVENT_HEIGHT];
            let handle_max = [
                handle_min[0] + text_size[0] + PANEL_TEXT_PADDING * 2.0,
                top + SCENE_HEIGHT,
            ];
            let hovered =
                ui.is_window_hovered() && ui.is_mouse_hovering_rect(handle_min, handle_max);
            if hovered {
                hovered_event = Some((index, event_index, *file_event, *creation_time, duration));
            }

            let is_dragging = drag.is_some();
            if hovered || is_dragging {
                ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeEW));
            }
            let accent = ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark);
            let text = ui.get_color_u32(dear_imgui_rs::StyleColor::Text);
            draw_panel_rect(
                draw_list,
                handle_min,
                handle_max,
                Some(if is_dragging { text } else { accent }),
                if is_dragging { accent } else { text },
                1.0,
            );
            let event_clip = draw_list.push_clip_rect(handle_min, handle_max, true);
            draw_list.add_text(
                [
                    handle_min[0] + PANEL_TEXT_PADDING,
                    top + EVENT_HEIGHT + (EVENT_HEIGHT - text_size[1]) * 0.5,
                ],
                if is_dragging { accent } else { text },
                event_name,
            );
            drop(event_clip);
        }
    }

    drop(clip);

    let left = dear_imgui_rs::MouseButton::Left;
    let mouse_global_time = time.start
        + (ui.io().mouse_pos()[0] - layout.timeline_left) / layout.timeline_width
            * (time.end - time.start);

    if let Some((scene, event, file_event, creation_time, duration)) = hovered_event
        && ui.is_mouse_clicked(left)
    {
        state.begin_event_drag(
            scene,
            event,
            file_event,
            duration,
            mouse_global_time - scenes[scene].1[0] - creation_time,
        );
    }

    if let Some(drag) = state.event_drag() {
        if ui.is_mouse_down(left) {
            let creation_time = scenes[drag.scene].2[drag.event].2;
            state.drag_event(mouse_global_time - scenes[drag.scene].1[0] - creation_time);
        } else if let Some(drag) = state.finish_event_drag() {
            editor.set_event_duration(drag.scene, drag.file_event, drag.duration);
        }
    }
}

fn active_scene_color(ui: &dear_imgui_rs::Ui) -> u32 {
    ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg)
}
