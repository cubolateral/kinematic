use kinematic::{hecs, prelude::*};

#[derive(Clone)]
struct CustomShape {
    size: Vector3,
}

#[derive(Object, hecs::Bundle)]
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
                get_box: |world, entity| world.get::<&CustomShape>(entity).unwrap().size,
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
    scene.get_world_3d().add(&custom);

    assert_eq!(custom.get_box(), vec3(2.0, 3.0, 4.0));
    assert_eq!(custom.get_global_position(), vec3(1.0, 2.0, 3.0));
    assert!(scene.get_world().get::<&Draw3D>(custom.get_id()).is_ok());
}
