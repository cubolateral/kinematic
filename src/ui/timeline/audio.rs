use super::{
    layout::{Layout, TimeRange},
    state::{Interaction, State},
};
use crate::{editor::Editor, scripting::AudioClip};
use kinematic_ui::controls;
use kinematic_ui::widgets;

use kinematic_ui::timeline::{
    metrics::{AUDIO_HEIGHT as HEIGHT, CLIP_HANDLE as HANDLE},
    painting,
};

#[derive(Clone, Copy)]
enum Edit {
    Move,
    Left,
    Right,
}

pub(super) struct ClipDrag {
    original: AudioClip,
    draft: AudioClip,
    edit: Edit,
    mouse_x: f32,
    mouse_y: f32,
    insert_above: bool,
    started: bool,
    source_duration: f64,
}

fn edited(original: &AudioClip, edit: Edit, delta: f64, source_duration: f64) -> AudioClip {
    let mut clip = original.clone();
    let minimum = 1.0 / crate::core::audio::SAMPLE_RATE as f64;
    match edit {
        Edit::Move => clip.at = (original.at + delta).max(0.0),
        Edit::Left => {
            let delta = delta.clamp(
                -original.offset.min(original.at),
                (original.duration - minimum).max(0.0),
            );
            clip.at += delta;
            clip.offset += delta;
            clip.duration -= delta;
        }
        Edit::Right => {
            clip.duration = (original.duration + delta)
                .clamp(minimum, (source_duration - original.offset).max(minimum))
        }
    }
    clip
}

fn constrained(
    mut draft: AudioClip,
    reference: &AudioClip,
    edit: Edit,
    peers: &[AudioClip],
) -> Option<AudioClip> {
    let mut lower: f64 = 0.0;
    let mut upper = f64::INFINITY;
    for peer in peers
        .iter()
        .filter(|peer| peer.id != draft.id && peer.lane == draft.lane)
    {
        let end = peer.at + peer.duration;
        if end <= reference.at + 1e-9 {
            lower = lower.max(end);
        } else if peer.at >= reference.at + reference.duration - 1e-9 {
            upper = upper.min(peer.at);
        } else if draft.at < end - 1e-9 && draft.at + draft.duration > peer.at + 1e-9 {
            return None;
        }
    }
    match edit {
        Edit::Move => {
            upper -= draft.duration;
            if upper < lower {
                return None;
            }
            draft.at = draft.at.clamp(lower, upper);
        }
        Edit::Left => {
            let delta = (lower - draft.at).max(0.0);
            draft.at += delta;
            draft.offset += delta;
            draft.duration -= delta;
        }
        Edit::Right => draft.duration = draft.duration.min(upper - draft.at),
    }
    (draft.duration > 0.0).then_some(draft)
}

