use dear_imgui_rs::{ColorStackToken, Context, FontId, Ui};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(super) struct Appearance {
    pub background: [f32; 4],
    pub accent: [f32; 4],
    pub contrast: f32,
    pub scale: f32,
    pub editor: super::code::Settings,
}

impl Default for Appearance {
    fn default() -> Self {
        let visual = kinematic_ui::theme::Theme::default();
        Self {
            background: visual.background,
            accent: visual.accent,
            contrast: visual.contrast,
            scale: visual.scale,
            editor: super::code::Settings::default(),
        }
    }
}

impl Appearance {
    pub fn apply_scale(&self, context: &mut Context) {
        self.visual().apply_scale(context);
    }

    fn visual(&self) -> kinematic_ui::theme::Theme {
        kinematic_ui::theme::Theme {
            background: self.background,
            accent: self.accent,
            contrast: self.contrast,
            scale: self.scale,
        }
    }

    pub fn push<'ui>(&self, ui: &'ui Ui) -> Vec<ColorStackToken<'ui>> {
        self.visual().push(ui)
    }
}

pub(super) fn initialize(context: &mut Context) -> FontId {
    kinematic_ui::theme::initialize(
        context,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/JetBrainsMonoNerdFont-Regular.ttf"
        )),
    )
}

pub(super) fn load_bold_font(context: &mut Context) -> FontId {
    kinematic_ui::theme::load_font(
        context,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/JetBrainsMonoNerdFont-Bold.ttf"
        )),
    )
}
