use crate::renderer::RenderError;

pub(super) struct Encoder {
    stdin: Option<std::process::ChildStdin>,
    child: Option<std::process::Child>,
}

impl Encoder {
    pub fn new(
        output_path: &std::path::Path,
        resolution: (u32, u32),
        fps: u32,
        silent: bool,
    ) -> Result<Self, RenderError> {
        let resolution = format!("{}x{}", resolution.0, resolution.1);
        let fps = fps.to_string();
        let mut child = std::process::Command::new("ffmpeg")
            .args([
                "-y",
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
            .arg(output_path)
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

        Ok(Self {
            stdin: Some(stdin),
            child: Some(child),
        })
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<(), RenderError> {
        use std::io::Write;

        self.stdin
            .as_mut()
            .expect("FFmpeg standard input must remain open while encoding.")
            .write_all(frame)?;

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
    }
}
