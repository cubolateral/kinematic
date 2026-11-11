use super::browser;
use crate::editor::{Editor, documents::Documents};
use kinematic_ui::{controls, widgets};
use std::path::PathBuf;

mod diagnostics;
mod files;
mod presets;

pub(super) const FILES_WINDOW: &str = "Files";
pub(super) const CONSOLE_WINDOW: &str = "Console";
pub(super) type Settings = crate::editor::external::Settings;

#[derive(Clone)]
pub(crate) enum ProjectAction {
    Open(PathBuf),
    Create(PathBuf, String, crate::core::ProjectSettings),
    Close,
    Exit,
}
struct ProjectForm {
    name: String,
    path: String,
    directory: Option<PathBuf>,
    manual_path: bool,
}

impl Default for ProjectForm {
    fn default() -> Self {
        Self {
            name: "Project".into(),
            path: String::new(),
            directory: None,
            manual_path: false,
        }
    }
}

impl ProjectForm {
    fn update_path(&mut self) {
        if self.manual_path {
            return;
        }

        let Some(directory) = self.directory.as_ref().filter(|path| path.is_dir()) else {
            self.path.clear();

            return;
        };
        let characters: Vec<_> = self.name.chars().collect();
        let mut folder = String::new();

        for (index, character) in characters.iter().copied().enumerate() {
            if !character.is_alphanumeric() {
                if !folder.is_empty() && !folder.ends_with('_') {
                    folder.push('_');
                }

                continue;
            }

            if character.is_uppercase() && !folder.is_empty() && !folder.ends_with('_') {
                let previous = characters[index - 1];
                let next_is_lowercase = characters.get(index + 1).is_some_and(|c| c.is_lowercase());

                if previous.is_lowercase() || previous.is_numeric() || next_is_lowercase {
                    folder.push('_');
                }
            }

            folder.extend(character.to_lowercase());
        }

        let folder = folder.trim_end_matches('_');

        self.path = if folder.is_empty() {
            String::new()
        } else {
            directory.join(folder).to_string_lossy().into_owned()
        };
    }
}

