/// Shared editor selection used by every entity view.
#[derive(Default)]
pub(crate) struct Selection {
    object: Option<(usize, hecs::Entity)>,
    audio: Option<u64>,
}

impl Selection {
    pub fn get(&self) -> Option<(usize, hecs::Entity)> {
        self.object
    }

    pub fn select(&mut self, scene: usize, entity: hecs::Entity) {
        self.object = Some((scene, entity));
        self.audio = None;
    }

    pub(crate) fn audio(&self) -> Option<u64> {
        self.audio
    }

    pub(crate) fn select_audio(&mut self, id: u64) {
        self.audio = Some(id);
        self.object = None;
    }

    pub(crate) fn remap_scenes(&mut self, order: &[usize]) {
        self.object = self.object.and_then(|(scene, entity)| {
            order
                .iter()
                .position(|old| *old == scene)
                .map(|index| (index, entity))
        });
    }

    pub(crate) fn clear_audio(&mut self) {
        self.audio = None;
    }

    pub fn clear(&mut self) {
        self.object = None;
        self.audio = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selecting_audio_or_an_object_replaces_the_previous_selection() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let mut selection = Selection::default();

        selection.select(1, entity);
        selection.select_audio(42);
        assert_eq!(selection.get(), None);
        assert_eq!(selection.audio(), Some(42));

        selection.select(1, entity);
        assert_eq!(selection.get(), Some((1, entity)));
        assert_eq!(selection.audio(), None);

        selection.remap_scenes(&[1, 0]);
        assert_eq!(selection.get(), Some((0, entity)));
        selection.clear();
        assert_eq!(selection.get(), None);
        assert_eq!(selection.audio(), None);

        selection.select_audio(42);
        selection.remap_scenes(&[0]);
        assert_eq!(selection.audio(), Some(42));
        selection.clear_audio();
        assert_eq!(selection.get(), None);
        assert_eq!(selection.audio(), None);

        selection.select_audio(42);
        selection.clear();
        assert_eq!(selection.get(), None);
        assert_eq!(selection.audio(), None);
    }
}
