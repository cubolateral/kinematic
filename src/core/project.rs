use serde::{Deserialize, Serialize};

use crate::core::ProjectContext;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct ProjectSettings {
    pub(crate) resolution: (u32, u32),
    pub(crate) fps: u32,
    pub(crate) margin: (u32, u32),
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            fps: 60,
            margin: (0, 0),
        }
    }
}

impl ProjectSettings {
    pub(crate) fn is_valid(settings: &Self) -> bool {
        settings.resolution.0 > 0
            && settings.resolution.1 > 0
            && settings.resolution.0.is_multiple_of(2)
            && settings.resolution.1.is_multiple_of(2)
            && settings.fps > 0
            && settings.resolution.0 <= 8192
            && settings.resolution.1 <= 8192
            && settings.fps <= 1000
            && settings.margin.0 < settings.resolution.0 / 2
            && settings.margin.1 < settings.resolution.1 / 2
    }
}

pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) context: ProjectContext,
    pub(crate) settings: ProjectSettings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_settings_have_editor_defaults() {
        assert_eq!(
            ProjectSettings::default(),
            ProjectSettings {
                resolution: (1280, 720),
                fps: 60,
                margin: (0, 0),
            }
        );
    }

    #[test]
    fn project_settings_require_exportable_values() {
        assert!(ProjectSettings::is_valid(&ProjectSettings::default()));

        assert!(!ProjectSettings::is_valid(&ProjectSettings {
            resolution: (1279, 720),
            fps: 60,
            margin: (64, 64),
        }));

        assert!(!ProjectSettings::is_valid(&ProjectSettings {
            resolution: (1280, 720),
            fps: 0,
            margin: (64, 64),
        }));
    }
}
