mod encoder;

use glow::HasContext;

use crate::renderer::encoder::Encoder;

const READBACK_BUFFER_COUNT: usize = 2;

pub enum FrameResult {
    Continue(f32),
    Finished,
}

struct Export {
    encoder: Encoder,
    output_path: std::path::PathBuf,
    frame_count: u64,
    frame_index: u64,
    fps: u32,
}

pub struct Renderer {
    export: Option<Export>,
    readback_buffers: Option<[glow::NativeBuffer; READBACK_BUFFER_COUNT]>,
    message: Option<String>,
    frame_size: usize,
    progress: f32,
}

impl Renderer {
    pub fn new(resolution: (u32, u32)) -> Self {
        Self {
            export: None,
            readback_buffers: None,
            message: None,
            frame_size: frame_size(resolution).unwrap_or(0),
            progress: 0.0,
        }
    }

    pub fn start(
        &mut self,
        project_name: &str,
        resolution: (u32, u32),
        fps: u32,
        duration: f32,
        silent: bool,
    ) -> bool {
        match self.start_export(project_name, resolution, fps, duration, silent) {
            Ok(export) => {
                self.export = Some(export);
                self.message = None;
                self.progress = 0.0;
                true
            }
            Err(error) => {
                self.message = Some(error.to_string());
                false
            }
        }
    }

    pub fn cancel(&mut self) {
        self.export.take();
        self.message = Some("Export canceled.".to_owned());
    }

    pub fn process_frame(
        &mut self,
        gl: &glow::Context,
        framebuffer: glow::NativeFramebuffer,
        resolution: (u32, u32),
    ) -> Result<FrameResult, RenderError> {
        self.ensure_readback_buffers(gl)?;

        let export = self
            .export
            .as_mut()
            .expect("An export must be active while processing a frame.");
        let frame_index = export.frame_index;
        let buffers = self
            .readback_buffers
            .as_ref()
            .expect("Readback buffers must exist while processing a frame.");

        enqueue_readback(gl, framebuffer, resolution, buffers, frame_index);

        if frame_index > 0 {
            write_readback(
                gl,
                buffers,
                self.frame_size,
                frame_index - 1,
                &mut export.encoder,
            )?;
        }

        export.frame_index += 1;
        self.progress = export.frame_index as f32 / export.frame_count as f32;

        if export.frame_index < export.frame_count {
            let next_time = frame_time(export.frame_index, export.fps);
            return Ok(FrameResult::Continue(next_time));
        }

        write_readback(
            gl,
            buffers,
            self.frame_size,
            frame_index,
            &mut export.encoder,
        )?;

        let export = self
            .export
            .take()
            .expect("An export must remain active until encoding finishes.");
        export.encoder.finish()?;

        self.message = Some(format!(
            "Export completed: {}.",
            export.output_path.display()
        ));

        Ok(FrameResult::Finished)
    }

    pub fn fail(&mut self, error: &RenderError) {
        self.export.take();
        self.message = Some(error.to_string());
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn shutdown(&mut self, gl: &glow::Context) {
        self.export.take();

        if let Some(buffers) = self.readback_buffers.take() {
            unsafe {
                gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);

                for buffer in buffers {
                    gl.delete_buffer(buffer);
                }
            }
        }
    }

    fn start_export(
        &mut self,
        project_name: &str,
        resolution: (u32, u32),
        fps: u32,
        duration: f32,
        silent: bool,
    ) -> Result<Export, RenderError> {
        validate_project(project_name, resolution, fps, duration)?;

        let output_path = output_path(project_name);
        let frame_size = frame_size(resolution)?;

        std::fs::create_dir_all("output")?;

        let encoder = Encoder::new(&output_path, resolution, fps, silent)?;
        let frame_count = frame_count(duration, fps);

        self.frame_size = frame_size;

        Ok(Export {
            encoder,
            output_path,
            frame_count,
            frame_index: 0,
            fps,
        })
    }

    fn ensure_readback_buffers(&mut self, gl: &glow::Context) -> Result<(), RenderError> {
        if self.readback_buffers.is_some() {
            return Ok(());
        }

        let buffer_size = i32::try_from(self.frame_size).map_err(|_| {
            RenderError::InvalidProject("Project frame size is too large for OpenGL.".to_owned())
        })?;
        let first = create_readback_buffer(gl, buffer_size)?;
        let second = match create_readback_buffer(gl, buffer_size) {
            Ok(buffer) => buffer,
            Err(error) => {
                unsafe {
                    gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
                    gl.delete_buffer(first);
                }

                return Err(error);
            }
        };

        unsafe {
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        }
        self.readback_buffers = Some([first, second]);

        Ok(())
    }
}

