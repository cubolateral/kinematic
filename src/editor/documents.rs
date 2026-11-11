use crate::core::{ProjectContext, atomic_write};
use crate::scripting::Sources;
use notify::Watcher;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

mod files;
pub(crate) use files::SceneTemplate;

#[derive(Clone)]
pub(crate) struct FileEntry {
    pub path: PathBuf,
    pub directory: bool,
}

pub(crate) struct Document {
    pub text: String,
    pub version: u64,
    pub disk: Option<Vec<u8>>,
    pub conflict: bool,
}
impl Document {
    pub fn dirty(&self) -> bool {
        self.disk.as_deref() != Some(self.text.as_bytes())
    }
}

pub(crate) struct Documents {
    pub context: ProjectContext,
    pub open: BTreeMap<PathBuf, Document>,
    pub active: Option<PathBuf>,
    pub diagnostics: Vec<String>,
    pub console: Vec<String>,
    pub revision: u64,
    pub applied: u64,
    pub changed: BTreeSet<PathBuf>,
    pub status: String,
    pub replaying: bool,
    pub project_dirty: bool,
    last_edit: Instant,
    file_paths: Option<Vec<FileEntry>>,
    watcher: notify::RecommendedWatcher,
    events: mpsc::Receiver<notify::Result<notify::Event>>,
    external: BTreeSet<PathBuf>,
}
impl Documents {
    pub fn new(context: ProjectContext, definitions: &str) -> Result<Self, String> {
        let (tx, events) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = tx.send(event);
        })
        .map_err(|e| e.to_string())?;

        watch_directory(&mut watcher, context.root(), context.root())?;

        let diagnostics = match super::external::prepare_luals(&context, definitions) {
            Ok(Some(message)) => vec![message],
            Ok(None) => Vec::new(),
            Err(error) => vec![format!("Could not prepare external LuaLS: {error}.")],
        };

        Ok(Self {
            context,
            open: BTreeMap::new(),
            active: None,
            diagnostics,
            console: Vec::new(),
            revision: 0,
            applied: 0,
            changed: BTreeSet::new(),
            status: "Ready.".into(),
            replaying: false,
            project_dirty: false,
            last_edit: Instant::now(),
            file_paths: None,
            watcher,
            events,
            external: BTreeSet::new(),
        })
    }

    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        if !self.open.contains_key(path) {
            let bytes = std::fs::read(self.context.resolve(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let text = String::from_utf8(bytes.clone()).map_err(|e| e.to_string())?;

            if text.contains('\0') {
                return Err("Text files cannot contain NUL bytes.".into());
            }

            self.open.insert(
                path.to_owned(),
                Document {
                    text,
                    version: 1,
                    disk: Some(bytes),
                    conflict: false,
                },
            );
        }

        self.active = Some(path.to_owned());

        Ok(())
    }

    #[cfg(test)]
    pub fn edit(&mut self, path: &Path, text: String) {
        if let Some(document) = self.open.get_mut(path) {
            if document.text != text {
                document.text = text;
                document.version += 1;
                self.status = "Unsaved edits; preview uses saved files.".into();
            }
        }
    }

    pub fn request(&mut self, path: PathBuf) {
        self.changed.insert(path);
        self.revision += 1;
        self.last_edit = Instant::now();
        self.status = "Waiting to apply edits.".into();
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let document = self.open.get_mut(path).ok_or("Document is not open.")?;
        if document.conflict {
            return Err("Resolve the external file conflict before saving.".into());
        }
        if let Err(error) = atomic_write(
            &self.context.resolve(path).map_err(|e| e.to_string())?,
            document.text.as_bytes(),
            document.disk.as_deref(),
        ) {
            document.conflict = true;
            return Err(error.to_string());
        }
        document.disk = Some(document.text.as_bytes().to_vec());
        self.request(path.to_owned());
        self.last_edit = Instant::now() - Duration::from_secs(1);
        Ok(())
    }

    pub fn save_all(&mut self) -> Result<(), String> {
        let paths = self
            .open
            .iter()
            .filter(|(_, d)| d.dirty())
            .map(|(p, _)| p.clone())
            .collect::<Vec<_>>();

        for path in paths {
            self.save(&path)?;
        }

        Ok(())
    }

    pub fn dirty(&self) -> bool {
        self.project_dirty || self.open.values().any(Document::dirty)
    }

    pub fn sources(&self) -> Sources {
        self.open
            .iter()
            .filter_map(|(path, document)| {
                document.disk.as_ref().and_then(|bytes| {
                    String::from_utf8(bytes.clone())
                        .ok()
                        .map(|text| (path.clone(), text))
                })
            })
            .collect()
    }

    pub fn files(&mut self) -> Result<Vec<FileEntry>, String> {
        if let Some(files) = &self.file_paths {
            return Ok(files.clone());
        }

        fn walk(root: &Path, base: &Path, out: &mut Vec<FileEntry>) -> Result<(), String> {
            for entry in std::fs::read_dir(base).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                let relative = path.strip_prefix(root).map_err(|e| e.to_string())?;

                if !watched(relative) || entry.file_type().map_err(|e| e.to_string())?.is_symlink()
                {
                    continue;
                }

                let directory = entry.file_type().map_err(|e| e.to_string())?.is_dir();

                out.push(FileEntry {
                    path: relative.to_owned(),
                    directory,
                });

                if directory {
                    walk(root, &path, out)?;
                }
            }

            Ok(())
        }

        let mut files = Vec::new();

        walk(self.context.root(), self.context.root(), &mut files)?;

        files.sort_by_cached_key(|entry| {
            (
                !entry.directory,
                entry.path.to_string_lossy().to_lowercase(),
            )
        });

        self.file_paths = Some(files.clone());

        Ok(files)
    }

    pub fn create(&mut self, path: &Path, directory: bool) -> Result<(), String> {
        self.check_file_path(path)?;

        self.file_paths = None;

        let full = self.context.resolve(path).map_err(|e| e.to_string())?;

        if directory {
            std::fs::create_dir(&full).map_err(|e| e.to_string())?;
        } else {
            atomic_write(&full, b"", None).map_err(|e| e.to_string())?;

            self.open(path)?;
        }

        self.request(path.to_owned());

        Ok(())
    }

    pub fn rename(&mut self, from: &Path, to: &Path) -> Result<(), String> {
        self.check_file_path(from)?;

        self.check_file_path(to)?;

        let manifest = self.scene_manifest(|manifest| {
            for path in &mut manifest.scenes {
                if let Some(next) = files::renamed_path(path, from, to) {
                    *path = next;
                }
            }

            for clip in &mut manifest.audio {
                if let Some(next) = files::renamed_path(&clip.file, from, to) {
                    clip.file = next;
                }
            }
        })?;

        self.file_paths = None;

        let old = self.context.resolve(from).map_err(|e| e.to_string())?;
        let new = self.context.resolve(to).map_err(|e| e.to_string())?;

        if new.exists() {
            return Err("Destination already exists.".into());
        }

        std::fs::rename(&old, &new).map_err(|e| e.to_string())?;

        if let Err(error) = self.write_scene_manifest(manifest) {
            return match std::fs::rename(&new, &old) {
                Ok(()) => Err(error),
                Err(rollback) => Err(format!(
                    "{error} Could not restore the old path: {rollback}."
                )),
            };
        }

        let moved = self
            .open
            .keys()
            .filter(|path| path.starts_with(from))
            .cloned()
            .collect::<Vec<_>>();

        for path in moved {
            let next = files::renamed_path(&path, from, to).unwrap();
            let doc = self.open.remove(&path).unwrap();

            self.open.insert(next.clone(), doc);

            if self.active.as_ref() == Some(&path) {
                self.active = Some(next);
            }
        }

        self.request(from.to_owned());
        self.request(to.to_owned());

        Ok(())
    }

    #[cfg(test)]
    pub fn resolve_conflict(&mut self, path: &Path, keep_local: bool) -> Result<(), String> {
        let bytes = std::fs::read(self.context.resolve(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let doc = self.open.get_mut(path).ok_or("Document is not open.")?;

        if !keep_local {
            doc.text = String::from_utf8(bytes.clone()).map_err(|e| e.to_string())?;

            doc.version += 1;
        }

        doc.disk = Some(bytes);
        doc.conflict = false;

        self.request(path.to_owned());

        Ok(())
    }

    pub fn diagnostic_messages(&self) -> Vec<String> {
        self.diagnostics.clone()
    }

    pub fn poll(&mut self) -> bool {
        for event in self.events.try_iter() {
            match event {
                Ok(event) => {
                    if matches!(event.kind, notify::EventKind::Access(_)) {
                        continue;
                    }

                    for path in event.paths {
                        if let Ok(path) = path.strip_prefix(self.context.root()) {
                            if matches!(
                                event.kind,
                                notify::EventKind::Create(_)
                                    | notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
                            ) {
                                let full = self.context.root().join(path);

                                if full.is_dir() {
                                    if let Err(error) = watch_directory(
                                        &mut self.watcher,
                                        self.context.root(),
                                        &full,
                                    ) {
                                        self.diagnostics.push(error);
                                    }
                                }
                            }

                            if watched(path) {
                                self.file_paths = None;

                                self.external.insert(path.to_owned());

                                if !self.changed.contains(path) {
                                    self.last_edit = Instant::now();
                                }
                            }
                        }
                    }
                }

                Err(error) => self.diagnostics.push(error.to_string()),
            }
        }

        if self.last_edit.elapsed() < Duration::from_millis(400) {
            return false;
        }

        for path in std::mem::take(&mut self.external) {
            let bytes = self.context.resolve(&path).and_then(std::fs::read);

            if let Some(document) = self.open.get_mut(&path) {
                if bytes.as_ref().ok() == document.disk.as_ref() {
                    continue;
                }

                if document.dirty() {
                    document.conflict = true;

                    self.status = "External edit conflicts with an unsaved document.".into();

                    continue;
                }

                match bytes {
                    Ok(bytes) => match String::from_utf8(bytes.clone()) {
                        Ok(text) => {
                            document.text = text;
                            document.disk = Some(bytes);

                            document.version += 1;
                        }

                        Err(error) => {
                            self.diagnostics.push(error.to_string());

                            continue;
                        }
                    },
                    Err(error) => {
                        document.conflict = true;

                        self.diagnostics.push(error.to_string());

                        continue;
                    }
                }
            }

            self.changed.insert(path);

            self.revision += 1;
        }

        self.revision != self.applied
    }
}

fn watch_directory(
    watcher: &mut notify::RecommendedWatcher,
    root: &Path,
    directory: &Path,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        // Windows cannot rename a directory while its descendants have open watch handles.
        if directory == root {
            watcher
                .watch(root, notify::RecursiveMode::Recursive)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let relative = directory.strip_prefix(root).map_err(|e| e.to_string())?;

        if !watched(relative) && relative != Path::new("kinematic") {
            return Ok(());
        }

        if std::fs::symlink_metadata(directory)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Ok(());
        }

        watcher
            .watch(directory, notify::RecursiveMode::NonRecursive)
            .map_err(|e| e.to_string())?;

        for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;

            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                watch_directory(watcher, root, &entry.path())?;
            }
        }

        Ok(())
    }
}

