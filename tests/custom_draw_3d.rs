use kinematic::{hecs, prelude::*};

#[derive(Clone)]
struct CustomShape {
    size: Vector3,
}

#[derive(Object)]
#[object(spatial = "3d", builder = "custom_mesh")]
struct CustomMesh {
    #[trackable]
    transform: Transform3D,

    shape: CustomShape,
    draw: Draw3D,
}

impl Default for CustomMesh {
    fn default() -> Self {
        Self {
            transform: Transform3D::default(),
            shape: CustomShape {
                size: vec3(2.0, 3.0, 4.0),
            },
            draw: Draw3D {
                on_draw: draw_custom_mesh,
                box_size: |world, entity| world.get::<&CustomShape>(entity).unwrap().size,
                ..Default::default()
            },
        }
    }
}

fn draw_custom_mesh(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let transformation = global_matrix3d(world, entity);
    let material = Material {
        unlit: true,
        ..Default::default()
    };
    context.render_material(
        GeometryKey::new::<CustomShape>(0),
        three_d::CpuMesh::cube,
        transformation,
        &material,
    )
}

#[test]
fn custom_3d_object_uses_the_public_draw_contract() {
    let mut scene = Scene::new();
    let custom = custom_mesh()
        .position(vec3(1.0, 2.0, 3.0))
        .build(&mut scene);
    scene.world_3d().add(&custom);

    assert_eq!(custom.get_position_z(), 3.0);
    custom.set_position_z(4.0);
    assert_eq!(custom.get_position_z(), 4.0);
    assert_eq!(custom.box_size(), vec3(2.0, 3.0, 4.0));
    assert_eq!(custom.global_position(), vec3(1.0, 2.0, 4.0));
    assert!(scene.world().get::<&Draw3D>(custom.entity()).is_ok());
}
