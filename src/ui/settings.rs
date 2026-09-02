use super::theme::Appearance;

pub(super) const WINDOW_NAME: &str = "Configuration";

pub(super) fn draw(appearance: &mut Appearance, ui: &dear_imgui_rs::Ui) {
    ui.window(WINDOW_NAME).build(|| {
        ui.color_edit4("Background", &mut appearance.background);
        ui.color_edit4("Accent", &mut appearance.accent);
        ui.slider_f32("Contrast", &mut appearance.contrast, 0.25, 1.0);
        ui.slider_f32("UI Scale", &mut appearance.scale, 0.75, 1.25);
    });
}
