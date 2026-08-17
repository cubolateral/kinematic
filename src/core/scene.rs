use crate::core::{
    Animator,
    components::{Animation, Draw, Transform},
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
    render_surfaces: std::cell::RefCell<std::collections::HashMap<hecs::Entity, RenderSurface>>,
}

struct RenderSurface {
    image: femtovg::ImageId,
    width: usize,
    height: usize,
}

impl Scene {
    /// Creates an empty scene.
    pub fn new() -> Self {
        Self {
            world: std::rc::Rc::new(std::cell::RefCell::new(hecs::World::new())),
            render_surfaces: std::cell::RefCell::new(std::collections::HashMap::new()),
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
        self.draw_to(vg, femtovg::RenderTarget::Screen);
    }

    /// Draws each entity into the supplied render target.
    pub fn draw_to(
        &self,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        target: femtovg::RenderTarget,
    ) {
        let world = self.world.borrow();
        let mut surfaces = self.render_surfaces.borrow_mut();
        let mut live = std::collections::HashSet::new();
        let flags = femtovg::ImageFlags::PREMULTIPLIED | femtovg::ImageFlags::FLIP_Y;

        for (entity, draw, transform) in world.query::<(hecs::Entity, &Draw, &Transform)>().iter() {
            let [x, y, width, height] = (draw.get_rect)(&world, entity, vg);
            if ![x, y, width, height].iter().all(|value| value.is_finite())
                || width <= 0.0
                || height <= 0.0
            {
                continue;
            }

            live.insert(entity);
            let image_width = width.ceil() as usize;
            let image_height = height.ceil() as usize;
            let surface = surfaces.entry(entity).or_insert_with(|| RenderSurface {
                image: vg
                    .create_image_empty(
                        image_width,
                        image_height,
                        femtovg::PixelFormat::Rgba8,
                        flags,
                    )
                    .expect("Entity render surface must be created."),
                width: image_width,
                height: image_height,
            });

            if surface.width != image_width || surface.height != image_height {
                vg.realloc_image(
                    surface.image,
                    image_width,
                    image_height,
                    femtovg::PixelFormat::Rgba8,
                    flags,
                )
                .expect("Entity render surface must be resized.");
                surface.width = image_width;
                surface.height = image_height;
            }

            vg.set_render_target(femtovg::RenderTarget::Image(surface.image));
            vg.clear_rect(
                0,
                0,
                image_width as u32,
                image_height as u32,
                femtovg::Color::rgba(0, 0, 0, 0),
            );
            vg.save_with(|vg| {
                vg.reset_transform();
                vg.set_global_alpha(1.0);
                vg.translate(-x, -y);
                (draw.on_draw)(&world, entity, vg);
            });

            vg.set_render_target(target);
            vg.save_with(|vg| {
                vg.set_global_alpha(draw.opacity);
                vg.translate(transform.position.x, transform.position.y);
                vg.rotate(transform.rotation);
                vg.scale(transform.scale.x, transform.scale.y);

                let mut path = femtovg::Path::new();
                path.rect(x, y, width, height);
                vg.fill_path(
                    &path,
                    &femtovg::Paint::image(surface.image, x, y, width, height, 0.0, 1.0),
                );
            });
        }

        surfaces.retain(|entity, surface| {
            if live.contains(entity) {
                true
            } else {
                vg.delete_image(surface.image);
                false
            }
        });
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
        assert!(std::ptr::fn_addr_eq(
            object.draw.get_rect,
            default.draw.get_rect,
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
