use glow::HasContext;

pub(crate) struct Canvas {
    size: (u32, u32),
    imgui_texture_id: dear_imgui_rs::TextureId,
    framebuffer: glow::NativeFramebuffer,
    surface: skia_safe::Surface,
}

impl Canvas {
    pub fn new(
        size: (u32, u32),
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
    ) -> Self {
        let width = i32::try_from(size.0).expect("Preview width must fit in an i32.");
        let height = i32::try_from(size.1).expect("Preview height must fit in an i32.");

        let texture = unsafe { gl.create_texture() }.expect("Preview texture must be created.");
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
        }

        let framebuffer =
            unsafe { gl.create_framebuffer() }.expect("Preview framebuffer must be created.");
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE,
                "Preview framebuffer must be complete."
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        let framebuffer_info = skia_safe::gpu::gl::FramebufferInfo {
            fboid: framebuffer.0.get(),
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
            ..Default::default()
        };
        let backend_render_target = skia_safe::gpu::backend_render_targets::make_gl(
            (width, height),
            0,
            0,
            framebuffer_info,
        );
        let surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
            skia_context,
            &backend_render_target,
            skia_safe::gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .expect("Preview Skia surface must be created.");

        let imgui_texture_id = imgui_renderer.texture_map_mut().register_texture(
            texture,
            size.0,
            size.1,
            dear_imgui_rs::TextureFormat::RGBA32,
        );

        Self {
            size,
            imgui_texture_id,
            framebuffer,
            surface,
        }
    }

    pub fn draw(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        f: impl FnOnce(&skia_safe::Canvas),
    ) {
        skia_context.reset(None);

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
        }

        f(self.surface.canvas());

        skia_context.flush_and_submit();

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }

    pub fn get_imgui_texture_id(&self) -> dear_imgui_rs::TextureId {
        self.imgui_texture_id
    }

    pub fn get_framebuffer(&self) -> glow::NativeFramebuffer {
        self.framebuffer
    }
}
