use kinematic_macros::Trackable;

/// Rendering callback and opacity for an entity.
///
/// The callback receives the current entity state and must not mutate the ECS world.
#[derive(Trackable)]
pub struct Draw {
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,

    /// Draws this entity on the supplied canvas.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>),
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| {},
            opacity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Scene;

    #[test]
    fn accepts_f32_literals_for_opacity() {
        let mut scene = Scene::new();
        let entity = scene.get_world_mut().spawn((Draw::default(),));

        Draw::handle(&mut scene, entity).opacity(0.0);

        assert_eq!(scene.get_world().get::<&Draw>(entity).unwrap().opacity, 0.0);
    }
}