#[derive(Default)]
pub(super) struct State {
    pub(super) projects_open: bool,
    opening_project: bool,
    root: String,
    browser: Option<browser::State>,
    project_form: ProjectForm,
    resolution: presets::Resolution,
    fps: presets::Fps,
    files: files::State,
    pending: Option<ProjectAction>,
    ready: Option<ProjectAction>,
    diagnostics: diagnostics::Footer,
    launcher: crate::editor::external::Launcher,
    pub error: Option<String>,
    pub(super) project_error: Option<String>,
}
impl State {
    pub(super) fn project_directory(&self) -> Option<PathBuf> {
        self.project_form.directory.clone()
    }
    pub(super) fn set_project_directory(&mut self, directory: Option<PathBuf>) {
        self.project_form.directory = directory.filter(|path| path.is_dir());
        self.project_form.manual_path = false;
        self.project_form.update_path();
    }
    pub(super) fn files_focused(&self) -> bool {
        self.files.focused
    }
    pub(super) fn dragged_audio(&self) -> Option<&std::path::Path> {
        self.files.dragged_audio.as_deref()
    }
    pub fn prepare(&mut self, context: &dear_imgui_rs::Context) {
        if let Some(text) = self.files.copy_path.take() {
            context.set_clipboard_text(text);
        }
        if let Some(text) = self.diagnostics.take_copy() {
            context.set_clipboard_text(text);
        }
    }
    pub fn reset(&mut self) {
        self.files = files::State::default();
        self.diagnostics = diagnostics::Footer::default();
        self.error = None;
        self.projects_open = false;
        self.opening_project = false;
        self.browser = None;
    }
    pub(super) fn request_project(&mut self, action: ProjectAction) {
        self.pending = Some(action);
    }
    pub(super) fn open_project(&mut self) {
        self.browse(true);
    }
    pub fn request_exit(&mut self) {
        self.request_project(ProjectAction::Exit);
    }
    pub fn take_action(&mut self) -> Option<ProjectAction> {
        self.ready.take()
    }
    pub fn projects(&mut self, mut editor: Option<&mut Editor>, ui: &dear_imgui_rs::Ui) {
        if self.projects_open && self.browser.is_none() {
            const TITLE: &str = "Create project";

            if !ui.is_popup_open(TITLE) {
                self.project_form.update_path();

                ui.open_popup(TITLE);
            }

            let mut open = true;
            let mut finished = false;

            if let Some(_modal) = super::windows::modal(ui, TITLE, &mut open) {
                let browse = super::properties::row(ui, "Project path", false, || {
                    let browse_width =
                        ui.calc_text_size("Browse")[0] + ui.clone_style().frame_padding()[0] * 2.0;

                    ui.set_next_item_width(
                        (ui.content_region_avail_width()
                            - browse_width
                            - ui.clone_style().item_spacing()[0])
                            .max(1.0),
                    );

                    if controls::text_input(ui, "##ProjectPath", &mut self.project_form.path) {
                        self.project_form.manual_path = true;
                    }

                    ui.same_line();

                    controls::button_sized(ui, "Browse", [browse_width, 0.0])
                });

                if browse.changed {
                    self.browse(false);
                }

                if super::properties::row(ui, "Name", false, || {
                    controls::text_input(ui, "##ProjectName", &mut self.project_form.name)
                })
                .changed
                {
                    self.project_form.update_path();
                }

                super::properties::row(ui, "Resolution", false, || {
                    ui.set_next_item_width(ui.content_region_avail_width());

                    self.resolution.draw(ui)
                });
                super::properties::row(ui, "FPS", false, || {
                    ui.set_next_item_width(ui.content_region_avail_width());

                    self.fps.draw(ui)
                });

                let valid = !self.project_form.name.trim().is_empty()
                    && !self.project_form.path.trim().is_empty();

                match super::windows::actions(ui, &[("Create project", valid), ("Cancel", true)]) {
                    Some(1) => {
                        finished = true;

                        ui.close_current_popup();
                    }
                    Some(0) => {
                        finished = true;

                        self.request_project(ProjectAction::Create(
                            self.project_form.path.clone().into(),
                            self.project_form.name.clone(),
                            crate::core::ProjectSettings {
                                resolution: self.resolution.value(),
                                fps: self.fps.value(),
                                ..crate::core::ProjectSettings::default()
                            },
                        ));

                        ui.close_current_popup();
                    }
                    _ => {}
                }
            }

            self.projects_open = open && !finished;
        }

        if let Some(browser) = &mut self.browser {
            if let Some(result) = browser.draw(ui) {
                self.finish_browse(result);
            }
        }

        if let Some(pending) = self.pending.clone() {
            let dirty = editor
                .as_ref()
                .and_then(|e| e.documents.as_ref())
                .is_some_and(Documents::dirty);
            let exporting = editor.as_ref().is_some_and(|e| e.is_exporting());

            if exporting {
                self.project_error =
                    Some("Finish or cancel export before closing the project.".into());
                self.pending = None;

                return;
            }

            if !dirty {
                self.finish(pending);

                return;
            }

            ui.open_popup("Unsaved changes");

            let mut open = true;

            if let Some(_modal) = super::windows::modal(ui, "Unsaved changes", &mut open) {
                widgets::text(ui, "Save changes before continuing?");

                let action = super::windows::actions(
                    ui,
                    &[("Save", true), ("Discard", true), ("Cancel", true)],
                );

                if action == Some(0) {
                    let result = editor.as_deref_mut().map_or(Ok(()), |editor| {
                        editor.save_project().and_then(|()| {
                            editor
                                .documents
                                .as_mut()
                                .map_or(Ok(()), Documents::save_all)
                        })
                    });

                    match result {
                        Ok(()) => {
                            self.finish(pending.clone());

                            ui.close_current_popup();
                        }

                        Err(error) => self.project_error = Some(error),
                    }
                }

                if action == Some(1) {
                    self.finish(pending);

                    ui.close_current_popup();
                }

                if action == Some(2) {
                    self.pending = None;

                    ui.close_current_popup();
                }
            }

            if !open {
                self.pending = None;
            }
        }
    }

