/// Shared editor selection used by every entity view.
#[derive(Default)]
pub(crate) struct Selection {
    object: Option<(usize, hecs::Entity)>,
    audio: Option<(usize, u64)>,
}

impl Selection {
    pub fn get(&self) -> Option<(usize, hecs::Entity)> {
        self.object
    }

    pub fn select(&mut self, scene: usize, entity: hecs::Entity) {
        self.object = Some((scene, entity));
    }

    pub(crate) fn audio(&self) -> Option<(usize, u64)> {
        self.audio
    }

    pub(crate) fn select_audio(&mut self, scene: usize, id: u64) {
        self.audio = Some((scene, id));
    }

    pub(crate) fn remap_scenes(&mut self, order: &[usize]) {
        self.object = self.object.and_then(|(scene, entity)| {
            order
                .iter()
                .position(|old| *old == scene)
                .map(|index| (index, entity))
        });
        self.audio = self.audio.and_then(|(scene, id)| {
            order
                .iter()
                .position(|old| *old == scene)
                .map(|index| (index, id))
        });
    }

    pub fn clear(&mut self) {
        self.object = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_and_object_selections_survive_each_other_and_scene_reordering() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let mut selection = Selection::default();
        selection.select(1, entity);
        selection.select_audio(0, 42);
        assert_eq!(selection.get(), Some((1, entity)));
        selection.select(1, entity);
        assert_eq!(selection.audio(), Some((0, 42)));
        selection.remap_scenes(&[1, 0]);
        assert_eq!(selection.get(), Some((0, entity)));
        assert_eq!(selection.audio(), Some((1, 42)));
        selection.clear();
        assert_eq!(selection.audio(), Some((1, 42)));
        selection.remap_scenes(&[0]);
        assert_eq!(selection.audio(), None);
    }
}
