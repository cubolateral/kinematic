//! Dear ImGui editor shell and panel composition.

use crate::ui::inputs::Action as InputAction;

mod browser;
mod cache;
mod code;
mod inputs;
pub(crate) use code::ProjectAction;
mod export;
mod home;
mod inspector;
mod preview;
mod scene_tree;
mod settings;
mod theme;
mod timeline;
mod title_bar;
mod workspace;

use crate::editor::Editor;
use kinematic_ui::{
    controls,
    message_box::{self, Kind},
    properties, widgets, windows,
};

#[cfg(test)]
static IMGUI_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) struct Ui {
    needs_initial_layout: bool,
    panels: workspace::State,
    messages: message_box::State,
    code: code::State,
    home: home::State,
    font: dear_imgui_rs::FontId,
    appearance: theme::Appearance,
    is_fullscreen: bool,
    audio_volume: i32,
    preview: preview::State,
    inspector: inspector::State,
    settings: settings::State,
    timeline: timeline::State,
    scene_tree: scene_tree::State,
    title_bar: title_bar::State,
}

fn draw_top_bar(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    code: &mut code::State,
    settings: &mut settings::State,
    messages: &mut message_box::State,
    title_bar: &mut title_bar::State,
) -> bool {
    let Some(_bar) = kinematic_ui::title_bar::begin(ui) else {
        return false;
    };

    title_bar.begin(ui);
    title_bar.logo(ui);

    let menu_width = ui.calc_text_size("ProjectEditorHelp")[0]
        + ui.calc_text_size(kinematic_ui::icons::EXPORT)[0]
        + ui.clone_style().frame_padding()[0] * 8.0;
    let controls_width = if title_bar.controls.macos {
        0.0
    } else {
        title_bar.controls.width(ui)
    };
    let available = ui.window_size()[0] - ui.cursor_pos()[0] - controls_width;
    let mut export_drag_area = None;
    let started = if available < menu_width + ui.frame_height() {
        if let Some(_menu) = controls::menu(ui, "Menu") {
            draw_project_menu(settings, editor, code, messages, ui);

            draw_help_menu(messages, ui);
            export::draw(editor, ui)
        } else {
            false
        }
    } else {
        draw_project_menu(settings, editor, code, messages, ui);
        draw_help_menu(messages, ui);

        let export_width = ui.calc_text_size(kinematic_ui::icons::EXPORT)[0]
            + ui.clone_style().frame_padding()[0] * 2.0;
        let right = ui.window_size()[0] - ui.clone_style().window_padding()[0] - controls_width;
        let drag_start = ui.cursor_pos_x();
        let export_start = (right - export_width).max(drag_start);
        ui.set_cursor_pos_x(export_start);
        export_drag_area = Some((drag_start, export_start));

        export::draw(editor, ui)
    };

    title_bar.finish(ui, None);

    if let Some((start, end)) = export_drag_area {
        let position = ui.window_pos();
        let viewport = ui.main_viewport().pos();

        title_bar.drag_area = (title_bar.controls.visible && end > start).then_some([
            [position[0] - viewport[0] + start, position[1] - viewport[1]],
            [
                position[0] - viewport[0] + end,
                position[1] - viewport[1] + ui.window_size()[1],
            ],
        ]);
    }

    started
}

fn draw_project_menu(
    settings: &mut settings::State,
    editor: &mut Editor,
    code: &mut code::State,
    messages: &mut message_box::State,
    ui: &dear_imgui_rs::Ui,
) {
    if let Some(_menu) = controls::menu(ui, "Project") {
        {
            let _disabled = ui.begin_disabled_with_cond(editor.is_exporting());

            if kinematic_ui::menu::item(
                ui,
                "Save project",
                &inputs::map().binding(InputAction::SaveProject).display,
            ) {
                if let Err(error) = save_project(editor) {
                    messages.show(Kind::Error, error);
                }
            }

            if controls::menu_item(ui, "Project settings") {
                settings.project_open = true;
            }

            if kinematic_ui::menu::item(
                ui,
                "Quit to project list",
                &inputs::map().binding(InputAction::ProjectList).display,
            ) {
                code.request_project(ProjectAction::Close);
            }
        }

        if kinematic_ui::menu::item(
            ui,
            "Quit",
            &inputs::map().binding(InputAction::Quit).display,
        ) {
            code.request_exit();
        }
    }

    if let Some(_menu) = controls::menu(ui, "Editor") {
        if controls::menu_item(ui, "Editor settings") {
            settings.editor_open = true;
        }
    }
}

