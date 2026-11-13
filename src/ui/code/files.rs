use crate::editor::documents::{Documents, FileEntry, SceneTemplate};
use crate::ui::inputs::{self, Action as InputAction};
use crate::utilities::terminal::open_terminal;
use dear_imgui_rs::{Condition, MouseButton, StyleColor, Ui};
use kinematic_ui::widgets;
use kinematic_ui::{controls, icons, menu};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
enum Action {
    File,
    Folder,
    Script,
    Duplicate,
    Scene,
    Rename,
    Delete,
    Open,
    CopyPath,
    CopyAbsolutePath,
    Terminal,
}

impl Action {
    fn title(self) -> &'static str {
        match self {
            Self::File => "Add File",
            Self::Folder => "Add Folder",
            Self::Script => "Add Script",
            Self::Scene => "Add Scene",
            Self::Duplicate => "Duplicate",
            Self::Rename => "Rename",
            Self::Delete => "Delete",
            Self::Open => "Open",
            Self::CopyPath => "Copy Path",
            Self::CopyAbsolutePath => "Copy Absolute Path",
            Self::Terminal => "Open in Terminal",
        }
    }
    fn binding(self) -> Option<InputAction> {
        Some(match self {
            Self::CopyPath => InputAction::FileCopyPath,
            Self::CopyAbsolutePath => InputAction::FileCopyAbsolutePath,
            Self::Rename => InputAction::FileRename,
            Self::Duplicate => InputAction::FileDuplicate,
            Self::Delete => InputAction::FileDelete,
            Self::Terminal => InputAction::FileTerminal,
            _ => return None,
        })
    }

    fn menu(self, ui: &Ui) -> bool {
        let shortcut = self
            .binding()
            .map(|binding| inputs::map().binding(binding).display.as_str())
            .unwrap_or("");

        menu::item(ui, self.title(), shortcut)
    }
}

struct Dialog {
    action: Action,
    target: PathBuf,
    name: String,
    template: SceneTemplate,
    opening: bool,
    targets: Vec<PathBuf>,
}

#[derive(Default)]
pub(super) struct State {
    pub(super) focused: bool,
    pub(super) open_requested: Option<PathBuf>,
    pub(super) dragged_audio: Option<PathBuf>,
    selected: BTreeSet<PathBuf>,
    anchor: Option<PathBuf>,
    visible: Vec<PathBuf>,
    pending_click: Option<PathBuf>,
    dragged: Vec<PathBuf>,
    drop_folder: Option<PathBuf>,
    context: Option<(PathBuf, bool)>,
    dialog: Option<Dialog>,
    expansion: Option<(PathBuf, bool, bool)>,
    pending_expansion: BTreeMap<PathBuf, bool>,
    terminal_result: Option<std::sync::mpsc::Receiver<String>>,
    pub(super) copy_path: Option<String>,
}