    fn browse(&mut self, opening_project: bool) {
        self.opening_project = opening_project;

        let mode = if opening_project {
            browser::Mode::Project
        } else {
            browser::Mode::Directory
        };
        let path = if opening_project {
            self.root.clone()
        } else {
            std::path::Path::new(&self.project_form.path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new(""))
                .to_string_lossy()
                .into_owned()
        };

        match browser::State::new(&path, mode) {
            Ok(browser) => self.browser = Some(browser),
            Err(error) => {
                self.project_error = Some(format!("Could not browse directories: {error}."))
            }
        }
    }

    fn finish_browse(&mut self, path: Option<PathBuf>) {
        self.browser = None;

        if let Some(path) = path {
            if let Some(path) = path.to_str() {
                self.root = path.to_owned();
                self.project_error = None;

                if self.opening_project {
                    self.request_project(ProjectAction::Open(path.into()));
                } else {
                    self.set_project_directory(Some(path.into()));
                }
            } else {
                self.project_error = Some("Directory path must be valid UTF-8.".into());
            }
        }

        self.opening_project = false;
    }

    fn finish(&mut self, pending: ProjectAction) {
        self.ready = Some(pending);
        self.pending = None;
    }
    pub fn draw(&mut self, documents: &mut Documents, settings: &Settings, ui: &dear_imgui_rs::Ui) {
        if let Some(Err(error)) = self.launcher.poll() {
            self.error = Some(error);
        }
        super::windows::panel(ui, FILES_WINDOW, || {
            self.files.draw(documents, ui, &mut self.error);
            if let Some(path) = self.files.open_requested.take() {
                self.error = self
                    .launcher
                    .open(settings, &documents.context, Some((&path, 1, 1)))
                    .err();
            }
        });
        super::windows::panel(ui, CONSOLE_WINDOW, || {
            let mut messages = documents.diagnostic_messages();
            messages.extend(self.error.iter().cloned());
            let height = self.diagnostics.height(ui, &messages);
            if let Some(message) = self
                .diagnostics
                .draw(ui, &messages, &documents.status, height)
            {
                if let Some((path, line, column)) = diagnostic_location(&message) {
                    self.error = self
                        .launcher
                        .open(
                            settings,
                            &documents.context,
                            Some((std::path::Path::new(path), line + 1, column + 1)),
                        )
                        .err();
                }
            }
            for line in &documents.console {
                widgets::message(ui, line);
            }
        });
    }
}
fn diagnostic_location(error: &str) -> Option<(&str, usize, usize)> {
    for line in error.lines() {
        let line = line
            .trim()
            .trim_start_matches("Warning: ")
            .trim_start_matches("LuaLS ");
        let Some(end) = line.find(".lua:") else {
            continue;
        };
        let path = line[..end + 4].rsplit(": ").next()?.trim_start_matches('@');
        let mut parts = line[end + 5..].split(':');

        if let Some(number) = parts.next().and_then(|n| n.parse::<usize>().ok()) {
            let column = parts
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(1);

            return Some((path, number.saturating_sub(1), column.saturating_sub(1)));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::diagnostic_location;

    #[test]
    fn runtime_locations_keep_spaces_windows_paths_and_columns() {
        assert_eq!(
            diagnostic_location("Runtime error: scenes/My scene.lua:12:3: Failed."),
            Some(("scenes/My scene.lua", 11, 2))
        );
        assert_eq!(
            diagnostic_location("Runtime error: C:\\Project Files\\intro.lua:9: Failed."),
            Some(("C:\\Project Files\\intro.lua", 8, 0))
        );
        assert_eq!(diagnostic_location("No source location."), None);
    }
}
