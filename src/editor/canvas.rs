use crate::renderer::target::{Target, reset_gl};

pub(crate) struct Canvas {
    pub(crate) target: Target,
    imgui_texture_id: dear_imgui_rs::TextureId,
}

impl Canvas {
    pub fn new(
        size: (u32, u32),
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) -> Self {
        Self::new_with_depth(size, false, imgui_renderer, skia_context, gl)
    }

    pub fn new_3d(
        size: (u32, u32),
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) -> Self {
        Self::new_with_depth(size, true, imgui_renderer, skia_context, gl)
    }

    fn new_with_depth(
        size: (u32, u32),
        depth: bool,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) -> Self {
        let target =
            Target::new(size, depth, skia_context, gl).expect("Editor target must be created.");
        let imgui_texture_id = imgui_renderer.texture_map_mut().register_texture(
            target.texture(),
            size.0,
            size.1,
            dear_imgui_rs::TextureFormat::RGBA32,
        );
        Self {
            target,
            imgui_texture_id,
        }
    }
    pub fn draw(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        f: impl FnOnce(&skia_safe::Canvas),
    ) {
        self.target.draw_skia(skia_context, f);
        reset_gl(gl, window_size);
    }
    pub fn get_size(&self) -> (u32, u32) {
        self.target.size
    }
    pub fn get_imgui_texture_id(&self) -> dear_imgui_rs::TextureId {
        self.imgui_texture_id
    }
    pub fn get_framebuffer(&self) -> glow::NativeFramebuffer {
        self.target.framebuffer()
    }
}
