use dear_imgui_rs::{ColorStackToken, Context, FontId, StyleColor, Ui};

pub(super) struct Appearance {
    pub background: [f32; 4],
    pub accent: [f32; 4],
    pub contrast: f32,
    pub scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 1.0],
            accent: [0.0, 0.549, 1.0, 1.0],
            contrast: 1.0,
            scale: 1.0,
        }
    }
}

impl Appearance {
    pub fn apply_scale(&self, context: &mut Context) {
        context
            .style_mut()
            .set_font_scale_main(self.scale.clamp(0.75, 1.25));
    }

    pub fn push<'ui>(&self, ui: &'ui Ui) -> Vec<ColorStackToken<'ui>> {
        let background = self.background;
        let contrast = self.contrast.clamp(0.0, 1.0);
        let foreground =
            if background[0] * 0.299 + background[1] * 0.587 + background[2] * 0.114 > 0.5 {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };

        let mix = |amount: f32| {
            [
                background[0] + (foreground[0] - background[0]) * amount * contrast,
                background[1] + (foreground[1] - background[1]) * amount * contrast,
                background[2] + (foreground[2] - background[2]) * amount * contrast,
                background[3],
            ]
        };

        let surface = mix(0.08);
        let hover = mix(0.14);
        let active = mix(0.22);
        let border = mix(0.30);
        let disabled = mix(0.50);
        let accent = self.accent;
        let colors = [
            (StyleColor::Text, foreground),
            (StyleColor::TextDisabled, disabled),
            (StyleColor::TextLink, accent),
            (StyleColor::TextSelectedBg, accent),
            (StyleColor::WindowBg, background),
            (StyleColor::ChildBg, background),
            (StyleColor::PopupBg, background),
            (StyleColor::DockingEmptyBg, background),
            (StyleColor::Border, border),
            (StyleColor::BorderShadow, border),
            (StyleColor::FrameBg, surface),
            (StyleColor::FrameBgHovered, hover),
            (StyleColor::FrameBgActive, active),
            (StyleColor::TitleBg, background),
            (StyleColor::TitleBgActive, surface),
            (StyleColor::TitleBgCollapsed, disabled),
            (StyleColor::MenuBarBg, background),
            (StyleColor::Button, surface),
            (StyleColor::ButtonHovered, hover),
            (StyleColor::ButtonActive, active),
            (StyleColor::Header, surface),
            (StyleColor::HeaderHovered, hover),
            (StyleColor::HeaderActive, active),
            (StyleColor::CheckMark, accent),
            (StyleColor::CheckboxSelectedBg, accent),
            (StyleColor::SliderGrab, hover),
            (StyleColor::SliderGrabActive, accent),
            (StyleColor::ScrollbarBg, background),
            (StyleColor::ScrollbarGrab, surface),
            (StyleColor::ScrollbarGrabHovered, hover),
            (StyleColor::ScrollbarGrabActive, active),
            (StyleColor::Separator, border),
            (StyleColor::SeparatorHovered, hover),
            (StyleColor::SeparatorActive, active),
            (StyleColor::ResizeGrip, surface),
            (StyleColor::ResizeGripHovered, hover),
            (StyleColor::ResizeGripActive, active),
            (StyleColor::Tab, surface),
            (StyleColor::TabHovered, hover),
            (StyleColor::TabSelected, active),
            (StyleColor::TabSelectedOverline, active),
            (StyleColor::TabDimmed, background),
            (StyleColor::TabDimmedSelected, active),
            (StyleColor::TabDimmedSelectedOverline, active),
            (StyleColor::DockingPreview, active),
            (StyleColor::TableHeaderBg, surface),
            (StyleColor::TableBorderStrong, border),
            (StyleColor::TableBorderLight, border),
            (StyleColor::TableRowBg, background),
            (StyleColor::TableRowBgAlt, surface),
            (StyleColor::InputTextCursor, foreground),
            (StyleColor::TreeLines, border),
            (StyleColor::DragDropTarget, accent),
            (StyleColor::DragDropTargetBg, surface),
            (StyleColor::NavCursor, accent),
            (StyleColor::NavWindowingHighlight, accent),
            (StyleColor::NavWindowingDimBg, disabled),
            (StyleColor::ModalWindowDimBg, disabled),
            (StyleColor::UnsavedMarker, accent),
            (StyleColor::PlotLines, foreground),
            (StyleColor::PlotLinesHovered, accent),
            (StyleColor::PlotHistogram, foreground),
            (StyleColor::PlotHistogramHovered, accent),
        ];

        colors
            .into_iter()
            .map(|(style_color, color)| ui.push_style_color(style_color, color))
            .collect()
    }
}

pub(super) fn initialize(context: &mut Context) -> FontId {
    apply_geometry(context);
    load_font(context)
}

fn load_font(context: &mut Context) -> FontId {
    const FONT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/JetBrainsMono-Regular.ttf"
    ));

    context
        .font_atlas_mut()
        .add_font(&[dear_imgui_rs::FontSource::ttf_data_with_size(FONT, 16.0)])
}

fn apply_geometry(context: &mut Context) {
    let style = context.style_mut();

    style.set_window_rounding(0.0);
    style.set_child_rounding(0.0);
    style.set_popup_rounding(0.0);
    style.set_frame_rounding(0.0);
    style.set_grab_rounding(0.0);
    style.set_tab_rounding(0.0);

    style.set_window_border_size(1.0);
    style.set_child_border_size(1.0);
    style.set_popup_border_size(1.0);
    style.set_frame_border_size(0.0);

    style.set_window_padding([8.0, 8.0]);
    style.set_frame_padding([6.0, 4.0]);
    style.set_item_spacing([8.0, 5.0]);
    style.set_item_inner_spacing([6.0, 4.0]);

    style.set_scrollbar_size(12.0);
    style.set_grab_min_size(10.0);
}
