use super::Editor;
use crate::scripting::{AudioClip, MANIFEST, Sources, audio};

impl Editor {
    pub(crate) fn audio_clips(&self) -> &[AudioClip] {
        self.lua_project
            .as_ref()
            .map_or(&[], |project| project.manifest.audio.as_slice())
    }

    pub(crate) fn audio_samples(&self, id: u64) -> Option<std::sync::Arc<Vec<[f32; 2]>>> {
        let index = self.audio_clips().iter().position(|clip| clip.id == id)?;
        self.project_audio
            .clips
            .get(index)
            .map(|event| event.samples.clone())
    }

    pub(crate) fn audio_source_duration(&self, id: u64) -> f64 {
        self.audio_samples(id).map_or(0.0, |samples| {
            samples.len() as f64 / crate::core::audio::SAMPLE_RATE as f64
        })
    }

    pub(crate) fn selected_audio(&self) -> Option<AudioClip> {
        let id = self.selection.audio()?;
        self.audio_clips()
            .iter()
            .find(|clip| clip.id == id)
            .cloned()
    }

    pub(crate) fn select_audio(&mut self, id: u64) {
        self.selection.select_audio(id);
    }
    pub(crate) fn clear_audio_selection(&mut self) {
        self.selection.clear_audio();
    }

    pub(crate) fn timeline_origin(&self) -> f32 {
        self.focused_scene
            .map_or(0.0, |index| self.scenes[index].start)
    }

    pub(crate) fn absolute_time(&self) -> f32 {
        self.timeline_origin() + self.timeline.time()
    }

    pub(crate) fn project_duration(&self) -> f32 {
        self.project_audio
            .clips
            .iter()
            .map(|event| {
                (event.time + event.frames as f64 / crate::core::audio::SAMPLE_RATE as f64) as f32
            })
            .fold(self.scenes.last().map_or(0.0, |scene| scene.end), f32::max)
    }

    pub(crate) fn audio_editable(&self) -> bool {
        !self.is_exporting
            && self.pending_lua.is_none()
            && self.documents.as_ref().is_some_and(|docs| {
                docs.changed.is_empty()
                    && !docs
                        .open
                        .get(std::path::Path::new(MANIFEST))
                        .is_some_and(|doc| doc.dirty() || doc.conflict)
            })
    }

    pub(crate) fn audio_error(&mut self, error: String) {
        if let Some(documents) = &mut self.documents {
            documents.status = error.clone();
            documents.diagnostics.push(error);
        }
    }

    pub(crate) fn commit_audio(&mut self, clips: Vec<AudioClip>) {
        if let Err(error) = self.edit_audio(clips) {
            self.audio_error(error);
        }
    }

