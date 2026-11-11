use super::Documents;
use crate::core::atomic_write;
use crate::scripting::{MANIFEST, Manifest};
use std::path::{Component, Path, PathBuf};

pub(super) fn renamed_path(path: &Path, from: &Path, to: &Path) -> Option<PathBuf> {
    let suffix = path.strip_prefix(from).ok()?;

    Some(if suffix.as_os_str().is_empty() {
        to.to_owned()
    } else {
        to.join(suffix)
    })
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum SceneTemplate {
    #[default]
    None,

    TwoD,
    ThreeD,
}

impl SceneTemplate {
    pub(crate) fn source(self) -> &'static str {
        match self {
            Self::None => "return function()\nend\n",
            Self::TwoD => include_str!("../../templates/scene_2d.lua"),
            Self::ThreeD => include_str!("../../templates/scene_3d.lua"),
        }
    }
}

impl Documents {
    pub(super) fn check_file_path(&self, path: &Path) -> Result<(), String> {
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            || !super::watched(path)
            || Path::new(MANIFEST).starts_with(path)
        {
            return Err(
                "Choose a regular project file or folder. The manifest is protected.".into(),
            );
        }

        self.context.resolve(path).map_err(|e| e.to_string())?;

        Ok(())
    }

    // Read the current disk revision rather than the last evaluated project revision.
    pub(super) fn scene_manifest(
        &self,
        change: impl FnOnce(&mut Manifest),
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, String> {
        let original =
            std::fs::read(self.context.root().join(MANIFEST)).map_err(|e| e.to_string())?;
        let mut manifest: Manifest = ron::de::from_bytes(&original).map_err(|e| e.to_string())?;
        let previous = (manifest.scenes.clone(), manifest.audio.clone());

        change(&mut manifest);

        if previous == (manifest.scenes.clone(), manifest.audio.clone()) {
            return Ok(None);
        }

        if self
            .open
            .get(Path::new(MANIFEST))
            .is_some_and(|doc| doc.dirty() || doc.conflict)
        {
            return Err("Save or resolve the manifest before changing scene files.".into());
        }

        manifest.validate(&self.context)?;

        let updated = ron::ser::to_string_pretty(&manifest, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())?
            .into_bytes();

        Ok(Some((original, updated)))
    }

    pub(super) fn write_scene_manifest(
        &mut self,
        change: Option<(Vec<u8>, Vec<u8>)>,
    ) -> Result<(), String> {
        let Some((original, updated)) = change else {
            return Ok(());
        };

        atomic_write(
            &self.context.root().join(MANIFEST),
            &updated,
            Some(&original),
        )
        .map_err(|e| e.to_string())?;

        if let Some(doc) = self.open.get_mut(Path::new(MANIFEST)) {
            doc.text = String::from_utf8(updated.clone()).map_err(|e| e.to_string())?;
            doc.disk = Some(updated);

            doc.version += 1;
        }

        self.request(MANIFEST.into());

        Ok(())
    }

    pub(crate) fn create_scene(
        &mut self,
        path: &Path,
        template: SceneTemplate,
    ) -> Result<(), String> {
        self.check_file_path(path)?;

        if path.extension().is_none_or(|ext| ext != "lua") {
            return Err("Scene filenames must end in .lua.".into());
        }

        let manifest = self.scene_manifest(|manifest| manifest.scenes.push(path.to_owned()))?;
        let full = self.context.resolve(path).map_err(|e| e.to_string())?;

        atomic_write(&full, template.source().as_bytes(), None).map_err(|e| e.to_string())?;

        if let Err(error) = self.write_scene_manifest(manifest) {
            return match std::fs::remove_file(&full) {
                Ok(()) => Err(error),
                Err(rollback) => Err(format!(
                    "{error} Could not remove the unlinked scene: {rollback}."
                )),
            };
        }

        self.file_paths = None;

        self.open(path)?;

        self.request(path.to_owned());

        Ok(())
    }

    pub(crate) fn move_files(
        &mut self,
        paths: &[PathBuf],
        folder: &Path,
    ) -> Result<Vec<PathBuf>, String> {
        let directory = if folder.as_os_str().is_empty() {
            self.context.root().to_owned()
        } else {
            self.context
                .resolve(folder)
                .map_err(|error| error.to_string())?
        };

        if !directory.is_dir() {
            return Err("Choose an existing destination folder.".into());
        }

        let directory = directory
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let mut moves = Vec::new();
        let mut destinations = std::collections::BTreeSet::new();

        for from in paths {
            self.check_file_path(from)?;

            if paths
                .iter()
                .any(|parent| parent != from && from.starts_with(parent))
            {
                return Err("Select each subtree only once.".into());
            }

            let to = folder.join(from.file_name().ok_or("Choose a project item.")?);

            self.check_file_path(&to)?;

            let source = self
                .context
                .resolve(from)
                .map_err(|error| error.to_string())?
                .canonicalize()
                .map_err(|error| error.to_string())?;

            if directory.starts_with(&source) {
                return Err("A folder cannot be moved into itself.".into());
            }

            if !destinations.insert(to.clone()) {
                return Err("Selected items have the same destination filename.".into());
            }

            if from != &to
                && self
                    .context
                    .resolve(&to)
                    .map_err(|error| error.to_string())?
                    .try_exists()
                    .map_err(|error| error.to_string())?
            {
                return Err(format!("Destination already exists: {}.", to.display()));
            }

            moves.push((from.clone(), to));
        }

        for (index, (from, to)) in moves.iter().enumerate() {
            if from == to {
                continue;
            }

            if let Err(error) = self.rename(from, to) {
                let mut message = error;

                for (old, new) in moves[..index].iter().rev().filter(|(old, new)| old != new) {
                    if let Err(error) = self.rename(new, old) {
                        message.push_str(&format!(" Could not restore {}: {error}", old.display()));
                    }
                }

                return Err(message);
            }
        }

        Ok(moves.into_iter().map(|(_, to)| to).collect())
    }

    pub(crate) fn delete_files(&mut self, paths: &[PathBuf]) -> Result<(), String> {
        for path in paths {
            self.check_file_path(path)?;

            if self
                .open
                .iter()
                .any(|(open, doc)| open.starts_with(path) && (doc.dirty() || doc.conflict))
            {
                return Err("Save or close unsaved files before deleting these items.".into());
            }

            if !self
                .context
                .resolve(path)
                .map_err(|error| error.to_string())?
                .try_exists()
                .map_err(|error| error.to_string())?
            {
                return Err(format!("Item no longer exists: {}.", path.display()));
            }
        }

        self.scene_manifest(|manifest| {
            manifest
                .scenes
                .retain(|scene| !paths.iter().any(|path| scene.starts_with(path)))
        })?;

        for path in paths {
            self.delete(path)?;
        }

        Ok(())
    }

    pub(crate) fn duplicate(&mut self, from: &Path, to: &Path) -> Result<(), String> {
        self.check_file_path(from)?;

        self.check_file_path(to)?;

        if to.starts_with(from) {
            return Err("Choose a destination outside the source folder.".into());
        }

        if self
            .open
            .iter()
            .any(|(path, doc)| path.starts_with(from) && (doc.dirty() || doc.conflict))
        {
            return Err("Save or resolve unsaved files before duplicating this item.".into());
        }

        let source = self.context.resolve(from).map_err(|e| e.to_string())?;
        let destination = self.context.resolve(to).map_err(|e| e.to_string())?;
        let metadata = source.symlink_metadata().map_err(|e| e.to_string())?;

        if metadata.is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
            return Err("Only regular files and folders can be duplicated.".into());
        }

        if !destination.parent().is_some_and(Path::is_dir) {
            return Err("Choose an existing destination folder.".into());
        }

        if destination
            .parent()
            .unwrap()
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(source.canonicalize().map_err(|e| e.to_string())?)
        {
            return Err("Choose a destination outside the source folder.".into());
        }

        let manifest = self.scene_manifest(|manifest| {
            let copies: Vec<_> = manifest
                .scenes
                .iter()
                .filter_map(|path| renamed_path(path, from, to))
                .collect();

            manifest.scenes.extend(copies);
        })?;

        fn copy(source: &Path, destination: &Path) -> std::io::Result<()> {
            let metadata = source.symlink_metadata()?;

            if metadata.is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
                return Err(std::io::Error::other(
                    "Only regular files and folders can be duplicated.",
                ));
            }

            if metadata.is_dir() {
                std::fs::create_dir(destination)?;

                for entry in std::fs::read_dir(source)? {
                    let entry = entry?;

                    copy(&entry.path(), &destination.join(entry.file_name()))?;
                }
            } else {
                let mut input = std::fs::File::open(source)?;
                let mut output = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(destination)?;

                std::io::copy(&mut input, &mut output)?;
            }

            Ok(())
        }
        // Reserve the destination before copying so cleanup never removes an existing item.
        if source.is_dir() {
            std::fs::create_dir(&destination).map_err(|e| e.to_string())?;
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|e| e.to_string())?;
        }

        let result = (|| {
            if source.is_dir() {
                for entry in std::fs::read_dir(&source).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;

                    copy(&entry.path(), &destination.join(entry.file_name()))
                        .map_err(|e| e.to_string())?;
                }
            } else {
                std::fs::copy(&source, &destination).map_err(|e| e.to_string())?;
            }

            self.write_scene_manifest(manifest)
        })();

        if let Err(error) = result {
            let cleanup = if destination.is_dir() {
                std::fs::remove_dir_all(&destination)
            } else {
                std::fs::remove_file(&destination)
            };

            return match cleanup {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!(
                    "{error} Could not remove the incomplete copy: {cleanup}."
                )),
            };
        }

        self.file_paths = None;

        self.request(to.to_owned());

        Ok(())
    }

    pub(crate) fn delete(&mut self, path: &Path) -> Result<(), String> {
        self.check_file_path(path)?;

        if self
            .open
            .iter()
            .any(|(p, doc)| p.starts_with(path) && (doc.dirty() || doc.conflict))
        {
            return Err("Save or close unsaved files before deleting this item.".into());
        }

        let manifest =
            self.scene_manifest(|manifest| manifest.scenes.retain(|p| !p.starts_with(path)))?;
        let full = self.context.resolve(path).map_err(|e| e.to_string())?;

        // Stage deletion so a manifest conflict can restore the complete subtree.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let staged = self.context.root().join(format!(
            ".kinematic-delete-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));

        if staged.try_exists().map_err(|e| e.to_string())? {
            return Err("Deletion staging path already exists. Retry the operation.".into());
        }

        std::fs::rename(&full, &staged).map_err(|e| e.to_string())?;

        if let Err(error) = self.write_scene_manifest(manifest) {
            return match std::fs::rename(&staged, &full) {
                Ok(()) => Err(error),
                Err(rollback) => Err(format!(
                    "{error} Could not restore the deleted item: {rollback}."
                )),
            };
        }

        let removed: Vec<_> = self
            .open
            .keys()
            .filter(|p| p.starts_with(path))
            .cloned()
            .collect();

        for p in removed {
            self.open.remove(&p);
        }

        if self.active.as_ref().is_some_and(|p| p.starts_with(path)) {
            self.active = self.open.keys().next().cloned();
        }

        self.file_paths = None;

        self.request(path.to_owned());

        let result = if staged.is_dir() {
            std::fs::remove_dir_all(&staged)
        } else {
            std::fs::remove_file(&staged)
        };

        if let Err(error) = result {
            self.diagnostics.push(format!(
                "Item deleted, but cleanup of {} failed: {error}.",
                staged.display()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ProjectContext;
    use crate::scripting::{LuaProject, LuaScene, Sources};

    #[test]
    fn batch_moves_preserve_buffers_and_reject_conflicts_before_changing_files() {
        let root =
            std::env::temp_dir().join(format!("kinematic-batch-files-{}", std::process::id()));
        let project = LuaProject::create(&root, "Batch", false).unwrap();
        let mut docs = Documents::new(project.context, "").unwrap();

        docs.create(Path::new("destination"), true).unwrap();
        docs.create(Path::new("a.txt"), false).unwrap();
        docs.edit(Path::new("a.txt"), "Unsaved".into());
        docs.create_scene(Path::new("b.lua"), SceneTemplate::None)
            .unwrap();

        std::fs::write(root.join("destination/b.lua"), "Existing").unwrap();

        let paths = vec![PathBuf::from("a.txt"), PathBuf::from("b.lua")];

        assert!(docs.move_files(&paths, Path::new("destination")).is_err());
        assert!(root.join("a.txt").exists());
        assert!(!root.join("destination/a.txt").exists());

        std::fs::remove_file(root.join("destination/b.lua")).unwrap();

        let moved = docs.move_files(&paths, Path::new("destination")).unwrap();

        assert_eq!(docs.open[Path::new("destination/a.txt")].text, "Unsaved");
        assert!(
            LuaProject::open(&root)
                .unwrap()
                .manifest
                .scenes
                .contains(&PathBuf::from("destination/b.lua"))
        );
        assert!(
            docs.move_files(&["destination".into()], Path::new("destination"))
                .is_err()
        );
        assert!(docs.delete_files(&moved).is_err());
        assert!(root.join("destination/b.lua").exists());

        docs.save(Path::new("destination/a.txt")).unwrap();
        docs.delete_files(&moved).unwrap();

        assert!(!root.join("destination/a.txt").exists());
        assert!(!root.join("destination/b.lua").exists());

        let all = vec![PathBuf::from("a.txt"), PathBuf::from("scenes")];

        docs.create(Path::new("a.txt"), false).unwrap();

        assert!(docs.delete_files(&all).is_err());
        assert!(root.join("a.txt").exists());
        assert!(root.join("scenes/intro.lua").exists());

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn duplicate_links_scenes_and_preserves_existing_files() {
        let root = std::env::temp_dir().join(format!("kinematic-duplicate-{}", std::process::id()));
        let project = LuaProject::create(&root, "Copies", false).unwrap();
        let mut docs = Documents::new(project.context, "").unwrap();

        docs.duplicate(Path::new("scenes"), Path::new("copies"))
            .unwrap();

        assert_eq!(
            std::fs::read(root.join("scenes/intro.lua")).unwrap(),
            std::fs::read(root.join("copies/intro.lua")).unwrap()
        );
        assert!(
            LuaProject::open(&root)
                .unwrap()
                .manifest
                .scenes
                .contains(&PathBuf::from("copies/intro.lua"))
        );
        assert!(
            docs.duplicate(Path::new("scenes"), Path::new("copies"))
                .is_err()
        );
        assert!(
            docs.duplicate(Path::new("scenes"), Path::new("scenes/nested"))
                .is_err()
        );

        docs.open(Path::new("scenes/intro.lua")).unwrap();
        docs.edit(Path::new("scenes/intro.lua"), "Unsaved".into());

        assert!(
            docs.duplicate(Path::new("scenes"), Path::new("unsaved"))
                .is_err()
        );
        assert!(!root.join("unsaved").exists());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("copies"), root.join("scenes/link")).unwrap();

            assert!(
                docs.duplicate(Path::new("copies"), Path::new("scenes/link/copy"))
                    .is_err()
            );

            docs.open.clear();

            assert!(
                docs.duplicate(Path::new("scenes"), Path::new("failed"))
                    .is_err()
            );
            assert!(!root.join("failed").exists());
        }

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scenes_link_rename_and_delete_without_losing_buffers_or_the_last_scene() {
        let root =
            std::env::temp_dir().join(format!("kinematic-file-scenes-{}", std::process::id()));
        let project = LuaProject::create(&root, "Files", false).unwrap();
        let mut docs = Documents::new(project.context.clone(), "").unwrap();

        assert!(
            docs.files()
                .unwrap()
                .iter()
                .all(|file| !file.path.starts_with("kinematic"))
        );
        assert!(docs.delete(Path::new("kinematic")).is_err());

        docs.open(Path::new(MANIFEST)).unwrap();
        docs.create(Path::new("scenes/nested"), true).unwrap();

        for (name, template) in [
            ("empty", SceneTemplate::None),
            ("two", SceneTemplate::TwoD),
            ("three", SceneTemplate::ThreeD),
        ] {
            let path = PathBuf::from(format!("scenes/nested/{name}.lua"));

            docs.create_scene(&path, template).unwrap();

            let scene = LuaScene::load(
                project.context.clone(),
                &path,
                Sources::default(),
                (1280, 720),
                60,
                crate::core::Margin::default(),
            )
            .unwrap();

            scene.update(0.0).unwrap();

            if template != SceneTemplate::None {
                assert!(scene.duration() > 0.0);
            }
        }

        assert_eq!(LuaProject::open(&root).unwrap().manifest.scenes.len(), 4);
        assert!(!docs.open[Path::new(MANIFEST)].dirty());

        let path = Path::new("scenes/nested/two.lua");

        docs.edit(path, "Unsaved".into());

        assert!(docs.delete(Path::new("scenes/nested")).is_err());

        docs.rename(Path::new("scenes/nested"), Path::new("scenes/renamed"))
            .unwrap();

        let moved = Path::new("scenes/renamed/two.lua");

        assert_eq!(docs.open[moved].text, "Unsaved");
        assert!(docs.open[moved].dirty());
        assert!(
            LuaProject::open(&root)
                .unwrap()
                .manifest
                .scenes
                .iter()
                .skip(1)
                .all(|p| p.starts_with("scenes/renamed"))
        );

        docs.open.remove(moved);
        docs.delete(Path::new("scenes/renamed")).unwrap();

        assert!(!root.join("scenes/renamed").exists());
        assert_eq!(
            LuaProject::open(&root).unwrap().manifest.scenes,
            vec![PathBuf::from("scenes/intro.lua")]
        );
        assert!(docs.delete(Path::new("scenes")).is_err());
        assert!(root.join("scenes/intro.lua").exists());

        docs.open(Path::new("scenes/intro.lua")).unwrap();
        docs.rename(Path::new("scenes/intro.lua"), Path::new("scenes/main.lua"))
            .unwrap();

        assert_eq!(docs.active.as_deref(), Some(Path::new("scenes/main.lua")));
        assert_eq!(
            LuaProject::open(&root).unwrap().manifest.scenes,
            vec![PathBuf::from("scenes/main.lua")]
        );

        LuaProject::open(&root)
            .unwrap()
            .load_scenes(&Sources::default())
            .unwrap();

        assert!(docs.delete(Path::new(MANIFEST)).is_err());
        assert!(docs.delete(Path::new(".")).is_err());

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn creation_and_manifest_updates_preserve_existing_and_conflicting_files() {
        let root =
            std::env::temp_dir().join(format!("kinematic-file-conflicts-{}", std::process::id()));
        let project = LuaProject::create(&root, "Conflicts", false).unwrap();
        let mut docs = Documents::new(ProjectContext::new(&root).unwrap(), "").unwrap();

        let original = std::fs::read(root.join(MANIFEST)).unwrap();

        std::fs::write(root.join("existing.lua"), "Existing").unwrap();

        assert!(
            docs.create_scene(Path::new("existing.lua"), SceneTemplate::None)
                .is_err()
        );
        assert_eq!(std::fs::read(root.join(MANIFEST)).unwrap(), original);
        assert_eq!(
            std::fs::read_to_string(root.join("existing.lua")).unwrap(),
            "Existing"
        );

        docs.open(Path::new(MANIFEST)).unwrap();
        docs.edit(Path::new(MANIFEST), "Unsaved manifest".into());

        assert!(
            docs.create_scene(Path::new("new.lua"), SceneTemplate::None)
                .is_err()
        );
        assert!(!root.join("new.lua").exists());
        assert!(
            docs.rename(Path::new("scenes/intro.lua"), Path::new("renamed.lua"))
                .is_err()
        );
        assert!(root.join("scenes/intro.lua").exists());

        docs.open.remove(Path::new(MANIFEST));

        let change = docs
            .scene_manifest(|manifest| manifest.scenes.push("new.lua".into()))
            .unwrap();

        std::fs::write(root.join(MANIFEST), "External manifest").unwrap();

        assert!(docs.write_scene_manifest(change).is_err());
        assert_eq!(
            std::fs::read_to_string(root.join(MANIFEST)).unwrap(),
            "External manifest"
        );
        assert!(docs.create(Path::new("../outside.lua"), false).is_err());
        assert_eq!(project.manifest.scenes.len(), 1);

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod audio_tests {
    use super::*;
    use crate::scripting::{AudioClip, LuaProject};

    #[test]
    fn file_operations_preserve_audio_associations() {
        let root =
            std::env::temp_dir().join(format!("kinematic-audio-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut project = LuaProject::create(&root, "Audio files", false).unwrap();

        std::fs::write(root.join("assets/a.wav"), b"Audio fixture.").unwrap();

        project.manifest.audio.extend([AudioClip {
            id: 1,
            lane: 0,
            file: "assets/a.wav".into(),
            at: 0.0,
            offset: 0.0,
            duration: 0.1,
            gain: 1.0,
            pan: 0.0,
        }]);
        project.save().unwrap();

        let mut docs = Documents::new(project.context.clone(), "").unwrap();

        docs.rename(Path::new("assets/a.wav"), Path::new("assets/b.wav"))
            .unwrap();
        docs.rename(Path::new("scenes/intro.lua"), Path::new("scenes/main.lua"))
            .unwrap();

        let project = LuaProject::open(&root).unwrap();
        let clips = &project.manifest.audio;

        assert_eq!(clips[0].file, Path::new("assets/b.wav"));

        docs.create_scene(Path::new("other.lua"), SceneTemplate::None)
            .unwrap();
        docs.delete(Path::new("scenes/main.lua")).unwrap();

        assert_eq!(LuaProject::open(&root).unwrap().manifest.audio, *clips);

        std::fs::remove_dir_all(root).unwrap();
    }
}
