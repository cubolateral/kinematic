use crate::editor::documents::{Documents, FileEntry, SceneTemplate};
use crate::ui::icons;
use crate::ui::inputs::{self, Action as InputAction};
use dear_imgui_rs::{Condition, MouseButton, StyleColor, Ui};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
enum Action {
    File,
    Folder,
    Scene,
    Rename,
    Delete,
}

impl Action {
    fn title(self) -> &'static str {
        match self {
            Self::File => "Add File",
            Self::Folder => "Add Folder",
            Self::Scene => "Add Scene",
            Self::Rename => "Rename",
            Self::Delete => "Delete",
        }
    }
}

struct Dialog {
    action: Action,
    target: PathBuf,
    name: String,
    template: SceneTemplate,
    error: Option<String>,
    opening: bool,
}

#[derive(Default)]
pub(super) struct State {
    selected: Option<PathBuf>,
    context: Option<(PathBuf, bool)>,
    dialog: Option<Dialog>,
}

impl State {
    pub(super) fn draw(
        &mut self,
        documents: &mut Documents,
        ui: &Ui,
        error: &mut Option<String>,
    ) -> bool {
        let mut focus = false;
        let files = match documents.files() {
            Ok(files) => files,
            Err(message) => {
                ui.text_wrapped(&message);

                return false;
            }
        };
        let mut children = BTreeMap::<&Path, Vec<&FileEntry>>::new();

        for entry in &files {
            children
                .entry(entry.path.parent().unwrap_or(Path::new("")))
                .or_default()
                .push(entry);
        }

        let root = FileEntry {
            path: PathBuf::new(),
            directory: true,
        };
        let mut context_clicked = false;
        let name = documents
            .context
            .root()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        self.row(
            documents,
            ui,
            &root,
            &name,
            &children,
            &mut focus,
            &mut context_clicked,
            error,
        );

        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && ui.is_mouse_released(MouseButton::Right)
        {
            self.context = Some((PathBuf::new(), true));
            context_clicked = true;
        }

        if context_clicked {
            ui.open_popup("File context");
        }

        let mut action = None;

        if let Some(_popup) = ui.begin_popup("File context") {
            if let Some((target, directory)) = &self.context {
                if *directory {
                    for kind in [Action::Folder, Action::File, Action::Scene] {
                        if ui.menu_item(kind.title()) {
                            action = Some(kind);
                        }
                    }

                    ui.separator();
                } else if ui.menu_item("Open") {
                    match documents.open(target) {
                        Ok(()) => focus = true,
                        Err(message) => *error = Some(message),
                    }
                }

                if !target.as_os_str().is_empty() {
                    let _disabled =
                        ui.begin_disabled_with_cond(target == Path::new("kinematic.ron"));

                    if ui.menu_item("Rename...") {
                        action = Some(Action::Rename);
                    }

                    if ui.menu_item("Delete...") {
                        action = Some(Action::Delete);
                    }
                }
            }
        }

        if let Some(action) = action {
            let target = self.context.as_ref().unwrap().0.clone();
            let name = if matches!(action, Action::Rename) {
                target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };

            self.dialog = Some(Dialog {
                action,
                target,
                name,
                template: SceneTemplate::None,
                error: None,
                opening: true,
            });
        }

        focus |= self.dialog(documents, ui);

        focus
    }

