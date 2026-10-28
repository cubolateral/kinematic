use super::{Editor, SceneInstance, recalculate_scene_ranges};
use crate::scripting::{AudioClip, MANIFEST, Sources, audio};

impl Editor {
    pub(crate) fn audio_clips(&self, scene: usize) -> &[AudioClip] {
        self.lua_project
            .as_ref()
            .and_then(|project| project.manifest.audio.get(&project.manifest.scenes[scene]))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn audio_samples(
        &self,
        scene: usize,
        id: u64,
    ) -> Option<std::sync::Arc<Vec<[f32; 2]>>> {
        let index = self
            .audio_clips(scene)
            .iter()
            .position(|clip| clip.id == id)?;
        self.scenes[scene]
            .scene
            .audio
            .borrow()
            .clips
            .get(index)
            .map(|event| event.samples.clone())
    }

    pub(crate) fn audio_source_duration(&self, scene: usize, id: u64) -> f64 {
        self.audio_samples(scene, id).map_or(0.0, |samples| {
            samples.len() as f64 / crate::core::audio::SAMPLE_RATE as f64
        })
    }

    pub(crate) fn selected_audio(&self) -> Option<(usize, AudioClip)> {
        let (scene, id) = self.selection.audio()?;
        self.audio_clips(scene)
            .iter()
            .find(|clip| clip.id == id)
            .cloned()
            .map(|clip| (scene, clip))
    }

    pub(crate) fn select_audio(&mut self, scene: usize, id: u64) {
        self.selection.select_audio(scene, id);
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

    pub(crate) fn commit_audio(&mut self, scene: usize, clips: Vec<AudioClip>) {
        if let Err(error) = self.save_audio(scene, clips) {
            self.audio_error(error);
        }
    }

    fn save_audio(&mut self, scene: usize, clips: Vec<AudioClip>) -> Result<(), String> {
        if !self.audio_editable() {
            return Err("Apply or resolve source edits before editing audio.".into());
        }
        let project = self.lua_project.as_mut().ok_or("No Lua project is open.")?;
        let path = project.manifest.scenes[scene].clone();
        let mut manifest = project.manifest.clone();
        manifest.audio.insert(path, clips.clone());
        manifest.validate(&project.context)?;
        let SceneInstance::Lua(instance) = &self.scenes[scene].scene else {
            return Err("Correct the scene before editing its audio.".into());
        };
        let events = instance
            .prepare_audio_clips(&clips)
            .map_err(|e| e.to_string())?;
        let previous = std::mem::replace(&mut project.manifest, manifest);
        if let Err(error) = project.save() {
            project.manifest = previous;
            return Err(error);
        }
        if let SceneInstance::Lua(scene) = &mut self.scenes[scene].scene {
            scene.apply_audio_clips(&clips, events);
        }
        if let Some(documents) = &mut self.documents {
            if let Some(doc) = documents.open.get_mut(std::path::Path::new(MANIFEST)) {
                let text = ron::ser::to_string_pretty(
                    &project.manifest,
                    ron::ser::PrettyConfig::default(),
                )
                .map_err(|e| e.to_string())?;
                doc.disk = Some(text.as_bytes().to_vec());
                doc.text = text;
                doc.version += 1;
            }
            documents.status = "Audio clips saved.".into();
        }
        let duration = recalculate_scene_ranges(&mut self.scenes);
        let duration = self.focused_scene.map_or(duration, |index| {
            self.scenes[index].end - self.scenes[index].start
        });
        self.timeline.set_duration(duration);
        Ok(())
    }

    pub(crate) fn import_audio(&mut self, scene: usize, file: std::path::PathBuf) {
        let result = (|| {
            if !self.audio_editable() {
                return Err("Apply or resolve source edits before importing audio.".into());
            }
            let project = self.lua_project.as_ref().ok_or("No Lua project is open.")?;
            let bytes = Sources::default()
                .read(&project.context, &file)
                .map_err(|e| e.to_string())?;
            let samples = audio::load_file(
                bytes,
                std::time::Instant::now() + std::time::Duration::from_secs(10),
            )?;
            self.scenes[scene]
                .scene
                .audio
                .borrow_mut()
                .register_samples(&samples)?;
            let mut clips = self.audio_clips(scene).to_vec();
            let id = next_id(&clips)?;
            clips.push(AudioClip {
                id,
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
            self.save_audio(scene, clips)?;
            self.select_audio(scene, id);
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            self.audio_error(error);
        }
    }

    pub(crate) fn split_audio(&mut self) {
        let Some((scene, clip)) = self.selected_audio() else {
            return;
        };
        let mut clips = self.audio_clips(scene).to_vec();
        let id = match next_id(&clips) {
            Ok(id) => id,
            Err(error) => {
                self.audio_error(error);
                return;
            }
        };
        let time = (self.timeline.time() - self.scene_range(scene)[0]) as f64;
        let selected = clips.iter_mut().find(|item| item.id == clip.id).unwrap();
        if let Some(right) = selected.split(time, id) {
            clips.push(right);
            self.commit_audio(scene, clips);
        }
    }

    pub(crate) fn delete_audio(&mut self) {
        let Some((scene, clip)) = self.selected_audio() else {
            return;
        };
        let clips = self
            .audio_clips(scene)
            .iter()
            .filter(|item| item.id != clip.id)
            .cloned()
            .collect();
        self.commit_audio(scene, clips);
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
