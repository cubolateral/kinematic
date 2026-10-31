//! Shared context menu presentation, independent of panel fonts and colors.

use dear_imgui_rs::{ColorStackToken, FontId, FontStackToken, StyleColor, Ui};

pub use crate::shortcut::{menu_item as item, menu_item_enabled_selected as item_enabled_selected};

pub struct Style {
    font: FontId,
    size: f32,
    colors: [(StyleColor, [f32; 4]); 4],
}

impl Style {
    pub fn capture(ui: &Ui) -> Self {
        Self {
            font: ui.current_font(),
            size: ui.current_font_size(),
            colors: [
                StyleColor::Text,
                StyleColor::Header,
                StyleColor::HeaderHovered,
                StyleColor::HeaderActive,
            ]
            .map(|color| (color, ui.style_color(color))),
        }
    }

    pub fn push<'ui>(&self, ui: &'ui Ui) -> (Vec<ColorStackToken<'ui>>, FontStackToken<'ui>) {
        let font = ui.push_font_with_size(Some(self.font), self.size);
        let mut colors: Vec<_> = self
            .colors
            .iter()
            .map(|&(color, value)| ui.push_style_color(color, value))
            .collect();
        // ImGui scopes must drop in reverse order, including colors before the font.
        colors.reverse();
        (colors, font)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menus_restore_interface_typography_and_colors_inside_specialized_panels() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let ui = context.frame();
        ui.window("Menu style").build(|| {
            let style = Style::capture(ui);
            let font = ui.current_font();
            let size = ui.current_font_size();
            let color = ui.style_color(StyleColor::Text);
            let _font = ui.push_font_with_size(Some(font), 30.0);
            let _color = ui.push_style_color(StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            {
                let _menu = style.push(ui);
                assert_eq!(ui.current_font(), font);
                assert_eq!(ui.current_font_size(), size);
                assert_eq!(ui.style_color(StyleColor::Text), color);
            }
            assert_eq!(ui.current_font_size(), 30.0);
            assert_eq!(ui.style_color(StyleColor::Text), [1.0, 0.0, 0.0, 1.0]);
        });
        context.end_frame();
    }
}
