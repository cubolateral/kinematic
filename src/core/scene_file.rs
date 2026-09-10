use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct SceneFile {
    pub(crate) events: Vec<TimeEvent>,
}

impl SceneFile {
    pub(crate) fn load(name: &str) -> Self {
        let path = scene_path(name);
        match std::fs::read_to_string(&path) {
            Ok(contents) => match ron::from_str(&contents) {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Could not parse {}: {error}.", path.display());
                    let file = Self::default();
                    file.save_to(&path);
                    file
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let file = Self::default();
                file.save_to(&path);
                file
            }
            Err(error) => {
                eprintln!("Could not read {}: {error}.", path.display());
                Self::default()
            }
        }
    }

    pub(crate) fn save(&self, name: &str) {
        self.save_to(&scene_path(name));
    }

    fn save_to(&self, path: &Path) {
        if let Err(error) = write(self, path) {
            eprintln!("Could not write {}: {error}.", path.display());
        }
    }
}

fn scene_path(name: &str) -> PathBuf {
    #[cfg(not(test))]
    let root = Path::new(".kinematic/scenes").to_path_buf();
    #[cfg(test)]
    let root = std::env::temp_dir()
        .join(format!("kinematic-scene-tests-{}", std::process::id()))
        .join("scenes");

    root.join(format!("{name}.ron"))
}

fn write(file: &SceneFile, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let contents = ron::ser::to_string_pretty(file, ron::ser::PrettyConfig::default())?;
    std::fs::write(path, format!("{contents}\n"))?;
    Ok(())
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

        write(&file, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded: SceneFile = ron::from_str(&contents).unwrap();
        assert_eq!(loaded.events, file.events);
        assert!(contents.contains("name: \"show_formula\""));
        assert!(contents.contains("duration: 5.0"));

        std::fs::remove_dir_all(directory).unwrap();
    }
}
