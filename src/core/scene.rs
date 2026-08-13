use crate::core::components::Node;

pub trait SceneBuilder {
    fn build(&mut self, scene: &mut Scene);
}

pub struct Scene {
    world: hecs::World,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            world: hecs::World::new(),
        }
    }

    pub fn draw(&self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) {
        for (entity, node) in self.world.query::<(hecs::Entity, &Node)>().iter() {
            (node.on_draw)(&self.world, entity, vg);
        }
    }

    pub fn build(&mut self, builder: &mut dyn SceneBuilder) -> f32 {
        builder.build(self);
        10.0 // TODO.
    }

    pub fn create(&mut self, bundle: impl hecs::DynamicBundle) {
        self.world.spawn(bundle);
    }
}
