use crate::core::{
    components::Draw3D,
    types::{Quaternion, Vector3},
};
use kinematic_macros::{Object, Trackable};

/// Local three-dimensional camera position and rotation.
#[derive(Clone, Debug, Trackable)]
pub struct CameraTransform3D {
    #[track]
    pub position: Vector3,
    #[track]
    pub rotation: Quaternion,
}

impl Default for CameraTransform3D {
    fn default() -> Self {
        Self {
            position: Vector3::ZERO,
            rotation: Quaternion::IDENTITY,
        }
    }
}

/// Perspective lens; angles are expressed in radians.
#[derive(Clone, Trackable)]
pub struct Perspective {
    #[track]
    pub fov: f32,
    #[track]
    pub near: f32,
    #[track]
    pub far: f32,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Perspective {
    pub fn validate(&self) -> Result<(), String> {
        if !self.fov.is_finite()
            || !(0.0..std::f32::consts::PI).contains(&self.fov)
            || self.fov == 0.0
        {
            return Err("Camera FOV must be between zero and pi radians.".into());
        }
        if !self.near.is_finite()
            || !self.far.is_finite()
            || self.near <= 0.0
            || self.far <= self.near
        {
            return Err("Camera planes must satisfy 0 < near < far.".into());
        }
        Ok(())
    }
}

/// Perspective camera looking along local negative Z, with positive Y up.
#[derive(Default, Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "camera_3d")]
pub struct Camera3D {
    #[trackable]
    pub perspective: Perspective,
    #[trackable]
    pub camera_transform: CameraTransform3D,
    #[trackable]
    pub draw: Draw3D,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene,
        objects::{ObjectHandler, camera_3d},
        types::vec3,
    };

    #[test]
    fn camera_builder_exposes_only_camera_transform_fields() {
        let mut scene = Scene::new();
        let camera = camera_3d()
            .position(vec3(1.0, 2.0, 3.0))
            .rotation(glam::Quat::IDENTITY)
            .build(&mut scene);

        let world = scene.get_world();
        let transform = world.get::<&CameraTransform3D>(camera.get_id()).unwrap();

        assert_eq!(transform.position, vec3(1.0, 2.0, 3.0));
        assert_eq!(transform.rotation, glam::Quat::IDENTITY);
        assert!(
            world
                .get::<&crate::core::components::Transform3D>(camera.get_id())
                .is_err()
        );
    }
}
