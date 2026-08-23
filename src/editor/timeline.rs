pub(crate) struct Timeline {
    pub is_controlling: bool,
    is_playing: bool,
    previous_time: f32,
    current_time: f32,
    max_time: f32,
    frame_duration: f32,
}

impl Timeline {
    pub fn new(max_time: f32, fps: u32) -> Self {
        Self {
            is_controlling: false,
            is_playing: false,
            previous_time: -1.0, // Start by updating.
            current_time: 0.0,
            max_time,
            frame_duration: 1.0 / fps.max(1) as f32,
        }
    }

    /// If the scene needs to be updated, it returns the current timeline time;
    /// otherwise, it returns None.
    pub fn update(&mut self, dt: f32) -> Option<f32> {
        if self.is_playing() {
            self.go_to(self.current_time + dt);

            if self.current_time == self.max_time {
                self.go_to(0.0);
            }
        }

        // If needs updating.
        if self.previous_time != self.current_time {
            self.previous_time = self.current_time;
            return Some(self.current_time);
        }

        None
    }

    pub fn play(&mut self) {
        self.is_playing = true;

        if self.current_time == self.max_time {
            self.go_to(0.0);
        }
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn toggle(&mut self) {
        if self.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn go_to(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.max_time);
    }

    pub fn go_to_start(&mut self) {
        self.go_to(0.0);
    }

    pub fn go_to_end(&mut self) {
        self.pause();
        self.go_to(self.max_time);
    }

    pub fn next_frame(&mut self) {
        self.pause();
        self.go_to(self.current_time + self.frame_duration);
    }

    pub fn previous_frame(&mut self) {
        self.pause();
        self.go_to(self.current_time - self.frame_duration);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing && !self.is_controlling
    }

    pub fn get_time(&self) -> f32 {
        self.current_time
    }

    pub fn get_duration(&self) -> f32 {
        self.max_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_buttons_move_by_one_project_frame_and_pause() {
        let mut timeline = Timeline::new(1.0, 10);
        timeline.play();

        timeline.next_frame();
        assert!((timeline.get_time() - 0.1).abs() < f32::EPSILON);
        assert!(!timeline.is_playing());

        timeline.previous_frame();
        assert!(timeline.get_time().abs() < f32::EPSILON);
    }

    #[test]
    fn frame_buttons_clamp_to_timeline_bounds() {
        let mut timeline = Timeline::new(0.15, 10);

        timeline.next_frame();
        timeline.next_frame();
        assert!((timeline.get_time() - 0.15).abs() < f32::EPSILON);

        timeline.previous_frame();
        timeline.previous_frame();
        assert!(timeline.get_time().abs() < f32::EPSILON);
    }
}
