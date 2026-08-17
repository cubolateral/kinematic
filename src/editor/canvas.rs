use glow::HasContext;

pub(crate) struct Canvas {
    size: (u32, u32),
    imgui_texture_id: dear_imgui_rs::TextureId,
    vg_image_id: femtovg::ImageId,
}

impl Canvas {
    pub fn new(
        size: (u32, u32),
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
    ) -> Self {
        let vg_image_id = vg
            .create_image_empty(
                size.0 as usize,
                size.1 as usize,
                femtovg::PixelFormat::Rgba8,
                femtovg::ImageFlags::empty(),
            )
            .unwrap();

        let imgui_texture_id = imgui_renderer.texture_map_mut().register_texture(
            vg.get_native_texture(vg_image_id).unwrap(),
            size.0,
            size.1,
            dear_imgui_rs::TextureFormat::RGBA32,
        );

        Self {
            size: (size.0, size.1),
            vg_image_id,
            imgui_texture_id,
        }
    }

    pub fn draw(
        &self,
        window_size: (u32, u32),
        gl: &glow::Context,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        f: impl FnOnce(&mut femtovg::Canvas<femtovg::renderer::OpenGl>, femtovg::RenderTarget),
    ) {
        let target = femtovg::RenderTarget::Image(self.vg_image_id);
        vg.set_render_target(target);
        f(vg, target);
        vg.flush();
        vg.set_render_target(femtovg::RenderTarget::Screen);

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
        }
    }

    pub fn _destroy(
        self,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
    ) {
        imgui_renderer
            .texture_map_mut()
            .remove(self.imgui_texture_id);
        vg.delete_image(self.vg_image_id);
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }

    pub fn get_imgui_texture_id(&self) -> dear_imgui_rs::TextureId {
        self.imgui_texture_id
    }
}
