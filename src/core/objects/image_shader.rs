use std::{collections::BTreeMap, sync::Arc};

use crate::core::{
    TrackValue,
    objects::{CanvasTexture, ProjectionCanvas},
    types::{Color, Quad, Quaternion, Vector2, Vector3},
};

/// Shared GLSL 3.30 fragment-shader definition for image effects.
///
/// The first line must be `#version 330 core`. The source declares
/// `in vec2 k_uv` and `out vec4 k_color`, and may use `sampler2D k_image`,
/// `vec2 k_resolution` in pixels, `float k_time` in evaluated scene seconds,
/// and `float k_alpha`. UV `(0, 0)` is the bottom-left pixel. Textures and
/// colors use premultiplied, sRGB-encoded RGBA. `k_alpha` is the object's
/// evaluated opacity, already present in `k_image`; canvas shaders receive
/// `1.0`. The output must remain premultiplied.
#[derive(Clone)]
pub struct ImageShader {
    pub(crate) source: Arc<str>,
    pub(crate) id: u64,
}

impl ImageShader {
    /// Creates a reusable shader definition from a complete GLSL 3.30 fragment shader.
    pub fn new(source: impl Into<String>) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            source: Arc::from(source.into()),
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

/// Initial value accepted by a custom image-shader uniform.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageShaderUniform {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    UInt(u32),
    Bool(bool),
    Color(Color),
    Quaternion(Quaternion),
    Quad(Quad),
}

impl ImageShaderUniform {
    pub(crate) fn into_track_value(self) -> TrackValue {
        match self {
            Self::Float(value) => TrackValue::F32(value),
            Self::Vec2(value) => TrackValue::Vector2(Vector2::from_array(value)),
            Self::Vec3(value) => TrackValue::Vector3(Vector3::from_array(value)),
            Self::Vec4(value) => TrackValue::Quad(Quad::from(value)),
            Self::Int(value) => TrackValue::I32(value),
            Self::UInt(value) => TrackValue::U32(value),
            Self::Bool(value) => TrackValue::Bool(value),
            Self::Color(value) => TrackValue::Color(value),
            Self::Quaternion(value) => TrackValue::Quaternion(value),
            Self::Quad(value) => TrackValue::Quad(value),
        }
    }
}

impl From<f32> for ImageShaderUniform {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for ImageShaderUniform {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<u32> for ImageShaderUniform {
    fn from(value: u32) -> Self {
        Self::UInt(value)
    }
}

impl From<bool> for ImageShaderUniform {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<[f32; 2]> for ImageShaderUniform {
    fn from(value: [f32; 2]) -> Self {
        Self::Vec2(value)
    }
}

impl From<[f32; 3]> for ImageShaderUniform {
    fn from(value: [f32; 3]) -> Self {
        Self::Vec3(value)
    }
}

impl From<[f32; 4]> for ImageShaderUniform {
    fn from(value: [f32; 4]) -> Self {
        Self::Vec4(value)
    }
}

impl From<glam::Vec2> for ImageShaderUniform {
    fn from(value: glam::Vec2) -> Self {
        Self::Vec2(value.to_array())
    }
}

impl From<glam::Vec3> for ImageShaderUniform {
    fn from(value: glam::Vec3) -> Self {
        Self::Vec3(value.to_array())
    }
}

impl From<glam::Vec4> for ImageShaderUniform {
    fn from(value: glam::Vec4) -> Self {
        Self::Vec4(value.to_array())
    }
}

impl From<crate::core::types::Color> for ImageShaderUniform {
    fn from(value: crate::core::types::Color) -> Self {
        Self::Color(value)
    }
}

impl From<Quad> for ImageShaderUniform {
    fn from(value: Quad) -> Self {
        Self::Quad(value)
    }
}

impl From<Quaternion> for ImageShaderUniform {
    fn from(value: Quaternion) -> Self {
        Self::Quaternion(value)
    }
}

/// Per-object image-shader values configured by an object builder.
#[doc(hidden)]
#[derive(Clone)]
pub struct ImageShaderData {
    pub(crate) shader: ImageShader,
    pub(crate) uniforms: BTreeMap<String, TrackValue>,
    pub(crate) textures: BTreeMap<String, CanvasTexture>,
    pub(crate) padding: f32,
}

impl ImageShaderData {
    #[doc(hidden)]
    pub fn new(shader: ImageShader) -> Self {
        Self {
            shader,
            uniforms: BTreeMap::new(),
            textures: BTreeMap::new(),
            padding: 0.0,
        }
    }

