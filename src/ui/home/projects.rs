use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn rename(path: &Path, name: &str) -> Result<(), String> {
    let mut project = crate::scripting::LuaProject::open(path)?;

    project.manifest.name = name.trim().into();

    project.save()
}

pub(super) fn duplicate(path: &Path) -> Result<PathBuf, String> {
    let source = crate::scripting::LuaProject::open(path)?;
    let root = source.context.root();
    let parent = root.parent().ok_or("Project has no parent directory.")?;
    let folder = root
        .file_name()
        .ok_or("Project folder is unavailable.")?
        .to_string_lossy();
    let mut index = 1;
    let destination = loop {
        let suffix = if index == 1 {
            " copy".into()
        } else {
            format!(" copy {index}")
        };
        let candidate = parent.join(format!("{folder}{suffix}"));

        match fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => index += 1,
            Err(error) => return Err(error.to_string()),
        }
    };
    let result = copy_files(root, &destination, Path::new(""))
        .and_then(|()| rename(&destination, &format!("{} copy", source.manifest.name)));

    if let Err(error) = result {
        let cleanup = fs::remove_dir_all(&destination);

        return Err(match cleanup {
            Ok(()) => error,
            Err(cleanup) => format!(
                "{error} Could not remove incomplete copy {}: {cleanup}.",
                destination.display()
            ),
        });
    }

    Ok(destination)
}

fn copy_files(source: &Path, destination: &Path, relative: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source.join(relative)).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = relative.join(entry.file_name());

        if [".git", "output", "kinematic/cache"]
            .iter()
            .any(|excluded| path == Path::new(excluded))
        {
            continue;
        }

        let kind = entry.file_type().map_err(|e| e.to_string())?;

        if kind.is_dir() {
            fs::create_dir(destination.join(&path)).map_err(|e| e.to_string())?;

            copy_files(source, destination, &path)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination.join(&path)).map_err(|e| e.to_string())?;
        } else {
            return Err(format!(
                "Cannot duplicate non-regular file {}.",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

pub(super) fn opened_at(seconds: u64) -> String {
    if seconds == 0 {
        return "—".into();
    }

    let Some(ticks) = seconds
        .checked_mul(1_000_000_000)
        .and_then(|n| i64::try_from(n).ok())
    else {
        return "—".into();
    };
    let mut date = sdl3::sys::time::SDL_DateTime::default();
    // SDL supplies the platform's local timezone without another date library.
    if !unsafe { sdl3::sys::time::SDL_TimeToDateTime(ticks, &mut date, true) } {
        return "—".into();
    }

    format!(
        "{:04}.{:02}.{:02} {:02}:{:02}:{:02}",
        date.year, date.month, date.day, date.hour, date.minute, date.second
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_and_duplicate_preserve_sources_and_existing_destinations() {
        let base = std::env::temp_dir().join(format!("kinematic-home-copy-{}", std::process::id()));
        let root = base.join("Project");

        crate::scripting::LuaProject::create(&root, "Original", false).unwrap();

        fs::create_dir(base.join("Project copy")).unwrap();
        fs::write(base.join("Project copy/keep.txt"), "Keep.").unwrap();
        fs::write(root.join("assets/data.txt"), "Asset.").unwrap();
        fs::create_dir_all(root.join("kinematic/cache")).unwrap();
        fs::write(root.join("kinematic/cache/editor.ron"), "Cache.").unwrap();

        assert!(rename(&root, "../Invalid").is_err());

        rename(&root, "Renamed").unwrap();

        let copy = duplicate(&root).unwrap();

        assert_eq!(copy, base.join("Project copy 2").canonicalize().unwrap());
        assert_eq!(
            crate::scripting::LuaProject::open(&copy)
                .unwrap()
                .manifest
                .name,
            "Renamed copy"
        );
        assert_eq!(
            fs::read(copy.join("scenes/intro.lua")).unwrap(),
            fs::read(root.join("scenes/intro.lua")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(copy.join("assets/data.txt")).unwrap(),
            "Asset."
        );
        assert!(!copy.join("kinematic/cache").exists());
        assert_eq!(
            fs::read_to_string(base.join("Project copy/keep.txt")).unwrap(),
            "Keep."
        );
        assert_eq!(
            crate::scripting::LuaProject::open(&root)
                .unwrap()
                .manifest
                .name,
            "Renamed"
        );

        fs::remove_dir_all(base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn duplicate_rejects_links_and_cleans_only_its_incomplete_copy() {
        let base =
            std::env::temp_dir().join(format!("kinematic-home-copy-link-{}", std::process::id()));
        let root = base.join("Project");

        crate::scripting::LuaProject::create(&root, "Original", false).unwrap();

        fs::write(base.join("outside.txt"), "Keep.").unwrap();

        std::os::unix::fs::symlink(base.join("outside.txt"), root.join("assets/link")).unwrap();

        assert!(duplicate(&root).is_err());
        assert!(!base.join("Project copy").exists());
        assert_eq!(
            fs::read_to_string(base.join("outside.txt")).unwrap(),
            "Keep."
        );
        assert!(root.is_dir());

        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn opening_dates_handle_unknown_and_out_of_range_timestamps() {
        assert_eq!(opened_at(0), "—");
        assert_eq!(opened_at(u64::MAX), "—");

        let text = opened_at(1_793_448_000);

        assert_eq!(text.len(), 19);
        assert_eq!(&text[4..5], ".");
        assert_eq!(&text[7..8], ".");
        assert_eq!(&text[16..17], ":");
    }
}
