use kinematic_macros::{Object, Trackable};

use crate::core::{components::Draw2D, types::Vector2};

/// View transformation used by a camera.
#[derive(Clone, Debug, Trackable)]
pub struct CameraTransform2D {
    /// Position observed at the center of the viewport.
    #[track]
    pub position: Vector2,
    /// Magnification applied to the scene.
    #[track]
    pub zoom: f32,
    /// Rotation of the view in radians.
    #[track]
    pub rotation: f32,
}

impl Default for CameraTransform2D {
    fn default() -> Self {
        Self {
            position: Vector2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

/// Two-dimensional scene camera whose active view controls scene rendering.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "camera_2d")]
#[morph]
pub struct Camera2D {
    #[trackable]
    pub camera_transform: CameraTransform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            camera_transform: Default::default(),
            draw: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        Scene,
        components::*,
        objects::*,
        types::{Color, vec2},
    };

    #[test]
    fn camera_builder_exposes_only_camera_transform_fields() {
        let mut scene = Scene::new();
        let camera = camera_2d()
            .position(vec2(10.0, 20.0))
            .zoom(2.0)
            .rotation(0.5)
            .build(&mut scene);

        let world = scene.get_world();
        let transform = world.get::<&CameraTransform2D>(camera.get_id()).unwrap();

        assert_eq!(transform.position, vec2(10.0, 20.0));
        assert_eq!(transform.zoom, 2.0);
        assert_eq!(transform.rotation, 0.5);
        assert!(world.get::<&Node>(camera.get_id()).is_ok());
    }

    #[test]
    fn camera_position_centers_its_view_and_keeps_picking_aligned() {
        let mut scene = Scene::new();
        let rectangle = rect()
            .size(vec2(4.0, 4.0))
            .position(vec2(10.0, 0.0))
            .fill(Color::RED)
            .build(&mut scene);
        let camera = camera_2d().position(vec2(10.0, 0.0)).build(&mut scene);

        scene.get_world_2d().add(&rectangle);
        scene.get_world_2d().add(&camera);

        let mut surface = render(&scene);
        assert_eq!(surface.peek_pixels().unwrap().get_color((16, 16)).r(), 255);
        assert_eq!(scene.pick(vec2(0.0, 0.0)), Some(rectangle.get_id()));
    }

    #[test]
    fn camera_zoom_magnifies_the_scene() {
        let mut scene = Scene::new();
        let rectangle = rect()
            .size(vec2(4.0, 4.0))
            .fill(Color::RED)
            .build(&mut scene);
        let camera = camera_2d().zoom(2.0).build(&mut scene);

        scene.get_world_2d().add(&rectangle);
        scene.get_world_2d().add(&camera);

        let mut surface = render(&scene);
        let pixels = surface.peek_pixels().unwrap();

        assert_eq!(pixels.get_color((19, 16)).r(), 255);
        assert_eq!(pixels.get_color((21, 16)).a(), 0);
    }

    #[test]
    fn camera_inherits_ancestor_transforms() {
        let mut scene = Scene::new();
        let rectangle = rect()
            .size(vec2(4.0, 4.0))
            .position(vec2(10.0, 0.0))
            .fill(Color::RED)
            .build(&mut scene);
        let camera = camera_2d().position(vec2(5.0, 0.0)).build(&mut scene);
        let rig = group_2d().position(vec2(5.0, 0.0)).build(&mut scene);

        rig.add(&camera);
        scene.get_world_2d().add(&rectangle);
        scene.get_world_2d().add(&rig);

        let mut surface = render(&scene);
        assert_eq!(surface.peek_pixels().unwrap().get_color((16, 16)).r(), 255);
    }

    #[test]
    fn last_active_camera_in_tree_order_controls_the_view() {
        let mut scene = Scene::new();
        let rectangle = rect()
            .size(vec2(4.0, 4.0))
            .position(vec2(10.0, 0.0))
            .fill(Color::RED)
            .build(&mut scene);
        let first = camera_2d().position(vec2(100.0, 0.0)).build(&mut scene);
        let second = camera_2d().position(vec2(10.0, 0.0)).build(&mut scene);

        scene.get_world_2d().add(&rectangle);
        scene.get_world_2d().add(&first);
        scene.get_world_2d().add(&second);

        let mut surface = render(&scene);
        assert_eq!(surface.peek_pixels().unwrap().get_color((16, 16)).r(), 255);
    }

    #[test]
    fn camera_rotation_rotates_the_view_and_picking_coordinates() {
        let mut scene = Scene::new();
        let rectangle = rect()
            .size(vec2(4.0, 4.0))
            .position(vec2(10.0, 0.0))
            .fill(Color::RED)
            .build(&mut scene);
        let camera = camera_2d()
            .rotation(std::f32::consts::FRAC_PI_2)
            .build(&mut scene);

        scene.get_world_2d().add(&rectangle);
        scene.get_world_2d().add(&camera);

        assert_eq!(scene.pick(vec2(0.0, -10.0)), Some(rectangle.get_id()));

        let mut surface = render(&scene);
        assert_eq!(surface.peek_pixels().unwrap().get_color((16, 6)).r(), 255);
    }

    fn render(scene: &Scene) -> skia_safe::Surface {
        let image_info = skia_safe::ImageInfo::new(
            (32, 32),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::colors::TRANSPARENT);
        canvas.translate((16.0, 16.0));
        scene.draw(canvas);
        surface
    }
}
