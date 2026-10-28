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
    Cut,
    RequestSuggestions,
    SignatureHelp,
    Replace,
    SelectLine,
    NextOccurrence,
    AllOccurrences,
    MoveLineUp,
    MoveLineDown,
    DuplicateUp,
    DuplicateDown,
    DeleteWordLeft,
    DeleteWordRight,
    InsertAbove,
    SoftNewline,
    InsertBelow,
    ScrollUp,
    ScrollDown,
    FindNext,
    FindPrevious,
    GoToLine,
    QuickOpen,
    CloseTab,
    NextTab,
    PreviousTab,
    SaveAll,
    FormatDocument,
    ToggleWordWrap,
    JumpBracket,
    Fold,
    Unfold,
    NativeEditing,
    ChordPrefix,
    CommentLines,
    UncommentLines,
    FoldAll,
    UnfoldAll,
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

        let (suggestion_modifier, suggestion_display) = (KeyMods::CTRL, "Ctrl + Space");
        let mut map = Self {
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
                    SignatureHelp,
                    CodeEditor,
                    "Show parameter hints",
                    "Ctrl + Shift + Space",
                    &[Key::Space],
                )
                .modifiers(
                    KeyMods::CTRL | KeyMods::SHIFT,
                    KeyMods::CTRL | KeyMods::SHIFT,
                ),
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
                Binding::new(Find, CodeEditor, "Find", "Ctrl + F", &[Key::F])
                    .modifiers(KeyMods::CTRL, KeyMods::CTRL),
                Binding::new(Save, CodeEditor, "Save file", "Ctrl + S", &[Key::S])
                    .modifiers(KeyMods::CTRL, KeyMods::CTRL),
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
        };

        for (action, name, display, key, modifiers) in [
            (
                Cut,
                "Cut selection / line",
                "Ctrl + X",
                Key::X,
                KeyMods::CTRL,
            ),
            (
                MoveLineUp,
                "Move lines up",
                "Alt + Up",
                Key::UpArrow,
                KeyMods::ALT,
            ),
            (
                MoveLineDown,
                "Move lines down",
                "Alt + Down",
                Key::DownArrow,
                KeyMods::ALT,
            ),
            (Replace, "Replace", "Ctrl + H", Key::H, KeyMods::CTRL),
            (SelectLine, "Select line", "Ctrl + L", Key::L, KeyMods::CTRL),
            (
                NextOccurrence,
                "Add next occurrence",
                "Ctrl + D",
                Key::D,
                KeyMods::CTRL,
            ),
            (
                AllOccurrences,
                "Select all occurrences",
                "Ctrl + Shift + L",
                Key::L,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                DuplicateUp,
                "Copy lines up",
                "Ctrl + Shift + Alt + Up",
                Key::UpArrow,
                KeyMods::CTRL | KeyMods::SHIFT | KeyMods::ALT,
            ),
            (
                DuplicateDown,
                "Copy lines down",
                "Ctrl + Shift + Alt + Down",
                Key::DownArrow,
                KeyMods::CTRL | KeyMods::SHIFT | KeyMods::ALT,
            ),
            (
                DeleteWordLeft,
                "Delete word left",
                "Ctrl + Backspace",
                Key::Backspace,
                KeyMods::CTRL,
            ),
            (
                DeleteWordRight,
                "Delete word right",
                "Ctrl + Delete",
                Key::Delete,
                KeyMods::CTRL,
            ),
            (
                SoftNewline,
                "Insert newline",
                "Shift + Enter",
                Key::Enter,
                KeyMods::SHIFT,
            ),
            (
                InsertAbove,
                "Insert line above",
                "Ctrl + Shift + Enter",
                Key::Enter,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                InsertBelow,
                "Insert line below",
                "Ctrl + Enter",
                Key::Enter,
                KeyMods::CTRL,
            ),
            (
                ScrollUp,
                "Scroll up",
                "Ctrl + Up",
                Key::UpArrow,
                KeyMods::CTRL,
            ),
            (
                ScrollDown,
                "Scroll down",
                "Ctrl + Down",
                Key::DownArrow,
                KeyMods::CTRL,
            ),
            (FindNext, "Find next", "F3", Key::F3, KeyMods::empty()),
            (
                FindPrevious,
                "Find previous",
                "Shift + F3",
                Key::F3,
                KeyMods::SHIFT,
            ),
            (
                GoToLine,
                "Go to line / column",
                "Ctrl + G",
                Key::G,
                KeyMods::CTRL,
            ),
            (QuickOpen, "Go to file", "Ctrl + P", Key::P, KeyMods::CTRL),
            (CloseTab, "Close editor", "Ctrl + W", Key::W, KeyMods::CTRL),
            (
                NextTab,
                "Next editor",
                "Ctrl + Page Down",
                Key::PageDown,
                KeyMods::CTRL,
            ),
            (
                PreviousTab,
                "Previous editor",
                "Ctrl + Page Up",
                Key::PageUp,
                KeyMods::CTRL,
            ),
            (
                SaveAll,
                "Save all",
                "Ctrl + Alt + S",
                Key::S,
                KeyMods::CTRL | KeyMods::ALT,
            ),
            (
                FormatDocument,
                "Format document",
                "Ctrl + Shift + I",
                Key::I,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                ToggleWordWrap,
                "Toggle word wrap",
                "Alt + Z",
                Key::Z,
                KeyMods::ALT,
            ),
            (
                JumpBracket,
                "Go to bracket",
                "Ctrl + Shift + Backslash",
                Key::Backslash,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                Fold,
                "Fold",
                "Ctrl + Shift + [",
                Key::LeftBracket,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                Unfold,
                "Unfold",
                "Ctrl + Shift + ]",
                Key::RightBracket,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
        ] {
            let trigger = if matches!(
                action,
                Replace
                    | GoToLine
                    | QuickOpen
                    | CloseTab
                    | SaveAll
                    | FormatDocument
                    | ToggleWordWrap
                    | JumpBracket
                    | Fold
                    | Unfold
            ) {
                Once
            } else {
                Repeat
            };

            map.bindings.push(
                Binding::new(action, CodeEditor, name, display, &[key])
                    .modifiers(modifiers, modifiers)
                    .trigger(trigger),
            );
        }

        for binding in &mut map.bindings {
            if matches!(binding.action, InsertAbove | InsertBelow | SoftNewline) {
                binding.keys.push(Key::KeypadEnter);
            }
        }

        for (action, name, display, key, modifiers) in [
            (
                ChordPrefix,
                "Start command chord",
                "Ctrl + K",
                Key::K,
                KeyMods::CTRL,
            ),
            (
                CommentLines,
                "Add line comments",
                "Ctrl + K, Ctrl + C",
                Key::C,
                KeyMods::CTRL,
            ),
            (
                UncommentLines,
                "Remove line comments",
                "Ctrl + K, Ctrl + U",
                Key::U,
                KeyMods::CTRL,
            ),
            (
                FoldAll,
                "Fold all",
                "Ctrl + K, Ctrl + 0",
                Key::Key0,
                KeyMods::CTRL,
            ),
            (
                UnfoldAll,
                "Unfold all",
                "Ctrl + K, Ctrl + J",
                Key::J,
                KeyMods::CTRL,
            ),
        ] {
            map.bindings.push(
                Binding::new(action, CodeEditor, name, display, &[key])
                    .modifiers(modifiers, modifiers),
            );
        }

        // CTE handles these bindings directly; list them alongside application commands.
        for (name, display, key, modifiers) in [
            ("Undo", "Ctrl + Z", Key::Z, KeyMods::CTRL),
            ("Redo", "Ctrl + Shift + Z / Ctrl + Y", Key::Y, KeyMods::CTRL),
            ("Copy selection / line", "Ctrl + C", Key::C, KeyMods::CTRL),
            ("Paste", "Ctrl + V", Key::V, KeyMods::CTRL),
            ("Select all code", "Ctrl + A", Key::A, KeyMods::CTRL),
            (
                "Delete lines",
                "Ctrl + Shift + K",
                Key::K,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            ("Toggle line comment", "Ctrl + /", Key::Slash, KeyMods::CTRL),
            (
                "Indent lines",
                "Ctrl + ] / Tab",
                Key::RightBracket,
                KeyMods::CTRL,
            ),
            (
                "Outdent lines",
                "Ctrl + [ / Shift + Tab",
                Key::LeftBracket,
                KeyMods::CTRL,
            ),
            (
                "Expand selection",
                "Shift + Alt + Right",
                Key::RightArrow,
                KeyMods::SHIFT | KeyMods::ALT,
            ),
            (
                "Shrink selection",
                "Shift + Alt + Left",
                Key::LeftArrow,
                KeyMods::SHIFT | KeyMods::ALT,
            ),
            (
                "Move by word",
                "Ctrl + Left / Right",
                Key::LeftArrow,
                KeyMods::CTRL,
            ),
            (
                "Select by word",
                "Ctrl + Shift + Left / Right",
                Key::LeftArrow,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                "Go to start / end of file",
                "Ctrl + Home / End",
                Key::Home,
                KeyMods::CTRL,
            ),
            (
                "Select to start / end of file",
                "Ctrl + Shift + Home / End",
                Key::Home,
                KeyMods::CTRL | KeyMods::SHIFT,
            ),
            (
                "Go to start / end of line",
                "Home / End",
                Key::Home,
                KeyMods::empty(),
            ),
            (
                "Select to start / end of line",
                "Shift + Home / End",
                Key::Home,
                KeyMods::SHIFT,
            ),
            (
                "Page up / down",
                "Page Up / Page Down",
                Key::PageUp,
                KeyMods::empty(),
            ),
            (
                "Select page up / down",
                "Shift + Page Up / Page Down",
                Key::PageUp,
                KeyMods::SHIFT,
            ),
            (
                "Select with arrows",
                "Shift + Arrow keys",
                Key::LeftArrow,
                KeyMods::SHIFT,
            ),
            (
                "Indent / accept completion",
                "Tab",
                Key::Tab,
                KeyMods::empty(),
            ),
            ("Outdent", "Shift + Tab", Key::Tab, KeyMods::SHIFT),
        ] {
            map.bindings.push(
                Binding::new(NativeEditing, CodeEditor, name, display, &[key])
                    .modifiers(modifiers, modifiers),
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

    pub fn routed(&self, ui: &Ui, action: Action) -> bool {
        let binding = self.binding(action);
        let options = dear_imgui_rs::ShortcutOptions::new()
            .route(dear_imgui_rs::ShortcutRoute::Global(
                dear_imgui_rs::ShortcutGlobalRouteFlags::OVER_FOCUSED,
            ))
            .flags(if matches!(binding.trigger, Trigger::Repeat) {
                dear_imgui_rs::ShortcutFlags::REPEAT
            } else {
                dear_imgui_rs::ShortcutFlags::NONE
            });

        binding.keys.iter().fold(false, |pressed, &key| {
            ui.shortcut_with_flags(
                KeyChord::new(key).with_mods(binding.required_modifiers),
                options,
            ) || pressed
        })
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
