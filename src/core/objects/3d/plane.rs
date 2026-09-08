use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::global_matrix3d,
    types::Vector2,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct PlaneShape {
    #[track]
    pub size: Vector2,
}

impl Default for PlaneShape {
    fn default() -> Self {
        Self { size: Vector2::ONE }
    }
}

/// Plane in local XY with its front facing positive Z.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "plane")]
pub struct Plane {
    #[trackable]
    pub shape: PlaneShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,

    pub draw: Draw3D,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            shape: PlaneShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_plane,
                get_box: |world, entity| {
                    world
                        .get::<&PlaneShape>(entity)
                        .unwrap()
                        .size
                        .abs()
                        .extend(0.0)
                },
            },
        }
    }
}

fn draw_plane(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&PlaneShape>(entity).unwrap();
    let size = (shape.size * 0.5).extend(1.0);
    validate_dimensions(size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(size);
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        GeometryKey::new::<PlaneShape>(0),
        three_d::CpuMesh::square,
        transformation,
        &material,
    )
}
