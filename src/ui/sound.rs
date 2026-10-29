use crate::editor::Editor;
use kinematic_ui::widgets;
use kinematic_ui::{controls, properties, windows};

pub(super) const WINDOW_NAME: &str = "Sound";

#[derive(Default)]
pub(super) struct State {
    edit: Option<(crate::scripting::AudioClip, crate::scripting::AudioClip)>,
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    windows::panel(ui, WINDOW_NAME, || {
        super::timeline::audio_shortcuts(editor, ui, true);
        let clips = editor
            .audio_clips()
            .iter()
            .map(|clip| (clip.id, format!("{} ({})", clip.file.display(), clip.id)))
            .collect::<Vec<_>>();
        let current = editor.selected_audio().map(|clip| clip.id);
        let mut selected = current
            .and_then(|key| clips.iter().position(|(id, _)| *id == key))
            .unwrap_or(usize::MAX);
        let labels = clips
            .iter()
            .map(|(_, label)| label.as_str())
            .collect::<Vec<_>>();
        if properties::row(ui, "Clip", false, || {
            controls::choice(ui, "##clip", &mut selected, &labels)
        })
        .changed
        {
            editor.select_audio(clips[selected].0);
        }
        if let Some(clip) = editor.selected_audio() {
            draw_clip(editor, ui, state, clip);
        } else {
            widgets::message(
                ui,
                "Select an audio clip in the Timeline. Drag audio from Files onto Audio to import.",
            );
        }
    });
}

fn draw_clip(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    clip: crate::scripting::AudioClip,
) {
    if state
        .edit
        .as_ref()
        .is_none_or(|(original, _)| *original != clip)
    {
        state.edit = Some((clip.clone(), clip.clone()));
    }
    properties::section(ui, "Audio clip");
    widgets::message(ui, clip.file.to_string_lossy());
    widgets::muted(
        ui,
        format!(
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
