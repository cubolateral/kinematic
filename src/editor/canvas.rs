use crate::renderer::target::Target;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum PreviewQuality {
    #[serde(alias = "Sixteenth")]
    Eighth,
    Quarter,
    Half,
    #[default]
    Full,
    Double,
}

impl PreviewQuality {
    pub const ALL: [Self; 5] = [
        Self::Eighth,
        Self::Quarter,
        Self::Half,
        Self::Full,
        Self::Double,
    ];
    pub const LABELS: [&str; 5] = ["1/8x", "1/4x", "1/2x", "1x", "2x"];

    pub fn scale(self) -> f32 {
        match self {
            Self::Eighth => 0.125,
            Self::Quarter => 0.25,
            Self::Half => 0.5,
            Self::Full => 1.0,
            Self::Double => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ViewportQualities {
    pub preview: PreviewQuality,
    pub editor_2d: PreviewQuality,
    pub editor_3d: PreviewQuality,
}

pub(crate) struct Canvas {
    pub(crate) target: Target,
    imgui_texture_id: dear_imgui_glow::ExternalTextureId,
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
        let imgui_texture_id = imgui_renderer
            .register_external_texture(glow_imgui::NativeTexture(target.texture().0))
            .expect("Preview texture must register with ImGui.");

        Self {
            target,
            imgui_texture_id,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.target.size
    }

    pub fn imgui_texture_id(&self) -> dear_imgui_rs::TextureId {
        self.imgui_texture_id.texture_id()
    }

    pub(crate) fn unregister(&self, renderer: &mut dear_imgui_glow::GlowRenderer) {
        renderer
            .unregister_external_texture(self.imgui_texture_id)
            .expect("Preview texture must still be registered.");
    }

    pub fn framebuffer(&self) -> glow::NativeFramebuffer {
        self.target.framebuffer()
    }
}
