use kinematic_macros::Trackable;

use crate::core::Vector2;

#[derive(Trackable, Default, Debug)]
/// Position of an entity in logical canvas coordinates.
pub struct Transform {
    #[track]
    pub position: Vector2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Scene;

    #[test]
    fn animates_a_position_and_its_individual_axes() {
        let mut scene = Scene::new();
        let entity = scene.get_world_mut().spawn((Transform::default(),));

        Transform::handle(&mut scene, entity).position([10.0, 20.0]);
        assert_eq!(
            scene
                .get_world()
                .get::<&Transform>(entity)
                .unwrap()
                .position,
            Vector2::new(10.0, 20.0)
        );

        Transform::handle(&mut scene, entity).position.x(30.0);
        assert_eq!(
            scene
                .get_world()
                .get::<&Transform>(entity)
                .unwrap()
                .position,
            Vector2::new(30.0, 20.0)
        );
    }
}