fn lane_count(editor: &Editor) -> usize {
    editor
        .audio_clips()
        .iter()
        .map(|clip| clip.lane.saturating_add(1))
        .max()
        .unwrap_or(1)
}

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    state: &mut State,
    dragged_audio: Option<&std::path::Path>,
) {
    #[cfg(test)]
    {
        state.audio_clip_rect = None;
    }
    state.audio_hovered = false;
    let editable = editor.audio_editable();
    let left = dear_imgui_rs::MouseButton::Left;
    let right = dear_imgui_rs::MouseButton::Right;
    if ui.is_mouse_clicked(right) {
        state.right_dragged = false;
    }
    if ui.mouse_drag_delta_with_threshold(right, super::metrics::DRAG_DIRECTION_THRESHOLD)
        != [0.0; 2]
    {
        state.right_dragged = true;
        if state.interaction == Interaction::Audio && state.audio_drag.is_none() {
            state.interaction = Interaction::Scrubber;
        }
    }
    if state.audio_drag.is_none()
        && state.interaction == Interaction::Audio
        && !ui.is_mouse_down(left)
        && !ui.is_mouse_down(right)
    {
        state.interaction = Interaction::None;
    }
    if !editable || ui.is_key_pressed(dear_imgui_rs::Key::Escape) {
        if state.audio_drag.take().is_some() {
            state.interaction = Interaction::None;
        }
    }
    if let Some(drag) = &mut state.audio_drag {
        let fps = editor.project_info().1.fps as f64;
        let delta = (ui.io().mouse_pos()[0] - drag.mouse_x) as f64 / layout.timeline_width as f64
            * (time.end - time.start) as f64;
        let anchor = match drag.edit {
            Edit::Move | Edit::Left => drag.original.at,
            Edit::Right => drag.original.at + drag.original.duration,
        };
        let delta = if delta == 0.0 {
            0.0
        } else {
            ((anchor + delta) * fps).round() / fps - anchor
        };
        let mouse = ui.io().mouse_pos();
        drag.started |= (mouse[0] - drag.mouse_x).hypot(mouse[1] - drag.mouse_y) >= 8.0;
        if drag.started {
            let mut draft = edited(&drag.original, drag.edit, delta, drag.source_duration);
            let mut insert_above = false;
            if matches!(drag.edit, Edit::Move) {
                let lane = drag.original.lane as f64
                    + ((ui.io().mouse_pos()[1] - drag.mouse_y) / HEIGHT).round() as f64;
                insert_above = lane < 0.0;
                draft.lane = lane.clamp(0.0, lane_count(editor) as f64) as usize;
            }
            let peers = if insert_above {
                &[]
            } else {
                editor.audio_clips()
            };
            let reference = if draft.lane == drag.draft.lane && insert_above == drag.insert_above {
                &drag.draft
            } else {
                &drag.original
            };
            if let Some(draft) = constrained(draft, reference, drag.edit, peers) {
                drag.draft = draft;
                drag.insert_above = insert_above;
            }
        }
        if !ui.is_mouse_down(left) {
            let drag = state.audio_drag.take().unwrap();
            state.interaction = Interaction::None;
            let mut clips = editor.audio_clips().to_vec();
            if let Some(clip) = clips.iter_mut().find(|clip| **clip == drag.original) {
                if drag.insert_above || *clip != drag.draft {
                    *clip = drag.draft;
                    if drag.insert_above {
                        for peer in &mut clips {
                            if peer.id != drag.original.id {
                                peer.lane = peer.lane.saturating_add(1);
                            }
                        }
                    }
                    editor.commit_audio(clips);
                }
            }
        }
    }
    let panel_top = ui.cursor_screen_pos();
    let header_height = 24.0;
    let panel_height = header_height
        + if state.audio_collapsed {
            0.0
        } else {
            HEIGHT
                * (lane_count(editor)
                    + usize::from(
                        state
                            .audio_drag
                            .as_ref()
                            .is_some_and(|drag| drag.started && matches!(drag.edit, Edit::Move)),
                    )) as f32
                + 8.0
        };
    let clicked = painting::audio_header(
        ui,
        draw_list,
        panel_top,
        layout.timeline_width,
        panel_height,
        state.audio_collapsed,
    );
    if ui.is_item_active() || clicked {
        state.interaction = Interaction::Audio;
    }
    if accept_file(editor, ui, dragged_audio) {
        state.audio_collapsed = false;
    }
    context_menu(editor, ui, state, None);
    if clicked {
        state.audio_collapsed = !state.audio_collapsed;
    }
    if state.audio_collapsed {
        ui.set_cursor_screen_pos(panel_top);
        ui.dummy([layout.timeline_width, header_height + 4.0]);
        return;
    }
    let selected = editor.selected_audio().map(|clip| clip.id);
    state
        .waveforms
        .retain(|_, wave| wave.samples.strong_count() > 0);
    let top = [panel_top[0], panel_top[1] + header_height + 4.0];
    ui.set_cursor_screen_pos(top);
    let lanes = ((panel_height - header_height - 8.0) / HEIGHT).round() as usize;
    ui.dummy([layout.timeline_width, HEIGHT * lanes as f32]);
    painting::audio_lanes(ui, draw_list, top, layout.timeline_width, HEIGHT, lanes);
    accept_file(editor, ui, dragged_audio);
    let mut clips = editor.audio_clips().to_vec();
    if let Some(drag) = &state.audio_drag {
        if drag.insert_above {
            for peer in &mut clips {
                peer.lane = peer.lane.saturating_add(1);
            }
        }
        if let Some(clip) = clips.iter_mut().find(|clip| clip.id == drag.draft.id) {
            *clip = drag.draft.clone();
        }
    }
    clips.sort_by(|a, b| {
        (selected == Some(a.id))
            .cmp(&(selected == Some(b.id)))
            .then(a.at.total_cmp(&b.at))
            .then(a.id.cmp(&b.id))
    });
    for clip in clips {
        let start = clip.at - editor.timeline_origin() as f64;
        let end = start + clip.duration;
        if end <= time.start as f64 || start >= time.end as f64 {
            continue;
        }
        let min = [
            time.x(layout, start as f32),
            top[1] + clip.lane as f32 * HEIGHT,
        ];
        let max = [time.x(layout, end as f32), min[1] + HEIGHT - 3.0];
        let _id = ui.push_id(&format!("audio:{}", clip.id));
        ui.set_cursor_screen_pos(min);
        ui.invisible_button("##clip", [(max[0] - min[0]).max(1.0), max[1] - min[1]]);
        let hovered = ui.is_item_hovered();
        state.audio_hovered |= hovered;
        let activated = ui.is_item_activated();
        context_menu(editor, ui, state, Some(clip.id));
        #[cfg(test)]
        if selected == Some(clip.id) {
            state.audio_clip_rect = Some([min, max]);
        }

        painting::audio(
            ui,
            draw_list,
            [min, max],
            selected == Some(clip.id),
            hovered,
        );
        let _clip = draw_list.push_clip_rect(min, max, true);
        if let Some(samples) = editor.audio_samples(clip.id) {
            let wave = state
                .waveforms
                .entry(std::sync::Arc::as_ptr(&samples) as usize)
                .or_insert_with(|| super::waveform::Waveform::new(&samples));
            let rate = crate::core::audio::SAMPLE_RATE as f64;
            painting::waveform(ui, draw_list, [min, max], clip.gain, |x, next_x| {
                let source_at = |x: f32| {
                    clip.offset
                        + time.start as f64
                        + (x - layout.timeline_left) as f64 / layout.timeline_width as f64
                            * (time.end - time.start) as f64
                        - start
                };
                let from = (source_at(x).max(clip.offset) * rate).round() as usize;
                let to =
                    (source_at(next_x).min(clip.offset + clip.duration) * rate).round() as usize;
                let source_end = ((clip.offset + clip.duration) * rate).round() as usize;
                wave.range(
                    &samples,
                    from.min(source_end),
                    to.max(from.saturating_add(1)).min(source_end),
                )
            });
        }
        painting::audio_label(
            ui,
            draw_list,
            [min, max],
            &clip.file.file_name().unwrap_or_default().to_string_lossy(),
        );
        if (hovered || activated) && editable && state.audio_drag.is_none() {
            let x = ui.io().mouse_pos()[0];
            let edit = if start >= time.start as f64 && x < min[0] + HANDLE {
                Edit::Left
            } else if end <= time.end as f64 && x > max[0] - HANDLE {
                Edit::Right
            } else {
                Edit::Move
            };
            ui.set_mouse_cursor(Some(match edit {
                Edit::Move => dear_imgui_rs::MouseCursor::ResizeAll,
                _ => dear_imgui_rs::MouseCursor::ResizeEW,
            }));
            if activated {
                editor.select_audio(clip.id);
                state.audio_drag = Some(ClipDrag {
                    draft: clip.clone(),
                    original: clip.clone(),
                    edit,
                    mouse_x: x,
                    mouse_y: ui.io().mouse_pos()[1],
                    insert_above: false,
                    started: false,
                    source_duration: editor.audio_source_duration(clip.id),
                });
                state.interaction = Interaction::Audio;
            }
        }
    }
    ui.set_cursor_screen_pos(panel_top);
    ui.dummy([layout.timeline_width, panel_height + 4.0]);
}

