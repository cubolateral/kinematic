use std::{collections::BTreeMap, sync::Arc};

use crate::core::{
    TrackValue,
    components::TreeNode,
    objects::{ImageShaderUniform, validate_shader_track_value},
};

/// Shared GLSL 3.30 shader definition for three-dimensional meshes.
///
/// Sources omit `#version`; the renderer supplies it. Fragment-only shaders use
/// the engine vertex shader and receive `k_world_position`, `k_normal`, and
/// `k_uv`. A custom vertex shader uses attributes `position`, and optionally
/// `normal` and `uv_coordinates`; it must actively use the `modelMatrix` and
/// `viewProjection` uniforms.
///
/// Both stages may use `normalMatrix`, `viewMatrix`, `projectionMatrix`,
/// `cameraPosition`, and `sceneTime`. Fragment shaders also receive the linear
/// `materialColor` and integer flags `hasNormal` and `hasUv`, and write
/// `outColor`. Custom shaders replace the standard material shader: Kinematic
/// preserves its transform, depth state, opacity blending, and outline, but
/// does not add PBR lighting or material textures automatically.
///
/// GPU vertex deformation does not update CPU geometry. Selection and any
/// CPU-side culling keep using the original mesh; configure
/// `mesh_shader_bounds` on the builder to add conservative bounds without
/// changing layout or object origins.
#[derive(Clone)]
pub struct MeshShader {
    pub(crate) vertex: Option<Arc<str>>,
    pub(crate) fragment: Arc<str>,
    pub(crate) id: u64,
}

impl MeshShader {
    /// Creates a fragment shader paired with Kinematic's standard mesh vertex shader.
    pub fn fragment(source: impl Into<String>) -> Self {
        Self::new(None, source.into())
    }

    /// Creates a complete custom vertex/fragment program.
    pub fn program(vertex: impl Into<String>, fragment: impl Into<String>) -> Self {
        Self::new(Some(vertex.into()), fragment.into())
    }

    fn new(vertex: Option<String>, fragment: String) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            vertex: vertex.map(Arc::from),
            fragment: Arc::from(fragment),
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

/// Initial value accepted by a custom mesh-shader uniform.
pub type MeshShaderUniform = ImageShaderUniform;

/// Builder-only mesh shader definition and local values.
#[doc(hidden)]
#[derive(Clone, Default)]
pub struct MeshShaderData {
    pub(crate) shader: Option<MeshShader>,
    pub(crate) uniforms: BTreeMap<String, TrackValue>,
    pub(crate) bounds_padding: Option<f32>,
}

impl MeshShaderData {
    #[doc(hidden)]
    pub fn with_shader(shader: MeshShader) -> Self {
        Self {
            shader: Some(shader),
            ..Default::default()
        }
    }

    #[doc(hidden)]
    pub fn uniform(&mut self, name: impl Into<String>, value: impl Into<MeshShaderUniform>) {
        let name = name.into();
        assert!(
            valid_uniform_name(&name),
            "Mesh shader uniforms must use non-reserved GLSL identifiers."
        );
        let value = value.into().into_track_value();
        validate_shader_track_value(&value).unwrap_or_else(|error| panic!("{error}"));
        self.uniforms.insert(name, value);
    }

