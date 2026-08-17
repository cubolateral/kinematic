use crate::core::{
    Animator,
    components::{Animation, Draw},
    objects::Object,
};

/// Shared ECS world used by scenes and their handlers.
pub type SceneWorld = std::rc::Rc<std::cell::RefCell<hecs::World>>;

/// Builds scene entities and schedules their animation timeline.
pub trait SceneBuilder {
    fn build(&mut self, s: &mut Scene, a: &mut Animator);
}

/// Runtime ECS scene containing render nodes and compiled animation tracks.
pub struct Scene {
    world: SceneWorld,
}

impl Scene {
    /// Creates an empty scene.
    pub fn new() -> Self {
        Self {
            world: std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new())),
        }
    }

    /// Evaluates every animation track at `time` and writes its value to the ECS world.
    ///
    /// This updates scene state only; rendering remains in [`Self::draw`].
    pub fn update(&self, time: f32) {
        let world = self.world.borrow_mut();
        for (entity, animation) in world.query::<(hecs::Entity, &mut Animation)>().iter() {
            for track in &mut animation.tracks {
                track.track.update(&world, entity, time);
            }
        }
    }

    /// Draws each entity using the scene state produced by [`Self::update`].
    pub fn draw(&self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) {
        let world = self.world.borrow();
        for (entity, draw) in world.query::<(hecs::Entity, &Draw)>().iter() {
            vg.save_with(|vg| {
                vg.set_global_alpha(draw.opacity);
                (draw.on_draw)(&world, entity, vg);
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
    /// let _ = text.position.x(100.0);
    /// ```
    pub fn create<T: Object + hecs::DynamicBundle>(&mut self, object: T) -> T::Handler {
        let entity = self.world.borrow_mut().spawn(
            hecs::EntityBuilder::new()
                .add_bundle(object)
                .add(Animation::default())
                .add(T::inspection())
                .build(),
        );

        T::handler(std::rc::Rc::clone(&self.world), entity)
    }

    /// Read-only access to the underlying ECS world.
    pub fn get_world(&self) -> std::cell::Ref<'_, hecs::World> {
        self.world.borrow()
    }

    /// Mutable access to the underlying ECS world.
    pub fn get_world_mut(&self) -> std::cell::RefMut<'_, hecs::World> {
        self.world.borrow_mut()
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
    fn object_handler_exposes_trackable_fields_directly() {
        let mut scene = Scene::new();
        let text: TextHandler = scene.create(TextBuilder::new().build());
        let circle: CircleHandler = scene.create(CircleBuilder::new().build());

        let _ = text.opacity(0.25);
        let _ = circle.position.x(10.0);
        let world = scene.get_world();
        let mut query = world.query::<&Draw>();

        assert_eq!(query.iter().next().unwrap().opacity, 0.25);
    }
}
