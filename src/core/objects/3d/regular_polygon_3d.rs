use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::{global_matrix3d, regular_polygon_vertices},
    types::Vector2,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
/// Geometry of a regular polygon in local XY.
pub struct RegularPolygon3DShape {
    #[track]
    pub size: Vector2,
    #[track]
    pub sides: u32,
}

impl Default for RegularPolygon3DShape {
    fn default() -> Self {
        Self {
            size: Vector2::ONE,
            sides: 4,
        }
    }
}

fn validate_sides(sides: u32) -> Result<(), String> {
    if !(3..=256).contains(&sides) {
        return Err("Regular polygon sides must be between 3 and 256.".into());
    }
    Ok(())
}

fn polygon_mesh(sides: u32) -> three_d::CpuMesh {
    let points: Vec<_> = regular_polygon_vertices(sides)
        .expect("Polygon mesh sides must be validated.")
        .into_iter()
        .map(|point| three_d::vec3(point.x, point.y, 0.0))
        .collect();
    let mut positions = Vec::with_capacity((sides as usize - 2) * 3);

    for index in 1..points.len() - 1 {
        positions.extend_from_slice(&[points[0], points[index], points[index + 1]]);
    }

    let mut mesh = three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}

/// Flat regular polygon with its front facing positive Z.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "regular_polygon_3d")]
pub struct RegularPolygon3D {
    #[trackable]
    pub shape: RegularPolygon3DShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for RegularPolygon3D {
    fn default() -> Self {
        Self {
            shape: RegularPolygon3DShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_regular_polygon_3d,
                get_box: |world, entity| {
                    world
                        .get::<&RegularPolygon3DShape>(entity)
                        .unwrap()
                        .size
                        .abs()
                        .extend(0.0)
                },
                ..Default::default()
            },
        }
    }
}

fn draw_regular_polygon_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&RegularPolygon3DShape>(entity).unwrap();
    validate_sides(shape.sides)?;
    validate_dimensions(shape.size.extend(0.0))?;
    let transformation =
        global_matrix3d(world, entity) * glam::Mat4::from_scale(shape.size.extend(1.0));
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        GeometryKey::new::<RegularPolygon3DShape>(u64::from(shape.sides)),
        || polygon_mesh(shape.sides),
        transformation,
        &material,
    )
}

/// Creates a three-sided [`RegularPolygon3D`] builder.
pub fn triangle_3d() -> RegularPolygon3DBuilder {
    regular_polygon_3d().sides(3)
}

/// Creates a four-sided [`RegularPolygon3D`] builder.
pub fn square_3d() -> RegularPolygon3DBuilder {
    regular_polygon_3d().sides(4)
}

/// Creates a 32-sided [`RegularPolygon3D`] builder.
pub fn circle_3d() -> RegularPolygon3DBuilder {
    regular_polygon_3d().sides(32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene,
        objects::{Object3DHandler, ObjectHandler},
        types::vec2,
    };

    #[test]
    fn regular_polygon_3d_builder_sets_trackable_sides() {
        let mut scene = Scene::new();
        let polygon = regular_polygon_3d()
            .size(vec2(2.0, 3.0))
            .sides(3)
            .build(&mut scene);

        assert_eq!(polygon.get_box(), vec2(2.0, 3.0).extend(0.0));
        assert_eq!(polygon.get(RegularPolygon3DShape::sides_property()), 3);
        assert_eq!(polygon_mesh(3).triangle_count(), 1);
    }

    #[test]
    fn regular_polygon_3d_defaults_to_four_sides_and_rejects_less_than_three() {
        assert_eq!(RegularPolygon3DShape::default().sides, 4);
        assert_eq!(polygon_mesh(4).triangle_count(), 2);
        assert!(validate_sides(2).is_err());
        assert!(validate_sides(3).is_ok());
    }

    #[test]
    fn polygon_3d_presets_set_triangle_and_square_sides() {
        let mut scene = Scene::new();
        let triangle = triangle_3d().build(&mut scene);
        let square = square_3d().build(&mut scene);

        assert_eq!(triangle.get(RegularPolygon3DShape::sides_property()), 3);
        assert_eq!(square.get(RegularPolygon3DShape::sides_property()), 4);
    }
}
