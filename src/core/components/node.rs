#[derive(Debug)]
pub struct Node {
    /// Timeline bounds stored as `[start, end]` and evaluated as `start <= time < end`.
    pub(crate) lifetime: [f32; 2],
    /// Whether the object is active at the current scene time.
    pub(crate) is_activated: bool,
}

impl Node {
    pub(crate) fn new(start: f32) -> Self {
        Self {
            lifetime: [start, f32::INFINITY],
            is_activated: start <= 0.0,
        }
    }

    pub(crate) fn update(&mut self, time: f32) {
        self.is_activated = time >= self.lifetime[0] && time < self.lifetime[1];
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new(0.0)
    }
}
