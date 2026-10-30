//! Dear ImGui editor shell and panel composition.

use crate::ui::inputs::Action as InputAction;

mod browser;
mod cache;
mod code;
mod inputs;
pub(crate) use code::ProjectAction;
mod export;
mod inspector;
mod preview;
mod scene_tree;
mod settings;
mod sound;
mod theme;
mod timeline;
mod workspace;

use crate::editor::Editor;
use kinematic_ui::{controls, properties, widgets, windows};

#[cfg(test)]
static IMGUI_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) struct Ui {
    needs_initial_layout: bool,
    code: code::State,
    font: dear_imgui_rs::FontId,
    appearance: theme::Appearance,
    is_fullscreen: bool,
    audio_volume: i32,
    preview: preview::State,
    inspector: inspector::State,
    sound: sound::State,
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

    export::draw(editor, ui)
}

fn draw_project_menu(
    code: &mut code::State,
    settings: &mut settings::State,
    ui: &dear_imgui_rs::Ui,
) {
    widgets::horizontal_border(
        ui,
        ui.window_pos()[1] + ui.window_size()[1] - kinematic_ui::theme::BORDER_WIDTH,
    );
    if let Some(_menu) = controls::menu(ui, "Projects") {
        if controls::menu_item(ui, "Open / Create Project") {
            code.projects_open = true;
        }

        if controls::menu_item(ui, "Project settings") {
            settings.project_open = true;
        }

        widgets::separator(ui);

        if controls::menu_item(ui, "Quit") {
            code.request_exit();
        }
    }

    if let Some(_menu) = controls::menu(ui, "Editor") {
        if controls::menu_item(ui, "Editor settings") {
            settings.editor_open = true;
        }
    }
}

impl Ui {
    #[cfg(test)]
    pub(crate) fn selected_audio_rect(&self) -> Option<[[f32; 2]; 2]> {
        self.timeline.selected_audio_rect()
    }

    pub fn new(context: &mut dear_imgui_rs::Context) -> Self {
        let mut flags = context.io().config_flags();

        flags.insert(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_config_flags(flags);
        context.io_mut().set_config_drag_click_to_input_text(true);

        let cache = cache::Cache::load();
        let mut code = code::State::default();

        code.zoom = cache.code_zoom;

        code.projects_open = true;
        let font = theme::initialize(context);
        code.bold_font = Some(theme::load_bold_font(context));

        Self {
            needs_initial_layout: true,
            code,
            font,
            appearance: settings::load(),
            is_fullscreen: false,
            audio_volume: cache.audio_volume,
            preview: preview::State::new(crate::editor::EditorMode::default()),
            inspector: inspector::State::default(),
            sound: sound::State::default(),
            settings: settings::State::default(),
            timeline: timeline::State::default(),
            scene_tree: scene_tree::State::default(),
        }
    }

    pub(crate) fn save_cache(&self) {
        cache::Cache {
            audio_volume: self.audio_volume,
            code_zoom: self.code.zoom,
        }
        .save();
    }

    pub(crate) fn audio_volume(&self) -> f32 {
        self.audio_volume as f32 / 100.0
    }

    pub fn draw(&mut self, editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _font = ui.push_font(self.font);
        let io = ui.io();
        if !editor.is_exporting() && inputs::active(ui, InputAction::SaveProject) {
            if let Err(error) = editor.save_project() {
                self.code.error = Some(error);
            }
        }
        let plain_keyboard_input =
            !self.code.focused && !io.want_text_input() && !ui.is_any_item_active();
        let fullscreen_shortcut = plain_keyboard_input
            && (self.is_fullscreen || !editor.is_exporting())
            && inputs::active(ui, InputAction::ToggleFullscreen);

        if self.is_fullscreen {
            let fullscreen_button = preview::draw_fullscreen(editor, ui, &mut self.audio_volume);

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

            let fullscreen_button = preview::draw(
                editor,
                ui,
                &mut self.preview,
                !self.code.focused,
                &mut self.audio_volume,
            );

            inspector::draw(editor, ui, &mut self.inspector);
            sound::draw(editor, ui, &mut self.sound);

            timeline::draw(
                editor,
                ui,
                &mut self.timeline,
                !self.code.focused,
                self.code.dragged_audio(),
            );
            fullscreen_button
        };

        if kinematic_ui::status_bar::draw(
            ui,
            editor
                .documents
                .as_ref()
                .map_or("Ready.", |documents| documents.status.as_str()),
            env!("CARGO_PKG_VERSION"),
        ) {
            if let Err(error) = sdl3::url::open_url("https://github.com/cubolateral/kinematic") {
                self.code.error = Some(format!("Could not open the repository: {error}."));
            }
        }

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
        let (mode, fullscreen, pan, zoom) = editor.cached_view();

        self.code.projects_open = false;
        self.settings = settings::State::default();
        self.preview = preview::State::new(mode);
        self.preview.view = kinematic_ui::canvas::View { pan, zoom };
        self.is_fullscreen = fullscreen;
    }

    pub(crate) fn reset_project(&mut self) {
        self.code.reset();
    }

    pub fn apply_scale(&self, context: &mut dear_imgui_rs::Context) {
        self.appearance.apply_scale(context);
    }

    pub(crate) fn preview_view(&self) -> ([f32; 2], f32) {
        (self.preview.view.pan, self.preview.view.zoom)
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