    fn row(
        &mut self,
        documents: &mut Documents,
        ui: &Ui,
        entry: &FileEntry,
        name: &str,
        children: &BTreeMap<&Path, Vec<&FileEntry>>,
        focus: &mut bool,
        context_clicked: &mut bool,
        error: &mut Option<String>,
    ) {
        let path = &entry.path;
        let _id = ui.push_id(path.to_string_lossy().as_ref());
        let position = ui.cursor_screen_pos();
        let node = ui
            .tree_node_config("##Entry")
            .selected(self.selected.as_ref().or(documents.active.as_ref()) == Some(path))
            .leaf(!entry.directory)
            .span_avail_width(true)
            .frame_padding(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .opened(path.as_os_str().is_empty(), Condition::Once)
            .push();
        let clicked = ui.is_item_clicked()
            || (ui.is_item_focused() && inputs::active(ui, InputAction::Confirm));

        if ui.is_item_hovered() && ui.is_mouse_released(MouseButton::Right) {
            self.selected = Some(path.clone());
            self.context = Some((path.clone(), entry.directory));
            *context_clicked = true;
        }

        let icon = if entry.directory {
            icons::FOLDER
        } else {
            icons::FILE
        };
        let color = if entry.directory {
            [0.95, 0.72, 0.3, 1.0]
        } else {
            [0.4, 0.7, 0.95, 1.0]
        };
        let x = position[0] + ui.tree_node_to_label_spacing();
        let y = position[1] + ui.clone_style().frame_padding()[1];

        {
            let draw = ui.get_window_draw_list();

            draw.add_text([x, y], color, icon);

            draw.add_text(
                [x + ui.calc_text_size(icon)[0] + 6.0, y],
                ui.get_color_u32(StyleColor::Text),
                name,
            );
        }

        if clicked {
            self.selected = Some(path.clone());

            if !entry.directory {
                match documents.open(path) {
                    Ok(()) => *focus = true,
                    Err(message) => *error = Some(message),
                }
            }
        }

        if let Some(_node) = node {
            if entry.directory {
                if let Some(entries) = children.get(path.as_path()) {
                    for child in entries {
                        let name = child.path.file_name().unwrap_or_default().to_string_lossy();

                        self.row(
                            documents,
                            ui,
                            child,
                            &name,
                            children,
                            focus,
                            context_clicked,
                            error,
                        );
                    }
                }
            }
        }
    }

    fn dialog(&mut self, documents: &mut Documents, ui: &Ui) -> bool {
        let Some(dialog) = &mut self.dialog else {
            return false;
        };

        let title = dialog.action.title();

        if dialog.opening {
            ui.open_popup(title);
        }

        let mut open = true;
        let mut finished = false;
        let mut focus = false;

        if let Some(_modal) = crate::ui::windows::modal(ui, title, &mut open) {
            let width = ui.content_region_avail_width().max(1.0);
            let _wrap = ui.push_text_wrap_pos(ui.cursor_pos()[0] + width);

            ui.text_wrapped(if dialog.target.as_os_str().is_empty() {
                "Project root".into()
            } else {
                dialog.target.to_string_lossy().into_owned()
            });

            ui.separator();

            let deleting = matches!(dialog.action, Action::Delete);
            let mut submit = false;
            let validation = if deleting {
                ui.text_wrapped("Permanently delete this item and all its contents? Scene entries will be removed from the manifest.");

                None
            } else {
                ui.text("Name");
                ui.set_next_item_width(width);

                if ui.is_window_appearing() {
                    ui.set_keyboard_focus_here();
                }

                submit = ui
                    .input_text("##Name", &mut dialog.name)
                    .enter_returns_true(true)
                    .build();

                submit |= inputs::active(ui, InputAction::Confirm);

                if matches!(dialog.action, Action::Scene) {
                    ui.text("Template");

                    for (label, template) in [
                        ("No template", SceneTemplate::None),
                        ("2D template", SceneTemplate::TwoD),
                        ("3D template", SceneTemplate::ThreeD),
                    ] {
                        if ui.radio_button(label, dialog.template == template) {
                            dialog.template = template;
                        }
                    }

                    ui.text_wrapped("The .lua extension is added automatically. The scene is linked in the manifest.");
                }

                if matches!(dialog.action, Action::Rename) {
                    ui.text_wrapped("Scene paths are updated. Lua imports are not rewritten.");
                }

                filename(&dialog.name, matches!(dialog.action, Action::Scene)).err()
            };

            dialog.opening = false;

            if !dialog.name.is_empty() {
                if let Some(message) = &validation {
                    ui.text_wrapped(message);
                }
            }

            if let Some(message) = &dialog.error {
                let _color = ui.push_style_color(StyleColor::Text, [1.0, 0.45, 0.4, 1.0]);

                ui.text_wrapped(message);
            }

            ui.separator();

            let label = match dialog.action {
                Action::Delete => "Delete",
                Action::Rename => "Rename",
                _ => "Create",
            };

            {
                let _disabled = ui.begin_disabled_with_cond(validation.is_some());

                submit |= ui.button(label);
            }

            ui.same_line();

            if ui.button("Cancel") || inputs::active(ui, InputAction::Cancel) {
                finished = true;

                ui.close_current_popup();
            } else if submit && validation.is_none() {
                let result = (|| {
                    let path = if deleting {
                        dialog.target.clone()
                    } else {
                        let name = filename(&dialog.name, matches!(dialog.action, Action::Scene))?;
                        let parent = if matches!(dialog.action, Action::Rename) {
                            dialog.target.parent().unwrap_or(Path::new(""))
                        } else {
                            &dialog.target
                        };

                        parent.join(name)
                    };

                    match dialog.action {
                        Action::Folder => documents.create(&path, true)?,
                        Action::File => documents.create(&path, false)?,
                        Action::Scene => documents.create_scene(&path, dialog.template)?,
                        Action::Rename => documents.rename(&dialog.target, &path)?,
                        Action::Delete => documents.delete(&path)?,
                    }

                    self.selected = if deleting { None } else { Some(path) };

                    Ok::<_, String>(())
                })();

                match result {
                    Ok(()) => {
                        focus = matches!(dialog.action, Action::File | Action::Scene);
                        finished = true;

                        ui.close_current_popup();
                    }

                    Err(message) => dialog.error = Some(message),
                }
            }
        }

        if finished || !open {
            self.dialog = None;
        }

        focus
    }
}

fn filename(name: &str, scene: bool) -> Result<String, String> {
    if name.trim().is_empty()
        || name != name.trim()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\', '\0', '<', '>', ':', '"', '|', '?', '*'])
        || name.chars().any(char::is_control)
        || name.ends_with('.')
    {
        return Err(
            "Enter a filename without path separators, special characters or surrounding spaces."
                .into(),
        );
    }

    if scene && Path::new(name).extension().is_some_and(|ext| ext != "lua") {
        return Err("Use a .lua extension or leave the extension empty.".into());
    }

    Ok(if scene && Path::new(name).extension().is_none() {
        format!("{name}.lua")
    } else {
        name.into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dear_imgui_rs::Key;

    #[test]
    fn filenames_keep_literal_labels_and_reject_paths_and_wrong_scene_extensions() {
        assert_eq!(filename("My scene##1", true).unwrap(), "My scene##1.lua");
        assert_eq!(filename("intro.lua", true).unwrap(), "intro.lua");
        assert_eq!(filename("config.ron", false).unwrap(), "config.ron");

        for name in [
            "",
            " ",
            ".",
            "..",
            "../test",
            "a/b",
            "a\\b",
            "a\0b",
            " test",
            "test.",
            "scene.txt",
        ] {
            assert!(filename(name, true).is_err(), "{name:?}");
        }
    }

    #[test]
    fn context_menu_stays_open_and_file_dialogs_start_centered() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();

        let font = crate::ui::theme::initialize(&mut context);

        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        context.io_mut().set_display_size([900.0, 700.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        let root = std::env::temp_dir().join(format!("kinematic-files-ui-{}", std::process::id()));
        let project = crate::scripting::LuaProject::create(&root, "Files", false).unwrap();
        let mut documents = Documents::new(project.context, "").unwrap();

        documents.lsp = None;

        let mut state = State::default();
        let mut error = None;
        let mut origin = [0.0, 0.0];

        for _ in 0..2 {
            let ui = context.frame();
            let _font = ui.push_font(font);

            ui.window("Files")
                .position([20.0, 20.0], Condition::Always)
                .size([280.0, 600.0], Condition::Always)
                .build(|| {
                    origin = ui.cursor_screen_pos();

                    state.draw(&mut documents, ui, &mut error);
                });

            drop(_font);
            context.end_frame();
        }

        context
            .io_mut()
            .add_mouse_pos_event([origin[0] + 60.0, origin[1] + 10.0]);

        for down in [true, false] {
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Right, down);

            let ui = context.frame();
            let _font = ui.push_font(font);

            ui.window("Files").build(|| {
                state.draw(&mut documents, ui, &mut error);
            });

            drop(_font);
            context.end_frame();
        }

        assert!(state.context.as_ref().unwrap().0.as_os_str().is_empty());
        context.io_mut().add_mouse_pos_event([800.0, 650.0]);

        {
            let ui = context.frame();
            let _font = ui.push_font(font);

            ui.window("Files").build(|| {
                state.draw(&mut documents, ui, &mut error);
                assert!(ui.is_popup_open("File context"));

                if let Some(_popup) = ui.begin_popup("File context") {
                    ui.close_current_popup();
                }
            });

            drop(_font);
            context.end_frame();
        }

        for (action, target, name, title, expected_path) in [
            (
                Action::Scene,
                "scenes",
                "New scene##1",
                c"Add Scene",
                "scenes/New scene##1.lua",
            ),
            (
                Action::Rename,
                "scenes/New scene##1.lua",
                "Renamed scene.lua",
                c"Rename",
                "scenes/Renamed scene.lua",
            ),
        ] {
            state.dialog = Some(Dialog {
                action,
                target: target.into(),
                name: name.into(),
                template: SceneTemplate::ThreeD,
                error: None,
                opening: true,
            });

            let mut visible_frames = 0;

            for enter in [false, false, false, true] {
                context.io_mut().add_key_event(Key::Enter, enter);

                let ui = context.frame();
                let _font = ui.push_font(font);

                ui.window("Files").build(|| {
                    state.draw(&mut documents, ui, &mut error);

                    if !enter {
                        // Inspect the actual modal without beginning it a second time.
                        unsafe {
                            let window = dear_imgui_rs::sys::igFindWindowByName(title.as_ptr());

                            assert!(!window.is_null());

                            if !(*window).Hidden {
                                visible_frames += 1;

                                let pos = (*window).Pos;
                                let size = (*window).Size;

                                assert!(
                                    (pos.x + size.x / 2.0 - 450.0).abs() <= 1.0,
                                    "{title:?}: X: {}, {}",
                                    pos.x,
                                    size.x
                                );

                                assert!(
                                    (pos.y + size.y / 2.0 - 350.0).abs() <= 1.0,
                                    "{title:?}: Y: {}, {}",
                                    pos.y,
                                    size.y
                                );
                            }
                        }
                    }
                });

                drop(_font);
                context.end_frame();
            }

            assert!(visible_frames > 0);
            assert!(state.dialog.is_none());
            assert!(error.is_none());
            assert!(root.join(expected_path).exists());

            assert!(
                crate::scripting::LuaProject::open(&root)
                    .unwrap()
                    .manifest
                    .scenes
                    .contains(&PathBuf::from(expected_path))
            );
        }

        drop(documents);

        std::fs::remove_dir_all(root).unwrap();
    }
}
