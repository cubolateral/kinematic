use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::core::{components::Material, objects::CanvasTexture};
use three_d::Geometry;

/// Stable identity for a reusable GPU geometry.
///
/// Use the object's shape component as `T` and encode geometry-changing settings in `variant`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryKey {
    kind: TypeId,
    variant: u64,
}

impl GeometryKey {
    pub fn new<T: 'static>(variant: u64) -> Self {
        Self {
            kind: TypeId::of::<T>(),
            variant,
        }
    }
}

/// Resources available to a [`Draw3D`] callback during one canvas pass.
///
/// Geometries are cached across frames. Callbacks can either use [`Self::render_material`]
/// or access three-d directly through the public camera, target, and context fields.
pub struct RenderContext3D<'a> {
    pub camera: &'a three_d::Camera,
    pub target: &'a three_d::RenderTarget<'static>,
    pub three_d: &'a three_d::Context,
    geometries: &'a mut HashMap<GeometryKey, three_d::Mesh>,
    used_geometries: &'a mut HashSet<GeometryKey>,
    physical: &'a mut three_d::PhysicalMaterial,
    ambient: &'a three_d::AmbientLight,
    sun: &'a three_d::DirectionalLight,
    texture: &'a dyn Fn(CanvasTexture) -> Option<glow::NativeTexture>,
}

impl<'a> RenderContext3D<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        camera: &'a three_d::Camera,
        target: &'a three_d::RenderTarget<'static>,
        three_d: &'a three_d::Context,
        geometries: &'a mut HashMap<GeometryKey, three_d::Mesh>,
        used_geometries: &'a mut HashSet<GeometryKey>,
        physical: &'a mut three_d::PhysicalMaterial,
        ambient: &'a three_d::AmbientLight,
        sun: &'a three_d::DirectionalLight,
        texture: &'a dyn Fn(CanvasTexture) -> Option<glow::NativeTexture>,
    ) -> Self {
        Self {
            camera,
            target,
            three_d,
            geometries,
            used_geometries,
            physical,
            ambient,
            sun,
            texture,
        }
    }

    /// Returns a shared mesh, creating it the first time its key is used.
    pub fn geometry(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
    ) -> &mut three_d::Mesh {
        self.used_geometries.insert(key);
        self.geometries
            .entry(key)
            .or_insert_with(|| three_d::Mesh::new(self.three_d, &create()))
    }

    /// Resolves a canvas texture for custom materials.
    pub fn canvas_texture(&self, texture: CanvasTexture) -> Option<glow::NativeTexture> {
        (self.texture)(texture)
    }

    /// Draws cached geometry with the standard Kinematic material and lights.
    pub fn render_material(
        &mut self,
        key: GeometryKey,
        create: impl FnOnce() -> three_d::CpuMesh,
        transformation: glam::Mat4,
        data: &Material,
    ) -> Result<(), String> {
        validate_transformation(transformation)?;
        if transformation.determinant().abs() <= f32::EPSILON {
            return Ok(());
        }
        let [r, g, b, a] = data.albedo.rgba();
        let transparent = data.opacity * a < 1.0;
        let color = three_d::Srgba::new(
            channel(r),
            channel(g),
            channel(b),
            channel(a * data.opacity),
        );
        let states = render_states(transparent, false);
        let camera = self.camera;
        if data.unlit {
            let material = three_d::ColorMaterial {
                color,
                texture: None,
                render_states: states,
                is_transparent: transparent,
            };
            let mesh = self.geometry(key, create);
            mesh.set_transformation(transformation.to_cols_array_2d().into());
            mesh.render_with_material(&material, camera, &[]);
        } else {
            self.physical.albedo = color;
            self.physical.metallic = data.metallic.clamp(0.0, 1.0);
            self.physical.roughness = data.roughness.clamp(0.04, 1.0);
            self.physical.is_transparent = transparent;
            self.physical.render_states = states;
            let physical = &*self.physical;
            let lights: [&dyn three_d::Light; 2] = [self.ambient, self.sun];
            self.used_geometries.insert(key);
            let mesh = self
                .geometries
                .entry(key)
                .or_insert_with(|| three_d::Mesh::new(self.three_d, &create()));
            mesh.set_transformation(transformation.to_cols_array_2d().into());
            mesh.render_with_material(physical, camera, &lights);
        }
        Ok(())
    }
}

/// Local three-dimensional rendering callback and bounds for an entity.
#[derive(Clone)]
pub struct Draw3D {
    /// Draws this entity using its current ECS state.
    pub on_draw: fn(&hecs::World, hecs::Entity, &mut RenderContext3D<'_>) -> Result<(), String>,

    /// Returns the object's local bounding-box size.
    pub get_box: fn(&hecs::World, hecs::Entity) -> glam::Vec3,
}

impl Default for Draw3D {
    fn default() -> Self {
        Self {
            on_draw: |_, _, _| Ok(()),
            get_box: |_, _| glam::Vec3::ZERO,
        }
    }
}

pub(crate) fn validate_dimensions(size: glam::Vec3) -> Result<(), String> {
    if !size.is_finite() || size.min_element() < 0.0 {
        return Err("Primitive dimensions must be finite and nonnegative.".into());
    }
    Ok(())
}

pub(crate) fn validate_transformation(transformation: glam::Mat4) -> Result<(), String> {
    if !transformation.is_finite() {
        return Err("Object transform must be finite.".into());
    }
    Ok(())
}

pub(crate) fn render_states(transparent: bool, premultiplied: bool) -> three_d::RenderStates {
    use three_d::{Blend, BlendEquationType as E, BlendMultiplierType as M};
    three_d::RenderStates {
        cull: three_d::Cull::None,
        write_mask: if transparent {
            three_d::WriteMask::COLOR
        } else {
            three_d::WriteMask::COLOR_AND_DEPTH
        },
        blend: if transparent {
            Blend::Enabled {
                source_rgb_multiplier: if premultiplied { M::One } else { M::SrcAlpha },
                source_alpha_multiplier: M::One,
                destination_rgb_multiplier: M::OneMinusSrcAlpha,
                destination_alpha_multiplier: M::OneMinusSrcAlpha,
                rgb_equation: E::Add,
                alpha_equation: E::Add,
            }
        } else {
            Blend::Disabled
        },
        ..Default::default()
    }
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Shape;
    struct OtherShape;

    #[test]
    fn geometry_keys_include_type_and_variant() {
        assert_eq!(GeometryKey::new::<Shape>(3), GeometryKey::new::<Shape>(3));
        assert_ne!(GeometryKey::new::<Shape>(3), GeometryKey::new::<Shape>(4));
        assert_ne!(
            GeometryKey::new::<Shape>(3),
            GeometryKey::new::<OtherShape>(3)
        );
    }
}
