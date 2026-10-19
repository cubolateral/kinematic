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
    edited_at: Instant,
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
    formatting_saves: BTreeMap<PathBuf, u64>,
    // None means LuaLS has not returned the preview yet.
    formatting_previews: BTreeMap<PathBuf, (u64, Option<String>)>,
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
            formatting_saves: BTreeMap::new(),
            formatting_previews: BTreeMap::new(),
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
                    edited_at: Instant::now(),
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
                document.edited_at = Instant::now();
                self.formatting_saves.remove(path);
                self.formatting_previews.remove(path);
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

    pub fn cancel_replay(&mut self) {
        self.revision += 1;

        self.applied = self.revision;
        self.replaying = false;
        self.status = "Replay cancelled. Save to retry.".into();
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let document = self.open.get(path).ok_or("Document is not open.")?;

        if document.conflict {
            return Err("Resolve the external file conflict before saving.".into());
        }

        self.formatting_saves.remove(path);

        let version = document.version;
        let preview = self
            .formatting_previews
            .remove(path)
            .filter(|(preview_version, _)| *preview_version == version);
        let format = self.formatting.is_some()
            && path.extension().is_some_and(|extension| extension == "lua");
        let pending = format && preview.as_ref().is_some_and(|(_, text)| text.is_none());
        let ready = format && preview.as_ref().is_some_and(|(_, text)| text.is_some());
        let text = if format {
            preview
                .and_then(|(_, text)| text)
                .unwrap_or_else(|| document.text.clone())
        } else {
            document.text.clone()
        };

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
            document.text = text.clone();
            document.version += 1;
        }

        document.disk = Some(text.as_bytes().to_vec());
        let saved_version = document.version;

        self.request(path.to_owned());

        self.last_edit = Instant::now() - Duration::from_secs(1);

        if pending {
            self.formatting_saves.insert(path.to_owned(), version);
        } else if format && !ready && saved_version == version {
            if let (Some(options), Some(lsp)) = (&self.formatting, &mut self.lsp) {
                match lsp.request_format(path, &text, version, options) {
                    Ok(_) => {
                        self.formatting_saves.insert(path.to_owned(), version);
                    }
                    Err(error) => self
                        .diagnostics
                        .push(format!("Formatting skipped: {error}")),
                }
            }
        }

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

    pub fn clear_formatting_previews(&mut self) {
        self.formatting_previews.clear();
        self.formatting_saves.clear();
        if let Some(lsp) = &mut self.lsp {
            lsp.cancel_format_requests();
        }
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

    fn apply_saved_format(&mut self, reply: super::lsp::Reply, utf8: bool) {
        if self.formatting_saves.get(&reply.path) != Some(&reply.version) {
            return;
        }

        self.formatting_saves.remove(&reply.path);

        let Some(document) = self.open.get_mut(&reply.path) else {
            return;
        };

        if document.version != reply.version || document.conflict || document.dirty() {
            return;
        }

        let formatted = match super::lsp::apply_text_edits(&document.text, &reply.value, utf8) {
            Ok(text) if text != document.text => text,
            Ok(_) => return,
            Err(error) => {
                self.diagnostics
                    .push(format!("Formatting skipped: {error}"));
                return;
            }
        };

        let result = self
            .context
            .resolve(&reply.path)
            .map_err(|error| error.to_string())
            .and_then(|path| {
                atomic_write(&path, formatted.as_bytes(), document.disk.as_deref())
                    .map_err(|error| error.to_string())
            });

        if let Err(error) = result {
            document.conflict = true;
            self.diagnostics
                .push(format!("Formatting skipped: {error}"));
            return;
        }

        document.text = formatted;
        document.disk = Some(document.text.as_bytes().to_vec());
        document.version += 1;
    }

    pub fn poll(&mut self) -> bool {
        if let Some(lsp) = &mut self.lsp {
            for (path, document) in &self.open {
                lsp.sync(path, &document.text, document.version);
            }

            if let (Some(path), Some(options)) = (&self.active, &self.formatting) {
                if let Some(document) = self.open.get(path) {
                    if self
                        .formatting_previews
                        .get(path)
                        .is_some_and(|(version, _)| *version != document.version)
                    {
                        self.formatting_previews.remove(path);
                    }

                    if document.dirty()
                        && path.extension().is_some_and(|extension| extension == "lua")
                        && document.edited_at.elapsed() >= Duration::from_millis(150)
                        && !self.formatting_previews.contains_key(path)
                        && lsp
                            .request_format(path, &document.text, document.version, options)
                            .is_ok()
                    {
                        self.formatting_previews
                            .insert(path.clone(), (document.version, None));
                    }
                }
            }

            let utf8 = lsp.utf8;
            let replies = lsp.poll();
            self.diagnostics.append(&mut lsp.messages);
            for reply in replies {
                if reply.method == "textDocument/formatting" {
                    if self.formatting_saves.get(&reply.path) == Some(&reply.version) {
                        self.apply_saved_format(reply, utf8);
                    } else if let Some((version, preview)) =
                        self.formatting_previews.get_mut(&reply.path)
                    {
                        if *version == reply.version {
                            if let Some(document) = self.open.get(&reply.path) {
                                if let Ok(text) =
                                    super::lsp::apply_text_edits(&document.text, &reply.value, utf8)
                                {
                                    *preview = Some(text);
                                } else {
                                    self.formatting_previews.remove(&reply.path);
                                }
                            }
                        }
                    }
                } else {
                    self.replies.push(reply);
                }
            }
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
fn watched(path: &Path) -> bool {
    if path == Path::new(crate::scripting::MANIFEST)
        || path == Path::new("kinematic/overrides.ron")
        || path.starts_with("kinematic/scenes")
    {
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
    fn delayed_format_only_changes_the_saved_revision() {
        let root =
            std::env::temp_dir().join(format!("kinematic-format-save-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("test.lua"), "local x=1").unwrap();

        let mut docs = Documents::new(
            ProjectContext::new(&root).unwrap(),
            &crate::scripting::definitions(),
        )
        .unwrap();
        let path = Path::new("test.lua");
        docs.open(path).unwrap();
        docs.formatting_saves.insert(path.into(), 1);

        let reply = || super::super::lsp::Reply {
            method: "textDocument/formatting".into(),
            path: path.into(),
            version: 1,
            value: serde_json::json!([{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":9}},"newText":"local x = 2"}]),
            position: None,
        };

        docs.edit(path, "local x=2".into());
        docs.apply_saved_format(reply(), true);
        assert_eq!(docs.open[path].text, "local x=2");
        assert_eq!(docs.sources()[path], "local x=1");

        docs.save(path).unwrap();
        assert!(docs.poll());
        docs.formatting_saves.insert(path.into(), 2);
        docs.apply_saved_format(
            super::super::lsp::Reply {
                version: 2,
                ..reply()
            },
            true,
        );
        assert_eq!(docs.sources()[path], "local x = 2");

        drop(docs);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ready_format_is_written_with_the_save() {
        let root =
            std::env::temp_dir().join(format!("kinematic-ready-format-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("test.lua"), "local x=1").unwrap();

        let mut docs = Documents::new(
            ProjectContext::new(&root).unwrap(),
            &crate::scripting::definitions(),
        )
        .unwrap();
        let path = Path::new("test.lua");
        docs.open(path).unwrap();
        docs.edit(path, "local x=2".into());
        docs.formatting = Some(serde_json::json!({"tabSize": 2, "insertSpaces": true}));
        docs.formatting_previews
            .insert(path.into(), (2, Some("local x = 2".into())));

        docs.save(path).unwrap();

        assert_eq!(docs.open[path].text, "local x = 2");
        assert_eq!(docs.sources()[path], "local x = 2");
        assert_eq!(
            std::fs::read_to_string(root.join(path)).unwrap(),
            "local x = 2"
        );
        assert_eq!(docs.open[path].version, 3);
        assert!(docs.formatting_saves.is_empty());

        drop(docs);
        std::fs::remove_dir_all(root).unwrap();
    }
}
