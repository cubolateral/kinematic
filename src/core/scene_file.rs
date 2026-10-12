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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_file_round_trips_readable_ron() {
        let directory = std::env::temp_dir().join(format!(
            "kinematic-scene-file-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = directory.join("example.ron");
        let file = SceneFile {
            events: vec![
                TimeEvent {
                    name: "intro".to_owned(),
                    duration: 0.0,
                },
                TimeEvent {
                    name: "show_formula".to_owned(),
                    duration: 5.0,
                },
            ],
        };

        let contents =
            ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(&path, contents).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded: SceneFile = ron::from_str(&contents).unwrap();
        assert_eq!(loaded.events, file.events);
        assert!(contents.contains("name: \"show_formula\""));
        assert!(contents.contains("duration: 5.0"));

        std::fs::remove_dir_all(directory).unwrap();
    }
}
