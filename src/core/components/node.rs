#[derive(Debug)]
pub struct Node {
    /// Parent container, assigned when the object is attached to the scene tree.
    pub(crate) parent: Option<hecs::Entity>,
    /// Timeline bounds stored as `[start, end]` and evaluated as `start <= time < end`.
    pub(crate) lifetime: [f32; 2],
    /// Whether the object is active at the current scene time.
    pub(crate) is_activated: bool,
    /// Whether this node is the scene's internal root container.
    pub(crate) is_root: bool,
    /// Ordered child objects, allocated when the first child is added.
    pub(crate) children: Option<Vec<hecs::Entity>>,
}

impl Node {
    pub(crate) fn activate(&mut self, start: f32) {
        self.lifetime = [start, f32::INFINITY];
        self.is_activated = start <= 0.0;
    }

    pub(crate) fn deactivate(&mut self, end: f32) {
        self.lifetime[1] = self.lifetime[1].min(end);
        self.is_activated = self.lifetime[0] <= 0.0 && self.lifetime[1] > 0.0;
    }

    pub(crate) fn update(&mut self, time: f32) {
        self.is_activated = time >= self.lifetime[0] && time < self.lifetime[1];
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            parent: None,
            lifetime: [f32::INFINITY; 2],
            is_activated: false,
            is_root: false,
            children: None,
        }
    }
}
