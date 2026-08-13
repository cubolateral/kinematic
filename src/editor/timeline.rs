pub(crate) struct Timeline {
    pub is_controlling: bool,
    is_playing: bool,
    current_time: f32,
    max_time: f32,
}

impl Timeline {
    pub fn new(max_time: f32) -> Self {
        Self {
            is_controlling: false,
            is_playing: false,
            current_time: 0.0,
            max_time,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_playing() {
            self.go_to(self.current_time + dt);

            if self.current_time == self.max_time {
                self.go_to(0.0);
            }
        }
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
        self.pause();
        self.go_to(0.0);
    }

    pub fn go_to_end(&mut self) {
        self.pause();
        self.go_to(self.max_time);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing && !self.is_controlling
    }

    pub fn get_max_time(&self) -> f32 {
        self.max_time
    }

    pub fn get_current_time(&self) -> f32 {
        self.current_time
    }
}
