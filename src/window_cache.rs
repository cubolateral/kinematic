use std::path::Path;

use serde::{Deserialize, Serialize};

const CACHE_PATH: &str = ".kinematic/cache/window.ron";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct WindowCache {
    pub(crate) position: Option<(i32, i32)>,
    pub(crate) size: (u32, u32),
    pub(crate) maximized: bool,
}

impl Default for WindowCache {
    fn default() -> Self {
        Self {
            position: None,
            size: (1280, 720),
            maximized: false,
        }
    }
}

impl WindowCache {
    pub(crate) fn load() -> Self {
        let path = Path::new(CACHE_PATH);
        match std::fs::read_to_string(path) {
            Ok(contents) => match ron::from_str(&contents) {
                Ok(cache) if Self::is_valid(&cache) => cache,
                Ok(_) => {
                    eprintln!("Could not use {CACHE_PATH}: window size must be valid.");
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

    pub(crate) fn update(&mut self, window: &sdl3::video::Window) {
        if window.is_minimized() {
            return;
        }
        if window.is_maximized() {
            self.maximized = true;
            return;
        }

        self.position = Some(window.position());
        self.size = window.size();
        self.maximized = false;
    }

    pub(crate) fn save(&self) {
        if let Err(error) = write(self, Path::new(CACHE_PATH)) {
            eprintln!("Could not write {CACHE_PATH}: {error}.");
        }
    }

    fn is_valid(cache: &Self) -> bool {
        cache.size.0 > 0
            && cache.size.1 > 0
            && cache.size.0 <= i32::MAX as u32
            && cache.size.1 <= i32::MAX as u32
    }
}

fn write(cache: &WindowCache, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
    fn window_cache_is_written_as_readable_ron() {
        let directory = std::env::temp_dir().join(format!(
            "kinematic-window-cache-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = directory.join("window.ron");
        let cache = WindowCache {
            position: Some((-1920, 120)),
            size: (1600, 900),
            maximized: true,
        };

        write(&cache, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let loaded: WindowCache = ron::from_str(&contents).unwrap();
        assert_eq!(loaded, cache);
        assert!(contents.contains("position: Some((-1920, 120))"));
        assert!(contents.contains("size: (1600, 900)"));
        assert!(contents.contains("maximized: true"));

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn window_cache_rejects_invalid_sizes() {
        assert!(WindowCache::is_valid(&WindowCache::default()));
        assert!(!WindowCache::is_valid(&WindowCache {
            size: (0, 720),
            ..WindowCache::default()
        }));
    }
}
