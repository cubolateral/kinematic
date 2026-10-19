use crate::ui::inputs::{self, Action as InputAction};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

use dear_imgui_rs::{ListClipper, MouseButton, StyleVar, TableFlags, Ui};

use super::icons;

struct Entry {
    path: PathBuf,
    name: String,
    directory: bool,
}

pub(super) struct State {
    cwd: PathBuf,
    path: String,
    entries: Vec<Entry>,
    loading: Option<mpsc::Receiver<io::Result<Vec<Entry>>>>,
    cancelled: Arc<AtomicBool>,
    selected: Option<PathBuf>,
    search: String,
    hidden: bool,
    history: Vec<PathBuf>,
    history_index: usize,
    error: Option<String>,
    project: Option<(PathBuf, bool)>,
}

impl State {
    pub(super) fn new(path: &str) -> io::Result<Self> {
        let cwd = Path::new(path)
            .canonicalize()
            .ok()
            .filter(|path| path.is_dir())
            .map(Ok)
            .unwrap_or_else(std::env::current_dir)?;
        let mut state = Self {
            path: cwd.to_string_lossy().into_owned(),
            history: vec![cwd.clone()],
            cwd,
            entries: Vec::new(),
            loading: None,
            cancelled: Arc::new(AtomicBool::new(false)),
            selected: None,
            search: String::new(),
            hidden: false,
            history_index: 0,
            error: None,
            project: None,
        };

        state.refresh();

        Ok(state)
    }

    fn refresh(&mut self) {
        self.loading = None;

        self.cancelled.store(true, Ordering::Relaxed);

        self.cancelled = Arc::new(AtomicBool::new(false));

        let cancelled = self.cancelled.clone();
        let path = self.cwd.clone();
        let (send, receive) = mpsc::channel();

        self.entries.clear();

        self.selected = None;
        self.error = None;
        self.project = None;

        match thread::Builder::new()
            .name("File browser".into())
            .spawn(move || {
                let _ = send.send(read_directory(&path, &cancelled));
            }) {
            Ok(_) => self.loading = Some(receive),
            Err(error) => self.error = Some(format!("Could not read directory: {error}.")),
        }
    }

    fn navigate(&mut self, path: &Path, remember: bool) -> bool {
        match path.canonicalize().and_then(|path| {
            if path.is_dir() {
                Ok(path)
            } else {
                Err(io::Error::other("Select a directory."))
            }
        }) {
            Ok(path) => {
                if remember && path != self.cwd {
                    self.history.truncate(self.history_index + 1);
                    self.history.push(path.clone());

                    self.history_index = self.history.len() - 1;
                }

                self.path = path.to_string_lossy().into_owned();
                self.cwd = path;

                self.refresh();

                true
            }

            Err(error) => {
                self.error = Some(format!("Could not open directory: {error}."));

                false
            }
        }
    }

