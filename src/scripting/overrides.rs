use crate::core::{ProjectContext, TrackChoices, TrackTarget, TrackValue};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub(super) const PATH: &str = "kinematic/overrides.ron";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) enum SavedValue {
    Bool(bool),
    U32(u32),
    I32(i32),
    F32(f32),
    String(String),
    Enum(String),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Quaternion([f32; 4]),
    Color([f32; 4]),
    Quad([f32; 4]),
}
impl SavedValue {
    pub fn capture(value: &TrackValue) -> Self {
        match value {
            TrackValue::Bool(v) => Self::Bool(*v),
            TrackValue::U32(v) => Self::U32(*v),
            TrackValue::I32(v) => Self::I32(*v),
            TrackValue::F32(v) => Self::F32(*v),
            TrackValue::String(v) => Self::String(v.clone()),
            TrackValue::Enum(v) => Self::Enum((*v).into()),
            TrackValue::Vector2(v) => Self::Vector2(v.to_array()),
            TrackValue::Vector3(v) => Self::Vector3(v.to_array()),
            TrackValue::Quaternion(v) => Self::Quaternion(v.to_array()),
            TrackValue::Color(v) => Self::Color(v.rgba()),
            TrackValue::Quad(v) => Self::Quad([v.a, v.b, v.c, v.d]),
        }
    }

    pub fn restore(
        &self,
        target: &TrackTarget,
        expected: &TrackValue,
    ) -> Result<TrackValue, String> {
        let value = match self {
            Self::Bool(v) => TrackValue::Bool(*v),
            Self::U32(v) => TrackValue::U32(*v),
            Self::I32(v) => TrackValue::I32(*v),
            Self::F32(v) => TrackValue::F32(*v),
            Self::String(v) => TrackValue::String(v.clone()),
            Self::Enum(v) => {
                let TrackTarget::Property { info, .. } = target else {
                    return Err("Enum property is no longer available.".into());
                };

                let TrackChoices::Enum(choices) = info.choices else {
                    return Err("Enum choices are unavailable.".into());
                };

                TrackValue::Enum(
                    choices
                        .iter()
                        .copied()
                        .find(|s| *s == v)
                        .ok_or("Override enum choice no longer exists.")?,
                )
            }
            Self::Vector2(v) => TrackValue::Vector2((*v).into()),
            Self::Vector3(v) => TrackValue::Vector3((*v).into()),
            Self::Quaternion(v) => TrackValue::Quaternion(glam::Quat::from_array(*v)),
            Self::Color(v) => TrackValue::Color((*v).into()),
            Self::Quad(v) => {
                TrackValue::Quad(crate::core::types::Quad::new(v[0], v[1], v[2], v[3]))
            }
        };

        if std::mem::discriminant(&value) != std::mem::discriminant(expected) {
            return Err("Override type no longer matches the script.".into());
        }

        value.validate()?;

        if let TrackTarget::Dynamic(schema) = target {
            schema.validate(&value)?;
        } else if target.clamp(value.clone()) != value {
            return Err("Override is outside the property limits.".into());
        }

        Ok(value)
    }
}
#[derive(Clone, Default)]
pub(super) struct Overrides {
    pub values: BTreeMap<PathBuf, BTreeMap<String, BTreeMap<String, SavedValue>>>,
    pub saved: Option<Vec<u8>>,
}
impl Overrides {
    pub fn read(context: &ProjectContext) -> Result<Self, String> {
        let path = context.resolve(PATH).map_err(|e| e.to_string())?;

        match std::fs::read(path) {
            Ok(bytes) => Ok(Self {
                values: ron::de::from_bytes(&bytes).map_err(|e| e.to_string())?,
                saved: Some(bytes),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub(super) fn key(target: &TrackTarget) -> String {
    target.uniform_name().map_or_else(
        || target.name().to_owned(),
        |name| format!("uniforms.{name}"),
    )
}

pub(super) fn matches(name: &str, target: &TrackTarget, targets: &[TrackTarget]) -> bool {
    name == key(target) || legacy_key(target, targets).is_some_and(|legacy| name == legacy)
}

pub(super) fn legacy_key<'a>(target: &'a TrackTarget, targets: &[TrackTarget]) -> Option<&'a str> {
    // Migrate old uniform overrides only when their unqualified name is unambiguous.
    target.uniform_name().filter(|name| {
        !targets
            .iter()
            .any(|other| other.uniform_name().is_none() && other.name() == *name)
    })
}

pub(super) fn lookup<'a>(
    saved: &'a BTreeMap<String, SavedValue>,
    target: &TrackTarget,
    targets: &[TrackTarget],
) -> Option<&'a SavedValue> {
    saved
        .get(&key(target))
        .or_else(|| legacy_key(target, targets).and_then(|name| saved.get(name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_keys_preserve_unambiguous_legacy_overrides_and_avoid_collisions() {
        let uniform = TrackTarget::uniform("pulse");
        let property = TrackTarget::Dynamic(std::sync::Arc::new(crate::core::PropertySchema {
            name: "pulse".into(),
            default: TrackValue::F32(0.0),
            limits: crate::core::TrackLimits::None,
            choices: Vec::new(),
        }));
        let saved = [
            ("pulse".into(), SavedValue::F32(2.0)),
            ("uniforms.pulse".into(), SavedValue::F32(6.0)),
        ]
        .into();
        let targets = [property.clone(), uniform.clone()];

        assert!(matches!(
            lookup(&saved, &property, &targets),
            Some(SavedValue::F32(2.0))
        ));

        assert!(matches!(
            lookup(&saved, &uniform, &targets),
            Some(SavedValue::F32(6.0))
        ));

        let legacy = [("pulse".into(), SavedValue::F32(4.0))].into();

        assert!(lookup(&legacy, &uniform, &targets).is_none());

        assert!(matches!(
            lookup(&legacy, &uniform, &[uniform.clone()]),
            Some(SavedValue::F32(4.0))
        ));
    }
}
