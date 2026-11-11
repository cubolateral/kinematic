use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) fn global_path(name: &str) -> io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    let base = base
        .filter(|path| path.is_absolute())
        .ok_or_else(|| io::Error::other("User data directory is unavailable."))?;

    Ok(base.join(".kinematic").join(name))
}

pub(crate) fn current_project() -> Result<Option<PathBuf>, String> {
    read_current_project(&global_path("cache/current.ron").map_err(|e| e.to_string())?)
}

fn read_current_project(path: &Path) -> Result<Option<PathBuf>, String> {
    match fs::read(path) {
        Ok(bytes) => ron::de::from_bytes(&bytes).map_err(|e| e.to_string()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn save_current_project(root: Option<&Path>) -> Result<(), String> {
    write_current_project(
        &global_path("cache/current.ron").map_err(|e| e.to_string())?,
        root,
    )
}

fn write_current_project(path: &Path, root: Option<&Path>) -> Result<(), String> {
    let contents = ron::to_string(&root).map_err(|e| e.to_string())?;
    let expected = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };

    crate::core::atomic_write(path, contents.as_bytes(), expected.as_deref())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_project_round_trips_and_close_clears_it() {
        let root = std::env::temp_dir().join(format!("kinematic-current-{}", std::process::id()));

        fs::create_dir_all(&root).unwrap();

        let path = root.join("current.ron");
        let _ = fs::remove_file(&path);

        assert_eq!(read_current_project(&path).unwrap(), None);

        write_current_project(&path, Some(&root)).unwrap();

        assert_eq!(read_current_project(&path).unwrap(), Some(root.clone()));

        write_current_project(&path, None).unwrap();

        assert_eq!(read_current_project(&path).unwrap(), None);

        fs::write(&path, "Invalid").unwrap();

        assert!(read_current_project(&path).is_err());

        fs::remove_dir_all(root).unwrap();
    }
}
