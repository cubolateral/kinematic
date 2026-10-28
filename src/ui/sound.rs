use crate::editor::Editor;

pub(super) const WINDOW_NAME: &str = "Sound";

#[derive(Default)]
pub(super) struct State {
    edit: Option<(
        usize,
        crate::scripting::AudioClip,
        crate::scripting::AudioClip,
    )>,
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    ui.window(WINDOW_NAME).build(|| {
        super::timeline::audio_shortcuts(editor, ui, true);
        let clips = editor
            .scenes()
            .flat_map(|(scene, name, _, _)| {
                editor.audio_clips(scene).iter().map(move |clip| {
                    (
                        scene,
                        clip.id,
                        format!("{} / {} ({})", name, clip.file.display(), clip.id),
                    )
                })
            })
            .collect::<Vec<_>>();
        let current = editor
            .selected_audio()
            .map(|(scene, clip)| (scene, clip.id));
        if let Some(_combo) = ui.begin_combo(
            "Clip",
            current
                .and_then(|key| clips.iter().find(|(scene, id, _)| (*scene, *id) == key))
                .map_or("Select a clip", |(_, _, label)| label.as_str()),
        ) {
            for (scene, id, label) in &clips {
                if ui
                    .selectable_config(label)
                    .selected(current == Some((*scene, *id)))
                    .build()
                {
                    editor.select_audio(*scene, *id);
                }
            }
        }
        if let Some((scene, clip)) = editor.selected_audio() {
            draw_clip(editor, ui, state, scene, clip);
        } else {
            ui.text_wrapped(
                "Select an audio clip in the Timeline. Drag audio from Files onto Audio to import.",
            );
        }
    });
}

fn draw_clip(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    scene: usize,
    clip: crate::scripting::AudioClip,
) {
    if state
        .edit
        .as_ref()
        .is_none_or(|(index, original, _)| *index != scene || *original != clip)
    {
        state.edit = Some((scene, clip.clone(), clip.clone()));
    }
    ui.text("Audio clip");
    ui.text_wrapped(clip.file.to_string_lossy());
    ui.text_disabled(format!(
        "At: {:.3}s. Offset: {:.3}s. Duration: {:.3}s.",
        clip.at, clip.offset, clip.duration
    ));
    let editable = editor.audio_editable();
    let _disabled = ui.begin_disabled_with_cond(!editable);
    let (_, _, draft) = state.edit.as_mut().unwrap();
    let gain_changed = ui.slider("Volume", 0.0, 1.0, &mut draft.gain);
    let gain_done = ui.is_item_deactivated_after_edit();
    let pan_changed = ui.slider("Pan", -1.0, 1.0, &mut draft.pan);
    let pan_done = ui.is_item_deactivated_after_edit();
    if gain_changed || pan_changed {
        editor.timeline_mut().pause();
    }
    if ui.is_key_pressed(dear_imgui_rs::Key::Escape) || !editable {
        state.edit = None;
    } else if gain_done || pan_done {
        let (_, original, draft) = state.edit.take().unwrap();
        if draft != original {
            let mut clips = editor.audio_clips(scene).to_vec();
            if let Some(clip) = clips.iter_mut().find(|clip| clip.id == draft.id) {
                *clip = draft;
                editor.commit_audio(scene, clips);
            }
        }
    }
}
