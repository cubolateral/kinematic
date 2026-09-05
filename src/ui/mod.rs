//! Dear ImGui editor shell and panel composition.

mod export;
mod inspector;
mod preview;
mod scene_tree;
mod settings;
mod theme;
mod timeline;
mod widgets;
mod workspace;

use crate::editor::Editor;

pub(crate) struct Ui {
    needs_initial_layout: bool,
    font: dear_imgui_rs::FontId,
    appearance: theme::Appearance,
    export: export::State,
    is_fullscreen: bool,
    preview: preview::State,
    timeline: timeline::State,
}

impl Ui {
    pub fn new(context: &mut dear_imgui_rs::Context) -> Self {
        let mut flags = context.io().config_flags();
        flags.insert(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_config_flags(flags);

        Self {
            needs_initial_layout: true,
            font: theme::initialize(context),
            appearance: theme::Appearance::default(),
            export: export::State::default(),
            is_fullscreen: false,
            preview: preview::State::default(),
            timeline: timeline::State::default(),
        }
    }

    pub fn draw(&mut self, editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _font = ui.push_font(self.font);
        let io = ui.io();
        let plain_keyboard_input = !io.want_text_input()
            && !io.key_ctrl()
            && !io.key_shift()
            && !io.key_alt()
            && !io.key_super();
        let fullscreen_shortcut = plain_keyboard_input
            && (self.is_fullscreen || !editor.is_exporting())
            && ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::F, false);

        if self.is_fullscreen {
            let fullscreen_button = preview::draw_fullscreen(editor, ui);

            if fullscreen_shortcut || fullscreen_button {
                self.is_fullscreen = false;
            }

            return;
        }

        let dock = ui.dockspace_over_main_viewport();

        if self.needs_initial_layout {
            workspace::apply_default_layout(ui, dock);
            self.needs_initial_layout = false;
        }

        scene_tree::draw(editor, ui);
        export::draw(editor, ui, &mut self.export);
        preview::draw(editor, ui, &mut self.preview);
        inspector::draw(editor, ui);
        settings::draw(&mut self.appearance, ui);
        let fullscreen_button = timeline::draw(editor, ui, &mut self.timeline);

        if fullscreen_shortcut || fullscreen_button {
            self.is_fullscreen = true;
        }
    }

    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub fn apply_scale(&self, context: &mut dear_imgui_rs::Context) {
        self.appearance.apply_scale(context);
    }
}
