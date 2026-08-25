/// Shared editor selection used by every entity view.
#[derive(Default)]
pub(crate) struct Selection {
    entity: Option<hecs::Entity>,
}

impl Selection {
    pub fn get(&self) -> Option<hecs::Entity> {
        self.entity
    }

    pub fn select(&mut self, entity: hecs::Entity) {
        self.entity = Some(entity);
    }

    pub fn clear(&mut self) {
        self.entity = None;
    }
}
