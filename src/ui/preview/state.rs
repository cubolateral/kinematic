#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Preview,
    Two,
    Three,
}

pub(in crate::ui) struct State {
    mode: Mode,
    requested_mode: Option<Mode>,
    canvas_2d: Option<(usize, hecs::Entity)>,
    canvas_3d: Option<(usize, hecs::Entity)>,
}

impl Default for State {
    fn default() -> Self {
        Self::new(crate::editor::EditorMode::Preview)
    }
}

impl State {
    pub(in crate::ui) fn new(mode: crate::editor::EditorMode) -> Self {
        let mode = match mode {
            crate::editor::EditorMode::Preview => Mode::Preview,
            crate::editor::EditorMode::Two => Mode::Two,
            crate::editor::EditorMode::Three => Mode::Three,
        };
        Self {
            mode,
            requested_mode: Some(mode),
            canvas_2d: None,
            canvas_3d: None,
        }
    }

    pub(in crate::ui) fn cached_mode(&self) -> crate::editor::EditorMode {
        match self.mode {
            Mode::Preview => crate::editor::EditorMode::Preview,
            Mode::Two => crate::editor::EditorMode::Two,
            Mode::Three => crate::editor::EditorMode::Three,
        }
    }

    pub(super) fn mode(&self) -> Mode {
        self.mode
    }

    pub(super) fn set_mode(&mut self, mode: Mode) -> bool {
        if self.mode == mode {
            return false;
        }
        self.mode = mode;
        true
    }

    pub(in crate::ui) fn edit_canvas(&mut self, canvas: crate::editor::SelectedCanvas) {
        self.requested_mode = match canvas {
            crate::editor::SelectedCanvas::Two(_) => Some(Mode::Two),
            crate::editor::SelectedCanvas::Three(_) => Some(Mode::Three),
            crate::editor::SelectedCanvas::None => None,
        };
    }

    pub(super) fn take_requested_mode(&mut self) -> Option<Mode> {
        self.requested_mode.take()
    }

    pub(super) fn sync_canvas_2d(&mut self, scene: usize, canvas: hecs::Entity) -> bool {
        let changed = self
            .canvas_2d
            .is_some_and(|current| current != (scene, canvas));
        self.canvas_2d = Some((scene, canvas));
        changed
    }

    pub(super) fn sync_canvas_3d(&mut self, scene: usize, canvas: hecs::Entity) -> bool {
        let changed = self
            .canvas_3d
            .is_some_and(|current| current != (scene, canvas));
        self.canvas_3d = Some((scene, canvas));
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, State};

    #[test]
    fn preview_is_the_default_mode() {
        let mut state = State::default();
        assert_eq!(state.mode(), Mode::Preview);
        assert!(state.set_mode(Mode::Two));
        assert_eq!(state.mode(), Mode::Two);
        assert!(!state.set_mode(Mode::Two));
    }

    #[test]
    fn first_canvas_sync_preserves_the_cached_camera() {
        let mut state = State::default();
        let canvas = hecs::Entity::DANGLING;

        assert!(!state.sync_canvas_2d(0, canvas));
        assert!(!state.sync_canvas_3d(0, canvas));
        assert!(!state.sync_canvas_2d(0, canvas));
        assert!(!state.sync_canvas_3d(0, canvas));
        assert!(state.sync_canvas_2d(1, canvas));
        assert!(state.sync_canvas_3d(1, canvas));
    }

    #[test]
    fn cached_mode_is_requested_for_the_first_frame() {
        let mut state = State::new(crate::editor::EditorMode::Three);

        assert_eq!(state.mode(), Mode::Three);
        assert_eq!(state.take_requested_mode(), Some(Mode::Three));
        assert_eq!(state.cached_mode(), crate::editor::EditorMode::Three);
    }
}
