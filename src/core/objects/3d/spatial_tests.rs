use crate::core::components::{Inspection, Morph};
use crate::core::objects::draw_canvas2d;
use crate::prelude::*;
use crate::renderer::plan::{active_subtree, canvas_order, order_dependencies, visible_subtree_3d};

#[test]
fn primitive_3d_builders_expose_outline_material_fields() {
    let mut scene = Scene::new();
    let color = Color::new(0.25, 0.5, 0.75, 0.8);
    let primitive = sphere()
        .outline_color(color)
        .outline_width(3.0)
        .build(&mut scene);

    assert_eq!(primitive.get_outline_color(), color);
    assert_eq!(primitive.get_outline_width(), 3.0);
}

#[test]
fn spatial_handlers_compose_transforms_and_bounds() {
    let mut scene = Scene::new();
    let canvas = canvas_3d().resolution((640, 480)).build(&mut scene);
    let parent = group_3d()
        .position(vec3(1.0, 2.0, 3.0))
        .rotation(Quaternion::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .scale(vec3(2.0, 3.0, 4.0))
        .build(&mut scene);
    let child = cube()
        .position(vec3(1.0, 0.0, 1.0))
        .rotation(Quaternion::from_rotation_y(0.5))
        .size(vec3(2.0, 3.0, 4.0))
        .build(&mut scene);
    parent.add(&child);
    canvas.add(&parent);
    scene.add_canvas_3d(&canvas);
    assert!(
        child
            .global_position()
            .abs_diff_eq(vec3(1.0, 4.0, 7.0), 1e-5)
    );
    assert!(child.global_scale().abs_diff_eq(vec3(2.0, 3.0, 4.0), 1e-5));
    assert!(child.global_rotation().abs_diff_eq(
        Quaternion::from_rotation_z(std::f32::consts::FRAC_PI_2) * Quaternion::from_rotation_y(0.5),
        1e-5
    ));
    assert_eq!(child.box_size(), vec3(2.0, 3.0, 4.0));
}

#[test]
fn group_origin_moves_3d_children_to_the_selected_edge() {
    let mut scene = Scene::new();
    let group = group_3d().origin(vec3(1.0, 0.0, 0.0)).build(&mut scene);
    let children = [-2.0, 0.0, 2.0].map(|x| {
        let child = cube()
            .size(vec3(2.0, 2.0, 2.0))
            .position(vec3(x, 0.0, 0.0))
            .build(&mut scene);
        group.add(&child);
        child
    });

    assert!(
        children[2]
            .global_position()
            .abs_diff_eq(vec3(-1.0, 0.0, 0.0), 1e-5)
    );
    assert_eq!(
        group.get(Transform3D::origin_property()),
        vec3(1.0, 0.0, 0.0)
    );
}

#[test]
fn container_visibility_hides_its_3d_subtree() {
    let mut scene = Scene::new();
    let canvas = canvas_3d().build(&mut scene);
    let parent = group_3d().visibility(false).build(&mut scene);
    let child = cube().build(&mut scene);
    parent.add(&child);
    canvas.add(&parent);
    scene.add_canvas_3d(&canvas);

    let world = scene.world();
    let mut visible = Vec::new();
    visible_subtree_3d(&world, canvas.entity(), &mut visible);
    assert!(visible.contains(&canvas.entity()));
    assert!(!visible.contains(&parent.entity()));
    assert!(!visible.contains(&child.entity()));
}

#[test]
fn dimensional_containers_reject_mixing_and_foreign_scenes() {
    let mut scene = Scene::new();
    let two = canvas_2d().resolution((64, 32)).build(&mut scene);
    let three = canvas_3d().resolution((64, 32)).build(&mut scene);
    let cube = cube().build(&mut scene);
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
fn canvases_require_explicit_resolution_and_own_their_cameras() {
    let mut scene = Scene::new();
    let missing = canvas_2d().build(&mut scene);
    let invalid = canvas_2d().resolution((0, 20)).build(&mut scene);
    let two = canvas_2d().resolution((1024, 512)).build(&mut scene);
    let three = canvas_3d().resolution((1920, 1080)).build(&mut scene);
    assert!(missing.validate().is_err());
    assert!(invalid.validate().is_err());
    assert!(two.validate().is_ok());
    assert!(three.validate().is_ok());
    scene.add_canvas_3d(&three);
    assert!(three.validate().is_ok());
    let world = scene.world();
    let camera = world.get::<&Camera3D>(three.entity()).unwrap();
    assert_eq!(camera.camera_position, vec3(0.0, 0.0, 3.0));
    assert!(world.get::<&Camera2D>(two.entity()).is_ok());
    assert_eq!(
        world
            .get::<&CanvasSettings>(two.entity())
            .unwrap()
            .resolution,
        (1024, 512)
    );
    let settings = world.get::<&CanvasSettings>(three.entity()).unwrap();
    assert_eq!(settings.aspect_ratio().unwrap(), 1920.0 / 1080.0);
}

#[test]
fn projection_3d_builder_sizes_the_plane_from_canvas_resolution() {
    let mut scene = Scene::new();
    let source = canvas_2d().resolution((1920, 1080)).build(&mut scene);
    let default_scale = projection_3d().source(&source).build(&mut scene);
    let custom_after_source = projection_3d()
        .source(&source)
        .pixels_per_unit(200.0)
        .build(&mut scene);
    let custom_before_source = projection_3d()
        .pixels_per_unit(200.0)
        .source(&source)
        .build(&mut scene);
    let world = scene.world();

    assert!(
        world
            .get::<&PlaneShape>(default_scale.entity())
            .unwrap()
            .size
            .abs_diff_eq(vec2(7.5, 4.21875), 1e-5)
    );
    for projection in [custom_after_source, custom_before_source] {
        assert!(
            world
                .get::<&PlaneShape>(projection.entity())
                .unwrap()
                .size
                .abs_diff_eq(vec2(9.6, 5.4), 1e-5)
        );
    }
}

#[test]
fn projection_2d_builder_sizes_the_rect_from_canvas_resolution() {
    let mut scene = Scene::new();
    let source = canvas_3d().resolution((960, 540)).build(&mut scene);
    let projection = projection_2d()
        .source(&source)
        .round([10, 20, 30, 40])
        .stroke(Color::WHITE)
        .stroke_width(4.0)
        .build(&mut scene);
    let world = scene.world();

    assert_eq!(
        world.get::<&RectShape>(projection.entity()).unwrap().size,
        vec2(960.0, 540.0)
    );
    assert_eq!(
        world.get::<&Style>(projection.entity()).unwrap().fill,
        Color::TRANSPARENT
    );
    assert_eq!(
        world.get::<&RectShape>(projection.entity()).unwrap().round,
        Quad::new(10.0, 20.0, 30.0, 40.0)
    );
}

#[test]
fn camera_lens_rejects_invalid_ranges() {
    for lens in [
        Camera3D {
            camera_fov: 0.0,
            ..Default::default()
        },
        Camera3D {
            camera_fov: std::f32::consts::PI,
            ..Default::default()
        },
        Camera3D {
            camera_near: 0.0,
            ..Default::default()
        },
        Camera3D {
            camera_near: 10.0,
            camera_far: 1.0,
            ..Default::default()
        },
        Camera3D {
            camera_far: f32::NAN,
            ..Default::default()
        },
    ] {
        assert!(lens.validate().is_err());
    }
}

#[test]
fn spatial_tracks_snapshots_and_lifetime_are_seekable_without_morph() {
    struct Setup(Option<PrismHandler>);
    impl SceneBuilder for Setup {
        fn build(&mut self, scene: &mut Scene) {
            let canvas = canvas_3d().resolution((64, 64)).build(scene);
            let cube = cube().build(scene);
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
    assert!(!active_subtree(&scene.world(), scene.root().entity()).contains(&cube.entity()));
    scene.update(0.0);
    assert!(active_subtree(&scene.world(), scene.root().entity()).contains(&cube.entity()));
    let world = scene.world();
    assert!(world.get::<&Morph>(cube.entity()).is_err());
    assert!(!<Prism as Object>::SPATIAL_2D);
    let inspection = world.get::<&Inspection>(cube.entity()).unwrap();
    assert!(
        (inspection.get)(&world, cube.entity())
            .iter()
            .all(|c| c.name != "Morph")
    );
    assert_eq!(cube.name(), "Prism");
    assert!(scene.pick(Vector2::ZERO).is_none());
    let mut surface = skia_safe::surfaces::raster_n32_premul((32, 32)).unwrap();
    scene.draw(surface.canvas());
    assert!(scene.selection_outline(cube.entity()).is_none());
}

#[test]
fn projections_order_dependencies_and_reject_cycles_and_inactive_sources() {
    let mut scene = Scene::new_with_resolution((64, 64));
    let world = scene.world_3d();
    let source = canvas_2d().resolution((32, 16)).build(&mut scene);
    let projection = projection_3d().source(&source).build(&mut scene);
    world.add(&projection);
    scene.add_canvas_2d(&source);
    scene.root().view_2d(false).immediate();
    let order = canvas_order(&scene).unwrap();
    assert_eq!(order, vec![source.entity(), world.entity()]);
    let cyclic = std::collections::HashMap::from([
        (source.entity(), vec![world.entity()]),
        (world.entity(), vec![source.entity()]),
    ]);
    assert!(order_dependencies(&cyclic).unwrap_err().contains("cycle"));
    source.remove();
    scene.update(0.0);
    assert!(canvas_order(&scene).unwrap_err().contains("inactive"));
    projection.remove();
    scene.update(0.0);
    assert_eq!(canvas_order(&scene).unwrap(), vec![world.entity()]);
}

#[test]
fn projection_2d_orders_its_canvas_3d_dependency() {
    let mut scene = Scene::new_with_resolution((64, 64));
    let world_2d = scene.world_2d();
    let world_3d = scene.world_3d();
    let projection = projection_2d().source(&world_3d).build(&mut scene);
    world_2d.add(&projection);
    scene.update(0.0);

    assert_eq!(
        canvas_order(&scene).unwrap(),
        vec![world_3d.entity(), world_2d.entity()]
    );
}

#[test]
fn projections_accept_sources_with_the_same_dimension() {
    let mut scene_2d = Scene::new_with_resolution((64, 64));
    let output_2d = scene_2d.world_2d();
    let source_2d = canvas_2d().resolution((32, 32)).build(&mut scene_2d);
    scene_2d.add_canvas_2d(&source_2d);
    let projection_2d = projection_2d().source(&source_2d).build(&mut scene_2d);
    output_2d.add(&projection_2d);
    scene_2d.update(0.0);
    assert_eq!(
        canvas_order(&scene_2d).unwrap(),
        vec![source_2d.entity(), output_2d.entity()]
    );

    let mut scene_3d = Scene::new_with_resolution((64, 64));
    let output_3d = scene_3d.world_3d();
    let source_3d = canvas_3d().resolution((32, 32)).build(&mut scene_3d);
    scene_3d.add_canvas_3d(&source_3d);
    let projection_3d = projection_3d().source(&source_3d).build(&mut scene_3d);
    output_3d.add(&projection_3d);
    scene_3d.root().view_2d(false).immediate();
    scene_3d.update(0.0);
    assert_eq!(
        canvas_order(&scene_3d).unwrap(),
        vec![source_3d.entity(), output_3d.entity()]
    );
}

#[test]
fn canvas2d_camera_is_scoped_to_its_canvas() {
    let mut scene = Scene::new_with_resolution((32, 32));
    let source = scene.world_2d();
    let shape = rect()
        .size(vec2(4.0, 4.0))
        .position(vec2(10.0, 0.0))
        .fill(Color::RED)
        .build(&mut scene);
    source.add(&shape);
    source.camera_position(vec2(10.0, 0.0)).immediate();
    let other = canvas_2d()
        .resolution((32, 32))
        .camera_position(vec2(200.0, 0.0))
        .build(&mut scene);
    scene.add_canvas_2d(&other);
    let mut surface = skia_safe::surfaces::raster_n32_premul((32, 32)).unwrap();
    draw_canvas2d(&scene.world(), source.entity(), surface.canvas());
    assert_eq!(surface.peek_pixels().unwrap().get_color((16, 16)).r(), 255);
    assert_eq!(scene.pick(Vector2::ZERO), Some(shape.entity()));
    assert!(scene.pick(vec2(10.0, 10.0)).is_none());
}

#[test]
fn objects_can_ignore_the_2d_camera_with_their_subtree() {
    let mut scene = Scene::new_with_resolution((32, 32));
    let canvas = scene.world_2d();
    let parent = group_2d()
        .position(vec2(4.0, 0.0))
        .opacity(0.5)
        .build(&mut scene);
    let fixed = group_2d()
        .position(vec2(2.0, 0.0))
        .follows_camera(false)
        .build(&mut scene);
    let child = rect()
        .size(vec2(4.0, 4.0))
        .position(vec2(1.0, 0.0))
        .fill(Color::RED)
        .build(&mut scene);
    fixed.add(&child);
    parent.add(&fixed);
    canvas.add(&parent);
    canvas.camera_position(vec2(10.0, 0.0)).immediate();

    let mut surface = skia_safe::surfaces::raster_n32_premul((32, 32)).unwrap();
    draw_canvas2d(&scene.world(), canvas.entity(), surface.canvas());
    let pixels = surface.peek_pixels().unwrap();

    assert_eq!(pixels.get_color((23, 16)).r(), 255);
    assert!((127..=128).contains(&pixels.get_color((23, 16)).a()));
    assert_eq!(pixels.get_color((13, 16)).a(), 0);
    assert_eq!(scene.pick(vec2(7.0, 0.0)), Some(child.entity()));
    assert_ne!(scene.pick(vec2(-3.0, 0.0)), Some(child.entity()));
    assert_eq!(
        scene.selection_outline(child.entity()).unwrap()[0][0][0],
        0.65625
    );
}

#[test]
fn canvas_camera_tracks_use_the_camera_prefix() {
    let mut scene = Scene::new();
    let two = canvas_2d()
        .resolution((32, 32))
        .camera_position(vec2(1.0, 2.0))
        .camera_zoom(2.0)
        .camera_rotation(0.5)
        .build(&mut scene);
    let three = canvas_3d()
        .resolution((32, 32))
        .camera_position(vec3(1.0, 2.0, 4.0))
        .camera_fov(1.0)
        .camera_near(0.2)
        .camera_far(200.0)
        .build(&mut scene);
    let world = scene.world();

    let camera_2d = world.get::<&Camera2D>(two.entity()).unwrap();
    assert_eq!(camera_2d.camera_position, vec2(1.0, 2.0));
    assert_eq!(camera_2d.camera_zoom, 2.0);
    assert_eq!(camera_2d.camera_rotation, 0.5);

    let camera_3d = world.get::<&Camera3D>(three.entity()).unwrap();
    assert_eq!(camera_3d.camera_position, vec3(1.0, 2.0, 4.0));
    assert_eq!(camera_3d.camera_fov, 1.0);
    assert_eq!(camera_3d.camera_near, 0.2);
    assert_eq!(camera_3d.camera_far, 200.0);

    for entity in [two.entity(), three.entity()] {
        let inspection = world.get::<&Inspection>(entity).unwrap();
        let camera = (inspection.get)(&world, entity)
            .into_iter()
            .find(|component| component.name == "Camera2D" || component.name == "Camera3D")
            .unwrap();
        assert!(
            (camera.get)()
                .iter()
                .all(|track| track.name.starts_with("camera_"))
        );
    }
}

#[test]
fn root_view_track_switches_between_builtin_worlds() {
    struct SwitchView;
    impl SceneBuilder for SwitchView {
        fn build(&mut self, scene: &mut Scene) {
            scene.root().view_2d(false).immediate();
            scene.wait(2.0);
            scene.root().view_2d(true).immediate();
        }
    }

    let mut scene = Scene::new();
    let world_2d = scene.world_2d().texture();
    let world_3d = scene.world_3d().texture();
    scene.build(&mut SwitchView);

    scene.update(0.0);
    assert_eq!(scene.view_texture(), world_3d);
    scene.update(2.0);
    assert_eq!(scene.view_texture(), world_2d);
}
