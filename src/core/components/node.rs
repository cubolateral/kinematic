#[derive(Debug)]
pub struct Node {
    /// Timeline bounds stored as `[start, end]` and evaluated as `start <= time < end`.
    pub(crate) lifetime: [f32; 2],
    /// Whether the object is active at the current scene time.
    pub(crate) is_activated: bool,
}

impl Node {
    pub(crate) fn activate(&mut self, start: f32) {
        self.lifetime = [start, f32::INFINITY];
        self.is_activated = start <= 0.0;
    }

    pub(crate) fn deactivate(&mut self, end: f32) {
        self.lifetime[1] = end;
        self.is_activated = self.lifetime[0] <= 0.0 && end > 0.0;
    }

    pub(crate) fn update(&mut self, time: f32) {
        self.is_activated = time >= self.lifetime[0] && time < self.lifetime[1];
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            lifetime: [f32::INFINITY; 2],
            is_activated: false,
        }
    }
}
