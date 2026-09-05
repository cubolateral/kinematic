const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 100.0;
const ZOOM_STEP: f32 = 0.15;
const PAN_THRESHOLD: f32 = 4.0;

pub(in crate::ui) struct State {
    zoom: f32,
    pan: [f32; 2],
    pointer_down: bool,
    pointer_moved: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: [0.0; 2],
            pointer_down: false,
            pointer_moved: false,
        }
    }
}

impl State {
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn pan(&self) -> [f32; 2] {
        self.pan
    }

    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = [0.0; 2];
        self.pointer_down = false;
        self.pointer_moved = false;
    }

    pub fn zoom_at(&mut self, wheel: f32, anchor: [f32; 2]) {
        if wheel == 0.0 {
            return;
        }

        let old_zoom = self.zoom;
        self.zoom = (old_zoom * (wheel * ZOOM_STEP).exp()).clamp(MIN_ZOOM, MAX_ZOOM);
        let ratio = self.zoom / old_zoom;

        for axis in 0..2 {
            self.pan[axis] = anchor[axis] + (self.pan[axis] - anchor[axis]) * ratio;
        }
    }

    pub fn press(&mut self) {
        self.pointer_down = true;
        self.pointer_moved = false;
    }

    pub fn drag(&mut self, total_delta: [f32; 2], frame_delta: [f32; 2]) {
        if !self.pointer_down {
            return;
        }

        let started =
            !self.pointer_moved && total_delta[0].abs().max(total_delta[1].abs()) >= PAN_THRESHOLD;
        if started {
            self.pointer_moved = true;
            self.pan[0] += total_delta[0];
            self.pan[1] += total_delta[1];
        } else if self.pointer_moved {
            self.pan[0] += frame_delta[0];
            self.pan[1] += frame_delta[1];
        }
    }

    pub fn release(&mut self, hovered: bool) -> bool {
        let select = self.pointer_down && !self.pointer_moved && hovered;
        self.pointer_down = false;
        self.pointer_moved = false;
        select
    }

    pub fn is_panning(&self) -> bool {
        self.pointer_down && self.pointer_moved
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn reset_restores_the_default_view() {
        let mut state = State::default();
        state.zoom_at(3.0, [120.0, 80.0]);
        state.press();
        state.drag([10.0, 0.0], [10.0, 0.0]);

        state.reset();

        assert_eq!(state.zoom(), 1.0);
        assert_eq!(state.pan(), [0.0; 2]);
    }

    #[test]
    fn zoom_keeps_the_anchor_in_place() {
        let mut state = State::default();
        let anchor = [100.0, -50.0];

        state.zoom_at(1.0, anchor);

        let zoom = state.zoom();
        let pan = state.pan();
        assert!((pan[0] - anchor[0] * (1.0 - zoom)).abs() < 0.001);
        assert!((pan[1] - anchor[1] * (1.0 - zoom)).abs() < 0.001);
    }

    #[test]
    fn a_pan_does_not_become_a_selection() {
        let mut state = State::default();
        state.press();
        state.drag([5.0, 0.0], [1.0, 0.0]);

        assert_eq!(state.pan(), [5.0, 0.0]);
        assert!(!state.release(true));
    }

    #[test]
    fn an_unmoved_click_selects_only_when_released_over_the_view() {
        let mut state = State::default();
        state.press();
        assert!(state.release(true));

        state.press();
        assert!(!state.release(false));
    }
}
