use glow::HasContext;
use std::rc::Rc;

/// Shared offscreen storage for Skia, three-d and the final editor output.
pub(crate) struct Target {
    pub size: (u32, u32),
    pub surface: Option<skia_safe::Surface>,
    texture: Option<glow::NativeTexture>,
    framebuffer: Option<glow::NativeFramebuffer>,
    depth: Option<glow::NativeRenderbuffer>,
    gl: Rc<glow::Context>,
}

impl Target {
    pub fn new(
        size: (u32, u32),
        depth: bool,
        skia: &mut skia_safe::gpu::DirectContext,
        gl: &Rc<glow::Context>,
    ) -> Result<Self, String> {
        let limit = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) } as u32;
        if size.0 == 0 || size.1 == 0 || size.0 > limit || size.1 > limit {
            return Err(format!(
                "Canvas dimensions must be between 1 and {limit} pixels."
            ));
        }
        let mut target = Self {
            size,
            surface: None,
            texture: None,
            framebuffer: None,
            depth: None,
            gl: Rc::clone(gl),
        };
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_sampler(0, None);
            gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
            target.texture = Some(gl.create_texture()?);
            gl.bind_texture(glow::TEXTURE_2D, target.texture);
            for parameter in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::LINEAR as i32);
            }
            for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::CLAMP_TO_EDGE as i32);
            }
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                size.0 as i32,
                size.1 as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            target.framebuffer = Some(gl.create_framebuffer()?);
            gl.bind_framebuffer(glow::FRAMEBUFFER, target.framebuffer);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                target.texture,
                0,
            );
            if depth {
                target.depth = Some(gl.create_renderbuffer()?);
                gl.bind_renderbuffer(glow::RENDERBUFFER, target.depth);
                gl.renderbuffer_storage(
                    glow::RENDERBUFFER,
                    glow::DEPTH_COMPONENT24,
                    size.0 as i32,
                    size.1 as i32,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::RENDERBUFFER,
                    target.depth,
                );
                gl.bind_renderbuffer(glow::RENDERBUFFER, None);
            }
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                return Err("Canvas framebuffer is incomplete.".into());
            }
        }
        if !depth {
            skia.reset(None);
            let info = skia_safe::gpu::gl::FramebufferInfo {
                fboid: target.framebuffer().0.get(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            };
            let backend = skia_safe::gpu::backend_render_targets::make_gl(
                (size.0 as i32, size.1 as i32),
                0,
                0,
                info,
            );
            target.surface = Some(
                skia_safe::gpu::surfaces::wrap_backend_render_target(
                    skia,
                    &backend,
                    skia_safe::gpu::SurfaceOrigin::BottomLeft,
                    skia_safe::ColorType::RGBA8888,
                    None,
                    None,
                )
                .ok_or("Canvas Skia surface could not be created.")?,
            );
        }
        Ok(target)
    }

    pub fn texture(&self) -> glow::NativeTexture {
        self.texture.unwrap()
    }
    pub fn framebuffer(&self) -> glow::NativeFramebuffer {
        self.framebuffer.unwrap()
    }

    pub fn draw_skia(
        &mut self,
        skia: &mut skia_safe::gpu::DirectContext,
        f: impl FnOnce(&skia_safe::Canvas),
    ) {
        skia.reset(None);
        unsafe {
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, self.framebuffer);
        }
        f(self
            .surface
            .as_mut()
            .expect("Skia drawing requires a 2D target.")
            .canvas());
        skia.flush_and_submit();
    }

    /// Presents premultiplied RGB over the project's opaque black background.
    pub fn present_to(&self, destination: &Target) {
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.disable(glow::FRAMEBUFFER_SRGB);
            self.gl
                .bind_framebuffer(glow::READ_FRAMEBUFFER, self.framebuffer);
            self.gl
                .bind_framebuffer(glow::DRAW_FRAMEBUFFER, destination.framebuffer);
            self.gl.blit_framebuffer(
                0,
                0,
                self.size.0 as i32,
                self.size.1 as i32,
                0,
                0,
                destination.size.0 as i32,
                destination.size.1 as i32,
                glow::COLOR_BUFFER_BIT,
                glow::LINEAR,
            );
            // RGB already equals the result of compositing premultiplied color over black.
            self.gl.color_mask(false, false, false, true);
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.color_mask(true, true, true, true);
        }
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        self.surface.take();
        unsafe {
            if let Some(depth) = self.depth.take() {
                self.gl.delete_renderbuffer(depth);
            }
            if let Some(framebuffer) = self.framebuffer.take() {
                self.gl.delete_framebuffer(framebuffer);
            }
            if let Some(texture) = self.texture.take() {
                self.gl.delete_texture(texture);
            }
        }
    }
}

/// Configures the shared context at backend boundaries; each backend sets its own drawing state.
pub(crate) fn reset_gl(gl: &glow::Context, size: (u32, u32)) {
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.viewport(0, 0, size.0 as i32, size.1 as i32);
        gl.disable(glow::DEPTH_TEST);
        gl.depth_mask(true);
        gl.depth_func(glow::LESS);
        gl.disable(glow::BLEND);
        gl.blend_equation(glow::FUNC_ADD);
        gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
        gl.disable(glow::CULL_FACE);
        gl.front_face(glow::CCW);
        gl.disable(glow::SCISSOR_TEST);
        gl.disable(glow::FRAMEBUFFER_SRGB);
        gl.color_mask(true, true, true, true);
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.bind_sampler(0, None);
        gl.use_program(None);
        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
    }
}