fn watched(path: &Path) -> bool {
    if path == Path::new(crate::scripting::MANIFEST) || path.starts_with("kinematic/scenes") {
        return true;
    }

    !path.components().any(|part| {
        part.as_os_str().to_str().is_some_and(|s| {
            matches!(s, ".git" | "target" | "output" | "kinematic")
                || s.starts_with(".kinematic-save-")
                || s.starts_with(".kinematic-delete-")
                || s.ends_with('~')
                || s.ends_with(".swp")
                || s.ends_with(".tmp")
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_skips_ignored_trees_but_keeps_project_files_and_metadata() {
        let root = std::env::temp_dir().join(format!("kinematic-watch-{}", std::process::id()));

        for path in [
            "scenes/nested",
            "kinematic/scenes",
            "kinematic/target",
            "target",
            ".git",
        ] {
            std::fs::create_dir_all(root.join(path)).unwrap();
        }

        let (send, receive) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = send.send(event);
        })
        .unwrap();

        watch_directory(&mut watcher, &root, &root).unwrap();

        for path in [
            "kinematic/target/ignored.lua",
            "target/ignored.lua",
            ".git/ignored.lua",
            "scenes/nested/intro.lua",
            "kinematic/project.ron",
            "kinematic/scenes/intro.ron",
        ] {
            std::fs::write(root.join(path), "Test.").unwrap();
        }

        let mut changed = BTreeSet::new();

        while let Ok(event) = receive.recv_timeout(Duration::from_millis(200)) {
            let event = event.unwrap();

            if !matches!(event.kind, notify::EventKind::Access(_)) {
                changed.extend(
                    event
                        .paths
                        .into_iter()
                        .filter(|path| path.strip_prefix(&root).is_ok_and(watched)),
                );
            }
        }

        for path in [
            "scenes/nested/intro.lua",
            "kinematic/project.ron",
            "kinematic/scenes/intro.ron",
        ] {
            assert!(
                changed.contains(&root.join(path)),
                "Missing event for {path}."
            );
        }

        for path in [
            "kinematic/target/ignored.lua",
            "target/ignored.lua",
            ".git/ignored.lua",
        ] {
            assert!(
                !changed.contains(&root.join(path)),
                "Unexpected event for {path}."
            );
        }

        drop(watcher);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn monitoring_picks_up_created_and_renamed_directories() {
        let root = std::env::temp_dir().join(format!("kinematic-watch-new-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();

        let mut docs = Documents::new(ProjectContext::new(&root).unwrap(), "").unwrap();

        let wait_for_change = |docs: &mut Documents, path: &Path| {
            let deadline = Instant::now() + Duration::from_secs(2);

            while !docs.external.contains(path) && !docs.changed.contains(path) {
                docs.poll();

                assert!(
                    Instant::now() < deadline,
                    "Missing event for {}.",
                    path.display()
                );

                std::thread::sleep(Duration::from_millis(10));
            }
        };

        std::fs::create_dir_all(root.join("new/nested")).unwrap();

        wait_for_change(&mut docs, Path::new("new"));

        std::fs::write(root.join("new/nested/intro.lua"), "Original.").unwrap();

        wait_for_change(&mut docs, Path::new("new/nested/intro.lua"));

        std::fs::rename(root.join("new"), root.join("moved")).unwrap();

        wait_for_change(&mut docs, Path::new("moved"));

        std::fs::write(root.join("moved/nested/intro.lua"), "Updated.").unwrap();

        wait_for_change(&mut docs, Path::new("moved/nested/intro.lua"));

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsaved_buffers_win_and_external_conflicts_keep_both_versions() {
        let root = std::env::temp_dir().join(format!("kinematic-documents-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("test.lua"), "Old").unwrap();

        let mut docs = Documents::new(
            ProjectContext::new(&root).unwrap(),
            &crate::scripting::definitions(),
        )
        .unwrap();
        let path = Path::new("test.lua");

        docs.open(path).unwrap();
        docs.edit(path, "Local".into());

        assert_eq!(docs.sources()[path], "Old");
        assert_eq!(docs.revision, 0);
        assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), "Old");

        std::fs::write(root.join(path), "External").unwrap();

        assert!(docs.save(path).is_err());
        assert_eq!(docs.open[path].text, "Local");

        docs.resolve_conflict(path, true).unwrap();
        docs.save(path).unwrap();

        assert_eq!(docs.sources()[path], "Local");
        assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), "Local");

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn external_atomic_saves_refresh_sources_and_invalid_lua_keeps_the_scene() {
        let directory =
            std::env::temp_dir().join(format!("kinematic-external-reload-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let context = ProjectContext::new(&directory).unwrap();
        let path = Path::new("intro.lua");
        let source = "return function() k.world_2d():add(k.circle {radius=24}) end";
        std::fs::write(directory.join(path), source).unwrap();
        let mut docs = Documents::new(context.clone(), &crate::scripting::definitions()).unwrap();
        docs.open(path).unwrap();
        let mut scene = crate::scripting::LuaScene::load(
            context,
            path,
            docs.sources(),
            (640, 480),
            60,
            crate::core::Margin::default(),
        )
        .unwrap();
        let wait = |docs: &mut Documents| {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !docs.poll() {
                assert!(Instant::now() < deadline, "External save was not observed.");
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        for (text, valid) in [
            (source.replace("24", "48"), true),
            ("Invalid Lua ???".into(), false),
        ] {
            let before = docs.revision;
            crate::core::atomic_write(
                &directory.join(path),
                text.as_bytes(),
                Some(&std::fs::read(directory.join(path)).unwrap()),
            )
            .unwrap();
            wait(&mut docs);
            assert!(docs.revision > before);
            assert_eq!(docs.sources()[path], text);
            assert_eq!(scene.reload(docs.sources()).is_ok(), valid);
            docs.applied = docs.revision;
            docs.changed.clear();
        }
        let scene = scene.scene();
        let radii: Vec<_> = scene
            .world()
            .query::<&crate::core::objects::CircleShape>()
            .iter()
            .map(|circle| circle.radius)
            .collect();
        assert_eq!(radii, vec![48.0]);
        drop(docs);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
