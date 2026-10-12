use std::{
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

/// Filesystem context shared by a project's scenes, assets and exports.
#[derive(Clone, Debug)]
pub struct ProjectContext {
    root: PathBuf,
}

impl ProjectContext {
    /// Opens an existing project directory without changing the process directory.
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().canonicalize()?;
        if !root.is_dir() {
            return Err(io::Error::other("Project root must be a directory."));
        }
        let context = Self { root };
        crate::storage::migrate_project(&context)?;
        Ok(context)
    }

    /// Returns the canonical project directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a project-relative path, rejecting escapes including symbolic links.
    pub fn resolve(&self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = path.as_ref();
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err(io::Error::other(
                "Expected a nonempty project-relative path.",
            ));
        }
        let result = self.root.join(path);
        let mut existing = result.as_path();
        while !existing.try_exists()? {
            existing = existing
                .parent()
                .ok_or_else(|| io::Error::other("Invalid project path."))?;
        }
        if !existing.canonicalize()?.starts_with(&self.root) {
            return Err(io::Error::other("Path escapes the project directory."));
        }
        Ok(result)
    }
}

/// Replaces a file only if it still matches the version read by the caller.
/// A missing expected version means that the destination must not exist.
pub(crate) fn atomic_write(
    path: &Path,
    contents: &[u8],
    expected: Option<&[u8]>,
) -> io::Result<()> {
    fn unchanged(path: &Path, expected: Option<&[u8]>) -> io::Result<()> {
        match std::fs::read(path) {
            Ok(actual) if expected == Some(actual.as_slice()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound && expected.is_none() => Ok(()),
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File changed on disk; resolve the conflict before saving.",
            )),
        }
    }
    unchanged(path, expected)?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("File needs a parent directory."))?;
    std::fs::create_dir_all(parent)?;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let temporary = parent.join(format!(
        ".kinematic-save-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(contents)?;
        if let Ok(metadata) = std::fs::metadata(path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.sync_all()?;
        drop(file);
        unchanged(path, expected)?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_detects_conflicts_and_never_truncates_them() {
        let root = std::env::temp_dir().join(format!("kinematic-atomic-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("scene.lua");
        let _ = std::fs::remove_file(&path);
        atomic_write(&path, b"Original", None).unwrap();
        std::fs::write(&path, b"External").unwrap();
        assert!(atomic_write(&path, b"Local", Some(b"Original")).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"External");
        atomic_write(&path, b"Resolved", Some(b"External")).unwrap();
        let context = ProjectContext::new(&root).unwrap();
        assert!(context.resolve("../outside.lua").is_err());
        assert_eq!(
            context.resolve("scenes/intro.lua").unwrap(),
            root.join("scenes/intro.lua")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