    fn edit_audio(&mut self, clips: Vec<AudioClip>) -> Result<(), String> {
        if !self.audio_editable() {
            return Err("Apply or resolve source edits before editing audio.".into());
        }
        let project = self.lua_project.as_ref().ok_or("No Lua project is open.")?;
        let mut manifest = project.manifest.clone();
        manifest.audio = clips.clone();
        manifest.validate(&project.context)?;
        let mut samples = self
            .audio_clips()
            .iter()
            .zip(&self.project_audio.clips)
            .map(|(clip, event)| (clip.file.clone(), event.samples.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut events = Vec::with_capacity(clips.len());
        for clip in &clips {
            let sample = if let Some(sample) = samples.get(&clip.file) {
                sample.clone()
            } else {
                let project = self.lua_project.as_ref().unwrap();
                let bytes = self
                    .documents
                    .as_ref()
                    .unwrap()
                    .sources()
                    .read_audio(&project.context, &clip.file)
                    .map_err(|e| e.to_string())?;
                let sample = audio::load_file(
                    bytes,
                    std::time::Instant::now() + std::time::Duration::from_secs(10),
                )?;
                samples.insert(clip.file.clone(), sample.clone());
                sample
            };
            events.push(audio::clip_event(clip, sample)?);
        }
        self.lua_project.as_mut().unwrap().manifest = manifest;
        self.project_audio.clips = events;
        if let Some(documents) = &mut self.documents {
            documents.project_dirty |= clips != self.saved_audio;
            documents.status = if documents.project_dirty {
                "Unsaved project changes."
            } else {
                "Project is current."
            }
            .into();
        }
        let duration = self.project_duration();
        let duration = self.focused_scene.map_or(duration, |index| {
            self.scenes[index].end - self.scenes[index].start
        });
        self.timeline.set_duration(duration);
        Ok(())
    }

    pub(crate) fn save_project(&mut self) -> Result<(), String> {
        if !self
            .documents
            .as_ref()
            .is_some_and(|documents| documents.project_dirty)
        {
            return Ok(());
        }
        let documents = self.documents.as_mut().unwrap();
        let path = std::path::Path::new(MANIFEST);
        if documents
            .open
            .get(path)
            .is_some_and(|document| document.dirty() || document.conflict)
        {
            return Err(
                "Save or resolve manifest text edits before saving project changes.".into(),
            );
        }
        let project = self.lua_project.as_mut().ok_or("No Lua project is open.")?;
        project.save()?;
        self.saved_audio = project.manifest.audio.clone();
        documents.project_dirty = false;
        if let Some(document) = documents.open.get_mut(path) {
            let text =
                ron::ser::to_string_pretty(&project.manifest, ron::ser::PrettyConfig::default())
                    .map_err(|error| error.to_string())?;
            document.disk = Some(text.as_bytes().to_vec());
            document.text = text;
            document.version += 1;
        }
        documents.status = "Project saved.".into();
        Ok(())
    }

    pub(crate) fn import_audio(&mut self, file: std::path::PathBuf) {
        let result = (|| {
            if !self.audio_editable() {
                return Err("Apply or resolve source edits before importing audio.".into());
            }
            let project = self.lua_project.as_ref().ok_or("No Lua project is open.")?;
            let bytes = Sources::default()
                .read_audio(&project.context, &file)
                .map_err(|e| e.to_string())?;
            let samples = audio::load_file(
                bytes,
                std::time::Instant::now() + std::time::Duration::from_secs(10),
            )?;
            let mut clips = self.audio_clips().to_vec();
            let id = next_id(&clips)?;
            clips.push(AudioClip {
                id,
                lane: 0,
                file,
                at: clips
                    .iter()
                    .map(|clip| clip.at + clip.duration)
                    .fold(0.0, f64::max),
                offset: 0.0,
                duration: samples.len() as f64 / crate::core::audio::SAMPLE_RATE as f64,
                gain: 1.0,
                pan: 0.0,
            });
            self.edit_audio(clips)?;
            self.select_audio(id);
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            self.audio_error(error);
        }
    }

    pub(crate) fn split_audio(&mut self) {
        let Some(clip) = self.selected_audio() else {
            return;
        };
        let mut clips = self.audio_clips().to_vec();
        let id = match next_id(&clips) {
            Ok(id) => id,
            Err(error) => {
                self.audio_error(error);
                return;
            }
        };
        let time = self.absolute_time() as f64;
        let selected = clips.iter_mut().find(|item| item.id == clip.id).unwrap();
        if let Some(right) = selected.split(time, id) {
            clips.push(right);
            self.commit_audio(clips);
        }
    }

    pub(crate) fn delete_audio(&mut self) {
        let Some(clip) = self.selected_audio() else {
            return;
        };
        let clips = self
            .audio_clips()
            .iter()
            .filter(|item| item.id != clip.id)
            .cloned()
            .collect();
        self.commit_audio(clips);
    }
}

fn next_id(clips: &[AudioClip]) -> Result<u64, String> {
    clips
        .iter()
        .map(|clip| clip.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "Audio clip IDs are exhausted.".into())
}