fn accept_file(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    path: Option<&std::path::Path>,
) -> bool {
    if editor.audio_editable() {
        if let Some(target) = ui.drag_drop_target() {
            if let Some(payload) = target
                .accept_payload_empty("KINEMATIC_AUDIO", dear_imgui_rs::DragDropTargetFlags::NONE)
            {
                if payload.delivery {
                    if let Some(path) = path {
                        editor.import_audio(path.to_owned());
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn context_menu(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State, clip: Option<u64>) {
    let right = dear_imgui_rs::MouseButton::Right;
    if ui.is_item_hovered() && ui.is_mouse_clicked(right) {
        state.interaction = Interaction::Audio;
    }
    if ui.is_item_hovered() && ui.is_mouse_released(right) && !state.right_dragged {
        if let Some(id) = clip {
            editor.select_audio(id);
        }
        ui.open_popup("Audio context");
    }
    if let Some(_popup) = ui.begin_popup("Audio context") {
        state.interaction = Interaction::Audio;
        let _disabled = ui.begin_disabled_with_cond(!editor.audio_editable());
        if clip.is_some() {
            if controls::menu_item(ui, "Split at playhead (Ctrl+B)") {
                editor.split_audio();
            }
            if controls::menu_item(ui, "Delete clip (Delete)") {
                editor.delete_audio();
            }
            widgets::separator(ui);
        }
        if let Some(_menu) = controls::menu(ui, "Import audio") {
            match editor.documents.as_mut().map(|docs| docs.files()) {
                Some(Ok(files)) => {
                    let mut any = false;
                    for entry in files
                        .into_iter()
                        .filter(|entry| !entry.directory && super::is_audio_file(&entry.path))
                    {
                        any = true;
                        if controls::menu_item(ui, entry.path.to_string_lossy()) {
                            editor.import_audio(entry.path);
                        }
                    }
                    if !any {
                        widgets::muted(ui, "Drag project audio from Files.");
                    }
                }
                Some(Err(error)) => widgets::message(ui, error),
                None => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn clip(id: u64, at: f64, duration: f64, lane: usize) -> AudioClip {
        AudioClip {
            id,
            lane,
            file: "assets/a.wav".into(),
            at,
            offset: 2.0,
            duration,
            gain: 1.0,
            pan: 0.0,
        }
    }

    #[test]
    fn movement_stops_at_neighbors_but_other_lanes_allow_overlap() {
        let original = clip(1, 2.0, 1.0, 0);
        let peers = [clip(2, 0.0, 1.5, 0), clip(3, 4.0, 1.0, 0)];
        let moved = edited(&original, Edit::Move, 100.0, 10.0);
        assert_eq!(
            constrained(moved.clone(), &original, Edit::Move, &peers)
                .unwrap()
                .at,
            3.0
        );
        let left = edited(&original, Edit::Move, -100.0, 10.0);
        assert_eq!(
            constrained(left, &original, Edit::Move, &peers).unwrap().at,
            1.5
        );
        let mut other_lane = moved;
        other_lane.lane = 1;
        other_lane.at = 4.0;
        assert_eq!(
            constrained(other_lane, &original, Edit::Move, &peers)
                .unwrap()
                .at,
            4.0
        );
        let occupied = [clip(2, 1.0, 3.0, 1)];
        let mut draft = original.clone();
        draft.lane = 1;
        assert!(constrained(draft, &original, Edit::Move, &occupied).is_none());
    }

    #[test]
    fn trimming_stops_at_neighbors_and_preserves_the_source() {
        let original = clip(1, 2.0, 1.0, 0);
        let peers = [clip(2, 0.0, 1.5, 0), clip(3, 4.0, 1.0, 0)];
        let left = constrained(
            edited(&original, Edit::Left, -10.0, 10.0),
            &original,
            Edit::Left,
            &peers,
        )
        .unwrap();
        assert_eq!((left.at, left.offset, left.duration), (1.5, 1.5, 1.5));
        let right = constrained(
            edited(&original, Edit::Right, 10.0, 10.0),
            &original,
            Edit::Right,
            &peers,
        )
        .unwrap();
        assert_eq!((right.at, right.offset, right.duration), (2.0, 2.0, 2.0));
    }

    #[test]
    fn moving_and_trimming_preserve_source_bounds() {
        let clip = AudioClip {
            id: 1,
            lane: 0,
            file: "assets/a.wav".into(),
            at: 2.0,
            offset: 1.0,
            duration: 3.0,
            gain: 0.5,
            pan: -0.2,
        };
        assert_eq!(edited(&clip, Edit::Move, -10.0, 5.0).at, 0.0);
        let left = edited(&clip, Edit::Left, -10.0, 5.0);
        assert_eq!((left.at, left.offset, left.duration), (1.0, 0.0, 4.0));
        let right = edited(&clip, Edit::Right, 10.0, 5.0);
        assert_eq!(right.duration, 4.0);
        assert!(edited(&clip, Edit::Left, 10.0, 5.0).duration > 0.0);
    }
}
