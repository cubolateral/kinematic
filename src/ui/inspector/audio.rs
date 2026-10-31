use crate::editor::Editor;
use kinematic_ui::properties;

#[derive(Default)]
pub(super) struct State {
    edit: Option<(crate::scripting::AudioClip, crate::scripting::AudioClip)>,
}

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    clip: crate::scripting::AudioClip,
) {
    let _id = ui.push_id(&format!("Audio:{}", clip.id));

    if state
        .edit
        .as_ref()
        .is_none_or(|(original, _)| *original != clip)
    {
        state.edit = Some((clip.clone(), clip.clone()));
    }
    super::header(
        ui,
        &clip.file.to_string_lossy(),
        &format!(
            "At: {:.3}s. Offset: {:.3}s. Duration: {:.3}s.",
            clip.at, clip.offset, clip.duration
        ),
    );
    let editable = editor.audio_editable();
    let _disabled = ui.begin_disabled_with_cond(!editable);
    let (_, draft) = state.edit.as_mut().unwrap();
    let gain = properties::slider(ui, "Volume", &mut draft.gain, 1.0, 0.0..=1.0);
    let pan = properties::slider(ui, "Pan", &mut draft.pan, 0.0, -1.0..=1.0);
    if gain.changed || pan.changed {
        editor.timeline_mut().pause();
    }
    if ui.is_key_pressed(dear_imgui_rs::Key::Escape) || !editable {
        state.edit = None;
    } else if gain.committed || pan.committed {
        let (original, draft) = state.edit.take().unwrap();
        if draft != original {
            let mut clips = editor.audio_clips().to_vec();
            if let Some(clip) = clips.iter_mut().find(|clip| clip.id == draft.id) {
                *clip = draft;
                editor.commit_audio(clips);
            }
        }
    }
}
