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
        Self {
            mode: Mode::Preview,
            requested_mode: None,
            canvas_2d: None,
            canvas_3d: None,
        }
    }
}

impl State {
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
        if self.canvas_2d == Some((scene, canvas)) {
            return false;
        }
        self.canvas_2d = Some((scene, canvas));
        true
    }

    pub(super) fn sync_canvas_3d(&mut self, scene: usize, canvas: hecs::Entity) -> bool {
        if self.canvas_3d == Some((scene, canvas)) {
            return false;
        }
        self.canvas_3d = Some((scene, canvas));
        true
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
}
