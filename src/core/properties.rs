use std::collections::BTreeMap;

use crate::core::{TrackLimits, TrackValue};

/// Runtime schema for an animatable property supplied by an extension.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertySchema {
    pub name: String,
    pub default: TrackValue,
    pub limits: TrackLimits,
    pub choices: Vec<String>,
}

impl PropertySchema {
    /// Checks the type, finite values, bounds and declared choices.
    pub fn validate(&self, value: &TrackValue) -> Result<(), String> {
        if std::mem::discriminant(value) != std::mem::discriminant(&self.default) {
            return Err(format!(
                "Property '{}' has an incompatible value type.",
                self.name
            ));
        }

        value.validate()?;

        let in_range = match (value, self.limits) {
            (TrackValue::F32(v), TrackLimits::F32 { min, max }) => {
                min.is_none_or(|m| *v >= m) && max.is_none_or(|m| *v <= m)
            }
            (TrackValue::I32(v), TrackLimits::I32 { min, max }) => {
                min.is_none_or(|m| *v >= m) && max.is_none_or(|m| *v <= m)
            }
            (TrackValue::U32(v), TrackLimits::U32 { min, max }) => {
                min.is_none_or(|m| *v >= m) && max.is_none_or(|m| *v <= m)
            }
            _ => true,
        };

        if !in_range || (!self.choices.is_empty() && !self.choices.contains(&value.to_string())) {
            return Err(format!(
                "Property '{}' is outside its declared range or choices.",
                self.name
            ));
        }

        Ok(())
    }

    pub(crate) fn clamp(&self, value: TrackValue) -> TrackValue {
        self.limits.clamp(value)
    }
}

/// Dynamic properties use the same values, targets and interpolation as native fields.
#[derive(Clone, Debug, Default)]
pub struct Properties(pub BTreeMap<String, (std::sync::Arc<PropertySchema>, TrackValue)>);

impl TrackValue {
    /// Rejects non-finite values before they enter a scene or animation.
    pub fn validate(&self) -> Result<(), String> {
        let finite = match self {
            Self::F32(v) => v.is_finite(),
            Self::Vector2(v) => v.is_finite(),
            Self::Vector3(v) => v.is_finite(),
            Self::Quaternion(v) => v.is_finite() && v.length_squared() > f32::EPSILON,
            Self::Color(v) => v.rgba().iter().all(|v| v.is_finite()),
            Self::Quad(v) => [v.a, v.b, v.c, v.d].iter().all(|v| v.is_finite()),
            _ => true,
        };

        if finite {
            Ok(())
        } else {
            Err("Property values must be finite; rotations must be nonzero.".to_owned())
        }
    }
}
