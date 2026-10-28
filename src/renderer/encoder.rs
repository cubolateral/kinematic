use crate::renderer::RenderError;

pub(super) struct Encoder {
    stdin: Option<std::process::ChildStdin>,
    directory: std::path::PathBuf,
    output: std::path::PathBuf,
    audio: Option<std::io::BufWriter<std::fs::File>>,
    silent: bool,
    owns_output: bool,
    child: Option<std::process::Child>,
}

impl Encoder {
    pub fn new(
        output_path: &std::path::Path,
        resolution: (u32, u32),
        fps: u32,
        silent: bool,
    ) -> Result<Self, RenderError> {
        static NEXT_EXPORT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let directory = loop {
            let id = NEXT_EXPORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("kinematic-audio-{}-{id}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        };
        let mut encoder = Self {
            stdin: None,
            child: None,
            directory,
            output: output_path.to_owned(),
            audio: None,
            silent,
            owns_output: false,
        };
        encoder.audio = Some(std::io::BufWriter::new(std::fs::File::create(
            encoder.directory.join("audio.f32"),
        )?));
        let resolution = format!("{}x{}", resolution.0, resolution.1);
        let fps = fps.to_string();
        let mut child = std::process::Command::new("ffmpeg")
            .args([
                "-n",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgba",
                "-video_size",
                &resolution,
                "-framerate",
                &fps,
                "-i",
                "-",
                "-vf",
                "vflip,format=yuv420p",
                "-c:v",
                "libx264",
                "-preset",
                "slow",
                "-crf",
                "18",
                "-tune",
                "animation",
                "-movflags",
                "+faststart",
            ])
            .arg(encoder.directory.join("video.mp4"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(if silent {
                std::process::Stdio::null()
            } else {
                std::process::Stdio::inherit()
            })
            .spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| {
            RenderError::Io(std::io::Error::other(
                "FFmpeg standard input could not be opened.",
            ))
        })?;

        encoder.stdin = Some(stdin);
        encoder.child = Some(child);
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output_path)?;
        encoder.owns_output = true;
        Ok(encoder)
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<(), RenderError> {
        use std::io::Write;

        self.stdin
            .as_mut()
            .expect("FFmpeg standard input must remain open while encoding.")
            .write_all(frame)?;

        Ok(())
    }

    pub fn write_audio(&mut self, samples: &[f32]) -> Result<(), RenderError> {
        use std::io::Write;
        let audio = self
            .audio
            .as_mut()
            .expect("Audio file must remain open during export.");
        for sample in samples {
            audio.write_all(&sample.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), RenderError> {
        self.stdin.take();

        let status = self
            .child
            .take()
            .expect("FFmpeg process must exist while encoding.")
            .wait()?;

        if !status.success() {
            return Err(RenderError::FfmpegFailed(status));
        }

        use std::io::Write;
        if let Some(mut audio) = self.audio.take() {
            audio.flush()?;
        }
        let status = std::process::Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(self.directory.join("video.mp4"))
            .args(["-f", "f32le", "-ar", "48000", "-ac", "2", "-i"])
            .arg(self.directory.join("audio.f32"))
            .args([
                "-map",
                "0:v:0",
                "-map",
                "1:a:0",
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-movflags",
                "+faststart",
            ])
            .arg(&self.output)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(if self.silent {
                std::process::Stdio::null()
            } else {
                std::process::Stdio::inherit()
            })
            .status()?;
        if !status.success() {
            return Err(RenderError::FfmpegFailed(status));
        }
        self.owns_output = false;
        Ok(())
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        self.stdin.take();

        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.audio.take();
        if self.owns_output {
            let _ = std::fs::remove_file(&self.output);
        }
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::audio::{AudioEvents, SAMPLE_RATE, Sound, SoundEvent, limit};

    #[test]
    #[ignore = "Requires FFmpeg with libx264 and AAC."]
    fn exported_audio_keeps_sample_timing_and_cleans_temporary_files() {
        let output =
            std::env::temp_dir().join(format!("kinematic-audio-test-{}.mp4", std::process::id()));
        let mut encoder = Encoder::new(&output, (2, 2), 30, true).unwrap();
        let directory = encoder.directory.clone();
        let sound = Sound::preset("boink").unwrap();
        let mut audio = AudioEvents::default();
        audio.scheduled.push(SoundEvent {
            time: 0.25,
            offset: 0,
            frames: sound.sample_count(),
            samples: sound.samples(),
            gain: 0.2,
            pan: 0.0,
        });
        audio.clips.push(SoundEvent {
            time: 0.5,
            offset: 240,
            frames: 480,
            samples: sound.samples(),
            gain: 0.4,
            pan: -0.5,
        });
        let mut samples = vec![[0.0; 2]; SAMPLE_RATE as usize];
        audio.mix(0.0, 0, &mut samples);
        let expected = limit(samples);
        for (frame, chunk) in expected.chunks(3200).enumerate() {
            encoder.write_audio(chunk).unwrap();
            encoder.write_frame(&[frame as u8; 16]).unwrap();
        }
        encoder.finish().unwrap();
        assert!(!directory.exists());
        let decoded = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-i"])
            .arg(&output)
            .args(["-map", "0:a:0", "-f", "f32le", "-acodec", "pcm_f32le", "-"])
            .output()
            .unwrap();
        std::fs::remove_file(output).unwrap();
        assert!(decoded.status.success());
        let actual = decoded
            .stdout
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert!(actual.len() >= expected.len());
        assert!(actual[..19_200].iter().all(|value| value.abs() < 0.001));
        assert!(
            actual[24_000..30_000]
                .iter()
                .any(|value| value.abs() > 0.01)
        );
        let error = actual
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            / expected.len() as f32;
        assert!(error.sqrt() < 0.01, "AAC error: {}.", error.sqrt());
        let encoder = Encoder::new(
            &std::env::temp_dir().join("unused-kinematic-audio.mp4"),
            (2, 2),
            30,
            true,
        )
        .unwrap();
        let directory = encoder.directory.clone();
        drop(encoder);
        assert!(!directory.exists());
    }
}
