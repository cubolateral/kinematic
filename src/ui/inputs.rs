//! Action bindings shared by editor panels and shortcut settings.

use dear_imgui_rs::{Key, KeyChord, KeyMods, MouseButton, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Category {
    Editor,
    CodeEditor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    ToggleFullscreen,
    ToggleMargin,
    Preview,
    Edit2d,
    Edit3d,
    ToggleCamera,
    ResetCamera,
    PlayPause,
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
    IncreaseValue,
    DecreaseValue,
    Confirm,
    Cancel,
    Find,
    Save,
    ZoomIn,
    ZoomOut,
    Definition,
    ClickDefinition,
    IndentBackspace,
    DismissAssistance,
    PreviousSuggestion,
    NextSuggestion,
    AcceptSuggestion,
    CancelSuggestion,
    SelectAll,
    Copy,
    RequestSuggestions,
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
    mouse: Option<MouseButton>,
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
            name,
            display: display.to_owned(),
            keys: keys.to_vec(),
            mouse: None,
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

    fn mouse(mut self, mouse: MouseButton) -> Self {
        self.mouse = Some(mouse);
        self
    }
}

pub(super) struct InputMap {
    pub bindings: Vec<Binding>,
}

impl Default for InputMap {
    fn default() -> Self {
        use crate::ui::inputs::{Action::*, Category::*, Trigger::*};

        let (suggestion_modifier, suggestion_display) = if cfg!(target_vendor = "apple") {
            (KeyMods::SUPER, "Super + Space")
        } else {
            (KeyMods::CTRL, "Ctrl + Space")
        };
        Self {
            bindings: vec![
                Binding::new(
                    RequestSuggestions,
                    CodeEditor,
                    "Request suggestions",
                    suggestion_display,
                    &[Key::Space],
                )
                .modifiers(suggestion_modifier, suggestion_modifier),
                Binding::new(
                    ToggleFullscreen,
                    Editor,
                    "Toggle fullscreen",
                    "F",
                    &[Key::F],
                ),
                Binding::new(ToggleMargin, Editor, "Toggle margins", "M", &[Key::M]),
                Binding::new(
                    Preview,
                    Editor,
                    "Preview mode",
                    "1 / Keypad 1",
                    &[Key::Key1, Key::Keypad1],
                ),
                Binding::new(
                    Edit2d,
                    Editor,
                    "2D editor mode",
                    "2 / Keypad 2",
                    &[Key::Key2, Key::Keypad2],
                ),
                Binding::new(
                    Edit3d,
                    Editor,
                    "3D editor mode",
                    "3 / Keypad 3",
                    &[Key::Key3, Key::Keypad3],
                ),
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
                Binding::new(Find, CodeEditor, "Find / replace", "Ctrl + F", &[Key::F])
                    .modifiers(KeyMods::CTRL, KeyMods::all()),
                Binding::new(Save, CodeEditor, "Save file", "Ctrl + S", &[Key::S])
                    .modifiers(KeyMods::CTRL, KeyMods::all()),
                Binding::new(
                    ZoomIn,
                    CodeEditor,
                    "Increase font size",
                    "Ctrl + = / Keypad +",
                    &[Key::Equal, Key::KeypadAdd],
                )
                .modifiers(KeyMods::CTRL, KeyMods::CTRL | KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    ZoomOut,
                    CodeEditor,
                    "Decrease font size",
                    "Ctrl + - / Keypad -",
                    &[Key::Minus, Key::KeypadSubtract],
                )
                .modifiers(KeyMods::CTRL, KeyMods::CTRL | KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    Definition,
                    CodeEditor,
                    "Go to definition",
                    "F12",
                    &[Key::F12],
                )
                .modifiers(KeyMods::empty(), KeyMods::all()),
                Binding::new(
                    ClickDefinition,
                    CodeEditor,
                    "Go to definition with mouse",
                    "Ctrl + Left Click",
                    &[],
                )
                .modifiers(KeyMods::CTRL, KeyMods::all())
                .mouse(MouseButton::Left),
                Binding::new(
                    IndentBackspace,
                    CodeEditor,
                    "Delete indentation",
                    "Backspace",
                    &[Key::Backspace],
                )
                .modifiers(KeyMods::empty(), KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    DismissAssistance,
                    CodeEditor,
                    "Dismiss suggestions / signature",
                    "Escape",
                    &[Key::Escape],
                )
                .modifiers(KeyMods::empty(), KeyMods::all()),
                Binding::new(
                    PreviousSuggestion,
                    CodeEditor,
                    "Previous suggestion",
                    "Up Arrow",
                    &[Key::UpArrow],
                )
                .modifiers(KeyMods::empty(), KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    NextSuggestion,
                    CodeEditor,
                    "Next suggestion",
                    "Down Arrow",
                    &[Key::DownArrow],
                )
                .modifiers(KeyMods::empty(), KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    AcceptSuggestion,
                    CodeEditor,
                    "Accept suggestion",
                    "Enter / Keypad Enter / Tab",
                    &[Key::Enter, Key::KeypadEnter, Key::Tab],
                )
                .modifiers(KeyMods::empty(), KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    CancelSuggestion,
                    CodeEditor,
                    "Cancel suggestions",
                    "Escape",
                    &[Key::Escape],
                )
                .modifiers(KeyMods::empty(), KeyMods::SHIFT)
                .trigger(Repeat),
                Binding::new(
                    SelectAll,
                    CodeEditor,
                    "Select documentation text",
                    "Ctrl + A",
                    &[Key::A],
                )
                .modifiers(KeyMods::CTRL, KeyMods::all())
                .trigger(Repeat),
                Binding::new(
                    Copy,
                    CodeEditor,
                    "Copy documentation text",
                    "Ctrl + C",
                    &[Key::C],
                )
                .modifiers(KeyMods::CTRL, KeyMods::all())
                .trigger(Repeat),
            ],
        }
    }
}

impl InputMap {
    pub fn binding(&self, action: Action) -> &Binding {
        self.bindings
            .iter()
            .find(|binding| binding.action == action)
            .expect("Action must have a binding.")
    }

    pub fn chord(&self, action: Action) -> KeyChord {
        let binding = self.binding(action);
        KeyChord::new(binding.keys[0]).with_mods(binding.required_modifiers)
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
                }) || binding
                    .mouse
                    .is_some_and(|mouse| ui.is_mouse_clicked(mouse))
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
    ui.tooltip_text(format!("{text} [{}]", map().binding(action).display));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_match_modifiers_alternatives_and_runtime_remapping() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        for (action, key, modifiers, expected) in [
            (Action::ToggleCamera, Key::Keypad0, KeyMods::empty(), true),
            (Action::ToggleCamera, Key::Keypad0, KeyMods::CTRL, false),
            (Action::GoToStart, Key::LeftArrow, KeyMods::SHIFT, true),
            (Action::PreviousFrame, Key::LeftArrow, KeyMods::SHIFT, false),
            (Action::Save, Key::S, KeyMods::CTRL, true),
            (Action::Save, Key::S, KeyMods::empty(), false),
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
