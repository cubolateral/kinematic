use crate::core::{
    Animator,
    components::{Animation, Node},
    objects::Object,
};

/// Builds scene entities and schedules their animation timeline.
pub trait SceneBuilder {
    fn build(&mut self, s: &mut Scene, a: &mut Animator);
}

/// Runtime ECS scene containing render nodes and compiled animation tracks.
pub struct Scene {
    world: hecs::World,
}

impl Scene {
    /// Creates an empty scene.
    pub fn new() -> Self {
        Self {
            world: hecs::World::new(),
        }
    }

    /// Evaluates every animation track at `time` and writes its value to the ECS world.
    ///
    /// This updates scene state only; rendering remains in [`Self::draw`].
    pub fn update(&self, time: f32) {
        for (entity, animation) in self.world.query::<(hecs::Entity, &Animation)>().iter() {
            for track in &animation.tracks {
                track.track.update(&self.world, entity, time);
            }
        }
    }

    /// Draws each node using the scene state produced by [`Self::update`].
    pub fn draw(&self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) {
        for (entity, node) in self.world.query::<(hecs::Entity, &Node)>().iter() {
            (node.on_draw)(&self.world, entity, vg);
        }
    }

    /// Populates the scene and compiles the builder's animation timeline.
    ///
    /// Returns the duration of the resulting timeline.
    pub fn build(&mut self, builder: &mut dyn SceneBuilder) -> f32 {
        let mut animator = Animator::new();
        builder.build(self, &mut animator);
        animator.get_duration(self)
    }

    /// Spawns a bundle into the ECS world and returns its typed handle.
    pub fn create<T: Object + hecs::DynamicBundle>(&mut self, bundle: T) -> T::Handle {
        T::handle(
            self.world.spawn(
                hecs::EntityBuilder::new()
                    .add_bundle(bundle)
                    .add(Animation::default())
                    .add(T::inspection())
                    .build(),
            ),
        )
    }

    /// Read-only access to the underlying ECS world.
    pub fn get_world(&self) -> &hecs::World {
        &self.world
    }

    /// Mutable access to the underlying ECS world.
    pub fn get_world_mut(&mut self) -> &mut hecs::World {
        &mut self.world
    }
}
