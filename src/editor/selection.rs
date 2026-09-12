/// Shared editor selection used by every entity view.
#[derive(Default)]
pub(crate) struct Selection {
    object: Option<(usize, hecs::Entity)>,
}

impl Selection {
    pub fn get(&self) -> Option<(usize, hecs::Entity)> {
        self.object
    }

    pub fn select(&mut self, scene: usize, entity: hecs::Entity) {
        self.object = Some((scene, entity));
    }

    pub fn clear(&mut self) {
        self.object = None;
    }
}
