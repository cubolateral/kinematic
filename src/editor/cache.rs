use std::path::Path;

use serde::{Deserialize, Serialize};

const CACHE_PATH: &str = "kinematic/cache/editor.ron";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct EditorCache {
    pub(super) preview_pan: [f32; 2],
    pub(super) preview_zoom: f32,
    pub(super) camera_2d: Camera2DCache,
    pub(super) camera_3d: Camera3DCache,
    pub(super) timeline_time: f32,
    pub(super) mode: EditorMode,
    pub(super) fullscreen: bool,
    pub(super) margin_visible: bool,
    pub(super) loop_enabled: bool,
    pub(super) focused_scene: Option<String>,
}

impl Default for EditorCache {
    fn default() -> Self {
        Self {
            preview_pan: [0.0; 2],
            preview_zoom: 1.0,
            camera_2d: Camera2DCache::default(),
            camera_3d: Camera3DCache::default(),
            timeline_time: 0.0,
            mode: EditorMode::default(),
            fullscreen: false,
            margin_visible: true,
            loop_enabled: true,
            focused_scene: None,
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

impl EditorCache {
    pub(super) fn load(context: &crate::core::ProjectContext) -> Self {
        let Ok(path) = context.resolve(CACHE_PATH) else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
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

    pub(super) fn save(&self, context: &crate::core::ProjectContext) {
        let Ok(path) = context.resolve(CACHE_PATH) else {
            return;
        };

        if path.exists()
            && std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| ron::from_str::<Self>(&text).ok())
                .is_none()
        {
            return;
        }

        if let Err(error) = write(self, &path) {
            eprintln!("Could not write {CACHE_PATH}: {error}.");
        }
    }

    pub(super) fn timeline_time(&self, duration: f32) -> f32 {
        if self.timeline_time <= duration {
            self.timeline_time
        } else {
            0.0
        }
    }

    fn is_valid(cache: &Self) -> bool {
        cache.preview_pan.iter().all(|value| value.is_finite())
            && cache.preview_zoom.is_finite()
            && (0.01..=100.0).contains(&cache.preview_zoom)
            && cache.camera_2d.pan.iter().all(|value| value.is_finite())
            && cache.camera_2d.zoom.is_finite()
            && cache.camera_2d.zoom > 0.0
            && cache.camera_2d.rotation.is_finite()
            && cache
                .camera_3d
                .position
                .iter()
                .all(|value| value.is_finite())
            && cache.camera_3d.yaw.is_finite()
            && cache.camera_3d.pitch.is_finite()
            && cache.camera_3d.roll.is_finite()
            && cache.camera_3d.orbit_distance.is_finite()
            && cache.camera_3d.orbit_distance > 0.0
            && (0.01..=10000.0).contains(&cache.camera_3d.speed)
            && cache.camera_3d.perspective_fov.is_finite()
            && (0.0..std::f32::consts::PI).contains(&cache.camera_3d.perspective_fov)
            && cache.camera_3d.perspective_fov > 0.0
            && cache.camera_3d.projection().validate().is_ok()
            && cache.timeline_time.is_finite()
            && cache.timeline_time >= 0.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Camera2DCache {
    pub(super) pan: [f32; 2],
    pub(super) zoom: f32,
    pub(super) rotation: f32,
    pub(super) camera_view: bool,
    #[serde(default = "default_axes_2d")]
    pub(super) axes: [bool; 2],
}

impl Default for Camera2DCache {
    fn default() -> Self {
        Self {
            pan: [0.0; 2],
            zoom: 1.0,
            rotation: 0.0,
            camera_view: false,
            axes: default_axes_2d(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Camera3DCache {
    pub(super) position: [f32; 3],
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) roll: f32,
    pub(super) orbit_distance: f32,
    pub(super) speed: f32,
    pub(super) perspective_fov: f32,
    pub(super) mode: crate::core::components::Camera3DMode,
    pub(super) fov: f32,
    pub(super) near: f32,
    pub(super) far: f32,
    pub(super) camera_view: bool,
    #[serde(default = "default_axes_3d")]
    pub(super) axes: [bool; 3],
}

impl Default for Camera3DCache {
    fn default() -> Self {
        let camera = crate::core::components::Camera3D::default();
        Self {
            position: [4.0, 3.0, 6.0],
            yaw: 0.588,
            pitch: -0.395,
            roll: 0.0,
            orbit_distance: glam::vec3(4.0, 3.0, 6.0).length(),
            speed: 3.0,
            perspective_fov: camera.camera_fov,
            mode: camera.camera_mode,
            fov: camera.camera_fov,
            near: camera.camera_near,
            far: camera.camera_far,
            camera_view: false,
            axes: default_axes_3d(),
        }
    }
}

impl Camera3DCache {
    pub(super) fn projection(&self) -> crate::core::components::Camera3D {
        crate::core::components::Camera3D {
            camera_mode: self.mode,
            camera_fov: self.fov,
            camera_near: self.near,
            camera_far: self.far,
            ..Default::default()
        }
    }
}

fn default_axes_2d() -> [bool; 2] {
    [true; 2]
}

fn default_axes_3d() -> [bool; 3] {
    [true; 3]
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
    fn removed_timeline_preference_does_not_discard_cached_time() {
        let cache: EditorCache =
            ron::from_str("(timeline_time: 1.5, timeline_tracks_expanded: true)").unwrap();
        assert_eq!(cache.timeline_time, 1.5);
        assert_eq!(cache.preview_pan, [0.0; 2]);
        assert_eq!(cache.preview_zoom, 1.0);
    }

    #[test]
    fn preview_cache_rejects_nonfinite_pan_and_invalid_zoom() {
        let mut cache = EditorCache::default();
        assert!(EditorCache::is_valid(&cache));
        cache.preview_zoom = 0.0;
        assert!(!EditorCache::is_valid(&cache));
        cache.preview_zoom = 1.0;
        cache.preview_pan[0] = f32::NAN;
        assert!(!EditorCache::is_valid(&cache));
    }

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
            preview_pan: [25.0, -30.0],
            preview_zoom: 1.75,
            camera_2d: Camera2DCache {
                pan: [12.0, -8.0],
                zoom: 2.0,
                rotation: 0.3,
                camera_view: true,
                axes: [true, false],
            },
            camera_3d: Camera3DCache {
                position: [1.0, 2.0, 3.0],
                yaw: 0.5,
                pitch: -0.25,
                roll: 0.3,
                mode: crate::core::components::Camera3DMode::Orthogonal,
                fov: 10.0,
                camera_view: true,
                axes: [true, false, true],
                ..Default::default()
            },
            timeline_time: 1.5,
            mode: EditorMode::Three,
            fullscreen: true,
            margin_visible: false,
            loop_enabled: false,
            focused_scene: Some("Main".to_owned()),
        };

        write(&cache, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded: EditorCache = ron::from_str(&contents).unwrap();

        assert_eq!(loaded, cache);
        assert!(contents.contains("timeline_time: 1.5"));
        assert!(contents.contains("focused_scene: Some(\"Main\")"));

        std::fs::remove_dir_all(directory).unwrap();
    }
}
