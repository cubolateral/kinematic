//! Dear ImGui editor shell and panel composition.

use crate::ui::inputs::Action as InputAction;

mod browser;
mod code;
mod controls;
mod inputs;
pub(crate) use code::ProjectAction;
mod export;
mod icons;
mod inspector;
mod preview;
mod properties;
mod scene_tree;
mod settings;
mod theme;
mod timeline;
mod widgets;
mod windows;
mod workspace;

use crate::editor::Editor;

#[cfg(test)]
static IMGUI_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(super) const CORNER_RADIUS: f32 = 4.0;

pub(crate) struct Ui {
    needs_initial_layout: bool,
    code: code::State,
    font: dear_imgui_rs::FontId,
    appearance: theme::Appearance,
    is_fullscreen: bool,
    preview: preview::State,
    inspector: inspector::State,
    settings: settings::State,
    timeline: timeline::State,
    scene_tree: scene_tree::State,
}

fn draw_top_bar(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    code: &mut code::State,
    settings: &mut settings::State,
) -> bool {
    let Some(_bar) = ui.begin_main_menu_bar() else {
        return false;
    };

    draw_project_menu(code, settings, ui);
    ui.set_cursor_pos_x(ui.cursor_pos_x() + ui.content_region_avail_width() - ui.frame_height());

    export::draw(editor, ui)
}

fn draw_project_menu(
    code: &mut code::State,
    settings: &mut settings::State,
    ui: &dear_imgui_rs::Ui,
) {
    if let Some(_menu) = ui.begin_menu("Projects") {
        if ui.menu_item("Open / Create Project") {
            code.projects_open = true;
        }

        if ui.menu_item("Project settings") {
            settings.project_open = true;
        }

        ui.separator();

        if ui.menu_item("Quit") {
            code.request_exit();
        }
    }

    if let Some(_menu) = ui.begin_menu("Editor") {
        if ui.menu_item("Editor settings") {
            settings.editor_open = true;
        }
    }
}

impl Ui {
    pub fn new(context: &mut dear_imgui_rs::Context) -> Self {
        let mut flags = context.io().config_flags();

        flags.insert(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_config_flags(flags);
        context.io_mut().set_config_drag_click_to_input_text(true);

        let mut code = code::State::default();

        code.projects_open = true;

        Self {
            needs_initial_layout: true,
            code,
            font: theme::initialize(context),
            appearance: settings::load(),
            is_fullscreen: false,
            preview: preview::State::new(crate::editor::EditorMode::default()),
            inspector: inspector::State::default(),
            settings: settings::State::default(),
            timeline: timeline::State::default(),
            scene_tree: scene_tree::State::default(),
        }
    }

    pub fn draw(&mut self, editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _font = ui.push_font(self.font);
        let io = ui.io();
        let plain_keyboard_input =
            !self.code.focused && !io.want_text_input() && !ui.is_any_item_active();
        let fullscreen_shortcut = plain_keyboard_input
            && (self.is_fullscreen || !editor.is_exporting())
            && inputs::active(ui, InputAction::ToggleFullscreen);

        if self.is_fullscreen {
            let fullscreen_button = preview::draw_fullscreen(editor, ui);

            if fullscreen_shortcut || fullscreen_button {
                self.is_fullscreen = false;

                ui.set_window_focus(Some(inspector::WINDOW_NAME));
            }

            return;
        }

        let export_started = draw_top_bar(editor, ui, &mut self.code, &mut self.settings);

        if export_started {
            self.preview.show_preview();
        }

        workspace::draw(ui);

        let initial_layout = self.needs_initial_layout;

        if initial_layout {
            self.needs_initial_layout = false;
        }

        if let Some(documents) = &mut editor.documents {
            self.code.draw(documents, &self.appearance.editor, ui);
        }

        let is_exporting = editor.is_exporting();
        let fullscreen_button = {
            let _disabled = ui.begin_disabled_with_cond(is_exporting);

            if scene_tree::draw(editor, ui, &mut self.scene_tree) {
                self.preview.edit_canvas(editor.selected_canvas());
            }

            if settings::draw(&mut self.appearance, &mut self.settings, Some(editor), ui) {
                settings::save(&self.appearance);
            }

            preview::draw(editor, ui, &mut self.preview, !self.code.focused);

            inspector::draw(editor, ui, &mut self.inspector);

            timeline::draw(editor, ui, &mut self.timeline, !self.code.focused)
        };

        if initial_layout && !self.code.focused {
            ui.set_window_focus(Some(inspector::WINDOW_NAME));
        }

        if fullscreen_shortcut || fullscreen_button {
            self.is_fullscreen = true;
        }
    }

    pub(crate) fn prepare_documents(
        &mut self,
        context: &dear_imgui_rs::Context,
        editor: Option<&Editor>,
    ) {
        self.code.prepare(
            context,
            editor.and_then(|e| e.documents.as_ref()),
            &self.appearance.editor,
        );
    }

    pub(crate) fn draw_projects(&mut self, editor: Option<&mut Editor>, ui: &dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _font = ui.push_font(self.font);

        if editor.is_none() {
            if let Some(_bar) = ui.begin_main_menu_bar() {
                draw_project_menu(&mut self.code, &mut self.settings, ui);
            }
        }

        if editor.is_none() && settings::draw(&mut self.appearance, &mut self.settings, None, ui) {
            settings::save(&self.appearance);
        }

        self.code.projects(editor, ui);
    }

    pub(crate) fn request_exit(&mut self) {
        self.code.request_exit();
    }

    pub(crate) fn take_project_action(&mut self) -> Option<ProjectAction> {
        self.code.take_action()
    }

    pub(crate) fn project_error(&mut self, error: String) {
        self.code.error = Some(error);
    }

    pub(crate) fn reload_appearance(&mut self) {
        self.appearance = settings::load();
    }

    pub(crate) fn restore_project_view(&mut self, editor: &Editor) {
        let (mode, fullscreen) = editor.cached_view();

        self.code.projects_open = false;
        self.settings = settings::State::default();
        self.preview = preview::State::new(mode);
        self.is_fullscreen = fullscreen;
    }

    pub(crate) fn reset_project(&mut self) {
        self.code.reset();
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