#[derive(Debug)]
pub enum RenderError {
    InvalidProject(String),
    Graphics(String),
    Io(std::io::Error),
    FfmpegFailed(std::process::ExitStatus),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProject(message) | Self::Graphics(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "Export I/O failed: {error}."),
            Self::FfmpegFailed(status) => {
                write!(
                    formatter,
                    "FFmpeg exited unsuccessfully with status {status}."
                )
            }
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RenderError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

fn validate_project(
    name: &str,
    resolution: (u32, u32),
    fps: u32,
    duration: f32,
) -> Result<(), RenderError> {
    let path = std::path::Path::new(name);
    let mut components = path.components();
    let valid_name = matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none();

    if !valid_name {
        return Err(RenderError::InvalidProject(
            "Project name must be a non-empty file name.".to_owned(),
        ));
    }
    if resolution.0 == 0 || resolution.1 == 0 {
        return Err(RenderError::InvalidProject(
            "Project resolution must be greater than zero.".to_owned(),
        ));
    }
    if !resolution.0.is_multiple_of(2) || !resolution.1.is_multiple_of(2) {
        return Err(RenderError::InvalidProject(
            "Project resolution must use even dimensions for YUV 4:2:0 output.".to_owned(),
        ));
    }
    if fps == 0 {
        return Err(RenderError::InvalidProject(
            "Project frame rate must be greater than zero.".to_owned(),
        ));
    }
    if !duration.is_finite() || duration < 0.0 {
        return Err(RenderError::InvalidProject(
            "Project duration must be a finite, non-negative value.".to_owned(),
        ));
    }

    frame_size(resolution)?;

    Ok(())
}

fn output_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new("output").join(format!("{name}.mp4"))
}

fn frame_count(duration: f32, fps: u32) -> u64 {
    ((duration * fps as f32).ceil() as u64).max(1)
}

fn frame_time(frame_index: u64, fps: u32) -> f32 {
    frame_index as f32 / fps as f32
}

fn frame_size(resolution: (u32, u32)) -> Result<usize, RenderError> {
    let size = usize::try_from(resolution.0)
        .ok()
        .and_then(|width| {
            usize::try_from(resolution.1)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            RenderError::InvalidProject("Project frame size is too large.".to_owned())
        })?;
    i32::try_from(size).map_err(|_| {
        RenderError::InvalidProject("Project frame size is too large for OpenGL.".to_owned())
    })?;

    Ok(size)
}

fn enqueue_readback(
    gl: &glow::Context,
    framebuffer: glow::NativeFramebuffer,
    resolution: (u32, u32),
    buffers: &[glow::NativeBuffer; READBACK_BUFFER_COUNT],
    frame_index: u64,
) {
    let buffer = buffers[(frame_index % READBACK_BUFFER_COUNT as u64) as usize];

    unsafe {
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(framebuffer));
        gl.read_buffer(glow::COLOR_ATTACHMENT0);
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(buffer));
        gl.read_pixels(
            0,
            0,
            resolution.0 as i32,
            resolution.1 as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::BufferOffset(0),
        );
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
    }
}

fn write_readback(
    gl: &glow::Context,
    buffers: &[glow::NativeBuffer; READBACK_BUFFER_COUNT],
    frame_size: usize,
    frame_index: u64,
    encoder: &mut Encoder,
) -> Result<(), RenderError> {
    let buffer = buffers[(frame_index % READBACK_BUFFER_COUNT as u64) as usize];

    unsafe {
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(buffer));

        let pointer = gl.map_buffer_range(
            glow::PIXEL_PACK_BUFFER,
            0,
            frame_size as i32,
            glow::MAP_READ_BIT,
        );
        if pointer.is_null() {
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);

            return Err(RenderError::Graphics(
                "OpenGL frame readback mapping failed.".to_owned(),
            ));
        }

        let frame = std::slice::from_raw_parts(pointer, frame_size);
        let write_result = encoder.write_frame(frame);

        gl.unmap_buffer(glow::PIXEL_PACK_BUFFER);
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);

        write_result
    }
}

fn create_readback_buffer(
    gl: &glow::Context,
    buffer_size: i32,
) -> Result<glow::NativeBuffer, RenderError> {
    let buffer = unsafe { gl.create_buffer() }.map_err(|error| {
        RenderError::Graphics(format!("OpenGL readback buffer creation failed: {error}."))
    })?;
    unsafe {
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(buffer));
        gl.buffer_data_size(glow::PIXEL_PACK_BUFFER, buffer_size, glow::STREAM_READ);
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_uses_the_project_name() {
        assert_eq!(
            output_path("Example project"),
            std::path::Path::new("output/Example project.mp4"),
        );
    }

    #[test]
    fn project_validation_rejects_directory_components() {
        assert!(validate_project("../Example", (1280, 720), 60, 1.0).is_err());
        assert!(validate_project("", (1280, 720), 60, 1.0).is_err());
    }

    #[test]
    fn project_validation_accepts_export_values() {
        assert!(validate_project("Example", (1280, 720), 60, 1.0).is_ok());
    }

    #[test]
    fn frame_count_covers_the_project_duration() {
        assert_eq!(frame_count(1.0, 60), 60);
        assert_eq!(frame_count(1.01, 60), 61);
        assert_eq!(frame_count(0.0, 60), 1);
    }

    #[test]
    fn frame_time_advances_by_the_project_frame_duration() {
        assert!((frame_time(1, 60) - 1.0 / 60.0).abs() < f32::EPSILON);
        assert!((frame_time(30, 60) - 0.5).abs() < f32::EPSILON);
    }
}
