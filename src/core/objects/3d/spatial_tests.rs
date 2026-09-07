use crate::core::components::{Inspection, Morph};
use crate::core::objects::draw_canvas2d;
use crate::prelude::*;
use crate::renderer::plan::{active_subtree, canvas_order, order_dependencies};

#[test]
fn spatial_handlers_compose_transforms_and_bounds() {
    let mut scene = Scene::new();
    let canvas = canvas_3d().resolution((640, 480)).build(&mut scene);
    let parent = group_3d()
        .position(vec3(1.0, 2.0, 3.0))
        .rotation(Quaternion::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .scale(vec3(2.0, 3.0, 4.0))
        .build(&mut scene);
    let child = cuboid()
        .position(vec3(1.0, 0.0, 1.0))
        .rotation(Quaternion::from_rotation_y(0.5))
        .size(vec3(2.0, 3.0, 4.0))
        .build(&mut scene);
    parent.add(&child);
    canvas.add(&parent);
    scene.add_canvas_3d(&canvas);
    assert!(
        child
            .get_global_position()
            .abs_diff_eq(vec3(1.0, 4.0, 7.0), 1e-5)
    );
    assert!(
        child
            .get_global_scale()
            .abs_diff_eq(vec3(2.0, 3.0, 4.0), 1e-5)
    );
    assert!(child.get_global_rotation().abs_diff_eq(
        Quaternion::from_rotation_z(std::f32::consts::FRAC_PI_2) * Quaternion::from_rotation_y(0.5),
        1e-5
    ));
    assert_eq!(child.get_box(), vec3(2.0, 3.0, 4.0));
}

#[test]
fn dimensional_containers_reject_mixing_and_foreign_scenes() {
    let mut scene = Scene::new();
    let two = canvas_2d().resolution((64, 32)).build(&mut scene);
    let three = canvas_3d().resolution((64, 32)).build(&mut scene);
    let cube = cuboid().build(&mut scene);
    let rectangle = rect().build(&mut scene);
    for action in [0, 1] {
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if action == 0 {
                    two.add(&cube);
                } else {
                    three.add(&rectangle);
                }
            }))
            .is_err()
        );
    }
    let mut other = Scene::new();
    let foreign = rect().build(&mut other);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| two.add(&foreign))).is_err());
}

#[test]
fn canvases_require_explicit_resolution_and_camera_scope() {
    let mut scene = Scene::new();
    let missing = canvas_2d().build(&mut scene);
    let invalid = canvas_2d().resolution((0, 20)).build(&mut scene);
    let two = canvas_2d().resolution((1024, 512)).build(&mut scene);
    let three = canvas_3d().resolution((1920, 1080)).build(&mut scene);
    assert!(missing.validate().is_err());
    assert!(invalid.validate().is_err());
    assert!(two.validate().is_ok());
    assert!(three.validate().unwrap_err().contains("explicitly"));
    let camera = camera_3d().position(vec3(0.0, 0.0, 5.0)).build(&mut scene);
    three.set_camera(&camera);
    assert!(three.validate().unwrap_err().contains("subtree"));
    three.add(&camera);
    scene.add_canvas_3d(&three);
    assert!(three.validate().is_ok());
    let second = camera_3d().build(&mut scene);
    three.add(&second);
    assert_eq!(
        scene
            .get_world()
            .get::<&CanvasSettings>(three.get_id())
            .unwrap()
            .camera,
        Some(camera.get_id())
    );
    assert_eq!(
        scene
            .get_world()
            .get::<&CanvasSettings>(two.get_id())
            .unwrap()
            .resolution,
        (1024, 512)
    );
    let world = scene.get_world();
    let settings = world.get::<&CanvasSettings>(three.get_id()).unwrap();
    assert_eq!(settings.aspect_ratio().unwrap(), 1920.0 / 1080.0);
}

#[test]
fn projection_builder_sizes_the_plane_from_canvas_resolution() {
    let mut scene = Scene::new();
    let source = canvas_2d().resolution((1920, 1080)).build(&mut scene);
    let default_scale = projection().source(&source).build(&mut scene);
    let custom_after_source = projection()
        .source(&source)
        .pixels_per_unit(200.0)
        .build(&mut scene);
    let custom_before_source = projection()
        .pixels_per_unit(200.0)
        .source(&source)
        .build(&mut scene);
    let world = scene.get_world();

    assert_eq!(
        world
            .get::<&PlaneShape>(default_scale.get_id())
            .unwrap()
            .size,
        vec2(19.2, 10.8)
    );
    for projection in [custom_after_source, custom_before_source] {
        assert_eq!(
            world.get::<&PlaneShape>(projection.get_id()).unwrap().size,
            vec2(9.6, 5.4)
        );
    }
}

#[test]
fn camera_lens_rejects_invalid_ranges() {
    for lens in [
        Perspective {
            fov: 0.0,
            ..Default::default()
        },
        Perspective {
            fov: std::f32::consts::PI,
            ..Default::default()
        },
        Perspective {
            near: 0.0,
            ..Default::default()
        },
        Perspective {
            near: 10.0,
            far: 1.0,
            ..Default::default()
        },
        Perspective {
            far: f32::NAN,
            ..Default::default()
        },
    ] {
        assert!(lens.validate().is_err());
    }
}

