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
            Self::None => "return function(s)\nend\n",
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
        let previous = (
            manifest.scenes.clone(),
            manifest.audio.clone(),
            manifest.legacy_audio.clone(),
        );

        change(&mut manifest);
        manifest
            .legacy_audio
            .retain(|path, _| manifest.scenes.contains(path));

        if previous
            == (
                manifest.scenes.clone(),
                manifest.audio.clone(),
                manifest.legacy_audio.clone(),
            )
        {
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

            if let Some(lsp) = &mut self.lsp {
                lsp.close(&p);
            }
        }

        self.lsp_diagnostics.retain(|p, _| !p.starts_with(path));

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
    fn scenes_link_rename_and_delete_without_losing_buffers_or_the_last_scene() {
        let root =
            std::env::temp_dir().join(format!("kinematic-file-scenes-{}", std::process::id()));
        let project = LuaProject::create(&root, "Files", false).unwrap();
        let mut docs = Documents::new(project.context.clone(), "").unwrap();

        docs.lsp = None;

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

        docs.lsp = None;

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
        docs.lsp = None;
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