    pub(super) fn draw(&mut self, ui: &Ui) -> Option<Option<PathBuf>> {
        if let Some(receive) = &self.loading {
            match receive.try_recv() {
                Ok(result) => {
                    self.loading = None;

                    match result {
                        Ok(entries) => self.entries = entries,
                        Err(error) => {
                            self.error = Some(format!("Could not read directory: {error}."))
                        }
                    }
                }

                Err(mpsc::TryRecvError::Disconnected) => {
                    self.loading = None;
                    self.error = Some("Directory reader stopped.".into());
                }

                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        const TITLE: &str = "Select Project Folder";

        if !ui.is_popup_open(TITLE) {
            ui.open_popup(TITLE);
        }

        let _frames = ui.push_style_var(StyleVar::FrameBorderSize(0.0));
        let _children = ui.push_style_var(StyleVar::ChildBorderSize(0.0));
        let _windows = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        let _popup_border = ui.push_style_var(StyleVar::PopupBorderSize(0.0));
        let mut open = true;

        {
            let popup = super::windows::sized_modal(ui, TITLE, &mut open, [760.0, 520.0]);

            if let Some(_popup) = popup {
                for (label, step, enabled) in [
                    ("Back", -1, self.history_index > 0),
                    ("Forward", 1, self.history_index + 1 < self.history.len()),
                ] {
                    let _disabled = ui.begin_disabled_with_cond(!enabled);

                    if button(ui, label) {
                        let index = (self.history_index as isize + step) as usize;
                        let path = self.history[index].clone();

                        if self.navigate(&path, false) {
                            self.history_index = index;
                        }
                    }

                    ui.same_line();
                }

                {
                    let _disabled = ui.begin_disabled_with_cond(self.cwd.parent().is_none());

                    if button(ui, "Up") {
                        if let Some(parent) = self.cwd.parent().map(Path::to_path_buf) {
                            self.navigate(&parent, true);
                        }
                    }
                }

                ui.same_line();

                if button(ui, "Refresh") {
                    self.refresh();
                }

                ui.same_line();

                let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" });

                if let Some(home) = home {
                    if button(ui, "Home") {
                        self.navigate(Path::new(&home), true);
                    }
                }

                ui.set_next_item_width(-1.0);

                if ui
                    .input_text("##BrowserPath", &mut self.path)
                    .enter_returns_true(true)
                    .build()
                {
                    self.navigate(&PathBuf::from(&self.path), true);
                }

                ui.set_next_item_width((ui.content_region_avail_width() - 170.0).max(1.0));
                ui.input_text("Search", &mut self.search).build();
                ui.same_line();
                ui.checkbox("Hidden files", &mut self.hidden);

                let query = self.search.to_lowercase();
                let entries: Vec<_> = self
                    .entries
                    .iter()
                    .filter(|entry| {
                        (self.hidden || !entry.name.starts_with('.'))
                            && entry.name.to_lowercase().contains(&query)
                    })
                    .collect();
                let mut navigate = None;

                ui.table("BrowserFiles")
                    .flags(
                        TableFlags::SCROLL_Y | TableFlags::ROW_BG | TableFlags::NO_SAVED_SETTINGS,
                    )
                    .outer_size([
                        0.0,
                        (ui.content_region_avail()[1] - ui.frame_height() * 3.0).max(80.0),
                    ])
                    .column("Name")
                    .weight(1.0)
                    .done()
                    .column("Extension")
                    .width(100.0)
                    .done()
                    .headers(true)
                    .build(|ui| {
                        for index in ListClipper::new(entries.len()).begin(ui).iter() {
                            let entry = entries[index];
                            let _id = ui.push_id(index);

                            ui.table_next_row();
                            ui.table_next_column();

                            let (icon, color) = if entry.directory {
                                (icons::FOLDER, [0.95, 0.72, 0.3, 1.0])
                            } else {
                                (icons::FILE, [0.4, 0.7, 0.95, 1.0])
                            };

                            ui.text_colored(color, icon);
                            ui.same_line();

                            let position = ui.cursor_screen_pos();
                            let clicked = ui
                                .selectable_config("##Entry")
                                .size([ui.content_region_avail_width(), ui.text_line_height()])
                                .flags(
                                    dear_imgui_rs::SelectableFlags::NO_AUTO_CLOSE_POPUPS
                                        | dear_imgui_rs::SelectableFlags::ALLOW_DOUBLE_CLICK,
                                )
                                .selected(self.selected.as_ref() == Some(&entry.path))
                                .build();

                            ui.get_window_draw_list().add_text(
                                position,
                                ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
                                &entry.name,
                            );

                            if clicked {
                                self.selected = Some(entry.path.clone());
                            }

                            if entry.directory
                                && ((clicked && ui.is_mouse_double_clicked(MouseButton::Left))
                                    || (ui.is_item_focused()
                                        && inputs::active(ui, InputAction::Confirm)))
                            {
                                navigate = Some(entry.path.clone());
                            }

                            ui.table_next_column();

                            if !entry.directory {
                                ui.text(
                                    entry.path.extension().unwrap_or_default().to_string_lossy(),
                                );
                            }
                        }
                    });

                if let Some(path) = navigate {
                    self.navigate(&path, true);
                }

                if self.loading.is_some() {
                    ui.text("Loading...");
                }

                if let Some(error) = &self.error {
                    ui.text_wrapped(error);
                }

                let target = self.selected.as_ref().unwrap_or(&self.cwd);
                if self.project.as_ref().is_none_or(|(path, _)| path != target) {
                    let valid = valid_project(target);

                    self.project = Some((target.clone(), valid));
                }
                let valid = self.project.as_ref().is_some_and(|(_, valid)| *valid);

                {
                    let _disabled = ui.begin_disabled_with_cond(
                        self.loading.is_some() || self.error.is_some() || !valid,
                    );

                    if button(ui, "Select Folder") {
                        ui.close_current_popup();

                        return Some(Some(target.clone()));
                    }
                }

                ui.same_line();

                if button(ui, "Cancel") || inputs::active(ui, InputAction::Cancel) {
                    ui.close_current_popup();

                    return Some(None);
                }
            }
        }

        if !open {
            return Some(None);
        }

        None
    }
}

fn button(ui: &Ui, label: &str) -> bool {
    let _border = ui.push_style_var(StyleVar::FrameBorderSize(1.0));

    ui.button(label)
}

fn valid_project(path: &Path) -> bool {
    path.join(crate::scripting::MANIFEST).is_file()
        && crate::scripting::LuaProject::open(path).is_ok()
}

impl Drop for State {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

fn read_directory(path: &Path, cancelled: &AtomicBool) -> io::Result<Vec<Entry>> {
    let mut entries = fs::read_dir(path)?
        .take_while(|_| !cancelled.load(Ordering::Relaxed))
        .map(|entry| {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();

            Ok(Entry {
                directory: kind.is_dir() || (kind.is_symlink() && path.is_dir()),
                name: entry.file_name().to_string_lossy().into_owned(),
                path,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    entries.sort_by_cached_key(|entry| (!entry.directory, entry.name.to_lowercase()));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_requires_a_valid_manifest_in_the_project_folder() {
        let root =
            std::env::temp_dir().join(format!("kinematic-browser-project-{}", std::process::id()));

        fs::create_dir_all(&root).unwrap();
        assert!(!valid_project(&root));

        let project =
            crate::scripting::LuaProject::create(root.join("valid"), "Test", false).unwrap();

        assert!(valid_project(project.context.root()));
        assert!(!valid_project(&project.context.root().join("kinematic")));

        let manifest = project.context.root().join(crate::scripting::MANIFEST);

        fs::write(&manifest, "Invalid").unwrap();
        assert!(!valid_project(project.context.root()));

        let mut invalid = project.manifest;

        invalid.fps = 0;
        fs::write(&manifest, ron::to_string(&invalid).unwrap()).unwrap();
        assert!(!valid_project(project.context.root()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn browser_draws_and_restores_the_editors_border_style() {
        let _context_lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();

        let font = crate::ui::theme::initialize(&mut context);

        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        let mut state = State::new(std::env::temp_dir().to_str().unwrap()).unwrap();

        state.entries = vec![Entry {
            path: state.cwd.join("scene##.lua"),
            name: "scene##.lua".into(),
            directory: false,
        }];

        state.loading = None;

        for _ in 0..2 {
            let ui = context.frame();
            let _font = ui.push_font(font);

            assert!(state.draw(ui).is_none());
            assert_eq!(ui.clone_style().frame_border_size(), 1.0);
            assert_eq!(ui.clone_style().child_border_size(), 1.0);

            drop(_font);
            context.end_frame();
        }
    }

    #[test]
    fn listing_keeps_extensions_and_real_brackets_and_navigation_rejects_files() {
        let root = std::env::temp_dir().join(format!("kinematic-browser-{}", std::process::id()));

        fs::create_dir_all(root.join("Folder [real]")).unwrap();

        fs::write(root.join("scene.lua"), "").unwrap();

        let entries = read_directory(&root, &AtomicBool::new(false)).unwrap();

        assert_eq!(entries[0].name, "Folder [real]");
        assert!(entries[0].directory);
        assert_eq!(entries[1].name, "scene.lua");
        assert_eq!(entries[1].path.extension().unwrap(), "lua");

        let mut state = State::new(root.to_str().unwrap()).unwrap();
        let cwd = state.cwd.clone();

        assert!(!state.navigate(&root.join("scene.lua"), true));
        assert_eq!(state.cwd, cwd);
        assert!(state.navigate(&root.join("Folder [real]"), true));
        assert_eq!(state.history.len(), 2);

        fs::remove_dir_all(root).unwrap();
    }
}
