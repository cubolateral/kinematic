//! Action bindings shared by editor panels and shortcut settings.

use dear_imgui_rs::{Key, KeyMods, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Category {
    Editor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    FileCopyPath,
    FileCopyAbsolutePath,
    FileRename,
    FileDuplicate,
    FileDelete,
    FileTerminal,
    ToggleFullscreen,
    TogglePanels,
    ToggleMargin,
    Preview,
    Edit2d,
    Edit3d,
    ToggleCamera,
    ResetCamera,
    PlayPause,
    SplitAudio,
    DeleteAudio,
    ToggleLoop,
    PreviousFrame,
    NextFrame,
    GoToStart,
    GoToEnd,
    MoveRight,
    MoveLeft,
    MoveUp,
    MoveDown,
    MoveForward,
    MoveBackward,
    CameraFast,
    CameraSlow,
    FocusSelection,
    FrameAll,
    ViewFront,
    ViewBack,
    ViewRight,
    ViewLeft,
    ViewTop,
    ViewBottom,
    ToggleProjection,
    OrbitLeft,
    OrbitRight,
    OrbitUp,
    OrbitDown,
    ViewOpposite,
    IncreaseValue,
    DecreaseValue,
    Confirm,
    Cancel,
    SaveProject,
    Quit,
    ProjectList,
}

#[derive(Clone, Copy)]
enum Trigger {
    Once,
    Repeat,
    Down,
}

pub(super) struct Binding {
    pub action: Action,
    pub category: Category,
    pub name: &'static str,
    pub display: String,
    keys: Vec<Key>,
    required_modifiers: KeyMods,
    allowed_modifiers: KeyMods,
    trigger: Trigger,
}

impl Binding {
    fn new(
        action: Action,
        category: Category,
        name: &'static str,
        display: &'static str,
        keys: &[Key],
    ) -> Self {
        Self {
            action,
            category,
            name: name.trim_end_matches('.'),
            display: display.to_owned(),
            keys: keys.to_vec(),
            required_modifiers: KeyMods::empty(),
            allowed_modifiers: KeyMods::empty(),
            trigger: Trigger::Once,
        }
    }

    fn modifiers(mut self, required: KeyMods, allowed: KeyMods) -> Self {
        self.required_modifiers = required;
        self.allowed_modifiers = allowed;

        self
    }

    fn trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;

        self
    }
}

pub(super) struct InputMap {
    pub bindings: Vec<Binding>,
}

