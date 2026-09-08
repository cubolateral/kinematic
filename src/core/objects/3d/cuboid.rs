use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::global_matrix3d,
    types::Vector3,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct CuboidShape {
    #[track]
    pub size: Vector3,
}

impl Default for CuboidShape {
    fn default() -> Self {
        Self { size: Vector3::ONE }
    }
}

/// Cuboid centered on its local origin.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "cuboid")]
pub struct Cuboid {
    #[trackable]
    pub shape: CuboidShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Cuboid {
    fn default() -> Self {
        Self {
            shape: CuboidShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_cuboid,
                get_box: |world, entity| world.get::<&CuboidShape>(entity).unwrap().size.abs(),
                ..Default::default()
            },
        }
    }
}

fn draw_cuboid(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&CuboidShape>(entity).unwrap();
    validate_dimensions(shape.size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(shape.size * 0.5);
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        GeometryKey::new::<CuboidShape>(0),
        three_d::CpuMesh::cube,
        transformation,
        &material,
    )
}
