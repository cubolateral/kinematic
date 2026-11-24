use std::collections::BTreeMap;

use crate::core::TrackValue;

pub(crate) type ShaderUniforms = BTreeMap<String, Option<TrackValue>>;
