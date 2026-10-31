pub(crate) struct Timeline {
    pub is_controlling: bool,
    is_playing: bool,
    pub(crate) transport_revision: u64,
    loop_enabled: bool,
    playback_speed: f32,
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
            transport_revision: 0,
            loop_enabled: true,
            playback_speed: 1.0,
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
            self.current_time =
                (self.current_time + dt * self.playback_speed).clamp(0.0, self.max_time);

            if self.current_time == self.max_time {
                if self.loop_enabled && self.max_time > 0.0 {
                    self.go_to(0.0);
                } else {
                    self.pause();
                }
            }
        }

        // If needs updating.
        if self.previous_time != self.current_time {
            self.previous_time = self.current_time;

            return Some(self.current_time);
        }

        None
    }

    pub(crate) fn playback_speed(&self) -> f32 {
        self.playback_speed
    }

    pub(crate) fn set_playback_speed(&mut self, speed: f32) {
        if speed.is_finite() {
            self.playback_speed = (speed.clamp(0.10, 5.0) * 20.0).round() / 20.0;
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;

        if self.current_time == self.max_time {
            self.go_to(0.0);
        }
    }

    pub fn loop_enabled(&self) -> bool {
        self.loop_enabled
    }

    pub(crate) fn set_fps(&mut self, fps: u32) {
        self.frame_duration = 1.0 / fps.max(1) as f32;
    }

    pub fn set_loop(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    pub fn toggle_loop(&mut self) {
        self.loop_enabled = !self.loop_enabled;
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
        self.transport_revision = self.transport_revision.wrapping_add(1);
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

    pub fn time(&self) -> f32 {
        self.current_time
    }

    pub fn duration(&self) -> f32 {
        self.max_time
    }

    pub(crate) fn set_duration(&mut self, duration: f32) {
        self.max_time = duration.max(0.0);

        self.go_to(self.current_time);

        self.previous_time = -1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::Timeline;

    #[test]
    fn playback_speed_scales_time_with_bounded_steps_and_keeps_frame_steps() {
        let mut timeline = Timeline::new(10.0, 20);
        assert_eq!(timeline.playback_speed(), 1.0);
        timeline.play();
        assert_eq!(timeline.update(1.0), Some(1.0));
        for (requested, expected) in [(0.0, 0.10), (1.23, 1.25), (6.0, 5.0)] {
            timeline.set_playback_speed(requested);
            assert_eq!(timeline.playback_speed(), expected);
            timeline.go_to(0.0);
            assert_eq!(timeline.update(1.0), Some(expected));
        }
        timeline.set_playback_speed(f32::NAN);
        assert_eq!(timeline.playback_speed(), 5.0);
        timeline.go_to(0.0);
        timeline.next_frame();
        assert_eq!(timeline.time(), 0.05);
    }

    #[test]
    fn scrubbing_pauses_and_resumes_playback() {
        let mut timeline = Timeline::new(2.0, 60);
        timeline.play();
        timeline.is_controlling = true;
        timeline.go_to(0.5);
        assert!(!timeline.is_playing());
        assert_eq!(timeline.update(0.1), Some(0.5));
        timeline.is_controlling = false;
        assert!(timeline.is_playing());
        assert_eq!(timeline.update(0.1), Some(0.6));
        timeline.pause();
        timeline.is_controlling = true;
        timeline.go_to(1.0);
        assert!(!timeline.is_playing());
    }

    #[test]
    fn playback_restarts_or_stops_at_the_end() {
        let mut timeline = Timeline::new(2.0, 60);

        timeline.play();
        assert_eq!(timeline.update(2.0), Some(0.0));
        assert!(timeline.is_playing());
        timeline.set_loop(false);
        assert_eq!(timeline.update(2.0), Some(2.0));
        assert!(!timeline.is_playing());
        timeline.play();
        assert_eq!(timeline.time(), 0.0);
        timeline.set_loop(true);
        assert_eq!(timeline.update(2.0), Some(0.0));
        assert!(timeline.is_playing());
    }
}
