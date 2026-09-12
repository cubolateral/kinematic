use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::{global_matrix3d, regular_polygon_vertices},
    types::Vector3,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct PrismShape {
    #[track]
    pub size: Vector3,
    #[track(min = 3, max = 256)]
    pub sides: u32,
    #[track]
    pub top_cap: bool,
    #[track]
    pub bottom_cap: bool,
}

impl Default for PrismShape {
    fn default() -> Self {
        Self {
            size: Vector3::ONE,
            sides: 4,
            top_cap: true,
            bottom_cap: true,
        }
    }
}

/// Regular-base prism centered on its local origin.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "prism")]
pub struct Prism {
    #[trackable]
    pub shape: PrismShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Prism {
    fn default() -> Self {
        Self {
            shape: PrismShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_prism,
                get_box: |world, entity| world.get::<&PrismShape>(entity).unwrap().size.abs(),
                ..Default::default()
            },
        }
    }
}

fn validate_sides(sides: u32) -> Result<(), String> {
    if !(3..=256).contains(&sides) {
        return Err("Prism sides must be between 3 and 256.".into());
    }
    Ok(())
}

fn geometry_variant(sides: u32, top_cap: bool, bottom_cap: bool) -> u64 {
    u64::from(sides) | (u64::from(top_cap) << 32) | (u64::from(bottom_cap) << 33)
}

fn prism_mesh(sides: u32, top_cap: bool, bottom_cap: bool) -> three_d::CpuMesh {
    let bottom: Vec<_> = regular_polygon_vertices(sides)
        .expect("Prism sides must be validated.")
        .into_iter()
        .map(|point| three_d::vec3(point.x, -0.5, point.y))
        .collect();
    let top: Vec<_> = bottom
        .iter()
        .map(|point| three_d::vec3(point.x, 0.5, point.z))
        .collect();
    let mut positions = Vec::with_capacity((sides as usize * 4 - 4) * 3);

    for index in 1..bottom.len() - 1 {
        if bottom_cap {
            positions.extend_from_slice(&[bottom[0], bottom[index], bottom[index + 1]]);
        }
        if top_cap {
            positions.extend_from_slice(&[top[0], top[index + 1], top[index]]);
        }
    }
    for index in 0..bottom.len() {
        let next = (index + 1) % bottom.len();
        positions.extend_from_slice(&[
            bottom[index],
            top[index],
            bottom[next],
            top[index],
            top[next],
            bottom[next],
        ]);
    }

    let mut mesh = three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}

fn draw_prism(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&PrismShape>(entity).unwrap();
    validate_sides(shape.sides)?;
    validate_dimensions(shape.size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(shape.size);
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        GeometryKey::new::<PrismShape>(geometry_variant(
            shape.sides,
            shape.top_cap,
            shape.bottom_cap,
        )),
        || prism_mesh(shape.sides, shape.top_cap, shape.bottom_cap),
        transformation,
        &material,
    )
}

/// Creates a three-sided [`Prism`] builder.
pub fn tetrahedron() -> PrismBuilder {
    prism().sides(3)
}

/// Creates a four-sided [`Prism`] builder.
pub fn cube() -> PrismBuilder {
    prism().sides(4)
}

/// Creates a 32-sided [`Prism`] builder.
pub fn cylinder() -> PrismBuilder {
    prism().sides(32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene,
        objects::{Object3DHandler, ObjectHandler},
        types::vec3,
    };

    #[test]
    fn prism_builder_sets_trackable_sides() {
        let mut scene = Scene::new();
        let prism = prism().size(vec3(2.0, 3.0, 4.0)).sides(3).build(&mut scene);

        assert_eq!(prism.get_box(), vec3(2.0, 3.0, 4.0));
        assert_eq!(prism.get(PrismShape::sides_property()), 3);
        assert!(prism.get(PrismShape::top_cap_property()));
        assert!(prism.get(PrismShape::bottom_cap_property()));
        assert_eq!(prism_mesh(3, true, true).triangle_count(), 8);
    }

    #[test]
    fn prism_defaults_to_four_sides_and_rejects_less_than_three() {
        assert_eq!(PrismShape::default().sides, 4);
        assert_eq!(prism_mesh(4, true, true).triangle_count(), 12);
        assert_eq!(prism_mesh(4, false, false).triangle_count(), 8);
        assert!(validate_sides(2).is_err());
        assert!(validate_sides(3).is_ok());
    }

    #[test]
    fn prism_presets_set_cube_and_cylinder_sides() {
        let mut scene = Scene::new();
        let cube = cube().build(&mut scene);
        let cylinder = cylinder().build(&mut scene);

        assert_eq!(cube.get(PrismShape::sides_property()), 4);
        assert_eq!(cylinder.get(PrismShape::sides_property()), 32);
    }
}
