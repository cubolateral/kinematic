use std::hash::{DefaultHasher, Hash, Hasher};

use crate::core::{
    components::{
        Draw3D, GeometryKey, Material, RenderContext3D, Transform3D, validate_dimensions,
    },
    objects::global_matrix3d,
};
use kinematic_macros::{Object, Trackable};

#[derive(Clone, Trackable)]
pub struct TorusShape {
    #[track(min = 0.0)]
    pub major_radius: f32,
    #[track(min = 0.0)]
    pub minor_radius: f32,
    #[track(min = 3, max = 256)]
    pub major_segments: u32,
    #[track(min = 3, max = 256)]
    pub minor_segments: u32,
}

impl Default for TorusShape {
    fn default() -> Self {
        Self {
            major_radius: 0.375,
            minor_radius: 0.125,
            major_segments: 32,
            minor_segments: 16,
        }
    }
}

impl TorusShape {
    fn geometry_key(&self) -> GeometryKey {
        let mut hash = DefaultHasher::new();
        self.major_radius.to_bits().hash(&mut hash);
        self.minor_radius.to_bits().hash(&mut hash);
        self.major_segments.hash(&mut hash);
        self.minor_segments.hash(&mut hash);
        GeometryKey::new::<Self>(hash.finish())
    }
}

/// Torus centered on its local origin with its ring around the Y axis.
#[derive(Object)]
#[object(spatial = "3d", builder = "torus")]
pub struct Torus {
    #[trackable]
    pub shape: TorusShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Torus {
    fn default() -> Self {
        Self {
            shape: TorusShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_torus,
                box_size: |world, entity| {
                    let shape = world.get::<&TorusShape>(entity).unwrap();
                    glam::vec3(
                        2.0 * (shape.major_radius + shape.minor_radius),
                        2.0 * shape.minor_radius,
                        2.0 * (shape.major_radius + shape.minor_radius),
                    )
                },
                ..Default::default()
            },
        }
    }
}

fn validate_segments(major: u32, minor: u32) -> Result<(), String> {
    if !(3..=256).contains(&major) {
        return Err("Torus major segments must be between 3 and 256.".into());
    }
    if !(3..=256).contains(&minor) {
        return Err("Torus minor segments must be between 3 and 256.".into());
    }
    Ok(())
}

fn torus_mesh(major_radius: f32, minor_radius: f32, major: u32, minor: u32) -> three_d::CpuMesh {
    let mut positions = Vec::with_capacity(major as usize * minor as usize);
    let mut indices = Vec::with_capacity(major as usize * minor as usize * 6);
    let major_step = std::f32::consts::TAU / major as f32;
    let minor_step = std::f32::consts::TAU / minor as f32;

    for major_index in 0..major {
        let major_angle = major_index as f32 * major_step;
        for minor_index in 0..minor {
            let minor_angle = minor_index as f32 * minor_step;
            let ring_radius = major_radius + minor_radius * minor_angle.cos();
            positions.push(three_d::vec3(
                ring_radius * major_angle.cos(),
                minor_radius * minor_angle.sin(),
                ring_radius * major_angle.sin(),
            ));
        }
    }

    for major_index in 0..major {
        for minor_index in 0..minor {
            let a = major_index * minor + minor_index;
            let b = ((major_index + 1) % major) * minor + minor_index;
            let c = ((major_index + 1) % major) * minor + (minor_index + 1) % minor;
            let d = major_index * minor + (minor_index + 1) % minor;
            indices.extend_from_slice(&[a, d, c, a, c, b]);
        }
    }

    let mut mesh = three_d::CpuMesh {
        positions: three_d::Positions::F32(positions),
        indices: three_d::Indices::U32(indices),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}

fn draw_torus(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&TorusShape>(entity).unwrap();
    validate_segments(shape.major_segments, shape.minor_segments)?;
    if !shape.major_radius.is_finite()
        || shape.major_radius < 0.0
        || !shape.minor_radius.is_finite()
        || shape.minor_radius < 0.0
    {
        return Err("Torus radii must be finite and nonnegative.".into());
    }
    let size = glam::vec3(
        shape.major_radius + shape.minor_radius,
        shape.minor_radius,
        shape.major_radius + shape.minor_radius,
    );
    validate_dimensions(size)?;
    let transformation = global_matrix3d(world, entity);
    let material = world.get::<&Material>(entity).unwrap();
    context.render_material(
        shape.geometry_key(),
        || {
            torus_mesh(
                shape.major_radius,
                shape.minor_radius,
                shape.major_segments,
                shape.minor_segments,
            )
        },
        transformation,
        &material,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Scene,
        objects::{Object3DHandler, ObjectHandler},
    };

    #[test]
    fn torus_builder_sets_radii_and_segments() {
        let mut scene = Scene::new();
        let torus = torus()
            .major_radius(1.0)
            .minor_radius(0.25)
            .major_segments(12)
            .minor_segments(8)
            .build(&mut scene);
        assert_eq!(torus.get(TorusShape::major_radius_property()), 1.0);
        assert_eq!(torus.get(TorusShape::minor_radius_property()), 0.25);
        assert_eq!(torus.get(TorusShape::major_segments_property()), 12);
        assert_eq!(torus.get(TorusShape::minor_segments_property()), 8);
        assert_eq!(torus.box_size(), glam::vec3(2.5, 0.5, 2.5));
    }

    #[test]
    fn torus_mesh_has_two_triangles_per_segment_quad() {
        let mesh = torus_mesh(1.0, 0.25, 12, 8);
        assert_eq!(mesh.triangle_count(), 12 * 8 * 2);
        assert_eq!(mesh.positions.len(), 12 * 8);
        assert!(mesh.normals.as_ref().unwrap()[0].x > 0.9);
        assert!(validate_segments(2, 8).is_err());
        assert!(validate_segments(8, 2).is_err());
    }

    #[test]
    fn default_torus_matches_the_standard_primitive_width() {
        let mut scene = Scene::new();
        let torus = torus().build(&mut scene);

        assert_eq!(torus.box_size(), glam::vec3(1.0, 0.25, 1.0));
    }

    #[test]
    fn torus_geometry_key_includes_radii_and_segments() {
        let shape = TorusShape::default();
        let mut changed = shape.clone();
        changed.major_radius = 1.0;
        assert_ne!(shape.geometry_key(), changed.geometry_key());

        changed = shape.clone();
        changed.minor_segments += 1;
        assert_ne!(shape.geometry_key(), changed.geometry_key());
    }
}
