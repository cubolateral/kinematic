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
    pub lsp_diagnostics: BTreeMap<PathBuf, serde_json::Value>,
    pub console: Vec<String>,
    pub revision: u64,
    pub applied: u64,
    pub changed: BTreeSet<PathBuf>,
    pub status: String,
    pub replaying: bool,
    pub lsp: Option<super::lsp::Client>,
    pub replies: Vec<super::lsp::Reply>,
    pub formatting: Option<serde_json::Value>,
    last_edit: Instant,
    file_paths: Option<Vec<FileEntry>>,
    _watcher: notify::RecommendedWatcher,
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

        watcher
            .watch(context.root(), notify::RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;

        let (lsp, diagnostics) = match super::lsp::Client::start(context.clone(), definitions) {
            Ok(client) => (Some(client), Vec::new()),
            Err(error) => (None, vec![error]),
        };

        Ok(Self {
            lsp,
            replies: Vec::new(),
            formatting: None,
            context,
            open: BTreeMap::new(),
            active: None,
            diagnostics,
            lsp_diagnostics: BTreeMap::new(),
            console: Vec::new(),
            revision: 0,
            applied: 0,
            changed: BTreeSet::new(),
            status: "Ready.".into(),
            replaying: false,
            last_edit: Instant::now(),
            file_paths: None,
            _watcher: watcher,
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

    pub fn edit(&mut self, path: &Path, text: String) {
        if let Some(document) = self.open.get_mut(path) {
            if document.text != text {
                document.text = text;

                document.version += 1;
                self.request(path.to_owned());
            }
        }
    }

    pub fn request(&mut self, path: PathBuf) {
        self.changed.insert(path);
        self.revision += 1;

        self.last_edit = Instant::now();
        self.status = "Waiting to apply edits.".into();
    }

    pub fn cancel_replay(&mut self) {
        self.revision += 1;

        self.applied = self.revision;
        self.replaying = false;
        self.status = "Replay cancelled. Edit or save to retry.".into();
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let document = self.open.get(path).ok_or("Document is not open.")?;

        if document.conflict {
            return Err("Resolve the external file conflict before saving.".into());
        }

        let mut text = document.text.clone();

        if path.extension().is_some_and(|extension| extension == "lua") {
            if let (Some(options), Some(lsp)) = (&self.formatting, &mut self.lsp) {
                match lsp.format(
                    path,
                    &document.text,
                    document.version,
                    options,
                    &mut self.replies,
                ) {
                    Ok(formatted) => text = formatted,
                    Err(error) => self
                        .diagnostics
                        .push(format!("Formatting skipped: {error}")),
                }
            }
        }

        let document = self.open.get_mut(path).unwrap();

        if let Err(error) = atomic_write(
            &self.context.resolve(path).map_err(|e| e.to_string())?,
            text.as_bytes(),
            document.disk.as_deref(),
        ) {
            document.conflict = true;

            return Err(error.to_string());
        }

        if document.text != text {
            document.text = text;

            document.version += 1;
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
        self.open.values().any(Document::dirty)
    }

    pub fn sources(&self) -> Sources {
        self.open
            .iter()
            .map(|(path, document)| (path.clone(), document.text.clone()))
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

        let manifest = self.scene_manifest(|scenes| {
            for path in scenes {
                if let Some(next) = files::renamed_path(path, from, to) {
                    *path = next;
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
            self.lsp_diagnostics.remove(&path);

            if let Some(lsp) = &mut self.lsp {
                lsp.close(&path);
            }

            if self.active.as_ref() == Some(&path) {
                self.active = Some(next);
            }
        }

        self.request(from.to_owned());
        self.request(to.to_owned());

        Ok(())
    }

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
        let mut messages = self.diagnostics.clone();

        for (path, diagnostics) in &self.lsp_diagnostics {
            if let Some(items) = diagnostics.as_array() {
                for item in items {
                    let severity = item["severity"].as_u64().unwrap_or(1);

                    if severity > 2 {
                        continue;
                    }

                    messages.push(format!(
                        "{}LuaLS {}:{}:{}: {}",
                        if severity == 2 { "Warning: " } else { "" },
                        path.display(),
                        item["range"]["start"]["line"].as_u64().unwrap_or(0) + 1,
                        item["range"]["start"]["character"].as_u64().unwrap_or(0) + 1,
                        item["message"].as_str().unwrap_or("Diagnostic."),
                    ));
                }
            }
        }

        messages
    }

    pub fn poll(&mut self) -> bool {
        if let Some(lsp) = &mut self.lsp {
            for (path, document) in &self.open {
                lsp.sync(path, &document.text, document.version);
            }

            self.replies.extend(lsp.poll());
            self.diagnostics.append(&mut lsp.messages);
        }

        for event in self.events.try_iter() {
            match event {
                Ok(event) => {
                    if matches!(event.kind, notify::EventKind::Access(_)) {
                        continue;
                    }

                    for path in event.paths {
                        if let Ok(path) = path.strip_prefix(self.context.root()) {
                            if watched(path) {
                                self.file_paths = None;

                                self.external.insert(path.to_owned());

                                self.last_edit = Instant::now();
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
fn watched(path: &Path) -> bool {
    if path == Path::new("kinematic/overrides.ron") || path.starts_with("kinematic/events") {
        return true;
    }

    !path.components().any(|part| {
        part.as_os_str().to_str().is_some_and(|s| {
            matches!(s, ".git" | "target" | "output" | ".kinematic" | "kinematic")
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
        assert_eq!(docs.sources()[path], "Local");
        assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), "Old");

        std::fs::write(root.join(path), "External").unwrap();
        assert!(docs.save(path).is_err());
        assert_eq!(docs.open[path].text, "Local");
        docs.resolve_conflict(path, true).unwrap();
        docs.save(path).unwrap();
        assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), "Local");

        drop(docs);

        std::fs::remove_dir_all(root).unwrap();
    }
}
