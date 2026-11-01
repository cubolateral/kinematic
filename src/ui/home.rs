use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use dear_imgui_rs::{Condition, MouseButton, StyleVar, Ui, WindowFlags};
use kinematic_ui::{controls, icons, lists, menu, widgets, windows};
use serde::{Deserialize, Serialize};

use super::{ProjectAction, code};

mod projects;

const LOGO: &str =
    " █████   ████  ███                                                 █████     ███          
░░███   ███░  ░░░                                                 ░░███     ░░░           
 ░███  ███    ████  ████████    ██████  █████████████    ██████   ███████   ████   ██████ 
 ░███████    ░░███ ░░███░░███  ███░░███░░███░░███░░███  ░░░░░███ ░░░███░   ░░███  ███░░███
 ░███░░███    ░███  ░███ ░███ ░███████  ░███ ░███ ░███   ███████   ░███     ░███ ░███ ░░░ 
 ░███ ░░███   ░███  ░███ ░███ ░███░░░   ░███ ░███ ░███  ███░░███   ░███ ███ ░███ ░███  ███
 █████ ░░████ █████ ████ █████░░██████  █████░███ █████░░████████  ░░█████  █████░░██████ 
░░░░░   ░░░░ ░░░░░ ░░░░ ░░░░░  ░░░░░░  ░░░░░ ░░░ ░░░░░  ░░░░░░░░    ░░░░░  ░░░░░  ░░░░░░";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct ProjectDetails {
    pub(super) name: String,
    pub(super) version: u32,
    pub(super) last_opened: u64,
}

#[derive(Clone, Copy)]
enum Action {
    Open,
    Rename,
    Duplicate,
    Remove,
    CopyPath,
    Terminal,
}

#[derive(Default)]
pub(super) struct State {
    pub(super) recent: Vec<PathBuf>,
    pub(super) details: BTreeMap<PathBuf, ProjectDetails>,
    available: BTreeSet<PathBuf>,
    search: String,
    context: Option<PathBuf>,
    context_opening: bool,
    rename: Option<(PathBuf, String)>,
    rename_opening: bool,
    terminal: Option<std::sync::mpsc::Receiver<String>>,
    pub(super) copy_path: Option<String>,
    pub(super) error: Option<String>,
    pub(super) dirty: bool,
}

impl State {
    pub(super) fn refresh(&mut self) {
        self.available.clear();
        self.details.retain(|path, _| self.recent.contains(path));
        for path in &self.recent {
            let details = self.details.entry(path.clone()).or_default();
            match crate::scripting::LuaProject::open(path) {
                Ok(project) => {
                    details.name = project.manifest.name;
                    details.version = project.manifest.version;
                    self.available.insert(path.clone());
                }
                Err(_) if details.name.is_empty() => {
                    details.name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                }
                Err(_) => {}
            }
        }
    }

    pub(super) fn remember(&mut self, path: &Path) {
        self.recent.retain(|previous| previous != path);
        self.recent.insert(0, path.to_owned());
        self.details.entry(path.to_owned()).or_default().last_opened = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.refresh();
    }

    fn filtered(&self, path: &Path) -> bool {
        self.details.get(path).is_some_and(|details| {
            details
                .name
                .to_lowercase()
                .contains(&self.search.to_lowercase())
        })
    }