impl Default for InputMap {
    fn default() -> Self {
        use crate::ui::inputs::{Action::*, Category::*, Trigger::*};

        let mut map = Self {
            bindings: vec![
                Binding::new(
                    ToggleFullscreen,
                    Editor,
                    "Toggle fullscreen",
                    "F",
                    &[Key::F],
                ),
                Binding::new(TogglePanels, Editor, "Toggle sidebars", "Tab", &[Key::Tab]),
                Binding::new(ToggleMargin, Editor, "Toggle margins", "M", &[Key::M]),
                Binding::new(Preview, Editor, "Preview mode", "F1", &[Key::F1]),
                Binding::new(Edit2d, Editor, "2D editor mode", "F2", &[Key::F2]),
                Binding::new(Edit3d, Editor, "3D editor mode", "F3", &[Key::F3]),
                Binding::new(
                    ToggleCamera,
                    Editor,
                    "Toggle camera view",
                    "0 / Keypad 0",
                    &[Key::Key0, Key::Keypad0],
                )
                .trigger(Repeat),
                Binding::new(
                    ResetCamera,
                    Editor,
                    "Reset camera transform",
                    "R",
                    &[Key::R],
                )
                .trigger(Repeat),
                Binding::new(
                    SplitAudio,
                    Editor,
                    "Split audio at playhead",
                    "Ctrl + B",
                    &[Key::B],
                )
                .modifiers(KeyMods::CTRL, KeyMods::CTRL),
                Binding::new(
                    DeleteAudio,
                    Editor,
                    "Delete audio clip",
                    "Delete / Backspace",
                    &[Key::Delete, Key::Backspace],
                ),
                Binding::new(PlayPause, Editor, "Play / pause", "Space", &[Key::Space])
                    .modifiers(KeyMods::empty(), KeyMods::SHIFT)
                    .trigger(Repeat),
                Binding::new(ToggleLoop, Editor, "Toggle loop", "L", &[Key::L]),
                Binding::new(
                    PreviousFrame,
                    Editor,
                    "Previous frame",
                    "Left Arrow",
                    &[Key::LeftArrow],
                )
                .trigger(Repeat),
                Binding::new(
                    NextFrame,
                    Editor,
                    "Next frame",
                    "Right Arrow",
                    &[Key::RightArrow],
                )
                .trigger(Repeat),
                Binding::new(
                    GoToStart,
                    Editor,
                    "Go to start",
                    "Shift + Left Arrow",
                    &[Key::LeftArrow],
                )
                .modifiers(KeyMods::SHIFT, KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    GoToEnd,
                    Editor,
                    "Go to end",
                    "Shift + Right Arrow",
                    &[Key::RightArrow],
                )
                .modifiers(KeyMods::SHIFT, KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(MoveRight, Editor, "Move camera right", "D", &[Key::D])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Down),
                Binding::new(MoveLeft, Editor, "Move camera left", "A", &[Key::A])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Down),
                Binding::new(MoveUp, Editor, "Move camera up", "E", &[Key::E])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Down),
                Binding::new(MoveDown, Editor, "Move camera down", "Q", &[Key::Q])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Down),
                Binding::new(MoveForward, Editor, "Move camera forward", "W", &[Key::W])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Down),
                Binding::new(MoveBackward, Editor, "Move camera backward", "S", &[Key::S])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Down),
                Binding::new(
                    CameraFast,
                    Editor,
                    "Faster camera movement",
                    "Shift",
                    &[Key::ModShift],
                )
                .modifiers(KeyMods::empty(), KeyMods::all())
                .trigger(Down),
                Binding::new(
                    IncreaseValue,
                    Editor,
                    "Increase numeric value",
                    "Up Arrow",
                    &[Key::UpArrow],
                )
                .modifiers(KeyMods::empty(), KeyMods::all())
                .trigger(Repeat),
                Binding::new(
                    DecreaseValue,
                    Editor,
                    "Decrease numeric value",
                    "Down Arrow",
                    &[Key::DownArrow],
                )
                .modifiers(KeyMods::empty(), KeyMods::all())
                .trigger(Repeat),
                Binding::new(
                    Confirm,
                    Editor,
                    "Confirm / open",
                    "Enter / Keypad Enter",
                    &[Key::Enter, Key::KeypadEnter],
                )
                .modifiers(KeyMods::empty(), KeyMods::all())
                .trigger(Repeat),
                Binding::new(Cancel, Editor, "Cancel dialog", "Escape", &[Key::Escape])
                    .modifiers(KeyMods::empty(), KeyMods::all())
                    .trigger(Repeat),
                Binding::new(Quit, Editor, "Quit", "Ctrl + Q", &[Key::Q])
                    .modifiers(KeyMods::CTRL, KeyMods::CTRL),
                Binding::new(
                    ProjectList,
                    Editor,
                    "Back to projects",
                    "Ctrl + Shift + Q",
                    &[Key::Q],
                )
                .modifiers(
                    KeyMods::CTRL | KeyMods::SHIFT,
                    KeyMods::CTRL | KeyMods::SHIFT,
                ),
                Binding::new(SaveProject, Editor, "Save project", "Ctrl + S", &[Key::S])
                    .modifiers(KeyMods::CTRL, KeyMods::CTRL),
            ],
        };

        for (action, name, display, key, modifiers) in [
            (
                FocusSelection,
                "Focus selection.",
                "Ctrl + Space",
                Key::Space,
                KeyMods::CTRL,
            ),
            (
                FocusSelection,
                "Focus selection.",
                "Keypad .",
                Key::KeypadDecimal,
                KeyMods::empty(),
            ),
            (FrameAll, "Frame all.", "Home", Key::Home, KeyMods::empty()),
            (
                ViewFront,
                "Front view.",
                "Keypad 1",
                Key::Keypad1,
                KeyMods::empty(),
            ),
            (
                ViewBack,
                "Back view.",
                "Ctrl + Keypad 1",
                Key::Keypad1,
                KeyMods::CTRL,
            ),
            (
                ViewRight,
                "Right view.",
                "Keypad 3",
                Key::Keypad3,
                KeyMods::empty(),
            ),
            (
                ViewLeft,
                "Left view.",
                "Ctrl + Keypad 3",
                Key::Keypad3,
                KeyMods::CTRL,
            ),
            (
                ViewTop,
                "Top view.",
                "Keypad 7",
                Key::Keypad7,
                KeyMods::empty(),
            ),
            (
                ViewBottom,
                "Bottom view.",
                "Ctrl + Keypad 7",
                Key::Keypad7,
                KeyMods::CTRL,
            ),
            (
                ToggleProjection,
                "Toggle perspective / orthographic.",
                "Keypad 5",
                Key::Keypad5,
                KeyMods::empty(),
            ),
            (
                OrbitLeft,
                "Orbit left.",
                "Keypad 4",
                Key::Keypad4,
                KeyMods::empty(),
            ),
            (
                OrbitRight,
                "Orbit right.",
                "Keypad 6",
                Key::Keypad6,
                KeyMods::empty(),
            ),
            (
                OrbitUp,
                "Orbit up.",
                "Keypad 8",
                Key::Keypad8,
                KeyMods::empty(),
            ),
            (
                OrbitDown,
                "Orbit down.",
                "Keypad 2",
                Key::Keypad2,
                KeyMods::empty(),
            ),
            (
                ViewOpposite,
                "Opposite view.",
                "Keypad 9",
                Key::Keypad9,
                KeyMods::empty(),
            ),
        ] {
            let number = match key {
                Key::Keypad1 => Some(Key::Key1),
                Key::Keypad2 => Some(Key::Key2),
                Key::Keypad3 => Some(Key::Key3),
                Key::Keypad4 => Some(Key::Key4),
                Key::Keypad5 => Some(Key::Key5),
                Key::Keypad6 => Some(Key::Key6),
                Key::Keypad7 => Some(Key::Key7),
                Key::Keypad8 => Some(Key::Key8),
                Key::Keypad9 => Some(Key::Key9),
                _ => None,
            };
            let mut binding =
                Binding::new(action, Editor, name.trim_end_matches('.'), display, &[key])
                    .modifiers(modifiers, modifiers);

            if let Some(number) = number {
                binding.keys.push(number);

                binding.display = format!("{} / {display}", display.replace("Keypad ", ""));
            }

            map.bindings.push(binding);
        }

        map.bindings.push(
            Binding::new(
                CameraSlow,
                Editor,
                "Precise camera movement.",
                "Alt",
                &[Key::ModAlt],
            )
            .modifiers(KeyMods::empty(), KeyMods::all())
            .trigger(Down),
        );

        for (action, name, display, key, modifiers) in [
            (
                FileCopyPath,
                "Copy file path",
                "Ctrl + Shift + C",
                Key::C,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                FileCopyAbsolutePath,
                "Copy absolute file path",
                "Ctrl + Alt + C",
                Key::C,
                KeyMods::CTRL | KeyMods::ALT,
            ),
            (FileRename, "Rename file", "F2", Key::F2, KeyMods::empty()),
            (
                FileDuplicate,
                "Duplicate file",
                "Ctrl + D",
                Key::D,
                KeyMods::CTRL,
            ),
            (
                FileDelete,
                "Delete file",
                "Delete",
                Key::Delete,
                KeyMods::empty(),
            ),
            (
                FileTerminal,
                "Open file folder in terminal",
                "Ctrl + Alt + T",
                Key::T,
                KeyMods::CTRL | KeyMods::ALT,
            ),
        ] {
            map.bindings.push(
                Binding::new(action, Editor, name, display, &[key]).modifiers(modifiers, modifiers),
            );
        }

        map
    }
}

