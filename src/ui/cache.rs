use std::{io::ErrorKind, path::Path};

use serde::{Deserialize, Serialize};

const CACHE_PATH: &str = "cache/editor.ron";

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Cache {
    pub audio_volume: i32,
    pub code_zoom: f32,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            audio_volume: 100,
            code_zoom: 0.0,
        }
    }
}

impl Cache {
    pub fn load() -> Self {
        let Ok(path) = crate::storage::global_path(CACHE_PATH) else {
            return Self::default();
        };
        read(&path).unwrap_or_else(|error| {
            eprintln!("Could not load editor cache: {error}.");
            Self::default()
        })
    }

    pub fn save(&self) {
        if let Ok(path) = crate::storage::global_path(CACHE_PATH) {
            if let Err(error) = write(self, &path) {
                eprintln!("Could not save editor cache: {error}.");
            }
        }
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !(0..=100).contains(&self.audio_volume)
            || !self.code_zoom.is_finite()
            || !(-18.0..=18.0).contains(&self.code_zoom)
        {
            return Err("Editor cache volume and zoom must be finite and within range.".into());
        }
        Ok(())
    }
}

fn read(path: &Path) -> Result<Cache, Box<dyn std::error::Error>> {
    let contents = match std::fs::read(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Cache::default()),
        Err(error) => return Err(error.into()),
    };
    let cache: Cache = ron::de::from_bytes(&contents)?;
    cache.validate()?;
    Ok(cache)
}

fn write(cache: &Cache, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    cache.validate()?;
    let expected = match std::fs::read(path) {
        Ok(contents) => {
            ron::de::from_bytes::<Cache>(&contents)?;
            Some(contents)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let contents = ron::ser::to_string_pretty(cache, ron::ser::PrettyConfig::default())?;
    crate::core::atomic_write(
        path,
        format!("{contents}\n").as_bytes(),
        expected.as_deref(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_and_text_zoom_round_trip_and_reject_invalid_cache() {
        let root = std::env::temp_dir().join(format!("kinematic-ui-cache-{}", std::process::id()));
        let path = root.join("cache/editor.ron");
        let cache = Cache {
            audio_volume: 35,
            code_zoom: 4.0,
        };
        assert_eq!(read(&path).unwrap(), Cache::default());
        write(&cache, &path).unwrap();
        assert_eq!(read(&path).unwrap(), cache);
        std::fs::write(&path, "(audio_volume: 0)").unwrap();
        assert_eq!(
            read(&path).unwrap(),
            Cache {
                audio_volume: 0,
                ..Cache::default()
            }
        );
        for text in [
            "(audio_volume: 101)",
            "(code_zoom: NaN)",
            "(code_zoom: 19.0)",
        ] {
            std::fs::write(&path, text).unwrap();
            assert!(read(&path).is_err());
        }
        std::fs::write(&path, "Invalid cache.").unwrap();
        assert!(write(&cache, &path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "Invalid cache.");
        std::fs::remove_dir_all(root).unwrap();
    }
}