    pub(super) fn draw(&mut self, code: &mut code::State, ui: &Ui) {
        if let Some(receiver) = &self.terminal {
            match receiver.try_recv() {
                Ok(error) => {
                    self.error = Some(error);
                    self.terminal = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.terminal = None,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        let viewport = ui.main_viewport();
        let size = viewport.work_size();
        let size = [
            size[0],
            (size[1] - kinematic_ui::status_bar::height(ui)).max(1.0),
        ];
        {
            let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
            let _rounding = ui.push_style_var(StyleVar::WindowRounding(0.0));
            ui.window("Kinematic home")
                .position(viewport.work_pos(), Condition::Always)
                .size(size, Condition::Always)
                .flags(
                    WindowFlags::NO_DECORATION
                        | WindowFlags::NO_MOVE
                        | WindowFlags::NO_DOCKING
                        | WindowFlags::NO_SAVED_SETTINGS
                        | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS,
                )
                .build(|| {
                    let width = (size[0] * 0.86).min(1100.0);
                    let left = (size[0] - width) * 0.5;
                    ui.set_cursor_pos([left, (size[1] * 0.14).clamp(24.0, 100.0)]);
                    {
                        let font_size =
                            ui.current_font_size() * (width / ui.calc_text_size(LOGO)[0]).min(1.0);
                        let _font = ui.push_font_with_size(None, font_size);
                        let logo_width = ui.calc_text_size(LOGO)[0];
                        ui.set_cursor_pos([left + (width - logo_width) * 0.5, ui.cursor_pos()[1]]);
                        widgets::text(ui, LOGO);
                    }
                    ui.dummy([0.0, ui.frame_height()]);
                    ui.set_cursor_pos([left, ui.cursor_pos()[1]]);
                    ui.child_window("Home content")
                        .size([width, 0.0])
                        .build(ui, || {
                            if controls::button(ui, &format!("{}  Create project", icons::PLUS)) {
                                code.projects_open = true;
                            }
                            ui.same_line();
                            if controls::button(
                                ui,
                                &format!("{}  Open project", icons::FOLDER_OPEN),
                            ) {
                                code.open_project();
                            }
                            ui.same_line();
                            ui.set_next_item_width(ui.content_region_avail_width());
                            ui.input_text("##SearchProjects", &mut self.search)
                                .hint(format!("{}  Search", icons::SEARCH))
                                .build();
                            ui.dummy([0.0, ui.frame_height() * 0.5]);
                            self.draw_list(code, ui);
                            if self.recent.is_empty() {
                                widgets::muted(ui, "Create or open a project to get started.");
                            }
                        });
                });
        }
        self.context_menu(code, ui);
        self.rename_dialog(ui);
    }

    fn draw_list(&mut self, code: &mut code::State, ui: &Ui) {
        for path in &self.recent {
            if !self.filtered(path) {
                continue;
            }
            let details = &self.details[path];
            let version = format!("v{}", details.version);
            let opened_at = projects::opened_at(details.last_opened);
            let path_text = path.to_string_lossy();
            let _id = ui.push_id(path_text.as_ref());
            let available = self.available.contains(path);
            lists::project_row(
                ui,
                &details.name,
                &path_text,
                &opened_at,
                &version,
                available,
            );
            let hovered = ui.is_item_hovered();
            if available && hovered && ui.is_mouse_double_clicked(MouseButton::Left) {
                code.request_project(ProjectAction::Open(path.clone()));
            }
            if hovered && ui.is_mouse_clicked(MouseButton::Right) {
                self.context = Some(path.clone());
                self.context_opening = true;
            }
        }
    }

    fn context_menu(&mut self, code: &mut code::State, ui: &Ui) {
        if self.context_opening {
            ui.open_popup("Project actions");
            self.context_opening = false;
        }
        let mut action = None;
        if let Some(_popup) = ui.begin_popup("Project actions") {
            if let Some(path) = &self.context {
                let available = self.available.contains(path);
                {
                    let _disabled = ui.begin_disabled_with_cond(!available);
                    for (kind, title) in [
                        (Action::Open, "Open"),
                        (Action::Rename, "Rename"),
                        (Action::Duplicate, "Duplicate"),
                    ] {
                        if menu::item(ui, title, "") {
                            action = Some(kind);
                        }
                    }
                }
                if menu::item(ui, "Remove", "") {
                    action = Some(Action::Remove);
                }
                widgets::separator(ui);
                if menu::item(ui, "Copy path", "") {
                    action = Some(Action::CopyPath);
                }
                let _disabled = ui.begin_disabled_with_cond(!path.is_dir());
                if menu::item(ui, "Open in terminal", "") {
                    action = Some(Action::Terminal);
                }
            }
        }
        if let Some(action) = action {
            let path = self.context.clone().unwrap();
            let result = match action {
                Action::Open => {
                    code.request_project(ProjectAction::Open(path));
                    Ok(())
                }
                Action::Rename => {
                    self.rename = Some((path.clone(), self.details[&path].name.clone()));
                    self.rename_opening = true;
                    Ok(())
                }
                Action::Duplicate => projects::duplicate(&path).map(|copy| {
                    self.recent.insert(0, copy.clone());
                    self.refresh();
                    self.dirty = true;
                }),
                Action::Remove => self.remove(&path),
                Action::CopyPath => {
                    self.copy_path = Some(path.to_string_lossy().into_owned());
                    Ok(())
                }
                Action::Terminal => crate::utilities::terminal::open_terminal(&path)
                    .map(|receiver| self.terminal = Some(receiver)),
            };
            if let Err(error) = result {
                self.error = Some(error);
            }
        }
    }

    fn remove(&mut self, path: &Path) -> Result<(), String> {
        if crate::storage::current_project()?.as_deref() == Some(path) {
            crate::storage::save_current_project(None)?;
        }
        self.recent.retain(|previous| previous != path);
        self.details.remove(path);
        self.context = None;
        self.dirty = true;
        Ok(())
    }

    fn rename_dialog(&mut self, ui: &Ui) {
        let Some((path, name)) = &mut self.rename else {
            return;
        };
        if self.rename_opening {
            ui.open_popup("Rename project");
            self.rename_opening = false;
        }
        let mut open = true;
        let mut finished = false;
        if let Some(_popup) = windows::modal(ui, "Rename project", &mut open) {
            let submitted = kinematic_ui::properties::row(ui, "Name", false, || {
                controls::text_submit(ui, "##Name", name)
            })
            .changed;
            let action =
                windows::actions(ui, &[("Rename", !name.trim().is_empty()), ("Cancel", true)]);
            if submitted || action == Some(0) {
                match projects::rename(path, name) {
                    Ok(()) => {
                        finished = true;
                        self.dirty = true;
                        ui.close_current_popup();
                    }
                    Err(error) => {
                        self.error = Some(error);
                        finished = true;
                        ui.close_current_popup();
                    }
                }
            }
            if action == Some(1) {
                finished = true;
                ui.close_current_popup();
            }
        }
        if finished || !open {
            self.rename = None;
            self.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_projects_keep_the_latest_order_and_read_saved_names() {
        let root = std::env::temp_dir().join(format!("kinematic-home-{}", std::process::id()));
        let mut project = crate::scripting::LuaProject::create(&root, "First name", false).unwrap();
        let missing = root.join("Missing");
        let mut home = State::default();
        home.remember(&root);
        home.remember(&missing);
        home.remember(&root);
        home.refresh();
        assert_eq!(home.recent, [root.clone(), missing]);
        assert_eq!(home.details[&root].name, "First name");
        project.manifest.name = "Renamed project".into();
        project.save().unwrap();
        home.refresh();
        assert_eq!(home.details[&root].name, "Renamed project");
        assert_eq!(home.details[&root.join("Missing")].name, "Missing");
        assert!(home.details[&root].last_opened > 0);
        home.search = "renamed".into();
        assert!(home.filtered(&root));
        home.search = root.to_string_lossy().into_owned();
        assert!(!home.filtered(&root));
        let last_opened = home.details[&root].last_opened;
        std::fs::remove_dir_all(&root).unwrap();
        home.refresh();
        assert!(!home.available.contains(&root));
        assert_eq!(home.details[&root].name, "Renamed project");
        assert_eq!(home.details[&root].last_opened, last_opened);
    }

    #[test]
    fn project_rows_require_double_click_and_offer_context_actions() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let root =
            std::env::temp_dir().join(format!("kinematic-home-click-{}", std::process::id()));
        crate::scripting::LuaProject::create(&root, "Click test", false).unwrap();
        let mut home = State::default();
        home.remember(&root);
        let mut code = code::State::default();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context.io_mut().set_display_size([800.0, 600.0]);
        context.io_mut().set_delta_time(0.05);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        let frame =
            |context: &mut dear_imgui_rs::Context, home: &mut State, code: &mut code::State| {
                let ui = context.frame();
                let mut rect = [[0.0; 2]; 2];
                let mut popup = false;
                ui.window("Project rows test")
                    .position([0.0; 2], Condition::Always)
                    .size([800.0, 500.0], Condition::Always)
                    .build(|| {
                        home.draw_list(code, ui);
                        let (min, max) = ui.item_rect();
                        rect = [min, max];
                        home.context_menu(code, ui);
                        popup = ui.is_popup_open("Project actions");
                    });
                code.projects(None, ui);
                context.end_frame();
                (rect, popup)
            };
        frame(&mut context, &mut home, &mut code);
        let (rect, _) = frame(&mut context, &mut home, &mut code);
        let point = [rect[0][0] + 20.0, rect[0][1] + 20.0];
        context.io_mut().add_mouse_pos_event(point);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, true);
        frame(&mut context, &mut home, &mut code);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        frame(&mut context, &mut home, &mut code);
        assert!(code.take_action().is_none());
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, true);
        frame(&mut context, &mut home, &mut code);
        assert!(matches!(code.take_action(), Some(ProjectAction::Open(path)) if path == root));
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        frame(&mut context, &mut home, &mut code);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Right, true);
        let (_, popup) = frame(&mut context, &mut home, &mut code);
        assert!(popup);
        assert_eq!(home.context.as_deref(), Some(root.as_path()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rename_uses_the_shared_modal_outside_the_borderless_home() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        let font = crate::ui::theme::initialize(&mut context);
        context.io_mut().set_display_size([800.0, 600.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        let mut home = State::default();
        home.rename = Some((PathBuf::from("Project"), "Project".into()));
        home.rename_opening = true;
        let mut code = code::State::default();
        let ui = context.frame();
        let _font = ui.push_font(font);
        home.draw(&mut code, ui);
        assert!(ui.is_popup_open("Rename project"));
        assert_eq!(
            ui.clone_style().window_border_size(),
            kinematic_ui::theme::BORDER_WIDTH
        );
        drop(_font);
        context.end_frame();
    }

    #[test]
    fn home_fills_the_viewport_above_the_shared_footer() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        let mut home = State::default();
        let mut code = code::State::default();
        for size in [[1280.0, 720.0], [640.0, 480.0]] {
            context.io_mut().set_display_size(size);
            for _ in 0..3 {
                let ui = context.frame();
                if let Some(_bar) = ui.begin_main_menu_bar() {}
                home.draw(&mut code, ui);
                kinematic_ui::status_bar::draw(ui, env!("CARGO_PKG_VERSION"), false);
                let mut bottom = 0.0;
                ui.window("Kinematic home").build(|| {
                    assert_eq!(ui.window_pos()[0], 0.0);
                    assert_eq!(ui.window_size()[0], size[0]);
                    bottom = ui.window_pos()[1] + ui.window_size()[1];
                });
                ui.window("Project status").build(|| {
                    assert_eq!(ui.window_pos()[1], bottom);
                    assert_eq!(ui.window_pos()[1] + ui.window_size()[1], size[1]);
                });
                context.end_frame();
            }
        }
    }
}
