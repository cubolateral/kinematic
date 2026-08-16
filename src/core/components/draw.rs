use kinematic_macros::Trackable;

use crate::core::types::Color;

/// Rendering callback and opacity for an entity.
///
/// The callback receives the current entity state and must not mutate the ECS world.
#[derive(Trackable)]
pub struct Draw {
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track]
    pub opacity: f32,
    /// Color applied by the entity drawing callback.
    #[track]
    pub color: Color,

    /// Draws this entity on the supplied canvas.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut femtovg::Canvas<femtovg::renderer::OpenGl>),
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| {},
            opacity: 1.0,
            color: Color::default(),
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

    #[test]
    fn animates_full_colors_and_individual_channels() {
        let mut scene = Scene::new();
        let entity = scene.get_world_mut().spawn((Draw::default(),));

        Draw::handle(&mut scene, entity).color([0.0, 0.25, 0.5, 0.75]);
        let alpha = Draw::handle(&mut scene, entity).color.get().a;
        assert_eq!(
            scene.get_world().get::<&Draw>(entity).unwrap().color.rgba(),
            [0.0, 0.25, 0.5, 0.75]
        );
        assert_eq!(alpha, 0.75);

        Draw::handle(&mut scene, entity).color.r(1.0);
        Draw::handle(&mut scene, entity).color.g(0.5);
        Draw::handle(&mut scene, entity).color.b(0.25);
        Draw::handle(&mut scene, entity).color.a(0.0);
        assert_eq!(
            scene.get_world().get::<&Draw>(entity).unwrap().color.rgba(),
            [1.0, 0.5, 0.25, 0.0]
        );
    }
}