    #[doc(hidden)]
    pub fn uniform(&mut self, name: impl Into<String>, value: impl Into<ImageShaderUniform>) {
        let name = name.into();
        assert!(
            valid_binding_name(&name),
            "Image shader bindings must be GLSL identifiers and cannot use the `k_` prefix."
        );
        let value = value.into().into_track_value();
        validate_shader_track_value(&value).unwrap_or_else(|error| panic!("{error}"));
        self.uniforms.insert(name, value);
    }

    #[doc(hidden)]
    pub fn texture(&mut self, name: impl Into<String>, canvas: &impl ProjectionCanvas) {
        let name = name.into();
        assert!(
            valid_binding_name(&name),
            "Image shader bindings must be GLSL identifiers and cannot use the `k_` prefix."
        );
        self.textures.insert(name, canvas.texture());
    }

    #[doc(hidden)]
    pub fn padding(&mut self, padding: f32) {
        assert!(
            padding.is_finite() && padding >= 0.0,
            "Image shader padding must be finite and non-negative."
        );
        self.padding = padding;
    }
}

pub(crate) fn validate_shader_values(data: &ImageShaderData) -> Result<(), String> {
    for name in data.uniforms.keys().chain(data.textures.keys()) {
        if !valid_binding_name(name) {
            return Err(format!(
                "Image shader binding `{name}` must be a GLSL identifier and cannot use the `k_` prefix."
            ));
        }
    }
    if let Some(name) = data
        .uniforms
        .keys()
        .find(|name| data.textures.contains_key(*name))
    {
        return Err(format!(
            "Image shader binding `{name}` cannot be both a uniform and a texture."
        ));
    }
    for value in data.uniforms.values() {
        validate_shader_track_value(value)?;
    }
    Ok(())
}

fn valid_binding_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
        && !name.starts_with("k_")
}

pub(crate) fn validate_shader_track_value(value: &TrackValue) -> Result<(), String> {
    let finite = match value {
        TrackValue::F32(value) => value.is_finite(),
        TrackValue::Quad(value) => value.to_array().iter().all(|value| value.is_finite()),
        TrackValue::Vector2(value) => value.is_finite(),
        TrackValue::Vector3(value) => value.is_finite(),
        TrackValue::Quaternion(value) => value.is_finite(),
        TrackValue::Color(value) => value.rgba().iter().all(|value| value.is_finite()),
        TrackValue::Bool(_) | TrackValue::U32(_) | TrackValue::I32(_) => true,
        TrackValue::Enum(_) | TrackValue::String(_) => {
            return Err("Shader uniforms must use numeric or boolean TrackValues.".into());
        }
    };
    finite
        .then_some(())
        .ok_or_else(|| "Shader uniforms must contain finite values.".into())
}

pub(crate) fn shader_uniform(
    world: &hecs::World,
    entity: hecs::Entity,
    name: &str,
) -> Result<TrackValue, String> {
    if let Ok(data) = world.get::<&ImageShaderData>(entity)
        && let Some(value) = data.uniforms.get(name)
    {
        return Ok(value.clone());
    }
    if let Ok(data) = world.get::<&crate::core::objects::MeshShaderData>(entity)
        && let Some(value) = data.uniforms.get(name)
    {
        return Ok(value.clone());
    }
    Err(format!(
        "Shader uniform `{name}` must be defined on this object's builder before use."
    ))
}

pub(crate) fn set_shader_uniform(
    world: &hecs::World,
    entity: hecs::Entity,
    name: &str,
    value: TrackValue,
) -> Result<(), String> {
    validate_shader_track_value(&value)?;
    if let Ok(mut data) = world.get::<&mut ImageShaderData>(entity)
        && let Some(current) = data.uniforms.get_mut(name)
    {
        return replace_uniform(name, current, value);
    }
    if let Ok(mut data) = world.get::<&mut crate::core::objects::MeshShaderData>(entity)
        && let Some(current) = data.uniforms.get_mut(name)
    {
        return replace_uniform(name, current, value);
    }
    Err(format!(
        "Shader uniform `{name}` must be defined on this object's builder before use."
    ))
}

fn replace_uniform(name: &str, current: &mut TrackValue, value: TrackValue) -> Result<(), String> {
    if std::mem::discriminant(current) != std::mem::discriminant(&value) {
        return Err(format!(
            "Shader uniform `{name}` must keep its builder-defined type."
        ));
    }
    *current = value;
    Ok(())
}

pub(crate) fn shader_uniforms(
    world: &hecs::World,
    entity: hecs::Entity,
) -> Vec<(String, TrackValue)> {
    if let Ok(data) = world.get::<&ImageShaderData>(entity) {
        return data
            .uniforms
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
    }
    world
        .get::<&crate::core::objects::MeshShaderData>(entity)
        .map(|data| {
            data.uniforms
                .iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn shader() -> ImageShader {
        ImageShader::new("#version 330 core\nvoid main() { k_color = vec4(1.0); }")
    }

    #[test]
    fn shader_values_reject_engine_names_and_non_finite_values() {
        let shader = ImageShader::new("#version 330 core\nvoid main() {}");
        let mut data = ImageShaderData::new(shader);
        data.uniforms.insert("k_time".into(), TrackValue::F32(1.0));
        assert!(validate_shader_values(&data).is_err());
        data.uniforms.clear();
        data.uniforms
            .insert("amount".into(), TrackValue::F32(f32::NAN));
        assert!(validate_shader_values(&data).is_err());
    }

    #[test]
    fn builders_store_independent_values_and_configure_default_canvases() {
        let shader = ImageShader::new("#version 330 core\nvoid main() {}");
        let mut scene = Scene::new_with_canvases(
            canvas_2d()
                .resolution((64, 64))
                .shader(&shader)
                .uniform("amount", 1.0)
                .shader_padding(8.0),
            canvas_3d().resolution((64, 64)).shader(&shader),
        );
        let object = rect()
            .shader(&shader)
            .uniform("amount", 2.0)
            .build(&mut scene);
        let world = scene.world();
        let canvas = world
            .get::<&ImageShaderData>(scene.world_2d().entity())
            .unwrap();
        let object = world.get::<&ImageShaderData>(object.entity()).unwrap();
        assert_eq!(canvas.padding, 8.0);
        assert_eq!(canvas.uniforms["amount"], TrackValue::F32(1.0));
        assert_eq!(object.uniforms["amount"], TrackValue::F32(2.0));
    }

    #[test]
    fn uniforms_share_tweens_with_properties_and_seek_normally() {
        struct Build {
            object: Option<RectHandler>,
        }
        impl SceneBuilder for Build {
            fn build(&mut self, scene: &mut Scene) {
                let object = rect()
                    .shader(&shader())
                    .uniform("u_progress", 0.0_f32)
                    .uniform("u_other", 0.0_f32)
                    .build(scene);
                scene.world_2d().add(&object);
                object
                    .uniform("u_progress", 1.0_f32)
                    .uniform("u_other", 2.0_f32)
                    .position_x(20.0)
                    .duration(2.0)
                    .easing(Easing::Linear)
                    .play();
                self.object = Some(object);
            }
        }

        let mut scene = Scene::new();
        let mut build = Build { object: None };
        assert_eq!(scene.build(&mut build), 2.0);
        let object = build.object.unwrap();
        scene.update(1.0);
        assert_eq!(object.get_uniform::<f32>("u_progress"), 0.5);
        assert_eq!(object.get_uniform::<f32>("u_other"), 1.0);
        assert_eq!(object.get_position().x, 10.0);
        scene.update(0.0);
        assert_eq!(object.get_uniform::<f32>("u_progress"), 0.0);
        assert_eq!(object.get_position().x, 0.0);
    }

    #[test]
    fn uniform_repeat_signal_and_override_restoration_use_existing_evaluation_order() {
        struct Build {
            object: Option<RectHandler>,
        }
        impl SceneBuilder for Build {
            fn build(&mut self, scene: &mut Scene) {
                let object = rect()
                    .shader(&shader())
                    .uniform("u_progress", 0.0_f32)
                    .build(scene);
                scene.world_2d().add(&object);
                object.signal(|object, frame| {
                    if frame.time < 0.75 {
                        object.set_uniform("u_progress", 0.9_f32);
                    }
                });
                scene.repeat(|_| {
                    object
                        .uniform_from("u_progress", 0.0_f32, 1.0_f32)
                        .duration(1.0)
                        .easing(Easing::Linear)
                        .play();
                });
                scene.wait(3.0);
                self.object = Some(object);
            }
        }

        let mut scene = Scene::new();
        let mut build = Build { object: None };
        scene.build(&mut build);
        let object = build.object.unwrap();
        scene.update(0.5);
        assert_eq!(object.get_uniform::<f32>("u_progress"), 0.9);
        scene.update(1.5);
        assert_eq!(object.get_uniform::<f32>("u_progress"), 0.5);
    }

    #[test]
    fn snapshots_include_uniforms_and_preserve_their_types() {
        let mut scene = Scene::new();
        let object = rect()
            .shader(&shader())
            .uniform("float", 0.25_f32)
            .uniform("uint", 2_u32)
            .uniform("int", -2_i32)
            .uniform("bool", true)
            .uniform("quad", Quad::new(1.0, 2.0, 3.0, 4.0))
            .uniform("vector", vec3(1.0, 2.0, 3.0))
            .uniform("rotation", Quaternion::IDENTITY)
            .uniform("color", Color::RED)
            .build(&mut scene);
        let snapshot = object.snapshot();
        object.set_uniform("float", 0.75_f32);
        let tween = object.restore_snapshot(snapshot);
        assert_eq!(object.get_uniform::<f32>("float"), 0.25);
        assert_eq!(object.get_uniform::<u32>("uint"), 2);
        assert!(object.get_uniform::<bool>("bool"));
        assert_eq!(object.get_uniform::<Color>("color"), Color::RED);
        drop(tween);
    }

    #[test]
    #[should_panic(expected = "must keep its builder-defined type")]
    fn uniform_type_cannot_change() {
        let mut scene = Scene::new();
        let object = rect()
            .shader(&shader())
            .uniform("value", 1.0_f32)
            .build(&mut scene);
        object.set_uniform("value", 1_i32);
    }

    #[test]
    #[should_panic(expected = "Animations on property 'value' overlap")]
    fn concurrent_uniform_writes_are_rejected() {
        struct Build;
        impl SceneBuilder for Build {
            fn build(&mut self, scene: &mut Scene) {
                let object = rect()
                    .shader(&shader())
                    .uniform("value", 0.0_f32)
                    .build(scene);
                scene.world_2d().add(&object);
                scene.all(|_| {
                    object.uniform("value", 1.0_f32).play();
                    object.uniform("value", 2.0_f32).play();
                });
            }
        }

        Scene::new().build(&mut Build);
    }
}
