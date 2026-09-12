use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::{global_matrix3d, regular_polygon_vertices},
    types::Vector3,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct PyramidShape {
    #[track]
    pub size: Vector3,
    #[track(min = 3, max = 256)]
    pub sides: u32,
    #[track]
    pub cap: bool,
}

impl Default for PyramidShape {
    fn default() -> Self {
        Self {
            size: Vector3::ONE,
            sides: 4,
            cap: true,
        }
    }
}

/// Regular-base pyramid centered on its local origin with its apex facing positive Y.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "pyramid")]
pub struct Pyramid {
    #[trackable]
    pub shape: PyramidShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Pyramid {
    fn default() -> Self {
        Self {
            shape: PyramidShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_pyramid,
                get_box: |world, entity| world.get::<&PyramidShape>(entity).unwrap().size.abs(),
                ..Default::default()
            },
        }
    }
}

fn validate_sides(sides: u32) -> Result<(), String> {
    if !(3..=256).contains(&sides) {
        return Err("Pyramid sides must be between 3 and 256.".into());
    }
    Ok(())
}

fn geometry_variant(sides: u32, cap: bool) -> u64 {
    u64::from(sides) | (u64::from(cap) << 32)
}

fn pyramid_mesh(sides: u32, bottom_cap: bool) -> three_d::CpuMesh {
    let base: Vec<_> = regular_polygon_vertices(sides)
        .expect("Pyramid sides must be validated.")
        .into_iter()
        .map(|point| three_d::vec3(point.x, -0.5, point.y))
        .collect();
    let apex = three_d::vec3(0.0, 0.5, 0.0);
    let mut positions = Vec::with_capacity((sides as usize * 2 - 2) * 3);

    if bottom_cap {
        for index in 1..base.len() - 1 {
            positions.extend_from_slice(&[base[0], base[index], base[index + 1]]);
        }
    }
    for index in 0..base.len() {
        positions.extend_from_slice(&[base[index], apex, base[(index + 1) % base.len()]]);
    }

    let mut mesh = three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}

fn draw_pyramid(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&PyramidShape>(entity).unwrap();
    validate_sides(shape.sides)?;
    validate_dimensions(shape.size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(shape.size);
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        GeometryKey::new::<PyramidShape>(geometry_variant(shape.sides, shape.cap)),
        || pyramid_mesh(shape.sides, shape.cap),
        transformation,
        &material,
    )
}

/// Creates a 32-sided [`Pyramid`] builder.
pub fn cone() -> PyramidBuilder {
    pyramid().sides(32)
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
    fn pyramid_builder_sets_its_size() {
        let mut scene = Scene::new();
        let pyramid = pyramid()
            .size(vec3(2.0, 3.0, 4.0))
            .sides(3)
            .build(&mut scene);

        assert_eq!(pyramid.get_box(), vec3(2.0, 3.0, 4.0));
        assert_eq!(pyramid.get(PyramidShape::sides_property()), 3);
        assert!(pyramid.get(PyramidShape::cap_property()));
        assert_eq!(
            scene
                .get_world()
                .get::<&PyramidShape>(pyramid.get_id())
                .unwrap()
                .sides,
            3
        );
        assert_eq!(pyramid_mesh(3, true).triangle_count(), 4);
    }

    #[test]
    fn pyramid_defaults_to_four_sides_and_rejects_less_than_three() {
        assert_eq!(PyramidShape::default().sides, 4);
        assert_eq!(pyramid_mesh(4, false).triangle_count(), 4);
        assert!(validate_sides(2).is_err());
        assert!(validate_sides(3).is_ok());
    }

    #[test]
    fn cone_preset_uses_32_sides() {
        let mut scene = Scene::new();
        let cone = cone().build(&mut scene);

        assert_eq!(cone.get(PyramidShape::sides_property()), 32);
    }
}
