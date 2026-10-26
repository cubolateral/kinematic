use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct TimeEvent {
    pub(crate) name: String,
    #[serde(alias = "time")]
    pub(crate) duration: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct ScheduledEvent {
    pub(crate) file_index: usize,
    pub(crate) name: String,
    pub(crate) creation_time: f32,
    pub(crate) duration: f32,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct SceneFile {
    pub(crate) events: Vec<TimeEvent>,
}

impl SceneFile {
    pub(crate) fn read(path: &Path) -> std::io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => ron::from_str(&text).map_err(std::io::Error::other),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }
}
