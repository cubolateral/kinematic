pub(crate) struct FrameTimer {
    last_tick: std::time::Instant,
    delta_time: f32,
    fps: f32,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            last_tick: std::time::Instant::now(),
            delta_time: 0.0,
            fps: 0.0,
        }
    }

    /// Call once per event you want to measure (e.g. once per redraw, or once
    /// per simulated step).
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        self.delta_time = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;

        if self.delta_time > 0.0 {
            self.fps = 1.0 / self.delta_time;
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }
}