    #[doc(hidden)]
    pub fn bounds_padding(&mut self, padding: f32) {
        assert!(
            padding.is_finite() && padding >= 0.0,
            "Mesh shader bounds padding must be finite and non-negative."
        );
        self.bounds_padding = Some(padding);
    }
}

#[derive(Clone)]
pub(crate) struct EffectiveMeshShader {
    pub(crate) shader: MeshShader,
    pub(crate) uniforms: BTreeMap<String, TrackValue>,
    pub(crate) bounds_padding: f32,
}

pub(crate) fn effective_mesh_shader(
    world: &hecs::World,
    entity: hecs::Entity,
) -> Result<Option<EffectiveMeshShader>, String> {
    let mut lineage = vec![entity];
    let mut current = entity;
    while let Some(parent) = world
        .get::<&TreeNode>(current)
        .ok()
        .and_then(|node| node.parent)
    {
        lineage.push(parent);
        current = parent;
    }
    lineage.reverse();
    let Some(start) = lineage.iter().rposition(|entity| {
        world
            .get::<&MeshShaderData>(*entity)
            .is_ok_and(|data| data.shader.is_some())
    }) else {
        return Ok(None);
    };
    let owner = world.get::<&MeshShaderData>(lineage[start]).unwrap();
    let mut result = EffectiveMeshShader {
        shader: owner.shader.clone().unwrap(),
        uniforms: BTreeMap::new(),
        bounds_padding: 0.0,
    };
    drop(owner);

    for entity in &lineage[start..] {
        let Ok(data) = world.get::<&MeshShaderData>(*entity) else {
            continue;
        };
        if let Some(padding) = data.bounds_padding {
            result.bounds_padding = padding;
        }
        for (name, value) in &data.uniforms {
            if let Some(previous) = result.uniforms.get(name)
                && std::mem::discriminant(previous) != std::mem::discriminant(value)
            {
                return Err(format!(
                    "Mesh shader uniform override `{name}` must keep its inherited value type."
                ));
            }
            result.uniforms.insert(name.clone(), value.clone());
        }
    }
    validate(&result)?;
    Ok(Some(result))
}

fn validate(data: &EffectiveMeshShader) -> Result<(), String> {
    for (name, value) in &data.uniforms {
        if !valid_uniform_name(name) {
            return Err(format!(
                "Mesh shader uniform `{name}` uses an invalid or reserved name."
            ));
        }
        validate_shader_track_value(value)?;
    }
    Ok(())
}

fn valid_uniform_name(name: &str) -> bool {
    const RESERVED: [&str; 10] = [
        "modelMatrix",
        "normalMatrix",
        "viewProjection",
        "viewMatrix",
        "projectionMatrix",
        "cameraPosition",
        "sceneTime",
        "materialColor",
        "hasNormal",
        "hasUv",
    ];
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
        && !RESERVED.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn inherited_shader_values_allow_compatible_local_overrides() {
        let shader = MeshShader::fragment("void main() { outColor = materialColor; }");
        let own_shader = MeshShader::fragment("void main() { outColor = vec4(1.0); }");
        let mut scene = Scene::new();
        let group = group_3d()
            .mesh_shader(&shader)
            .mesh_uniform("amount", 1.0)
            .build(&mut scene);
        let child = cube().mesh_uniform("amount", 2.0).build(&mut scene);
        let own = sphere()
            .mesh_shader(&own_shader)
            .mesh_uniform("local", 3.0)
            .build(&mut scene);
        let incompatible = plane()
            .mesh_uniform("amount", vec2(1.0, 2.0))
            .build(&mut scene);
        group.add(&child);
        group.add(&own);
        group.add(&incompatible);
        scene.world_3d().add(&group);
        let effective = effective_mesh_shader(&scene.world(), child.entity())
            .unwrap()
            .unwrap();
        assert_eq!(effective.shader.id, shader.id);
        assert_eq!(effective.uniforms["amount"], TrackValue::F32(2.0));
        let effective = effective_mesh_shader(&scene.world(), own.entity())
            .unwrap()
            .unwrap();
        assert_eq!(effective.shader.id, own_shader.id);
        assert!(!effective.uniforms.contains_key("amount"));
        assert_eq!(effective.uniforms["local"], TrackValue::F32(3.0));
        assert!(effective_mesh_shader(&scene.world(), incompatible.entity()).is_err());
    }

    #[test]
    fn deformation_padding_expands_selection_bounds_without_changing_layout_size() {
        let shader = MeshShader::fragment("void main() { outColor = vec4(1.0); }");
        let mut scene = Scene::new();
        let object = cube()
            .size(vec3(1.0, 1.0, 1.0))
            .mesh_shader(&shader)
            .mesh_shader_bounds(2.0)
            .build(&mut scene);
        assert_eq!(object.box_size(), vec3(1.0, 1.0, 1.0));
        let (min, max) = crate::core::objects::bounds3d(&scene.world(), object.entity()).unwrap();
        assert_eq!(min, vec3(-2.5, -2.5, -2.5));
        assert_eq!(max, vec3(2.5, 2.5, 2.5));
    }

    #[test]
    fn animated_group_uniforms_feed_inherited_shaders_with_local_isolation() {
        struct Build {
            group: Option<Group3DHandler>,
            child: Option<PrismHandler>,
        }
        impl SceneBuilder for Build {
            fn build(&mut self, scene: &mut Scene) {
                let shader = MeshShader::fragment("void main() { outColor = vec4(amount); }");
                let group = group_3d()
                    .mesh_shader(&shader)
                    .mesh_uniform("amount", 0.0_f32)
                    .build(scene);
                let child = cube().mesh_uniform("amount", 0.25_f32).build(scene);
                group.add(&child);
                scene.world_3d().add(&group);
                scene.all(|_| {
                    group
                        .uniform("amount", 1.0_f32)
                        .duration(2.0)
                        .easing(Easing::Linear)
                        .play();
                    child
                        .uniform("amount", 0.75_f32)
                        .duration(2.0)
                        .easing(Easing::Linear)
                        .play();
                });
                self.group = Some(group);
                self.child = Some(child);
            }
        }

        let mut scene = Scene::new();
        let mut build = Build {
            group: None,
            child: None,
        };
        scene.build(&mut build);
        scene.update(1.0);
        let group = build.group.unwrap();
        let child = build.child.unwrap();
        assert_eq!(group.get_uniform::<f32>("amount"), 0.5);
        assert_eq!(child.get_uniform::<f32>("amount"), 0.5);
        let effective = effective_mesh_shader(&scene.world(), child.entity())
            .unwrap()
            .unwrap();
        assert_eq!(effective.uniforms["amount"], TrackValue::F32(0.5));
    }
}
