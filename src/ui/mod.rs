//! Dear ImGui editor shell and panel composition.

mod controls;
mod export;
mod icons;
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
    inspector: inspector::State,
    settings: settings::State,
    timeline: timeline::State,
}

impl Ui {
    pub fn new(context: &mut dear_imgui_rs::Context) -> Self {
        let mut flags = context.io().config_flags();
        flags.insert(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_config_flags(flags);
        context.io_mut().set_config_drag_click_to_input_text(true);

        Self {
            needs_initial_layout: true,
            font: theme::initialize(context),
            appearance: settings::load(),
            export: export::State::default(),
            is_fullscreen: crate::editor::load_editor_fullscreen(),
            preview: preview::State::new(crate::editor::load_editor_mode()),
            inspector: inspector::State::default(),
            settings: settings::State::default(),
            timeline: timeline::State::default(),
        }
    }

    pub fn draw(&mut self, editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _font = ui.push_font(self.font);
        let io = ui.io();
        let plain_keyboard_input = !io.want_text_input()
            && !ui.is_any_item_active()
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
                ui.set_window_focus(Some(inspector::WINDOW_NAME));
            }

            return;
        }

        let dock = ui.dockspace_over_main_viewport();

        let initial_layout = self.needs_initial_layout;
        if initial_layout {
            workspace::apply_default_layout(ui, dock);
            self.needs_initial_layout = false;
        }

        let is_exporting = editor.is_exporting();
        let fullscreen_button = {
            let _disabled = ui.begin_disabled_with_cond(is_exporting);
            if scene_tree::draw(editor, ui) {
                self.preview.edit_canvas(editor.selected_canvas());
            }
            if settings::draw(&mut self.appearance, &mut self.settings, editor, ui) {
                settings::save(&self.appearance);
            }
            preview::draw(editor, ui, &mut self.preview);
            inspector::draw(editor, ui, &mut self.inspector);
            timeline::draw(editor, ui, &mut self.timeline)
        };

        if export::draw(editor, ui, &mut self.export) {
            self.preview.show_preview();
        }

        if initial_layout {
            ui.set_window_focus(Some(inspector::WINDOW_NAME));
        }

        if fullscreen_shortcut || fullscreen_button {
            self.is_fullscreen = true;
        }
    }

    pub fn apply_scale(&self, context: &mut dear_imgui_rs::Context) {
        self.appearance.apply_scale(context);
    }

    pub(crate) fn editor_mode(&self) -> crate::editor::EditorMode {
        self.preview.cached_mode()
    }

    pub(crate) fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub(crate) fn render_mode(&self) -> crate::editor::EditorMode {
        if self.is_fullscreen {
            crate::editor::EditorMode::Preview
        } else {
            self.editor_mode()
        }
    }
}
