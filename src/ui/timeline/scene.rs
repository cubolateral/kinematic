use super::{
    layout::{Layout, TimeRange},
    metrics::{EVENT_HEIGHT, PANEL_TEXT_PADDING, ROW_SPACING, SCENE_HEIGHT},
    state::State,
};
use crate::{editor::Editor, ui::widgets::text_size};
use kinematic_ui::timeline::painting;

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    state: &mut State,
    interactive: bool,
) {
    ui.dummy([0.0, ROW_SPACING]);

    let top = ui.cursor_screen_pos()[1];
    painting::separator(
        ui,
        draw_list,
        layout.timeline_left,
        layout.timeline_right(),
        top + SCENE_HEIGHT + 2.0,
    );

    ui.dummy([layout.timeline_right() - layout.content_left, SCENE_HEIGHT]);

    let clip = draw_list.push_clip_rect(
        [layout.timeline_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );
    let active_scene = editor.active_scene_index();
    let scenes = editor
        .scenes()
        .map(|(index, name, range, events)| {
            (
                index,
                name.to_owned(),
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
    let mut hovered_scene = None;
    let mut over_event = false;
    let mut back_position = None;

    for (index, name, range, _) in &scenes {
        let start = range[0].max(time.start);
        let end = range[1].min(time.end);

        if end <= start {
            continue;
        }

        let min = [time.x(layout, start), top];
        let max = [time.x(layout, end), top + SCENE_HEIGHT];
        let hovered = ui.is_window_hovered() && ui.is_mouse_hovering_rect(min, max);
        if hovered {
            hovered_scene = Some(*index);
        }
        let back = painting::scene(
            ui,
            draw_list,
            [min, max],
            name,
            *index == active_scene,
            hovered,
            editor.focused_scene_index() == Some(*index),
        );
        if back.is_some() {
            back_position = back;
        }
    }

    for (index, _, range, events) in &scenes {
        for (event_index, (file_event, event_name, creation_time, stored_duration)) in
            events.iter().enumerate()
        {
            let drag = state
                .event_drag()
                .filter(|drag| drag.scene == *index && drag.event == event_index);
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

            painting::event_span(ui, draw_list, duration_min, duration_max);

            let text_size = text_size(ui, event_name);
            let handle_min = [time.x(layout, global_end), top + EVENT_HEIGHT];
            let handle_max = [
                handle_min[0] + text_size[0] + PANEL_TEXT_PADDING * 2.0,
                top + SCENE_HEIGHT,
            ];
            let hovered =
                ui.is_window_hovered() && ui.is_mouse_hovering_rect(handle_min, handle_max);

            over_event |= hovered
                || (duration_max[0] > duration_min[0]
                    && ui.is_window_hovered()
                    && ui.is_mouse_hovering_rect(duration_min, duration_max));

            if hovered {
                hovered_event = Some((*index, event_index, *file_event, *creation_time, duration));
            }

            let is_dragging = drag.is_some();

            if hovered || is_dragging {
                ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeEW));
            }

            painting::event_handle(ui, draw_list, [handle_min, handle_max], event_name);
        }
    }

    drop(clip);

    let next_row = ui.cursor_screen_pos();
    let back_hovered = if let Some(position) = back_position {
        let (clicked, hovered) = painting::back(ui, draw_list, position);

        if clicked && interactive {
            ui.set_cursor_screen_pos(next_row);
            editor.clear_scene_focus();

            return;
        }

        hovered
    } else {
        false
    };

    ui.set_cursor_screen_pos(next_row);

    if !interactive {
        return;
    }

    let left = dear_imgui_rs::MouseButton::Left;
    let mouse_global_time = time.start
        + (ui.io().mouse_pos()[0] - layout.timeline_left) / layout.timeline_width
            * (time.end - time.start);

    if !back_hovered
        && !over_event
        && let Some(scene) = hovered_scene
        && ui.is_mouse_clicked(left)
    {
        state.press_scene(scene);
    }

    if ui.is_mouse_released(left)
        && let Some(scene) = state.release_scene(
            (!back_hovered && !over_event)
                .then_some(hovered_scene)
                .flatten(),
            ui.mouse_drag_delta(left),
        )
    {
        let start = scenes
            .iter()
            .find(|(index, ..)| *index == scene)
            .map_or(0.0, |(_, _, range, _)| range[0]);

        editor.focus_scene(scene, mouse_global_time - start);
    }

    if let Some((scene, event, file_event, creation_time, duration)) = hovered_event
        && ui.is_mouse_clicked(left)
    {
        let scene_start = scenes
            .iter()
            .find(|(index, ..)| *index == scene)
            .map_or(0.0, |(_, _, range, _)| range[0]);

        state.begin_event_drag(
            scene,
            event,
            file_event,
            duration,
            mouse_global_time - scene_start - creation_time,
        );
    }

    if let Some(drag) = state.event_drag() {
        if ui.is_mouse_down(left) {
            let (_, _, range, events) = scenes
                .iter()
                .find(|(index, ..)| *index == drag.scene)
                .expect("Dragged event scene must exist.");
            let creation_time = events[drag.event].2;

            state.drag_event(mouse_global_time - range[0] - creation_time);
        } else if let Some(drag) = state.finish_event_drag() {
            editor.set_event_duration(drag.scene, drag.file_event, drag.duration);
        }
    }
}