impl State {
    pub(super) fn draw(
        &mut self,
        documents: &mut Documents,
        ui: &Ui,
        error: &mut Option<String>,
    ) -> bool {
        let mut focus = false;

        if let Some(receiver) = &self.terminal_result {
            match receiver.try_recv() {
                Ok(message) => {
                    *error = Some(message);

                    self.terminal_result = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.terminal_result = None,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }

        self.focused =
            ui.is_window_focused_with_flags(dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS);

        let menu_style = menu::Style::capture(ui);
        let files = match documents.files() {
            Ok(files) => files,
            Err(message) => {
                *error = Some(message);

                return false;
            }
        };

        if let Some((target, expanded, recursive)) = self.expansion.take() {
            self.pending_expansion.insert(target.clone(), expanded);

            if recursive {
                for entry in &files {
                    if entry.directory && entry.path.starts_with(&target) {
                        self.pending_expansion.insert(entry.path.clone(), expanded);
                    }
                }
            }
        }

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

        self.selected.retain(|path| {
            path.as_os_str().is_empty() || files.iter().any(|entry| entry.path == *path)
        });

        let mut visible = Vec::new();
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
            &mut visible,
            &mut focus,
            &mut context_clicked,
            error,
        );

        self.visible = visible;

        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && ui.is_mouse_clicked(MouseButton::Left)
        {
            self.selected.clear();
            self.anchor = None;
            self.pending_click = None;
        }

        if let Some(folder) = self.drop_folder.take() {
            match documents.move_files(&self.dragged, &folder) {
                Ok(paths) => {
                    self.selected = paths.into_iter().collect();
                    self.anchor = self.selected.first().cloned();

                    self.pending_expansion.insert(folder, true);

                    documents.status = "Files moved.".into();
                }
                Err(message) => *error = Some(message),
            }
        }

        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && ui.is_mouse_clicked(MouseButton::Right)
        {
            if self.selected.len() <= 1 {
                self.select(Path::new(""), false);

                self.context = Some((PathBuf::new(), true));
            }

            if self.selected.len() > 1 {
                self.context = Some((self.selected.first().unwrap().clone(), false));
            }

            context_clicked = true;
        }

        if context_clicked {
            ui.open_popup("File context");
        }

        let mut action = None;

        if self.focused
            && self.dialog.is_none()
            && !ui.io().want_text_input()
            && !ui.is_popup_open("File context")
        {
            for kind in [
                Action::CopyPath,
                Action::CopyAbsolutePath,
                Action::Rename,
                Action::Duplicate,
                Action::Delete,
                Action::Terminal,
            ] {
                if (self.selected.len() <= 1 || matches!(kind, Action::Delete))
                    && inputs::active(ui, kind.binding().unwrap())
                {
                    let target = self
                        .selected
                        .first()
                        .or(documents.active.as_ref())
                        .cloned()
                        .unwrap_or_default();
                    let directory = target.as_os_str().is_empty()
                        || files
                            .iter()
                            .any(|entry| entry.path == target && entry.directory);

                    self.context = Some((target, directory));

                    action = Some(kind);
                }
            }
        }

        let _menu_style = menu_style.push(ui);

        if let Some(_popup) = ui.begin_popup("File context") {
            if self.selected.len() > 1 {
                let _disabled =
                    ui.begin_disabled_with_cond(self.selected.iter().any(|path| protected(path)));

                if Action::Delete.menu(ui) {
                    action = Some(Action::Delete);
                }
            } else if let Some((target, directory)) = &self.context {
                if let Some(_menu) = controls::menu(ui, "Create New") {
                    for (kind, icon, label) in [
                        (Action::Folder, icons::FOLDER, "Folder"),
                        (Action::Scene, icons::SCENE, "Scene"),
                        (Action::Script, icons::SCRIPT, "Script"),
                        (Action::File, icons::FILE, "File"),
                    ] {
                        if menu::item(ui, &format!("{icon}   {label}"), "") {
                            action = Some(kind);
                        }
                    }
                }

                if *directory {
                    for (label, expanded, recursive) in [
                        ("Expand Folder", true, false),
                        ("Expand Hierarchy", true, true),
                        ("Collapse Hierarchy", false, true),
                    ] {
                        if menu::item(ui, label, "") {
                            self.expansion = Some((target.clone(), expanded, recursive));
                        }
                    }
                } else if Action::Open.menu(ui) {
                    action = Some(Action::Open);
                }

                widgets::separator(ui);

                for kind in [Action::CopyPath, Action::CopyAbsolutePath] {
                    if kind.menu(ui) {
                        action = Some(kind);
                    }
                }

                {
                    let _disabled = ui.begin_disabled_with_cond(
                        target.as_os_str().is_empty()
                            || Path::new(crate::scripting::MANIFEST).starts_with(target),
                    );

                    for kind in [Action::Rename, Action::Duplicate, Action::Delete] {
                        if kind.menu(ui) {
                            action = Some(kind);
                        }
                    }
                }

                widgets::separator(ui);

                if Action::Terminal.menu(ui) {
                    action = Some(Action::Terminal);
                }
            }
        }

        if let Some(action) = action {
            let (target, directory) = self.context.as_ref().unwrap();
            let immediate = match action {
                Action::Open => {
                    self.open_requested = Some(target.clone());

                    true
                }
                Action::CopyPath | Action::CopyAbsolutePath => {
                    self.copy_path = Some(if matches!(action, Action::CopyAbsolutePath) {
                        documents
                            .context
                            .root()
                            .join(target)
                            .to_string_lossy()
                            .into_owned()
                    } else if target.as_os_str().is_empty() {
                        ".".into()
                    } else {
                        target.to_string_lossy().into_owned()
                    });

                    documents.status = "Path copied.".into();

                    true
                }
                Action::Terminal => {
                    let path = documents.context.root().join(target);
                    let folder = if *directory {
                        path.as_path()
                    } else {
                        path.parent().unwrap()
                    };

                    match open_terminal(folder) {
                        Ok(result) => {
                            self.terminal_result = Some(result);

                            documents.status = "Terminal opened.".into();
                        }
                        Err(message) => *error = Some(message),
                    }

                    true
                }
                _ => false,
            };

            if immediate {
                return focus | self.dialog(documents, ui, error);
            }

            if matches!(action, Action::Rename | Action::Duplicate | Action::Delete)
                && (target.as_os_str().is_empty()
                    || Path::new(crate::scripting::MANIFEST).starts_with(target))
            {
                return focus;
            }

            let target = if !directory
                && matches!(
                    action,
                    Action::Folder | Action::Scene | Action::Script | Action::File
                ) {
                target.parent().unwrap_or(Path::new("")).to_owned()
            } else {
                target.clone()
            };
            let name = if matches!(action, Action::Rename) {
                target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };
            let targets = if matches!(action, Action::Delete) {
                let targets = if self.selected.is_empty() {
                    vec![target.clone()]
                } else {
                    self.selection_roots()
                };

                if targets.iter().any(|path| protected(path)) {
                    return focus;
                }

                targets
            } else {
                Vec::new()
            };

            self.dialog = Some(Dialog {
                action,
                target,
                name,
                template: SceneTemplate::None,
                opening: true,
                targets,
            });
        }

        focus |= self.dialog(documents, ui, error);

        focus
    }

    fn select(&mut self, path: &Path, range: bool) {
        self.pending_click = None;

        if range {
            if let Some((start, end)) = self.anchor.as_ref().and_then(|anchor| {
                Some((
                    self.visible.iter().position(|entry| entry == anchor)?,
                    self.visible.iter().position(|entry| entry == path)?,
                ))
            }) {
                self.selected = self.visible[start.min(end)..=start.max(end)]
                    .iter()
                    .cloned()
                    .collect();

                return;
            }
        }

        self.selected = [path.to_owned()].into();
        self.anchor = Some(path.to_owned());
    }

    fn selection_roots(&self) -> Vec<PathBuf> {
        self.selected
            .iter()
            .filter(|path| {
                !self
                    .selected
                    .iter()
                    .any(|parent| parent != *path && path.starts_with(parent))
            })
            .cloned()
            .collect()
    }

    fn row(
        &mut self,
        documents: &mut Documents,
        ui: &Ui,
        entry: &FileEntry,
        name: &str,
        children: &BTreeMap<&Path, Vec<&FileEntry>>,
        visible: &mut Vec<PathBuf>,
        focus: &mut bool,
        context_clicked: &mut bool,
        error: &mut Option<String>,
    ) {
        let path = &entry.path;

        visible.push(path.clone());

        let _id = ui.push_id(path.to_string_lossy().as_ref());
        let position = ui.cursor_screen_pos();
        let forced = self.pending_expansion.remove(path);
        let _header = ui.push_style_color(StyleColor::Header, [0.0; 4]);
        let _text = ui.push_style_color(StyleColor::Text, [0.0; 4]);
        let node = ui
            .tree_node_config("##Entry")
            .selected(self.selected.contains(path))
            .leaf(!entry.directory)
            .span_full_width(true)
            .framed(true)
            .frame_padding(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .opened(
                forced.unwrap_or(path.as_os_str().is_empty()),
                if forced.is_some() {
                    Condition::Always
                } else {
                    Condition::Once
                },
            )
            .push();
        drop(_text);
        let clicked = ui.is_item_clicked();
        let confirmed = ui.is_item_focused() && inputs::active(ui, InputAction::Confirm);
        let hovered = ui.is_item_hovered();
        let open = confirmed
            || (hovered && ui.is_mouse_double_clicked(MouseButton::Left) && !ui.io().key_shift());

        if hovered && ui.is_mouse_clicked(MouseButton::Right) {
            if !self.selected.contains(path) {
                self.select(path, false);
            }

            self.context = Some((path.clone(), entry.directory));

            *context_clicked = true;
        }

        if clicked {
            if ui.io().key_shift() || !self.selected.contains(path) || self.selected.len() <= 1 {
                self.select(path, ui.io().key_shift());
            } else {
                self.pending_click = Some(path.clone());
            }
        }

        if hovered
            && ui.is_mouse_released(MouseButton::Left)
            && self.pending_click.as_ref() == Some(path)
        {
            self.select(path, false);

            self.pending_click = None;
        }

        if entry.directory {
            if let Some(target) = ui.drag_drop_target() {
                for payload in ["KINEMATIC_FILES", "KINEMATIC_AUDIO"] {
                    if target
                        .accept_payload_empty(payload, dear_imgui_rs::DragDropTargetFlags::NONE)
                        .is_some_and(|payload| payload.delivery)
                    {
                        self.drop_folder = Some(path.clone());
                    }
                }
            }
        }

        if !protected(path) && !self.selected.iter().any(|path| protected(path)) {
            let audio = self.selected.len() <= 1
                && !entry.directory
                && crate::ui::timeline::is_audio_file(path);

            if let Some(_source) = ui
                .drag_drop_source_config(if audio {
                    "KINEMATIC_AUDIO"
                } else {
                    "KINEMATIC_FILES"
                })
                .begin()
            {
                self.pending_click = None;
                self.dragged = self.selection_roots();

                if audio {
                    self.dragged_audio = Some(path.clone());
                }

                for path in &self.dragged {
                    widgets::text(ui, path.to_string_lossy());
                }
            }
        }

        let selected = self.selected.contains(path);
        kinematic_ui::lists::file_label(ui, position, name, entry.directory, selected);

        if entry.directory {
            let icon = if node.is_some() {
                icons::CHEVRON_DOWN
            } else {
                icons::CHEVRON_RIGHT
            };
            let size = ui.calc_text_size(icon);

            ui.get_window_draw_list().add_text(
                [
                    position[0] + (ui.tree_node_to_label_spacing() - size[0]) * 0.5,
                    position[1] + (ui.frame_height() - size[1]) * 0.5,
                ],
                ui.get_color_u32(StyleColor::TextDisabled),
                icon,
            );
        }

        if open
            && !entry.directory
            && self.selected.len() <= 1
            && !crate::ui::timeline::is_audio_file(path)
        {
            self.open_requested = Some(path.to_owned());
        }

        if let Some(_node) = node {
            if entry.directory {
                if let Some(entries) = children.get(path.as_path()) {
                    for child in entries {
                        let child_position = ui.cursor_screen_pos();

                        if self.selection_related(&child.path) {
                            kinematic_ui::lists::file_branch(
                                ui,
                                position,
                                child_position,
                                child.directory,
                            );
                        }

                        let name = child.path.file_name().unwrap_or_default().to_string_lossy();

                        self.row(
                            documents,
                            ui,
                            child,
                            &name,
                            children,
                            visible,
                            focus,
                            context_clicked,
                            error,
                        );
                    }
                }
            }
        }
    }

    fn selection_related(&self, path: &Path) -> bool {
        self.selected.iter().map(PathBuf::as_path).any(|selected| {
            selected == path || selected.starts_with(path) || path.starts_with(selected)
        })
    }

    fn dialog(&mut self, documents: &mut Documents, ui: &Ui, error: &mut Option<String>) -> bool {
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

            widgets::message(
                ui,
                if dialog.targets.len() > 1 {
                    format!("{} selected items.", dialog.targets.len())
                } else if dialog.target.as_os_str().is_empty() {
                    "Project root".into()
                } else {
                    dialog.target.to_string_lossy().into_owned()
                },
            );
            widgets::separator(ui);

            let deleting = matches!(dialog.action, Action::Delete);
            let mut submit = false;
            let validation = if deleting {
                widgets::message(
                    ui,
                    "Permanently delete this item and all its contents? Scene entries will be removed from the manifest.",
                );

                None
            } else {
                submit = kinematic_ui::properties::row(ui, "Name", false, || {
                    if ui.is_window_appearing() {
                        ui.set_keyboard_focus_here();
                    }

                    controls::text_submit(ui, "##Name", &mut dialog.name)
                })
                .changed;

                submit |= inputs::active(ui, InputAction::Confirm);

                if matches!(dialog.action, Action::Scene) {
                    kinematic_ui::properties::row(ui, "Template", false, || {
                        let mut index = match dialog.template {
                            SceneTemplate::None => 0,
                            SceneTemplate::TwoD => 1,
                            SceneTemplate::ThreeD => 2,
                        };
                        let changed = controls::choice(
                            ui,
                            "##Template",
                            &mut index,
                            &["No template", "2D template", "3D template"],
                        );

                        dialog.template = [
                            SceneTemplate::None,
                            SceneTemplate::TwoD,
                            SceneTemplate::ThreeD,
                        ][index];

                        changed
                    });

                    widgets::message(
                        ui,
                        "The .lua extension is added automatically. The scene is linked in the manifest.",
                    );
                }

                if matches!(dialog.action, Action::Rename) {
                    widgets::message(
                        ui,
                        "Scene paths are updated. Lua imports are not rewritten.",
                    );
                }

                dialog_name(&dialog.name, dialog.action).err()
            };

            dialog.opening = false;

            if deleting {
                submit |= inputs::active(ui, InputAction::Confirm);
            }

            if !dialog.name.is_empty() {
                if let Some(message) = &validation {
                    widgets::message(ui, message);
                }
            }

            let label = match dialog.action {
                Action::Delete => "Delete",
                Action::Rename => "Rename",
                Action::Duplicate => "Duplicate",
                _ => "Create",
            };
            let action =
                crate::ui::windows::actions(ui, &[(label, validation.is_none()), ("Cancel", true)]);

            submit |= action == Some(0);

            if action == Some(1) || inputs::active(ui, InputAction::Cancel) {
                finished = true;

                ui.close_current_popup();
            } else if submit && validation.is_none() {
                let result = (|| {
                    let path = if deleting {
                        dialog.target.clone()
                    } else {
                        let name = dialog_name(&dialog.name, dialog.action)?;
                        let parent = if matches!(dialog.action, Action::Rename | Action::Duplicate)
                        {
                            dialog.target.parent().unwrap_or(Path::new(""))
                        } else {
                            &dialog.target
                        };

                        parent.join(name)
                    };

                    match dialog.action {
                        Action::Folder => documents.create(&path, true)?,
                        Action::File | Action::Script => documents.create(&path, false)?,
                        Action::Duplicate => documents.duplicate(&dialog.target, &path)?,
                        Action::Scene => documents.create_scene(&path, dialog.template)?,
                        Action::Rename => documents.rename(&dialog.target, &path)?,
                        Action::Delete => documents.delete_files(&dialog.targets)?,
                        _ => unreachable!("Only file dialogs reach submission."),
                    }

                    self.selected = if deleting {
                        BTreeSet::new()
                    } else {
                        [path].into()
                    };
                    self.anchor = self.selected.first().cloned();

                    Ok::<_, String>(())
                })();

                match result {
                    Ok(()) => {
                        focus =
                            matches!(dialog.action, Action::File | Action::Scene | Action::Script);

                        documents.status = format!("{} completed.", dialog.action.title());

                        finished = true;

                        ui.close_current_popup();
                    }

                    Err(message) => *error = Some(message),
                }
            }
        }

        if finished || !open {
            self.dialog = None;
        }

        focus
    }
}

fn protected(path: &Path) -> bool {
    path.as_os_str().is_empty() || Path::new(crate::scripting::MANIFEST).starts_with(path)
}

fn dialog_name(name: &str, action: Action) -> Result<String, String> {
    filename(name, matches!(action, Action::Scene | Action::Script))
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
    fn selection_guides_follow_the_selected_path() {
        let mut state = State::default();
        let selected = Path::new("scenes/intro.lua");

        state.selected.insert(selected.into());

        assert!(state.selection_related(Path::new("")));
        assert!(state.selection_related(Path::new("scenes")));
        assert!(state.selection_related(selected));
        assert!(!state.selection_related(Path::new("assets")));

        state.selected.clear();

        assert!(!state.selection_related(Path::new("scenes")));
    }

    #[test]
    fn double_click_shift_selection_drag_and_batch_delete_work_together() {
        fn frame(
            context: &mut dear_imgui_rs::Context,
            font: dear_imgui_rs::FontId,
            state: &mut State,
            documents: &mut Documents,
            error: &mut Option<String>,
        ) -> BTreeMap<PathBuf, [f32; 2]> {
            let ui = context.frame();
            let _font = ui.push_font(font);
            let mut origin = [0.0; 2];
            let height = ui.frame_height_with_spacing();

            ui.window("Files")
                .position([20.0, 20.0], Condition::Always)
                .size([340.0, 600.0], Condition::Always)
                .build(|| {
                    origin = ui.cursor_screen_pos();

                    state.draw(documents, ui, error);
                });

            let positions = state
                .visible
                .iter()
                .enumerate()
                .map(|(index, path)| {
                    (
                        path.clone(),
                        [
                            origin[0] + 160.0,
                            origin[1] + index as f32 * height + height * 0.4,
                        ],
                    )
                })
                .collect();

            drop(_font);

            context.end_frame();

            positions
        }

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

        let root =
            std::env::temp_dir().join(format!("kinematic-files-drag-{}", std::process::id()));
        let project = crate::scripting::LuaProject::create(&root, "Drag", false).unwrap();

        std::fs::write(root.join("a.txt"), "A").unwrap();
        std::fs::write(root.join("b.txt"), "B").unwrap();
        std::fs::create_dir(root.join("destination")).unwrap();

        let mut documents = Documents::new(project.context, "").unwrap();

        let mut state = State::default();
        let mut error = None;

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        let positions = frame(&mut context, font, &mut state, &mut documents, &mut error);
        let a = Path::new("a.txt");
        let b = Path::new("b.txt");

        context.io_mut().add_mouse_pos_event(positions[a]);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        for down in [true, false] {
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, down);

            frame(&mut context, font, &mut state, &mut documents, &mut error);
        }

        assert!(
            documents.active.is_none(),
            "A single click must only select."
        );
        assert_eq!(state.selected, [a.to_owned()].into());

        for down in [true, false] {
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, down);

            frame(&mut context, font, &mut state, &mut documents, &mut error);
        }

