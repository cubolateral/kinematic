use std::path::Path;

use serde::{Deserialize, Serialize};

const CACHE_PATH: &str = ".kinematic/cache/editor.ron";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct EditorCache {
    pub(super) camera_2d: Camera2DCache,
    pub(super) camera_3d: Camera3DCache,
    pub(super) timeline_time: f32,
    pub(super) mode: EditorMode,
    pub(super) fullscreen: bool,
}

impl Default for EditorCache {
    fn default() -> Self {
        Self {
            camera_2d: Camera2DCache::default(),
            camera_3d: Camera3DCache::default(),
            timeline_time: 0.0,
            mode: EditorMode::default(),
            fullscreen: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) enum EditorMode {
    #[default]
    Preview,
    Two,
    Three,
}

pub(crate) fn load_editor_mode() -> EditorMode {
    EditorCache::load().mode
}

pub(crate) fn load_editor_fullscreen() -> bool {
    EditorCache::load().fullscreen
}

impl EditorCache {
    pub(super) fn load() -> Self {
        let path = Path::new(CACHE_PATH);
        match std::fs::read_to_string(path) {
            Ok(contents) => match ron::from_str(&contents) {
                Ok(cache) if Self::is_valid(&cache) => cache,
                Ok(_) => {
                    eprintln!("Could not use {CACHE_PATH}: values must be finite and valid.");
                    Self::default()
                }
                Err(error) => {
                    eprintln!("Could not parse {CACHE_PATH}: {error}.");
                    Self::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                eprintln!("Could not read {CACHE_PATH}: {error}.");
                Self::default()
            }
        }
    }

    pub(super) fn save(&self) {
        if let Err(error) = write(self, Path::new(CACHE_PATH)) {
            eprintln!("Could not write {CACHE_PATH}: {error}.");
        }
    }

    pub(super) fn timeline_time(self, duration: f32) -> f32 {
        if self.timeline_time <= duration {
            self.timeline_time
        } else {
            0.0
        }
    }

    fn is_valid(cache: &Self) -> bool {
        cache.camera_2d.pan.iter().all(|value| value.is_finite())
            && cache.camera_2d.zoom.is_finite()
            && cache.camera_2d.zoom > 0.0
            && cache
                .camera_3d
                .position
                .iter()
                .all(|value| value.is_finite())
            && cache.camera_3d.yaw.is_finite()
            && cache.camera_3d.pitch.is_finite()
            && cache.timeline_time.is_finite()
            && cache.timeline_time >= 0.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Camera2DCache {
    pub(super) pan: [f32; 2],
    pub(super) zoom: f32,
    pub(super) camera_view: bool,
}

impl Default for Camera2DCache {
    fn default() -> Self {
        Self {
            pan: [0.0; 2],
            zoom: 1.0,
            camera_view: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Camera3DCache {
    pub(super) position: [f32; 3],
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) camera_view: bool,
}

impl Default for Camera3DCache {
    fn default() -> Self {
        Self {
            position: [4.0, 3.0, 6.0],
            yaw: 0.588,
            pitch: -0.395,
            camera_view: false,
        }
    }
}

fn write(cache: &EditorCache, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }

    let contents = ron::ser::to_string_pretty(cache, ron::ser::PrettyConfig::default())?;
    std::fs::write(path, format!("{contents}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_time_resets_when_project_is_shorter() {
        let cache = EditorCache {
            timeline_time: 4.0,
            ..EditorCache::default()
        };

        assert_eq!(cache.timeline_time(3.0), 0.0);
        assert_eq!(cache.timeline_time(4.0), 4.0);
        assert_eq!(cache.timeline_time(5.0), 4.0);
    }

    #[test]
    fn editor_cache_is_written_as_readable_ron() {
        let directory = std::env::temp_dir().join(format!(
            "kinematic-editor-cache-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = directory.join("editor.ron");
        let cache = EditorCache {
            camera_2d: Camera2DCache {
                pan: [12.0, -8.0],
                zoom: 2.0,
                camera_view: true,
            },
            camera_3d: Camera3DCache {
                position: [1.0, 2.0, 3.0],
                yaw: 0.5,
                pitch: -0.25,
                camera_view: true,
            },
            timeline_time: 1.5,
            mode: EditorMode::Three,
            fullscreen: true,
        };

        write(&cache, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded: EditorCache = ron::from_str(&contents).unwrap();
        assert_eq!(loaded, cache);
        assert!(contents.contains("timeline_time: 1.5"));

        std::fs::remove_dir_all(directory).unwrap();
    }
}
