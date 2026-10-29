//! Minimal, borderless editor style and shared font setup.

use dear_imgui_rs::{ColorStackToken, Context, FontId, StyleColor, Ui};

pub const CORNER_RADIUS: f32 = 2.0;
pub const ERROR: [f32; 4] = [0.95, 0.33, 0.33, 1.0];
pub const WARNING: [f32; 4] = [0.89, 0.80, 0.33, 1.0];

pub fn contrast_text(background: [f32; 4]) -> [f32; 4] {
    let linear = |value: f32| {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = linear(background[0]) * 0.2126
        + linear(background[1]) * 0.7152
        + linear(background[2]) * 0.0722;
    if luminance > 0.179 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        [1.0; 4]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: [f32; 4],
    pub accent: [f32; 4],
    pub contrast: f32,
    pub scale: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 1.0],
            accent: [0.0, 0.549, 1.0, 1.0],
            contrast: 1.0,
            scale: 1.0,
        }
    }
}

impl Theme {
    pub fn apply_scale(&self, context: &mut Context) {
        context
            .style_mut()
            .set_font_scale_main(self.scale.clamp(0.75, 1.25));
    }

    pub fn push<'ui>(&self, ui: &'ui Ui) -> Vec<ColorStackToken<'ui>> {
        let background = self.background;
        let contrast = self.contrast.clamp(0.0, 1.0);
        let foreground = contrast_text(background);

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
        let border = mix(0.12);
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
            (StyleColor::BorderShadow, [0.0; 4]),
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
            (StyleColor::CheckboxSelectedBg, background),
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
            (StyleColor::UnsavedMarker, accent),
            (StyleColor::PlotLines, foreground),
            (StyleColor::PlotLinesHovered, accent),
            (StyleColor::PlotHistogram, foreground),
            (StyleColor::PlotHistogramHovered, accent),
        ];

        let mut tokens = colors
            .into_iter()
            .map(|(style_color, color)| ui.push_style_color(style_color, color))
            .collect::<Vec<_>>();

        // Stack guards must be released in reverse push order.
        tokens.reverse();

        tokens
    }
}

pub fn initialize(context: &mut Context, font: &'static [u8]) -> FontId {
    apply_geometry(context);
    context
        .style_mut()
        .set_color(StyleColor::ModalWindowDimBg, [0.0; 4]);
    load_font(context, font)
}

pub fn load_font(context: &mut Context, bytes: &'static [u8]) -> FontId {
    context
        .font_atlas()
        .add_font(&[dear_imgui_rs::FontSource::stb_truetype_with_size(
            dear_imgui_rs::StbTrueTypeFontData::from_slice(bytes)
                .expect("Embedded font must be valid."),
            16.0,
        )])
}

fn apply_geometry(context: &mut Context) {
    let style = context.style_mut();

    style.set_window_rounding(CORNER_RADIUS);
    style.set_child_rounding(CORNER_RADIUS);
    style.set_popup_rounding(CORNER_RADIUS);
    style.set_frame_rounding(CORNER_RADIUS);
    style.set_grab_rounding(CORNER_RADIUS);
    style.set_tab_rounding(CORNER_RADIUS);
    style.set_scrollbar_rounding(CORNER_RADIUS);
    style.set_image_rounding(CORNER_RADIUS);
    style.set_drag_drop_target_rounding(CORNER_RADIUS);

    style.set_window_border_size(0.0);
    style.set_child_border_size(0.0);
    style.set_popup_border_size(1.0);
    style.set_frame_border_size(0.0);

    style.set_window_padding([8.0, 8.0]);
    style.set_frame_padding([6.0, 4.0]);
    style.set_item_spacing([8.0, 5.0]);
    style.set_item_inner_spacing([6.0, 4.0]);

    style.set_scrollbar_size(12.0);
    style.set_grab_min_size(10.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_text_contrasts_with_bright_and_dark_colors() {
        assert_eq!(super::contrast_text([1.0; 4]), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(super::contrast_text([0.0, 0.0, 0.0, 1.0]), [1.0; 4]);
        assert_eq!(
            super::contrast_text([1.0, 1.0, 0.0, 1.0]),
            [0.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(super::contrast_text([0.0, 0.0, 1.0, 1.0]), [1.0; 4]);
    }

    #[test]
    fn the_shared_theme_is_borderless_and_supports_dark_and_light_backgrounds() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = Context::create();
        apply_geometry(&mut context);
        assert_eq!(context.style().frame_border_size(), 0.0);
        assert_eq!(context.style().window_border_size(), 0.0);
        assert_eq!(context.style().frame_rounding(), CORNER_RADIUS);
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        for background in [[0.0, 0.0, 0.0, 1.0], [1.0; 4]] {
            let theme = Theme {
                background,
                ..Theme::default()
            };
            let ui = context.frame();
            {
                let _colors = theme.push(ui);
                let border = ui.style_color(StyleColor::Border)[0];
                let frame = ui.style_color(StyleColor::FrameBg)[0];
                assert!((border - background[0]).abs() <= 0.121);
                assert_ne!(frame, background[0]);
            }
            context.end_frame();
        }
    }
}
