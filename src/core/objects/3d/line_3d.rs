use crate::core::{
    components::{Draw3D, GeometryKey, Material, RenderContext3D, Transform3D},
    objects::global_matrix3d,
    types::{Vector3, vec3},
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
/// Geometry of a 3D line, including optional arrowheads at either end.
pub struct Line3DShape {
    /// Starting point in local coordinates.
    #[track]
    pub from: Vector3,
    /// Ending point in local coordinates.
    #[track]
    pub to: Vector3,
    /// Diameter of the line body.
    #[track]
    pub thickness: f32,
    /// Length of the arrowhead at the starting point.
    #[track]
    pub from_arrow_size: f32,
    /// Length of the arrowhead at the ending point.
    #[track]
    pub to_arrow_size: f32,
    /// Number of sides around the line body.
    #[track]
    pub line_sides: u32,
    /// Number of sides around each arrowhead.
    #[track]
    pub arrow_sides: u32,
}

impl Default for Line3DShape {
    fn default() -> Self {
        Self {
            from: vec3(-0.5, 0.0, 0.0),
            to: vec3(0.5, 0.0, 0.0),
            thickness: 0.05,
            from_arrow_size: 0.0,
            to_arrow_size: 0.0,
            line_sides: 32,
            arrow_sides: 32,
        }
    }
}

struct Line3DGeometry {
    direction: Vector3,
    body_from: Vector3,
    body_to: Vector3,
    body_radius: f32,
    from_length: f32,
    from_radius: f32,
    to_length: f32,
    to_radius: f32,
}

fn line_geometry(shape: &Line3DShape) -> Result<Option<Line3DGeometry>, String> {
    if !shape.from.is_finite()
        || !shape.to.is_finite()
        || !shape.thickness.is_finite()
        || !shape.from_arrow_size.is_finite()
        || !shape.to_arrow_size.is_finite()
    {
        return Err("Line dimensions must be finite.".into());
    }
    if !(3..=256).contains(&shape.line_sides) {
        return Err("Line sides must be between 3 and 256.".into());
    }
    if !(3..=256).contains(&shape.arrow_sides) {
        return Err("Arrow sides must be between 3 and 256.".into());
    }

    let offset = shape.to - shape.from;
    let length = offset.length();
    let thickness = shape.thickness.max(0.0);

    if !length.is_finite() || length <= f32::EPSILON || thickness <= 0.0 {
        return Ok(None);
    }

    let direction = offset / length;
    let body_radius = thickness * 0.5;
    let from_arrow_size = shape.from_arrow_size.max(0.0);
    let to_arrow_size = shape.to_arrow_size.max(0.0);
    let requested_arrow_length = from_arrow_size + to_arrow_size;
    let arrow_scale = if requested_arrow_length > length {
        length / requested_arrow_length
    } else {
        1.0
    };
    let from_length = from_arrow_size * arrow_scale;
    let to_length = to_arrow_size * arrow_scale;

    Ok(Some(Line3DGeometry {
        direction,
        body_from: shape.from + direction * from_length,
        body_to: shape.to - direction * to_length,
        body_radius,
        from_length,
        from_radius: (from_length * 0.5).max(body_radius),
        to_length,
        to_radius: (to_length * 0.5).max(body_radius),
    }))
}

fn segment_transform(from: Vector3, direction: Vector3, length: f32, radius: f32) -> glam::Mat4 {
    glam::Mat4::from_translation(from)
        * glam::Mat4::from_quat(glam::Quat::from_rotation_arc(Vector3::X, direction))
        * glam::Mat4::from_scale(vec3(length, radius, radius))
}

fn geometry_variant(sides: u32, arrow: bool) -> u64 {
    u64::from(sides) | (u64::from(arrow) << 32)
}

fn closed_cylinder_mesh(sides: u32) -> three_d::CpuMesh {
    let mut positions = Vec::with_capacity(sides as usize * 12);
    let start_center = three_d::vec3(0.0, 0.0, 0.0);
    let end_center = three_d::vec3(1.0, 0.0, 0.0);

    for side in 0..sides {
        let angle = std::f32::consts::TAU * side as f32 / sides as f32;
        let next_angle = std::f32::consts::TAU * (side + 1) as f32 / sides as f32;
        let start = three_d::vec3(0.0, angle.cos(), angle.sin());
        let next_start = three_d::vec3(0.0, next_angle.cos(), next_angle.sin());
        let end = three_d::vec3(1.0, angle.cos(), angle.sin());
        let next_end = three_d::vec3(1.0, next_angle.cos(), next_angle.sin());

        positions.extend_from_slice(&[
            start,
            next_start,
            next_end,
            start,
            next_end,
            end,
            start_center,
            next_start,
            start,
            end_center,
            end,
            next_end,
        ]);
    }

    let mut mesh = three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}

fn closed_cone_mesh(sides: u32) -> three_d::CpuMesh {
    let mut positions = Vec::with_capacity(sides as usize * 6);
    let base_center = three_d::vec3(0.0, 0.0, 0.0);
    let apex = three_d::vec3(1.0, 0.0, 0.0);

    for side in 0..sides {
        let angle = std::f32::consts::TAU * side as f32 / sides as f32;
        let next_angle = std::f32::consts::TAU * (side + 1) as f32 / sides as f32;
        let base = three_d::vec3(0.0, angle.cos(), angle.sin());
        let next_base = three_d::vec3(0.0, next_angle.cos(), next_angle.sin());

        positions.extend_from_slice(&[base, next_base, apex, base_center, next_base, base]);
    }

    let mut mesh = three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}

fn line_box(shape: &Line3DShape) -> Vector3 {
    let Ok(Some(geometry)) = line_geometry(shape) else {
        return Vector3::ZERO;
    };
    let mut minimum = shape.from
        - Vector3::splat(if geometry.from_length > 0.0 {
            0.0
        } else {
            geometry.body_radius
        });
    let mut maximum = shape.from
        + Vector3::splat(if geometry.from_length > 0.0 {
            0.0
        } else {
            geometry.body_radius
        });

    for (point, radius) in [
        (geometry.body_from, geometry.from_radius),
        (geometry.body_to, geometry.to_radius),
        (
            shape.to,
            if geometry.to_length > 0.0 {
                0.0
            } else {
                geometry.body_radius
            },
        ),
    ] {
        minimum = minimum.min(point - Vector3::splat(radius));
        maximum = maximum.max(point + Vector3::splat(radius));
    }

    minimum.abs().max(maximum.abs()) * 2.0
}

/// Built-in 3D line scene object with independently animatable arrowheads.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "line_3d")]
pub struct Line3D {
    #[trackable]
    pub shape: Line3DShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Line3D {
    fn default() -> Self {
        Self {
            shape: Line3DShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_line_3d,
                get_box: |world, entity| line_box(&world.get::<&Line3DShape>(entity).unwrap()),
                ..Default::default()
            },
        }
    }
}

fn draw_line_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&Line3DShape>(entity).unwrap();
    let Some(geometry) = line_geometry(&shape)? else {
        return Ok(());
    };
    let global = global_matrix3d(world, entity);
    let material = world.get::<&Material>(entity).unwrap();
    let body_length = (geometry.body_to - geometry.body_from).length();

    if body_length > f32::EPSILON {
        context.render_material(
            GeometryKey::new::<Line3DShape>(geometry_variant(shape.line_sides, false)),
            || closed_cylinder_mesh(shape.line_sides),
            global
                * segment_transform(
                    geometry.body_from,
                    geometry.direction,
                    body_length,
                    geometry.body_radius,
                ),
            &material,
        )?;
    }
    if geometry.from_length > 0.0 {
        context.render_material(
            GeometryKey::new::<Line3DShape>(geometry_variant(shape.arrow_sides, true)),
            || closed_cone_mesh(shape.arrow_sides),
            global
                * segment_transform(
                    geometry.body_from,
                    -geometry.direction,
                    geometry.from_length,
                    geometry.from_radius,
                ),
            &material,
        )?;
    }
    if geometry.to_length > 0.0 {
        context.render_material(
            GeometryKey::new::<Line3DShape>(geometry_variant(shape.arrow_sides, true)),
            || closed_cone_mesh(shape.arrow_sides),
            global
                * segment_transform(
                    geometry.body_to,
                    geometry.direction,
                    geometry.to_length,
                    geometry.to_radius,
                ),
            &material,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Scene, objects::ObjectHandler};

    #[test]
    fn line_3d_builder_builds_the_configured_line() {
        let mut scene = Scene::new();
        let line = line_3d()
            .from(vec3(-1.0, 2.0, 3.0))
            .to(vec3(4.0, 5.0, 6.0))
            .thickness(0.2)
            .from_arrow_size(0.3)
            .to_arrow_size(0.4)
            .line_sides(12)
            .arrow_sides(8)
            .build(&mut scene);
        let world = scene.get_world();
        let shape = world.get::<&Line3DShape>(line.get_id()).unwrap();

        assert_eq!(shape.from, vec3(-1.0, 2.0, 3.0));
        assert_eq!(shape.to, vec3(4.0, 5.0, 6.0));
        assert_eq!(shape.thickness, 0.2);
        assert_eq!(shape.from_arrow_size, 0.3);
        assert_eq!(shape.to_arrow_size, 0.4);
        assert_eq!(shape.line_sides, 12);
        assert_eq!(shape.arrow_sides, 8);
    }

    #[test]
    fn arrowheads_are_scaled_to_fit_the_line() {
        let shape = Line3DShape {
            from: Vector3::ZERO,
            to: Vector3::X,
            from_arrow_size: 0.75,
            to_arrow_size: 0.75,
            ..Default::default()
        };
        let geometry = line_geometry(&shape).unwrap().unwrap();

        assert_eq!(geometry.from_length, 0.5);
        assert_eq!(geometry.to_length, 0.5);
        assert!((geometry.body_from - geometry.body_to).length() <= f32::EPSILON);
    }

    #[test]
    fn line_meshes_include_end_caps() {
        assert_eq!(closed_cylinder_mesh(32).triangle_count(), 128);
        assert_eq!(closed_cone_mesh(32).triangle_count(), 64);
    }

    #[test]
    fn line_sides_are_tracks_with_circular_defaults() {
        let shape = Line3DShape::default();

        assert_eq!(shape.line_sides, 32);
        assert_eq!(shape.arrow_sides, 32);
        let mut scene = Scene::new();
        let line = line_3d().build(&mut scene);
        assert_eq!(line.get(Line3DShape::line_sides_property()), 32);
        assert_eq!(line.get(Line3DShape::arrow_sides_property()), 32);
    }
}