#[test]
fn spatial_tracks_snapshots_and_lifetime_are_seekable_without_morph() {
    struct Setup(Option<CuboidHandler>);
    impl SceneBuilder for Setup {
        fn build(&mut self, scene: &mut Scene) {
            let canvas = canvas_3d().resolution((64, 64)).build(scene);
            let cube = cuboid().build(scene);
            canvas.add(&cube);
            scene.add_canvas_3d(&canvas);
            cube.save();
            cube.position(vec3(2.0, 4.0, 6.0))
                .duration(2.0)
                .easing(Easing::Linear)
                .play();
            cube.restore().duration(2.0).easing(Easing::Linear).play();
            cube.remove();
            self.0 = Some(cube);
        }
    }
    let mut scene = Scene::new();
    let mut setup = Setup(None);
    scene.build(&mut setup);
    let cube = setup.0.unwrap();
    scene.update(1.0);
    assert_eq!(
        cube.get(Transform3D::position_property()),
        vec3(1.0, 2.0, 3.0)
    );
    scene.update(3.0);
    assert_eq!(
        cube.get(Transform3D::position_property()),
        vec3(1.0, 2.0, 3.0)
    );
    scene.update(4.0);
    assert!(
        !active_subtree(&scene.get_world(), scene.get_root().get_id()).contains(&cube.get_id())
    );
    scene.update(0.0);
    assert!(active_subtree(&scene.get_world(), scene.get_root().get_id()).contains(&cube.get_id()));
    let world = scene.get_world();
    assert!(world.get::<&Morph>(cube.get_id()).is_err());
    assert!(!<Cuboid as Object>::MORPHABLE);
    let inspection = world.get::<&Inspection>(cube.get_id()).unwrap();
    assert!(
        (inspection.get)(&world, cube.get_id())
            .iter()
            .all(|c| c.name != "Morph")
    );
    assert_eq!(cube.get_name(), "Cuboid");
    assert!(scene.pick(Vector2::ZERO).is_none());
    let mut surface = skia_safe::surfaces::raster_n32_premul((32, 32)).unwrap();
    scene.draw(surface.canvas());
    scene.draw_outline(cube.get_id(), surface.canvas(), 1.0);
}

#[test]
fn projections_order_dependencies_and_reject_cycles_and_inactive_sources() {
    let mut scene = Scene::new_with_resolution((64, 64));
    let world = scene.get_world_3d();
    let source = canvas_2d().resolution((32, 16)).build(&mut scene);
    let camera = camera_3d().build(&mut scene);
    world.add(&camera);
    world.set_camera(&camera);
    let projection = projection().source(&source).build(&mut scene);
    world.add(&projection);
    scene.add_canvas_2d(&source);
    scene.get_root().view_2d(false).immediate();
    let order = canvas_order(&scene).unwrap();
    assert_eq!(order, vec![source.get_id(), world.get_id()]);
    let cyclic = std::collections::HashMap::from([
        (source.get_id(), vec![world.get_id()]),
        (world.get_id(), vec![source.get_id()]),
    ]);
    assert!(order_dependencies(&cyclic).unwrap_err().contains("cycle"));
    source.remove();
    scene.update(0.0);
    assert!(canvas_order(&scene).unwrap_err().contains("inactive"));
    projection.remove();
    scene.update(0.0);
    assert_eq!(canvas_order(&scene).unwrap(), vec![world.get_id()]);
}

#[test]
fn canvas2d_camera_is_scoped_and_identity_is_available() {
    let mut scene = Scene::new_with_resolution((32, 32));
    let source = scene.get_world_2d();
    let shape = rect()
        .size(vec2(4.0, 4.0))
        .fill(Color::RED)
        .build(&mut scene);
    source.add(&shape);
    let unrelated = camera_2d().position(vec2(200.0, 0.0)).build(&mut scene);
    let other = canvas_2d().resolution((32, 32)).build(&mut scene);
    other.add(&unrelated);
    scene.add_canvas_2d(&other);
    let mut surface = skia_safe::surfaces::raster_n32_premul((32, 32)).unwrap();
    draw_canvas2d(&scene.get_world(), source.get_id(), surface.canvas());
    assert_eq!(surface.peek_pixels().unwrap().get_color((16, 16)).r(), 255);
    assert_eq!(scene.pick(Vector2::ZERO), Some(shape.get_id()));
    assert!(scene.pick(vec2(10.0, 10.0)).is_none());
}

#[test]
fn root_view_track_switches_between_builtin_worlds() {
    struct SwitchView;
    impl SceneBuilder for SwitchView {
        fn build(&mut self, scene: &mut Scene) {
            scene.get_root().view_2d(false).immediate();
            scene.wait(2.0);
            scene.get_root().view_2d(true).immediate();
        }
    }

    let mut scene = Scene::new();
    let world_2d = scene.get_world_2d().get_texture();
    let world_3d = scene.get_world_3d().get_texture();
    scene.build(&mut SwitchView);

    scene.update(0.0);
    assert_eq!(scene.get_view(), world_3d);
    scene.update(2.0);
    assert_eq!(scene.get_view(), world_2d);
}