        assert_eq!(state.open_requested.take().as_deref(), Some(a));
        assert!(documents.active.is_none());

        documents.open(a).unwrap();

        context.io_mut().add_key_event(Key::ModShift, true);
        context.io_mut().add_mouse_pos_event(positions[b]);

        for down in [true, false] {
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, down);

            frame(&mut context, font, &mut state, &mut documents, &mut error);
        }

        context.io_mut().add_key_event(Key::ModShift, false);

        assert_eq!(state.selected, [a.to_owned(), b.to_owned()].into());

        context.io_mut().add_mouse_pos_event(positions[a]);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, true);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        context
            .io_mut()
            .add_mouse_pos_event([positions[a][0] + 16.0, positions[a][1]]);

        frame(&mut context, font, &mut state, &mut documents, &mut error);
        frame(&mut context, font, &mut state, &mut documents, &mut error);

        assert_eq!(
            state.dragged,
            vec![a.to_owned(), b.to_owned()],
            "Selected: {:?}, pending: {:?}, shift: {}",
            state.selected,
            state.pending_click,
            context.io().key_shift()
        );

        context
            .io_mut()
            .add_mouse_pos_event(positions[Path::new("destination")]);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, false);

        frame(&mut context, font, &mut state, &mut documents, &mut error);
        frame(&mut context, font, &mut state, &mut documents, &mut error);

        assert!(error.is_none(), "{error:?}");
        assert!(root.join("destination/a.txt").exists());
        assert!(root.join("destination/b.txt").exists());
        assert_eq!(
            documents.active.as_deref(),
            Some(Path::new("destination/a.txt"))
        );

        context.io_mut().add_key_event(Key::Delete, true);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        assert_eq!(state.dialog.as_ref().unwrap().targets.len(), 2);

        context.io_mut().add_key_event(Key::Delete, false);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        context.io_mut().add_key_event(Key::Enter, true);

        frame(&mut context, font, &mut state, &mut documents, &mut error);

        assert!(error.is_none(), "{error:?}");
        assert!(!root.join("destination/a.txt").exists());
        assert!(!root.join("destination/b.txt").exists());

        drop(documents);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generic_files_keep_their_extensions_and_reject_paths() {
        assert_eq!(dialog_name("notes.txt", Action::File).unwrap(), "notes.txt");
        assert_eq!(dialog_name("module", Action::Script).unwrap(), "module.lua");
        assert_eq!(
            dialog_name("module.lua", Action::Script).unwrap(),
            "module.lua"
        );
        assert!(dialog_name("module.txt", Action::Script).is_err());
        assert_eq!(
            dialog_name("module.lua", Action::File).unwrap(),
            "module.lua"
        );
        assert!(dialog_name("../outside", Action::File).is_err());
        assert!(dialog_name("/outside", Action::File).is_err());
    }

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

                assert!(ui.is_popup_open("File context"));
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

        state.select(Path::new("scenes/intro.lua"), false);

        for down in [true, false] {
            for key in [Key::ModCtrl, Key::ModShift, Key::C] {
                context.io_mut().add_key_event(key, down);
            }

            let ui = context.frame();
            let _font = ui.push_font(font);

            ui.window("Files").build(|| {
                ui.set_window_focus(Some("Files"));

                state.draw(&mut documents, ui, &mut error);
            });

            drop(_font);

            context.end_frame();
        }

        assert_eq!(state.copy_path.as_deref(), Some("scenes/intro.lua"));
        assert_eq!(documents.status, "Path copied.");

        let (sender, receiver) = std::sync::mpsc::channel();

        sender.send("Terminal failed.".into()).unwrap();

        state.terminal_result = Some(receiver);

        {
            let ui = context.frame();
            let _font = ui.push_font(font);

            ui.window("Files").build(|| {
                state.draw(&mut documents, ui, &mut error);
            });

            drop(_font);

            context.end_frame();
        }

        assert_eq!(error.take().as_deref(), Some("Terminal failed."));

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
                opening: true,
                targets: Vec::new(),
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