impl InputMap {
    pub fn binding(&self, action: Action) -> &Binding {
        self.bindings
            .iter()
            .find(|binding| binding.action == action)
            .expect("Action must have a binding.")
    }

    pub fn active(&self, ui: &Ui, action: Action) -> bool {
        let io = ui.io();
        let mut modifiers = KeyMods::empty();

        for (held, modifier) in [
            (io.key_ctrl(), KeyMods::CTRL),
            (io.key_shift(), KeyMods::SHIFT),
            (io.key_alt(), KeyMods::ALT),
            (io.key_super(), KeyMods::SUPER),
        ] {
            modifiers.set(modifier, held);
        }

        self.bindings
            .iter()
            .filter(|binding| binding.action == action)
            .any(|binding| {
                if !modifiers.contains(binding.required_modifiers)
                    || !binding.allowed_modifiers.contains(modifiers)
                {
                    return false;
                }

                binding.keys.iter().any(|&key| match binding.trigger {
                    Trigger::Once => ui.is_key_pressed_with_repeat(key, false),
                    Trigger::Repeat => ui.is_key_pressed_with_repeat(key, true),
                    Trigger::Down => ui.is_key_down(key),
                })
            })
    }
}

pub(super) fn map() -> &'static InputMap {
    static INPUTS: std::sync::OnceLock<InputMap> = std::sync::OnceLock::new();

    INPUTS.get_or_init(InputMap::default)
}

pub(super) fn active(ui: &Ui, action: Action) -> bool {
    map().active(ui, action)
}

