use super::{
    layout::{Layout, TimeRange},
    state::{Interaction, State},
};
use crate::{editor::Editor, scripting::AudioClip, ui::widgets::draw_panel_rect};

const HEIGHT: f32 = 64.0;
const HANDLE: f32 = 6.0;

#[derive(Clone, Copy)]
enum Edit {
    Move,
    Left,
    Right,
}

pub(super) struct ClipDrag {
    scene: usize,
    original: AudioClip,
    draft: AudioClip,
    edit: Edit,
    mouse_x: f32,
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
    let editable = editor.audio_editable();
    let left = dear_imgui_rs::MouseButton::Left;
    let right = dear_imgui_rs::MouseButton::Right;
    if state.audio_drag.is_none()
        && state.interaction == Interaction::Audio
        && !ui.is_mouse_down(left)
        && !ui.is_mouse_down(right)
    {
        state.interaction = Interaction::None;
    }
    let panel_top = ui.cursor_screen_pos();
    let header_height = 24.0;
    let panel_height = header_height
        + if state.audio_collapsed {
            0.0
        } else {
            HEIGHT + 8.0
        };
    draw_panel_rect(
        draw_list,
        panel_top,
        [layout.timeline_right(), panel_top[1] + panel_height],
        Some(ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg)),
        ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        1.0,
    );
    let clicked = ui.invisible_button("##audio_header", [layout.timeline_width, header_height]);
    if ui.is_item_active() || clicked {
        state.interaction = Interaction::Audio;
    }
    if accept_file(editor, ui, dragged_audio) {
        state.audio_collapsed = false;
    }
    context_menu(editor, ui, state, None);
    let arrow = if state.audio_collapsed {
        crate::ui::icons::CHEVRON_RIGHT
    } else {
        crate::ui::icons::CHEVRON_DOWN
    };
    draw_list.add_text(
        [panel_top[0] + 10.0, panel_top[1] + 4.0],
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        arrow,
    );
    draw_list.add_text(
        [panel_top[0] + 30.0, panel_top[1] + 4.0],
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        "Audio",
    );
    if clicked {
        state.audio_collapsed = !state.audio_collapsed;
    }
    if state.audio_collapsed {
        ui.set_cursor_screen_pos(panel_top);
        ui.dummy([layout.timeline_width, header_height + 4.0]);
        return;
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
        drag.draft = edited(&drag.original, drag.edit, delta, drag.source_duration);
        if !ui.is_mouse_down(left) {
            let drag = state.audio_drag.take().unwrap();
            state.interaction = Interaction::None;
            let mut clips = editor.audio_clips(drag.scene).to_vec();
            if let Some(clip) = clips.iter_mut().find(|clip| **clip == drag.original) {
                if *clip != drag.draft {
                    *clip = drag.draft;
                    editor.commit_audio(drag.scene, clips);
                }
            }
        }
    }
    let scenes = editor
        .scenes()
        .map(|(index, name, range, _)| (index, name.to_owned(), range))
        .collect::<Vec<_>>();
    let selected = editor
        .selected_audio()
        .map(|(scene, clip)| (scene, clip.id));
    state
        .waveforms
        .retain(|_, wave| wave.samples.strong_count() > 0);
    let top = [panel_top[0], panel_top[1] + header_height + 4.0];
    ui.set_cursor_screen_pos(top);
    ui.dummy([layout.timeline_width, HEIGHT]);
    accept_file(editor, ui, dragged_audio);
    for (scene, name, range) in scenes {
        let mut clips = editor.audio_clips(scene).to_vec();
        if let Some(drag) = &state.audio_drag {
            if drag.scene == scene {
                if let Some(clip) = clips.iter_mut().find(|clip| clip.id == drag.draft.id) {
                    *clip = drag.draft.clone();
                }
            }
        }
        if clips.is_empty() {
            continue;
        }
        clips.sort_by(|a, b| {
            (selected == Some((scene, a.id)))
                .cmp(&(selected == Some((scene, b.id))))
                .then(a.at.total_cmp(&b.at))
                .then(a.id.cmp(&b.id))
        });
        for clip in clips {
            let start = range[0] as f64 + clip.at;
            let end = start + clip.duration;
            if end <= time.start as f64 || start >= time.end as f64 {
                continue;
            }
            let min = [time.x(layout, start as f32), top[1]];
            let max = [time.x(layout, end as f32), min[1] + HEIGHT - 3.0];
            let _id = ui.push_id(&format!("audio:{scene}:{}", clip.id));
            ui.set_cursor_screen_pos(min);
            ui.invisible_button("##clip", [(max[0] - min[0]).max(1.0), max[1] - min[1]]);
            let hovered = ui.is_item_hovered();
            let activated = ui.is_item_activated();
            context_menu(editor, ui, state, Some((scene, clip.id)));
            #[cfg(test)]
            if selected == Some((scene, clip.id)) {
                state.audio_clip_rect = Some([min, max]);
            }

            let accent = ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark);
            let text = ui.get_color_u32(dear_imgui_rs::StyleColor::Text);
            draw_panel_rect(
                draw_list,
                min,
                max,
                Some(ui.get_color_u32(if hovered {
                    dear_imgui_rs::StyleColor::ButtonHovered
                } else {
                    dear_imgui_rs::StyleColor::Button
                })),
                if selected == Some((scene, clip.id)) {
                    accent
                } else {
                    text
                },
                1.0,
            );
            let _clip = draw_list.push_clip_rect(min, max, true);
            if let Some(samples) = editor.audio_samples(scene, clip.id) {
                let wave = state
                    .waveforms
                    .entry(std::sync::Arc::as_ptr(&samples) as usize)
                    .or_insert_with(|| super::waveform::Waveform::new(&samples));
                let rate = crate::core::audio::SAMPLE_RATE as f64;
                let center = min[1] + 41.0;
                let scale = 17.0 * clip.gain;
                let pixels = (max[0] - min[0]).ceil() as usize;
                for pixel in 0..pixels {
                    let source_at = |x: f32| {
                        clip.offset
                            + time.start as f64
                            + (x - layout.timeline_left) as f64 / layout.timeline_width as f64
                                * (time.end - time.start) as f64
                            - start
                    };
                    let x = min[0] + pixel as f32;
                    let from = (source_at(x).max(clip.offset) * rate).round() as usize;
                    let to = (source_at((x + 1.0).min(max[0])).min(clip.offset + clip.duration)
                        * rate)
                        .round() as usize;
                    let source_end = ((clip.offset + clip.duration) * rate).round() as usize;
                    let [low, high] = wave.range(
                        &samples,
                        from.min(source_end),
                        to.max(from.saturating_add(1)).min(source_end),
                    );
                    draw_list
                        .add_line(
                            [x, center - high * scale],
                            [x, center - low * scale],
                            accent,
                        )
                        .build();
                }
            }
            draw_list.add_text(
                [min[0] + HANDLE, min[1] + 3.0],
                text,
                clip.file.file_name().unwrap_or_default().to_string_lossy(),
            );
            draw_list
                .add_line([min[0] + HANDLE, min[1]], [min[0] + HANDLE, max[1]], text)
                .build();
            draw_list
                .add_line([max[0] - HANDLE, min[1]], [max[0] - HANDLE, max[1]], text)
                .build();
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
                    editor.select_audio(scene, clip.id);
                    editor.timeline_mut().pause();
                    state.audio_drag = Some(ClipDrag {
                        scene,
                        draft: clip.clone(),
                        original: clip.clone(),
                        edit,
                        mouse_x: x,
                        source_duration: editor.audio_source_duration(scene, clip.id),
                    });
                    state.interaction = Interaction::Audio;
                }
                if ui.is_mouse_hovering_rect(min, max) {
                    ui.tooltip_text(format!("{} — {}\nAt: {:.3}s. Offset: {:.3}s. Duration: {:.3}s.\nDrag the body to move; drag either edge to trim.", name, clip.file.display(), clip.at, clip.offset, clip.duration));
                }
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
                        editor.import_audio(editor.active_scene_index(), path.to_owned());
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn context_menu(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    clip: Option<(usize, u64)>,
) {
    if ui.is_item_hovered() && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Right) {
        if let Some((scene, id)) = clip {
            editor.select_audio(scene, id);
        }
        state.interaction = Interaction::Audio;
    }
    if let Some(_popup) = ui.begin_popup_context_item() {
        state.interaction = Interaction::Audio;
        let _disabled = ui.begin_disabled_with_cond(!editor.audio_editable());
        if clip.is_some() {
            if ui.menu_item("Split at playhead (Ctrl+B)") {
                editor.split_audio();
            }
            if ui.menu_item("Delete clip (Delete)") {
                editor.delete_audio();
            }
            ui.separator();
        }
        if let Some(_menu) = ui.begin_menu("Import audio") {
            match editor.documents.as_mut().map(|docs| docs.files()) {
                Some(Ok(files)) => {
                    let mut any = false;
                    for entry in files
                        .into_iter()
                        .filter(|entry| !entry.directory && super::is_audio_file(&entry.path))
                    {
                        any = true;
                        if ui.menu_item(entry.path.to_string_lossy()) {
                            editor.import_audio(editor.active_scene_index(), entry.path);
                        }
                    }
                    if !any {
                        ui.text_disabled("Drag project audio from Files.");
                    }
                }
                Some(Err(error)) => ui.text_wrapped(error),
                None => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_and_trimming_preserve_source_bounds() {
        let clip = AudioClip {
            id: 1,
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
