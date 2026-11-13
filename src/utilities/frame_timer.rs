pub(crate) struct FrameTimer {
    last_tick: std::time::Instant,
    delta_time: f32,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            last_tick: std::time::Instant::now(),
            delta_time: 0.0,
        }
    }

    /// Call once per event you want to measure (e.g. once per redraw, or once
    /// per simulated step).
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();

        self.delta_time = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
}
