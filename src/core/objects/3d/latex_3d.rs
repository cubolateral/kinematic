use std::hash::{DefaultHasher, Hash, Hasher};

use kinematic_macros::{Object, Trackable};

use crate::core::{
    Tween,
    components::{Draw3D, GeometryKey, Material, RenderContext3D, Transform3D},
    objects::{
        ObjectHandler, global_matrix3d,
        latex_geometry::geometry,
        string_morph::{ContentMorph, fade_string},
    },
    types::Vector3,
};

use super::{super::two_d::text_2d::weighted_path, text_3d::extruded_path_mesh};

// A visible polygonal curve keeps formula meshes compact.
const LATEX_TESSELLATION_TOLERANCE: f32 = 0.005;

/// Mathematical source, size and extrusion depth of a 3D LaTeX object.
#[derive(Clone, Trackable)]
pub struct Latex3DShape {
    /// LaTeX math source, without dollar delimiters.
    #[track]
    pub text: String,
    /// Formula size in world units.
    #[track]
    pub size: f32,
    /// Extra glyph thickness in world units.
    #[track]
    pub thickness: f32,
    /// Extrusion depth in world units.
    #[track]
    pub depth: f32,
}

impl Default for Latex3DShape {
    fn default() -> Self {
        Self {
            text: r"e^{i\pi}+1=0".to_owned(),
            size: 1.0,
            thickness: 0.0,
            depth: 0.1,
        }
    }
}

impl Latex3DShape {
    fn geometry_key(&self, text: &str, part: usize) -> GeometryKey {
        let mut hash = DefaultHasher::new();
        text.hash(&mut hash);
        part.hash(&mut hash);
        self.thickness.to_bits().hash(&mut hash);
        GeometryKey::new::<Latex3DShape>(hash.finish())
    }
}

/// Built-in extruded LaTeX math scene object.
///
/// Supports mathematical LaTeX accepted by RaTeX in display style.
/// Invalid or unsupported source panics when its geometry is first requested.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "latex_3d")]
pub struct Latex3D {
    #[trackable]
    pub shape: Latex3DShape,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Latex3D {
    fn default() -> Self {
        Self {
            shape: Latex3DShape::default(),
            material: Material::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_latex_3d,
                get_box: latex_3d_box,
                ..Default::default()
            },
        }
    }
}

impl Latex3DHandler {
    /// Cross-fades the current formula source into `text` on the same object.
    pub fn fade(&self, text: impl Into<String>) -> Tween<Latex3D> {
        let from = self.get(Latex3DShape::text_property());
        let to = text.into();
        let tween = self.text(to.clone());
        fade_string(tween, self.get_id(), from, to)
    }
}

fn latex_3d_box(world: &hecs::World, entity: hecs::Entity) -> Vector3 {
    let shape = world.get::<&Latex3DShape>(entity).unwrap();
    (geometry(&shape.text).size * shape.size.max(0.0) + glam::Vec2::splat(shape.thickness.max(0.0)))
        .extend(shape.depth.abs())
}

fn draw_latex_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&Latex3DShape>(entity).unwrap();
    if !shape.size.is_finite()
        || !shape.thickness.is_finite()
        || !shape.depth.is_finite()
        || shape.size < 0.0
        || shape.thickness < 0.0
        || shape.depth < 0.0
    {
        return Err("LaTeX dimensions must be finite and nonnegative.".into());
    }
    let material = world.get::<&Material>(entity).unwrap();
    let transform = global_matrix3d(world, entity)
        * glam::Mat4::from_scale(glam::Vec3::new(shape.size, shape.size, shape.depth));
    let progress = world
        .get::<&ContentMorph>(entity)
        .ok()
        .filter(|morph| morph.active && morph.progress > 0.0)
        .and_then(|morph| {
            morph.transitions[morph.transition as usize].fade_layers(morph.progress.clamp(0.0, 1.0))
        });

    if let Some(layers) = progress {
        for (text, opacity) in layers {
            draw_formula(&shape, &text, opacity, &material, transform, context)?;
        }
    } else {
        draw_formula(&shape, &shape.text, 1.0, &material, transform, context)?;
    }
    Ok(())
}

fn draw_formula(
    shape: &Latex3DShape,
    text: &str,
    opacity: f32,
    material: &Material,
    transform: glam::Mat4,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    if opacity <= 0.0 || text.is_empty() {
        return Ok(());
    }
    let formula = geometry(text);
    for (index, part) in formula.parts.iter().enumerate() {
        let mut material = material.clone();
        material.opacity *= opacity;
        material.albedo = part.color.unwrap_or(material.albedo);
        context.render_material(
            shape.geometry_key(text, index),
            || {
                let path = if shape.size > f32::EPSILON {
                    weighted_path(&part.path, shape.thickness / shape.size)
                } else {
                    part.path.clone()
                };
                extruded_path_mesh(&path, LATEX_TESSELLATION_TOLERANCE)
            },
            transform,
            &material,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{Object3DHandler, Scene};

    #[test]
    fn depth_is_trackable_and_part_of_the_local_box() {
        let mut scene = Scene::new();
        let formula = latex_3d().text("x").depth(0.25).build(&mut scene);

        assert_eq!(formula.depth.get(), 0.25);
        assert_eq!(formula.get_box().z, 0.25);
    }

    #[test]
    fn formula_builds_extruded_geometry() {
        let formula = geometry("x");
        let mesh = extruded_path_mesh(&formula.parts[0].path, 0.001);
        let normals = mesh.normals.unwrap();

        assert!(normals.iter().any(|normal| normal.z > 0.9));
        assert!(normals.iter().any(|normal| normal.z < -0.9));
        assert!(normals.iter().any(|normal| normal.z.abs() < 0.1));
    }

    #[test]
    fn fade_keeps_the_same_formula_object() {
        let mut scene = Scene::new();
        let formula = latex_3d().text("x").build(&mut scene);
        let entity = formula.get_id();

        formula.fade("y").play();

        assert_eq!(formula.get_id(), entity);
        scene.update(1.0);
        assert_eq!(formula.text.get(), "y");
    }

    #[test]
    fn latex_3d_is_not_morphable() {
        assert!(!<Latex3D as crate::core::objects::Object>::MORPHABLE);
    }
}
