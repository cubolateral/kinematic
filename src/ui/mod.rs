mod inspector;
mod preview;
mod timeline;

use crate::editor::Editor;

#[derive(Clone, Copy)]
struct ThemeColors {
    background: [f32; 4],
    accent: [f32; 4],
    contrast: f32,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 0.9],
            accent: [0.0, 0.549, 1.0, 1.0],
            contrast: 1.0,
        }
    }
}

pub(crate) struct Ui {
    is_first_time: bool,
    font: dear_imgui_rs::FontId,
    colors: ThemeColors,
    scale: f32,
    timeline: timeline::State,
}

impl Ui {
    const FONT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/JetBrainsMono-Regular.ttf"
    ));

    pub fn new(context: &mut dear_imgui_rs::Context) -> Self {
        // Enable docking.
        let mut flags = context.io().config_flags();
        flags.insert(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_config_flags(flags);

        Self::apply_theme(context);

        Self {
            is_first_time: true,
            font: Self::load_font(context),
            colors: ThemeColors::default(),
            scale: 1.0,
            timeline: timeline::State::default(),
        }
    }

    pub fn draw(&mut self, editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let _theme = self.push_theme(ui);
        let dock = ui.dockspace_over_main_viewport();

        if self.is_first_time {
            Self::dock_layout(ui, dock);
            self.is_first_time = false;
        }

        let _font = ui.push_font(self.font);
        inspector::draw(editor, ui);
        self.configuration(ui);
        preview::draw(editor, ui);
        timeline::draw(editor, ui, &mut self.timeline);
    }

    pub fn apply_scale(&self, context: &mut dear_imgui_rs::Context) {
        context
            .style_mut()
            .set_font_scale_main(self.scale.clamp(0.75, 1.25));
    }

    fn configuration(&mut self, ui: &dear_imgui_rs::Ui) {
        ui.window("Configuration").build(|| {
            ui.color_edit4("Background", &mut self.colors.background);
            ui.color_edit4("Accent", &mut self.colors.accent);
            ui.slider_f32("Contrast", &mut self.colors.contrast, 0.25, 1.0);
            ui.slider_f32("UI Scale", &mut self.scale, 0.75, 1.25);
        });
    }

    fn push_theme<'ui>(
        &self,
        ui: &'ui dear_imgui_rs::Ui,
    ) -> Vec<dear_imgui_rs::ColorStackToken<'ui>> {
        use dear_imgui_rs::StyleColor;

        let background = self.colors.background;
        let contrast = self.colors.contrast.clamp(0.0, 1.0);
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
        let accent = self.colors.accent;

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

    fn dock_layout(ui: &dear_imgui_rs::Ui, dock: dear_imgui_rs::Id) {
        dear_imgui_rs::DockBuilder::remove_node(ui, dock);
        dear_imgui_rs::DockBuilder::add_node(ui, dock, dear_imgui_rs::DockNodeFlags::NONE);
        dear_imgui_rs::DockBuilder::set_node_size(ui, dock, ui.main_viewport().size());

        let (inspector, right) = dear_imgui_rs::DockBuilder::split_node(
            ui,
            dock,
            dear_imgui_rs::SplitDirection::Left,
            0.2,
        );
        let (timeline, preview) = dear_imgui_rs::DockBuilder::split_node(
            ui,
            right,
            dear_imgui_rs::SplitDirection::Down,
            0.35,
        );

        dear_imgui_rs::DockBuilder::dock_window(ui, "Inspector", inspector);
        dear_imgui_rs::DockBuilder::dock_window(ui, "Configuration", inspector);
        dear_imgui_rs::DockBuilder::dock_window(ui, "Preview", preview);
        dear_imgui_rs::DockBuilder::dock_window(ui, "Timeline", timeline);
        dear_imgui_rs::DockBuilder::finish(ui, dock);
    }

    fn load_font(context: &mut dear_imgui_rs::Context) -> dear_imgui_rs::FontId {
        context
            .font_atlas_mut()
            .add_font(&[dear_imgui_rs::FontSource::ttf_data_with_size(
                Self::FONT,
                16.0,
            )])
    }

    fn apply_theme(ctx: &mut dear_imgui_rs::Context) {
        let style = ctx.style_mut();

        // Geometry.
        style.set_window_rounding(3.0);
        style.set_child_rounding(3.0);
        style.set_popup_rounding(3.0);
        style.set_frame_rounding(3.0);
        style.set_grab_rounding(3.0);
        style.set_tab_rounding(3.0);

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
}