pub(super) fn tooltip(ui: &Ui, action: Action, text: &str) {
    kinematic_ui::shortcut::tooltip(ui, text, &map().binding(action).display);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_match_modifiers_alternatives_and_runtime_remapping() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();

        for (action, key, modifiers, expected) in [
            (Action::TogglePanels, Key::Tab, KeyMods::empty(), true),
            (Action::TogglePanels, Key::Tab, KeyMods::SHIFT, false),
            (Action::FileRename, Key::F2, KeyMods::empty(), true),
            (Action::FileDuplicate, Key::D, KeyMods::CTRL, true),
            (Action::FileDelete, Key::Delete, KeyMods::empty(), true),
            (
                Action::FileCopyPath,
                Key::C,
                KeyMods::CTRL | KeyMods::SHIFT,
                true,
            ),
            (
                Action::FileCopyAbsolutePath,
                Key::C,
                KeyMods::CTRL | KeyMods::ALT,
                true,
            ),
            (
                Action::FileTerminal,
                Key::T,
                KeyMods::CTRL | KeyMods::ALT,
                true,
            ),
            (Action::FileCopyPath, Key::C, KeyMods::CTRL, false),
            (Action::Preview, Key::F1, KeyMods::empty(), true),
            (Action::Edit2d, Key::F2, KeyMods::empty(), true),
            (Action::Edit3d, Key::F3, KeyMods::empty(), true),
            (Action::Preview, Key::Keypad1, KeyMods::empty(), false),
            (Action::ViewFront, Key::Keypad1, KeyMods::empty(), true),
            (Action::ViewFront, Key::Key1, KeyMods::empty(), true),
            (Action::ViewBack, Key::Key1, KeyMods::CTRL, true),
            (Action::ToggleProjection, Key::Key5, KeyMods::empty(), true),
            (Action::OrbitDown, Key::Key2, KeyMods::empty(), true),
            (Action::Edit2d, Key::Key2, KeyMods::empty(), false),
            (Action::ViewFront, Key::Keypad1, KeyMods::CTRL, false),
            (Action::ViewBack, Key::Keypad1, KeyMods::CTRL, true),
            (Action::FocusSelection, Key::Space, KeyMods::CTRL, true),
            (
                Action::FocusSelection,
                Key::KeypadDecimal,
                KeyMods::empty(),
                true,
            ),
            (Action::PlayPause, Key::Space, KeyMods::CTRL, false),
            (Action::ToggleCamera, Key::Keypad0, KeyMods::empty(), true),
            (Action::ToggleCamera, Key::Keypad0, KeyMods::CTRL, false),
            (Action::GoToStart, Key::LeftArrow, KeyMods::SHIFT, true),
            (Action::PreviousFrame, Key::LeftArrow, KeyMods::SHIFT, false),
            (Action::SaveProject, Key::S, KeyMods::CTRL, true),
            (Action::SaveProject, Key::S, KeyMods::empty(), false),
            (Action::Quit, Key::Q, KeyMods::CTRL, true),
            (Action::Quit, Key::Q, KeyMods::CTRL | KeyMods::SHIFT, false),
            (
                Action::ProjectList,
                Key::Q,
                KeyMods::CTRL | KeyMods::SHIFT,
                true,
            ),
            (Action::ProjectList, Key::Q, KeyMods::CTRL, false),
            (Action::CameraFast, Key::ModShift, KeyMods::SHIFT, true),
            (Action::ToggleMargin, Key::N, KeyMods::empty(), true),
            (Action::ToggleMargin, Key::M, KeyMods::empty(), false),
        ] {
            let mut inputs = InputMap::default();
            let binding = inputs
                .bindings
                .iter_mut()
                .find(|binding| binding.action == Action::ToggleMargin)
                .unwrap();

            binding.keys = vec![Key::N];
            binding.display = "N".into();

            let mut context = dear_imgui_rs::Context::create();

            context.set_ini_filename(None::<&str>).unwrap();
            context
                .font_atlas()
                .try_claim_legacy_renderer()
                .unwrap()
                .build();
            context.io_mut().set_display_size([640.0, 480.0]);
            context.io_mut().set_delta_time(1.0 / 60.0);

            for (modifier, key) in [
                (KeyMods::CTRL, Key::ModCtrl),
                (KeyMods::SHIFT, Key::ModShift),
                (KeyMods::ALT, Key::ModAlt),
            ] {
                context
                    .io_mut()
                    .add_key_event(key, modifiers.contains(modifier));
            }

            context.io_mut().add_key_event(key, true);

            assert_eq!(
                inputs.active(context.frame(), action),
                expected,
                "{action:?}"
            );

            context.end_frame();
        }
    }

    #[test]
    fn held_and_repeated_actions_keep_their_trigger_behavior() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        for key in [Key::F, Key::D, Key::LeftArrow] {
            context.io_mut().add_key_event(key, true);
        }

        let inputs = InputMap::default();
        let mut repeated = false;

        for frame in 0..60 {
            let ui = context.frame();

            assert_eq!(inputs.active(ui, Action::ToggleFullscreen), frame == 0);
            assert!(inputs.active(ui, Action::MoveRight));

            if frame > 0 && inputs.active(ui, Action::PreviousFrame) {
                repeated = true;
            }

            context.end_frame();
        }

        assert!(repeated);
    }
}
