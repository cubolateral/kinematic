use crate::{core::audio::SAMPLE_RATE, editor::Editor};

pub(crate) struct AudioPreview {
    stream: sdl3::audio::AudioStreamOwner,
    cursor: Option<(u64, u64)>,
}

impl AudioPreview {
    pub fn new(sdl: &sdl3::Sdl) -> Result<Self, String> {
        let audio = sdl.audio().map_err(|e| e.to_string())?;
        let spec = sdl3::audio::AudioSpec::new(
            Some(SAMPLE_RATE as i32),
            Some(2),
            Some(sdl3::audio::AudioFormat::f32_sys()),
        );
        let stream = audio
            .open_playback_device(&spec)
            .and_then(|device| device.open_device_stream(Some(&spec)))
            .map_err(|e| e.to_string())?;
        stream.resume().map_err(|e| e.to_string())?;
        Ok(Self {
            stream,
            cursor: None,
        })
    }

    pub fn update(&mut self, editor: Option<&Editor>, volume: f32) -> Result<(), String> {
        self.stream.set_gain(volume).map_err(|e| e.to_string())?;
        let transport = editor.map(Editor::audio_transport);
        if let (Some(editor), Some((true, revision, end))) = (editor, transport) {
            if let Some((previous_revision, start)) = self.cursor
                && previous_revision == revision
                && end >= start
            {
                if end > start {
                    let max_queued = (end - start).max(u64::from(SAMPLE_RATE) / 4) * 8;
                    if self.stream.queued_bytes().map_err(|e| e.to_string())? as u64 > max_queued {
                        self.stream.clear().map_err(|e| e.to_string())?;
                    }
                    self.stream
                        .put_data_f32(&editor.mix_audio(start, end))
                        .map_err(|e| e.to_string())?;
                }
            } else {
                self.stream.clear().map_err(|e| e.to_string())?;
            }
            self.cursor = Some((revision, end));
        } else if let Some((revision, start)) = self.cursor.take() {
            if let (Some(editor), Some((false, current_revision, end))) = (editor, transport)
                && current_revision == revision
                && end > start
            {
                self.stream
                    .put_data_f32(&editor.mix_audio(start, end))
                    .map_err(|e| e.to_string())?;
            } else {
                self.stream.clear().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires an SDL audio driver; SDL_AUDIODRIVER=dummy is sufficient."]
    fn preview_accepts_the_shared_stereo_samples_and_clears_its_queue() {
        let sdl = sdl3::init().unwrap();
        let mut preview = AudioPreview::new(&sdl).unwrap();
        let sound = crate::core::audio::Sound::preset("boink").unwrap();
        let samples = crate::core::audio::limit((*sound.samples()).clone());
        preview.stream.put_data_f32(&samples).unwrap();
        preview.cursor = Some((0, 0));
        preview.update(None, 0.5).unwrap();
        assert_eq!(preview.stream.get_gain().unwrap(), 0.5);
        preview.update(None, 0.0).unwrap();
        assert_eq!(preview.stream.get_gain().unwrap(), 0.0);
        preview.update(None, 1.0).unwrap();
        assert_eq!(preview.stream.get_gain().unwrap(), 1.0);
        assert_eq!(preview.stream.queued_bytes().unwrap(), 0);
    }
}
