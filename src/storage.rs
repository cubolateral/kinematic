use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::core::ProjectContext;

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

pub(crate) fn migrate_project(context: &ProjectContext) -> io::Result<()> {
    for name in [
        "project.ron",
        "scenes",
        "events",
        "overrides.ron",
        "cache/editor.ron",
        "cache/luals",
    ] {
        migrate(
            &context.resolve(format!(".kinematic/{name}"))?,
            &context.resolve(format!("kinematic/{name}"))?,
        )?;
    }

    for (source, destination) in [
        ("kinematic.ron", crate::scripting::MANIFEST),
        ("kinematic/events", "kinematic/scenes"),
    ] {
        migrate(&context.resolve(source)?, &context.resolve(destination)?)?;
    }

    Ok(())
}

pub(crate) fn migrate_globals(context: &ProjectContext) -> io::Result<()> {
    for name in ["settings.ron", "cache/window.ron"] {
        migrate(
            &context.resolve(format!(".kinematic/{name}"))?,
            &global_path(name)?,
        )?;
    }

    // The old docking layout contains the removed panels.
    let layout = context.resolve(".kinematic/cache/layout.ini")?;

    if layout.try_exists()? {
        fs::remove_file(layout)?;
    }

    for name in [".kinematic/cache", ".kinematic"] {
        match fs::remove_dir(context.resolve(name)?) {
            Ok(()) => {}

            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(error) => return Err(error),
        }
    }

    Ok(())
}

fn migrate(source: &Path, destination: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(source) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    if metadata.is_symlink() {
        return Err(io::Error::other("Cannot migrate symbolic links."));
    }

    for ancestor in destination.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.is_symlink() => {
                return Err(io::Error::other("Cannot migrate into symbolic links."));
            }

            Ok(_) => {}

            Err(error) if error.kind() == io::ErrorKind::NotFound => {}

            Err(error) => return Err(error),
        }
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(source)? {
            let entry = entry?;

            migrate(&entry.path(), &destination.join(entry.file_name()))?;
        }

        if fs::read_dir(source)?.next().is_none() {
            fs::remove_dir(source)?;
        }
    } else {
        let contents = fs::read(source)?;

        if !destination.try_exists()? {
            crate::core::atomic_write(destination, &contents, None)?;
        }

        // Keep conflicting files so migration never discards project data.
        if fs::read(destination)? == contents {
            fs::remove_file(source)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_layout_migrates_without_losing_scene_metadata() {
        let root =
            std::env::temp_dir().join(format!("kinematic-project-layout-{}", std::process::id()));
        let project = crate::scripting::LuaProject::create(&root, "Migration", false).unwrap();
        let manifest = root.join(crate::scripting::MANIFEST);
        let events = "(events: [(name: \"Hold\", duration: 2.5)])";

        fs::write(
            root.join("scenes/intro.lua"),
            "return function(s) s:event('Hold') end",
        )
        .unwrap();
        fs::rename(&manifest, root.join("kinematic.ron")).unwrap();
        fs::create_dir_all(root.join("kinematic/events/scenes")).unwrap();
        fs::write(root.join("kinematic/events/scenes/intro.ron"), events).unwrap();

        let reopened = crate::scripting::LuaProject::open(&root).unwrap();

        assert_eq!(reopened.manifest.name, project.manifest.name);
        assert!(manifest.is_file());
        assert!(!root.join("kinematic.ron").exists());
        assert!(!root.join("kinematic/events").exists());
        assert_eq!(
            fs::read_to_string(root.join("kinematic/scenes/scenes/intro.ron")).unwrap(),
            events
        );
        let scenes = reopened
            .load_scenes(&crate::scripting::Sources::default())
            .unwrap();

        assert_eq!(scenes[0].duration(), 2.5);

        fs::remove_dir_all(root).unwrap();
    }

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

    #[test]
    fn migration_preserves_conflicts_and_removes_only_copied_files() {
        let root = std::env::temp_dir().join(format!("kinematic-migration-{}", std::process::id()));
        let source = root.join("old");
        let destination = root.join("new");

        fs::create_dir_all(&source).unwrap();

        fs::create_dir_all(&destination).unwrap();

        fs::write(source.join("scene.ron"), "Original").unwrap();

        fs::write(source.join("conflict.ron"), "Old").unwrap();

        fs::write(destination.join("conflict.ron"), "New").unwrap();

        migrate(&source, &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("scene.ron")).unwrap(),
            "Original"
        );

        assert!(!source.join("scene.ron").exists());

        assert_eq!(
            fs::read_to_string(source.join("conflict.ron")).unwrap(),
            "Old"
        );

        assert_eq!(
            fs::read_to_string(destination.join("conflict.ron")).unwrap(),
            "New"
        );

        fs::remove_file(destination.join("conflict.ron")).unwrap();

        migrate(&source, &destination).unwrap();
        assert!(!source.exists());

        fs::remove_dir_all(root).unwrap();
    }
}
