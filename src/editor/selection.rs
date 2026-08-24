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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_the_entity_selected_by_an_editor_view() {
        let mut selection = Selection::default();

        assert_eq!(selection.get(), None);

        selection.select(hecs::Entity::DANGLING);

        assert_eq!(selection.get(), Some(hecs::Entity::DANGLING));

        selection.clear();

        assert_eq!(selection.get(), None);
    }
}