fn draw_help_menu(messages: &mut message_box::State, ui: &dear_imgui_rs::Ui) {
    if let Some(_menu) = controls::menu(ui, "Help") {
        if controls::menu_item(ui, &format!("{}  Documentation", kinematic_ui::icons::BOOK)) {
            open_url("https://cubolateral.github.io/kinematic/", messages);
        }

        if controls::menu_item(ui, &format!("{}  Repository", kinematic_ui::icons::GITHUB)) {
            open_url("https://github.com/cubolateral/kinematic", messages);
        }
    }
}

fn open_url(url: &str, messages: &mut message_box::State) {
    if let Err(error) = sdl3::url::open_url(url) {
        messages.show(Kind::Error, format!("Could not open the link: {error}."));
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

        code.set_project_directory(cache.project_directory);

        let mut home = home::State::default();

        home.recent = cache.recent_projects;
        home.details = cache.project_details;

        if let Ok(Some(root)) = crate::storage::current_project() {
            if !home.recent.contains(&root) {
                home.recent.insert(0, root);
            }
        }

        home.refresh();

        let font = theme::initialize(context);

        Self {
            needs_initial_layout: true,
            panels: workspace::State::new(cache.workspace),
            messages: message_box::State::default(),
            code,
            home,
            font,
            appearance: settings::load(),
            is_fullscreen: false,
            audio_volume: cache.audio_volume,
            preview: preview::State::new(crate::editor::EditorMode::default()),
            inspector: inspector::State::default(),
            settings: settings::State::default(),
            timeline: timeline::State::default(),
            scene_tree: scene_tree::State::default(),
            title_bar: title_bar::State::default(),
        }
    }

    pub(crate) fn save_cache(&self) {
        cache::Cache {
            audio_volume: self.audio_volume,
            project_directory: self.code.project_directory(),
            recent_projects: self.home.recent.clone(),
            project_details: self.home.details.clone(),
            workspace: self.panels.layout.clone(),
        }
        .save();
    }

    pub(crate) fn audio_volume(&self) -> f32 {
        self.audio_volume as f32 / 100.0
    }

    pub fn draw(&mut self, editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _window_style = windows::style(
            ui,
            self.appearance.border_radius.clamp(0.0, 12.0),
            self.appearance.window_spacing.clamp(1.0, 16.0),
        );
        let _font = ui.push_font(self.font);
        let io = ui.io();
        let plain_keyboard_input =
            !self.code.files_focused() && !io.want_text_input() && !ui.is_any_item_active();
        let fullscreen_shortcut = plain_keyboard_input
            && (self.is_fullscreen || !editor.is_exporting())
            && inputs::active(ui, InputAction::ToggleFullscreen);

        if self.is_fullscreen {
            self.title_bar.drag_area = None;

            if !editor.is_exporting() && inputs::active(ui, InputAction::SaveProject) {
                if let Err(error) = save_project(editor) {
                    self.messages.show(Kind::Error, error);
                }
            }

            let fullscreen_button = preview::draw_fullscreen(editor, ui, &mut self.audio_volume);

            if fullscreen_shortcut || fullscreen_button {
                self.is_fullscreen = false;

                ui.set_window_focus(Some(if !self.panels.hidden {
                    inspector::WINDOW_NAME
                } else {
                    preview::WINDOW_NAME
                }));
            }

            return;
        }

        if !io.want_text_input()
            && !ui.is_any_item_active()
            && !ui.is_popup_open_with_flags("", dear_imgui_rs::PopupQueryFlags::ANY_POPUP)
            && inputs::active(ui, InputAction::TogglePanels)
        {
            self.panels.toggle();
        }

        let export_started = draw_top_bar(
            editor,
            ui,
            &mut self.code,
            &mut self.settings,
            &mut self.messages,
            &mut self.title_bar,
        );

        if export_started {
            self.preview.show_preview();
        }

        self.panels
            .draw(ui, self.appearance.window_spacing.clamp(1.0, 16.0));

        let initial_layout = self.needs_initial_layout;

        if initial_layout {
            self.needs_initial_layout = false;
        }

        if let Some(documents) = &mut editor.documents {
            self.code
                .draw(documents, &self.appearance.editor, ui, &mut self.panels);
        }

        if !editor.is_exporting() && inputs::active(ui, InputAction::SaveProject) {
            if let Err(error) = save_project(editor) {
                self.messages.show(Kind::Error, error);
            }
        }

        let is_exporting = editor.is_exporting();
        let fullscreen_button = {
            let _disabled = ui.begin_disabled_with_cond(is_exporting);

            if !self.panels.hidden
                && scene_tree::draw(editor, ui, &mut self.scene_tree, &mut self.panels)
            {
                self.preview.edit_canvas(editor.selected_canvas());
            }

            if settings::draw(&mut self.appearance, &mut self.settings, Some(editor), ui) {
                settings::save(&self.appearance);
            }

            let fullscreen_button = preview::draw(
                editor,
                ui,
                &mut self.preview,
                !self.code.files_focused(),
                &mut self.audio_volume,
                &mut self.panels,
            );

            if !self.panels.hidden {
                inspector::draw(editor, ui, &mut self.inspector, &mut self.panels);
            }

            timeline::draw(
                editor,
                ui,
                &mut self.timeline,
                !self.code.files_focused(),
                self.code.dragged_audio(),
                &mut self.panels,
            );

            fullscreen_button
        };

        if initial_layout && !self.panels.hidden {
            ui.set_window_focus(Some(inspector::WINDOW_NAME));
        }

        if fullscreen_shortcut || fullscreen_button {
            self.is_fullscreen = true;
        }
    }

    pub(crate) fn prepare_documents(&mut self, context: &dear_imgui_rs::Context) {
        if let Some(path) = self.home.copy_path.take() {
            context.set_clipboard_text(path);
        }

        self.code.prepare(context);
    }

    pub(crate) fn draw_projects(&mut self, editor: Option<&mut Editor>, ui: &dear_imgui_rs::Ui) {
        let _theme = self.appearance.push(ui);
        let _window_style = windows::style(
            ui,
            self.appearance.border_radius.clamp(0.0, 12.0),
            self.appearance.window_spacing.clamp(1.0, 16.0),
        );
        let _font = ui.push_font(self.font);

        if editor.is_none() {
            let _border = ui.push_style_var(dear_imgui_rs::StyleVar::WindowBorderSize(0.0));

            if let Some(_bar) = kinematic_ui::title_bar::begin(ui) {
                self.title_bar.begin(ui);
                self.title_bar.logo(ui);
                self.title_bar
                    .finish(ui, Some(&mut self.settings.editor_open));
            }
        }

        if editor.is_none() && settings::draw(&mut self.appearance, &mut self.settings, None, ui) {
            settings::save(&self.appearance);
        }

        if inputs::active(ui, InputAction::Quit) {
            self.code.request_exit();
        }

        if editor.is_some() && inputs::active(ui, InputAction::ProjectList) {
            self.code.request_project(ProjectAction::Close);
        }

        if editor.is_none() {
            self.is_fullscreen = false;

            self.home.draw(&mut self.code, ui);
        }

        let has_editor = editor.is_some();
        let project_directory = self.code.project_directory();

        if !self.messages.is_pending() {
            self.code.projects(editor, ui);
        }

        if let Some(message) = self.code.project_error.take() {
            self.messages.show(Kind::Error, message);
        }

        if let Some(error) = self
            .home
            .error
            .take()
            .or_else(|| self.settings.name_error.take())
        {
            self.messages.show(Kind::Error, error);
        }

        if std::mem::take(&mut self.home.dirty)
            || project_directory != self.code.project_directory()
        {
            self.save_cache();
        }

        if !has_editor {
            kinematic_ui::status_bar::home(ui, env!("CARGO_PKG_VERSION"));
        }

        self.messages.draw(ui);
    }

    pub(crate) fn remember_project(&mut self, root: &std::path::Path) {
        self.home.remember(root);
        self.save_cache();
    }

    pub(crate) fn request_exit(&mut self) {
        self.code.request_exit();
    }

    pub(crate) fn set_window_controls(&mut self, controls: kinematic_ui::title_bar::Controls) {
        self.title_bar.controls = controls;
    }

    pub(crate) fn take_window_action(&mut self) -> Option<kinematic_ui::title_bar::Action> {
        self.title_bar.action.take()
    }

    pub(crate) fn window_drag_area(&self, ui: &dear_imgui_rs::Ui) -> Option<[[f32; 2]; 2]> {
        self.title_bar.drag_area(ui)
    }

    pub(crate) fn take_project_action(&mut self) -> Option<ProjectAction> {
        self.code.take_action()
    }

    pub(crate) fn project_error(&mut self, error: String) {
        self.messages.show(Kind::Error, error);
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

        self.settings.project_open = false;

        self.home.refresh();
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

fn save_project(editor: &mut Editor) -> Result<(), String> {
    editor.save_project()?;

    if let Some(documents) = &mut editor.documents {
        documents.save_all()?;
    }

    Ok(())
}
