use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::global_matrix3d,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct SphereShape {
    #[track]
    pub radius: f32,
    pub segments: u32,
}

impl Default for SphereShape {
    fn default() -> Self {
        Self {
            radius: 0.5,
            segments: 32,
        }
    }
}

/// Sphere centered on its local origin.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "sphere")]
pub struct Sphere {
    #[trackable]
    pub shape: SphereShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            shape: SphereShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_sphere,
                get_box: |world, entity| {
                    glam::Vec3::splat(world.get::<&SphereShape>(entity).unwrap().radius.abs() * 2.0)
                },
                ..Default::default()
            },
        }
    }
}

fn draw_sphere(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&SphereShape>(entity).unwrap();
    if !(3..=256).contains(&shape.segments) {
        return Err("Sphere segments must be between 3 and 256.".into());
    }
    let size = glam::Vec3::splat(shape.radius);
    validate_dimensions(size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(size);
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        GeometryKey::new::<SphereShape>(u64::from(shape.segments)),
        || three_d::CpuMesh::sphere(shape.segments),
        transformation,
        &material,
    )
}
