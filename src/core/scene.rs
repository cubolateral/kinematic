use crate::core::{
    Animator,
    components::{Animation, Draw},
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
        for (entity, animation) in self.world.query::<(hecs::Entity, &mut Animation)>().iter() {
            for track in &mut animation.tracks {
                track.track.update(&self.world, entity, time);
            }
        }
    }

    /// Draws each entity using the scene state produced by [`Self::update`].
    pub fn draw(&self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) {
        for (entity, draw) in self.world.query::<(hecs::Entity, &Draw)>().iter() {
            vg.save_with(|vg| {
                vg.set_global_alpha(draw.opacity);
                (draw.on_draw)(&self.world, entity, vg);
            });
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

    /// Spawns an object into the ECS world and returns its typed handler.
    ///
    /// ```
    /// use kinematic::prelude::*;
    ///
    /// let mut scene = Scene::new();
    /// let text: TextHandler = scene.create(
    ///     TextBuilder::new()
    ///         .opacity(1.0)
    ///         .position(Vector2::ZERO)
    ///         .build(),
    /// );
    /// ```
    pub fn create<T: Object + hecs::DynamicBundle>(&mut self, object: T) -> T::Handler {
        T::handler(
            self.world.spawn(
                hecs::EntityBuilder::new()
                    .add_bundle(object)
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

#[cfg(test)]
mod tests {
    use crate::core::{components::*, objects::*, types::*};

    use super::*;

    #[test]
    fn object_builder_sets_component_values_and_preserves_object_defaults() {
        let default = Text::default();
        let object = TextBuilder::new()
            .opacity(0.5)
            .position(vec2(10.0, 20.0))
            .text("Kinematic!".to_owned())
            .build();

        assert_eq!(object.draw.opacity, 0.5);
        assert_eq!(object.transform.position, vec2(10.0, 20.0));
        assert_eq!(object.shape.text, "Kinematic!");
        assert!(std::ptr::fn_addr_eq(
            object.draw.on_draw,
            default.draw.on_draw,
        ));
    }

    #[test]
    fn create_returns_the_generated_object_handler() {
        let mut scene = Scene::new();
        let text: TextHandler = scene.create(TextBuilder::new().build());

        let _ = text.draw(&mut scene).opacity(0.25);
        let mut query = scene.get_world().query::<&Draw>();

        assert_eq!(query.iter().next().unwrap().opacity, 0.25);
    }
}
