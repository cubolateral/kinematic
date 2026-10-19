use crate::core::ProjectContext;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
};

pub(crate) struct Reply {
    pub method: String,
    pub path: PathBuf,
    pub version: u64,
    pub value: Value,
    pub position: Option<(usize, usize)>,
}
struct Request {
    position: (usize, usize),
    utf8_column: usize,
    method: String,
    path: PathBuf,
    version: u64,
}
pub(crate) struct Client {
    child: Child,
    send: mpsc::Sender<Value>,
    receive: mpsc::Receiver<Result<Value, String>>,
    requests: BTreeMap<u64, Request>,
    versions: BTreeMap<PathBuf, u64>,
    next: u64,
    ready: bool,
    started: std::time::Instant,
    failed: bool,
    queued: Vec<Value>,
    root: ProjectContext,
    pub utf8: bool,
    pub messages: Vec<String>,
}
fn executable() -> PathBuf {
    if let Some(path) = std::env::var_os("KINEMATIC_LUALS") {
        return path.into();
    }

    if let Some(home) = std::env::var_os("HOME") {
        let mason = PathBuf::from(home)
            .join(".local/share/nvim/mason/packages/lua-language-server/lua-language-server");

        if mason.is_file() {
            return mason;
        }
    }

    "lua-language-server".into()
}
impl Client {
    pub fn start(root: ProjectContext, definitions: &str) -> Result<Self, String> {
        let cache = root
            .resolve("kinematic/cache/luals")
            .map_err(|e| e.to_string())?;

        std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;

        let scenes = crate::scripting::LuaProject::open(root.root())
            .map(|project| project.manifest.scenes)
            .unwrap_or_default();

        write_scene_paths(&root, &scenes)?;

        std::fs::write(cache.join("components.lua"), luals_plugin(&root))
            .map_err(|e| e.to_string())?;

        std::fs::write(cache.join("config.json"), lua_settings(&root).to_string())
            .map_err(|e| e.to_string())?;

        let mut child = Command::new(executable())
            .arg(format!(
                "--configpath={}",
                cache.join("config.json").display()
            ))
            .arg(format!("--logpath={}", cache.join("logs").display()))
            .arg(format!("--metapath={}", cache.join("meta").display()))
            .current_dir(root.root())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!("LuaLS unavailable: {e}. Set KINEMATIC_LUALS to its executable.")
            })?;
        let mut input = child.stdin.take().ok_or("LuaLS stdin is unavailable.")?;
        let output = child.stdout.take().ok_or("LuaLS stdout is unavailable.")?;
        let (tx, rx) = mpsc::channel::<Value>();
        let (out_tx, receive) = mpsc::channel();
        let stderr = child.stderr.take().ok_or("LuaLS stderr is unavailable.")?;
        let errors = out_tx.clone();

        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().take(100) {
                if let Ok(line) = line {
                    let _ = errors.send(Err(format!("LuaLS: {line}")));
                }
            }
        });

        let errors = out_tx.clone();

        std::thread::spawn(move || {
            for message in rx {
                let bytes = message.to_string();

                if let Err(error) =
                    write!(input, "Content-Length: {}\r\n\r\n{}", bytes.len(), bytes)
                        .and_then(|_| input.flush())
                {
                    let _ = errors.send(Err(error.to_string()));

                    break;
                }
            }
        });

        std::thread::spawn(move || {
            let mut output = BufReader::new(output);

            loop {
                match read_message(&mut output) {
                    Ok(Some(message)) => {
                        if out_tx.send(Ok(message)).is_err() {
                            break;
                        }
                    }

                    Ok(None) => {
                        let _ = out_tx.send(Err("LuaLS closed its output stream.".into()));

                        break;
                    }

                    Err(error) => {
                        let _ = out_tx.send(Err(error));

                        break;
                    }
                }
            }
        });

        let library = root
            .resolve("kinematic/cache/luals")
            .map_err(|e| e.to_string())?;

        std::fs::create_dir_all(&library).map_err(|e| e.to_string())?;
        std::fs::write(library.join("kinematic.lua"), definitions).map_err(|e| e.to_string())?;

        let settings = lua_settings(&root);
        let uri = file_uri(root.root());

        tx.send(json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"processId":std::process::id(),"rootUri":uri,"workspaceFolders":[{"uri":uri,"name":"Kinematic"}],"capabilities":{"general":{"positionEncodings":["utf-8","utf-16"]},"textDocument":{"completion":{"completionItem":{"snippetSupport":false,"documentationFormat":["markdown","plaintext"],"resolveSupport":{"properties":["documentation","detail"]}}},"publishDiagnostics":{"versionSupport":true}}},"initializationOptions":{"Lua":settings,"trustByClient":true}}})).map_err(|e|e.to_string())?;

        Ok(Self {
            child,
            send: tx,
            receive,
            requests: BTreeMap::new(),
            versions: BTreeMap::new(),
            next: 1,
            ready: false,
            started: std::time::Instant::now(),
            failed: false,
            queued: Vec::new(),
            root,
            utf8: false,
            messages: Vec::new(),
        })
    }

    fn send(&mut self, message: Value) {
        if self.failed {
            return;
        }

        if self.ready {
            if let Err(error) = self.send.send(message) {
                self.messages.push(error.to_string());
            }
        } else {
            self.queued.push(message);
        }
    }

    pub(crate) fn set_scenes(&mut self, paths: &[PathBuf]) {
        if let Err(error) = write_scene_paths(&self.root, paths) {
            self.messages.push(error);
        }

        for path in self.versions.keys().cloned().collect::<Vec<_>>() {
            self.close(&path);
        }
    }

    pub fn sync(&mut self, path: &Path, text: &str, version: u64) {
        if !analyzed_path(&self.root, path) || self.versions.get(path) == Some(&version) {
            return;
        }

        let uri = file_uri(&self.root.root().join(path));
        let message = if self.versions.insert(path.to_owned(), version).is_some() {
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]}})
        } else {
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"lua","version":version,"text":text}}})
        };

        self.send(message);
    }

    pub fn close(&mut self, path: &Path) {
        self.versions.remove(path);
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":file_uri(&self.root.root().join(path))}}}));
    }

    pub fn request(
        &mut self,
        method: &str,
        path: &Path,
        text: &str,
        version: u64,
        line: usize,
        column: usize,
    ) {
        if !analyzed_path(&self.root, path) {
            return;
        }

        self.sync(path, text, version);

        let id = self.next;

        self.next += 1;

        self.requests.insert(
            id,
            Request {
                position: (line, column),
                utf8_column: position_units(text, line, column, true),
                method: method.into(),
                path: path.to_owned(),
                version,
            },
        );

        let character = position_units(text, line, column, self.utf8);

        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"textDocument":{"uri":file_uri(&self.root.root().join(path))},"position":{"line":line,"character":character}}}));
    }

    pub fn resolve_completion(
        &mut self,
        path: &Path,
        text: &str,
        version: u64,
        line: usize,
        column: usize,
        item: Value,
    ) {
        let id = self.next;

        self.next += 1;

        self.requests.insert(
            id,
            Request {
                position: (line, column),
                utf8_column: position_units(text, line, column, true),
                method: "completionItem/resolve".into(),
                path: path.to_owned(),
                version,
            },
        );

        self.send(
            json!({"jsonrpc":"2.0", "id":id, "method":"completionItem/resolve", "params":item}),
        );
    }

    pub fn format(
        &mut self,
        path: &Path,
        text: &str,
        version: u64,
        options: &Value,
        replies: &mut Vec<Reply>,
    ) -> Result<String, String> {
        let id = self.request_format(path, text, version, options)?;

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);

        while std::time::Instant::now() < deadline {
            let mut result = None;

            for reply in self.poll() {
                if reply.method == "textDocument/formatting"
                    && reply.path == path
                    && reply.version == version
                {
                    result = Some(apply_text_edits(text, &reply.value, self.utf8));
                } else {
                    replies.push(reply);
                }
            }

            if let Some(result) = result {
                return result;
            }

            if self.failed {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        self.requests.remove(&id);

        Err("LuaLS formatting timed out.".into())
    }

    pub fn request_format(
        &mut self,
        path: &Path,
        text: &str,
        version: u64,
        options: &Value,
    ) -> Result<u64, String> {
        if !analyzed_path(&self.root, path) {
            return Err("Formatting requires a project Lua file.".into());
        }

        if !self.ready || self.failed {
            return Err("LuaLS is not ready.".into());
        }

        self.requests.retain(|_, request| {
            request.method != "textDocument/formatting" || request.path != path
        });
        self.sync(path, text, version);

        let id = self.next;

        self.next += 1;

        self.requests.insert(
            id,
            Request {
                position: (0, 0),
                utf8_column: 0,
                method: "textDocument/formatting".into(),
                path: path.to_owned(),
                version,
            },
        );

        self.send(json!({"jsonrpc":"2.0", "id":id, "method":"textDocument/formatting",
            "params":{"textDocument":{"uri":file_uri(&self.root.root().join(path))}, "options":options}}));

        Ok(id)
    }

    pub fn cancel_format_requests(&mut self) {
        self.requests
            .retain(|_, request| request.method != "textDocument/formatting");
    }

    pub fn poll(&mut self) -> Vec<Reply> {
        let mut replies = Vec::new();

        if !self.ready && !self.failed && self.started.elapsed().as_secs() >= 10 {
            self.failed = true;

            let _ = self.child.kill();

            self.queued.clear();
            self.requests.clear();

            self.messages.push(
                "LuaLS initialization timed out. Editing and execution remain available.".into(),
            );
        }

        for message in self.receive.try_iter().collect::<Vec<_>>() {
            let value = match message {
                Ok(v) => v,
                Err(error) => {
                    self.messages.push(error);

                    continue;
                }
            };

            if value.get("id") == Some(&json!(0)) && value.get("method").is_none() {
                self.utf8 = value
                    .pointer("/result/capabilities/positionEncoding")
                    .and_then(Value::as_str)
                    == Some("utf-8");

                self.ready = true;

                let _ = self
                    .send
                    .send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
                let _ = self.send.send(json!({"jsonrpc":"2.0","method":"workspace/didChangeConfiguration","params":{"settings":{"Lua":lua_settings(&self.root)}}}));

                for mut message in self.queued.drain(..) {
                    if self.utf8 && message["params"].get("position").is_some() {
                        if let Some(request) =
                            message["id"].as_u64().and_then(|id| self.requests.get(&id))
                        {
                            message["params"]["position"]["character"] = json!(request.utf8_column);
                        }
                    }

                    let _ = self.send.send(message);
                }

                continue;
            }

            if let Some(method) = value.get("method").and_then(Value::as_str) {
                if let Some(id) = value.get("id") {
                    let result = if method == "workspace/configuration" {
                        Value::Array(
                            value
                                .pointer("/params/items")
                                .and_then(Value::as_array)
                                .map_or(Vec::new(), |items| {
                                    items
                                        .iter()
                                        .map(|item| {
                                            let settings = lua_settings(&self.root);

                                            match item["section"].as_str().unwrap_or("") {
                                                "Lua" => settings,
                                                "Lua.workspace" => settings["workspace"].clone(),
                                                "Lua.runtime" => settings["runtime"].clone(),
                                                "Lua.diagnostics" => {
                                                    settings["diagnostics"].clone()
                                                }
                                                "Lua.files" => settings["files"].clone(),
                                                "Lua.completion" => settings["completion"].clone(),
                                                _ => Value::Null,
                                            }
                                        })
                                        .collect()
                                }),
                        )
                    } else {
                        Value::Null
                    };
                    let _ = self
                        .send
                        .send(json!({"jsonrpc":"2.0","id":id,"result":result}));
                }

                if method == "textDocument/publishDiagnostics" {
                    let Some(uri) = value.pointer("/params/uri").and_then(Value::as_str) else {
                        continue;
                    };

                    let Some(path) = uri_path(uri).and_then(|path| {
                        path.strip_prefix(self.root.root()).ok().map(Path::to_owned)
                    }) else {
                        continue;
                    };

                    if !analyzed_path(&self.root, &path) {
                        continue;
                    }

                    if let Some(version) = diagnostic_version(
                        value.pointer("/params/version").and_then(Value::as_u64),
                        self.versions.get(&path).copied(),
                    ) {
                        replies.push(Reply {
                            position: None,
                            method: method.into(),
                            path,
                            version,
                            value: value["params"]["diagnostics"].clone(),
                        });
                    }
                }

                continue;
            }

            if let Some(id) = value.get("id").and_then(Value::as_u64) {
                if let Some(request) = self.requests.remove(&id) {
                    if self.versions.get(&request.path) == Some(&request.version) {
                        replies.push(Reply {
                            position: Some(request.position),
                            method: request.method,
                            path: request.path,
                            version: request.version,
                            value: value.get("result").cloned().unwrap_or(Value::Null),
                        });
                    }
                }
            }
        }

        replies
    }
}
pub(crate) fn analyzed_path(root: &ProjectContext, path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "lua")
        && !path.components().any(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some(".git" | "target" | "output" | "kinematic" | ".kinematic")
            )
        })
        && root.resolve(path).is_ok()
}
fn write_scene_paths(root: &ProjectContext, paths: &[PathBuf]) -> Result<(), String> {
    let contents = paths
        .iter()
        .map(|path| root.resolve(path).map(|path| file_uri(&path)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .join("\n");

    std::fs::write(
        root.root().join("kinematic/cache/luals/scene-paths.txt"),
        contents,
    )
    .map_err(|e| e.to_string())
}

fn luals_plugin(root: &ProjectContext) -> String {
    let list = serde_json::to_string(
        &root
            .root()
            .join("kinematic/cache/luals/scene-paths.txt")
            .to_string_lossy(),
    )
    .unwrap();

    format!(
        "local function is_scene(uri)\n    local file = io.open({list}, 'r')\n    if not file then return false end\n    for line in file:lines() do\n        if line == uri then file:close(); return true end\n    end\n    file:close()\n    return false\nend\n{}",
        include_str!("../scripting/luals.lua")
    )
}

fn lua_settings(root: &ProjectContext) -> Value {
    json!({
        "runtime": {"version": "Lua 5.4", "plugin": root.root().join("kinematic/cache/luals/components.lua")},
        "workspace": {"library": [root.root().join("kinematic/cache/luals")], "checkThirdParty": false,
            "ignoreDir": ["kinematic", ".kinematic", "output", "target", ".git"]},
        "completion": {"showWord": "Disable", "workspaceWord": false, "maxSuggestCount": 500, "callSnippet": "Disable"},
        "diagnostics": {"libraryFiles": "Disable", "ignoredFiles": "Disable"},
        "files": {"associations": {"*.lua": "lua"}},
    })
}
fn diagnostic_version(reported: Option<u64>, current: Option<u64>) -> Option<u64> {
    match (reported, current) {
        (Some(reported), Some(current)) if reported != current => None,
        (Some(reported), _) => Some(reported),
        (None, current) => Some(current.unwrap_or(0)),
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.send.send(json!({"jsonrpc":"2.0","method":"exit"}));
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut size = None;
    let mut header = String::new();
    let mut total = 0;

    loop {
        header.clear();

        if reader.read_line(&mut header).map_err(|e| e.to_string())? == 0 {
            return Ok(None);
        }

        total += header.len();

        if total > 8192 {
            return Err("LuaLS header exceeds its size limit.".into());
        }

        if header.trim().is_empty() {
            break;
        }

        if let Some((key, value)) = header.split_once(':') {
            if key.eq_ignore_ascii_case("Content-Length") {
                size = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?);
            }
        }
    }

    let size = size.ok_or("LuaLS message lacks Content-Length.")?;

    if size > 16 * 1024 * 1024 {
        return Err("LuaLS message exceeds its size limit.".into());
    }

    let mut bytes = vec![0; size];

    reader.read_exact(&mut bytes).map_err(|e| e.to_string())?;

    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| e.to_string())
}
pub(crate) fn file_uri(path: &Path) -> String {
    let mut result = String::from("file://");

    for byte in path.to_string_lossy().as_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(byte) {
            result.push(*byte as char);
        } else {
            result.push_str(&format!("%{byte:02X}"));
        }
    }

    result
}
pub(crate) fn uri_path(uri: &str) -> Option<PathBuf> {
    let bytes = uri.strip_prefix("file://")?.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            result.push(
                u8::from_str_radix(std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?, 16).ok()?,
            );

            i += 3;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(result).ok().map(PathBuf::from)
}
fn edit_offset(text: &str, position: &Value, utf8: bool) -> Result<usize, String> {
    let line = position["line"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or("Invalid formatting line.")?;
    let units = position["character"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or("Invalid formatting column.")?;
    let mut offset = 0;

    for (index, source) in text.split('\n').enumerate() {
        if index == line {
            let source = source.strip_suffix('\r').unwrap_or(source);
            let mut used = 0;

            for (byte, character) in source.char_indices() {
                if used == units {
                    return Ok(offset + byte);
                }

                used += if utf8 {
                    character.len_utf8()
                } else {
                    character.len_utf16()
                };
            }

            if used == units {
                return Ok(offset + source.len());
            }

            return Err("Formatting column is outside the text or splits a character.".into());
        }

        offset += source.len() + 1;
    }

    Err("Formatting line is outside the document.".into())
}
pub(crate) fn apply_text_edits(text: &str, value: &Value, utf8: bool) -> Result<String, String> {
    if value.is_null() {
        return Ok(text.to_owned());
    }

    let mut edits = value
        .as_array()
        .ok_or("Invalid formatting edits.")?
        .iter()
        .map(|edit| {
            let start = edit_offset(text, &edit["range"]["start"], utf8)?;
            let end = edit_offset(text, &edit["range"]["end"], utf8)?;
            let replacement = edit["newText"].as_str().ok_or("Invalid formatting text.")?;

            if replacement.contains('\0') || start > end {
                return Err("Invalid formatting range or text.".into());
            }

            Ok((start, end, replacement))
        })
        .collect::<Result<Vec<_>, String>>()?;

    edits.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));

    let mut result = text.to_owned();
    let mut previous_start = text.len();

    for (start, end, replacement) in edits {
        if end > previous_start {
            return Err("Formatting edits overlap.".into());
        }

        result.replace_range(start..end, replacement);

        previous_start = start;
    }

    Ok(result)
}

pub(crate) fn position_units(text: &str, line: usize, column: usize, utf8: bool) -> usize {
    let prefix = text
        .lines()
        .nth(line)
        .unwrap_or("")
        .chars()
        .take(column)
        .collect::<String>();

    if utf8 {
        prefix.len()
    } else {
        prefix.encode_utf16().count()
    }
}
pub(crate) fn glyph_column(text: &str, line: usize, units: usize, utf8: bool) -> usize {
    let mut count = 0;
    let mut used = 0;

    for c in text.lines().nth(line).unwrap_or("").chars() {
        let next = used + if utf8 { c.len_utf8() } else { c.len_utf16() };

        if next > units {
            break;
        }

        used = next;

        count += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_accept_optional_versions_and_reject_old_results() {
        assert_eq!(diagnostic_version(None, Some(3)), Some(3));
        assert_eq!(diagnostic_version(Some(3), Some(3)), Some(3));
        assert_eq!(diagnostic_version(Some(2), Some(3)), None);
        assert_eq!(diagnostic_version(None, None), Some(0));
        assert_eq!(diagnostic_version(Some(3), None), Some(3));
    }
    #[test]
    fn analysis_is_limited_to_project_lua_sources() {
        let root = std::env::temp_dir().join(format!("kinematic-lsp-scope-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();

        let context = ProjectContext::new(&root).unwrap();

        assert!(analyzed_path(&context, Path::new("scenes/main.lua")));

        for path in [
            "notes.txt",
            "project.ron",
            "scene.LUA",
            "../outside.lua",
            "/tmp/outside.lua",
            "kinematic/cache/luals/kinematic.lua",
            "kinematic/cache/luals/meta/LuaJIT.lua",
            "output/file.lua",
        ] {
            assert!(!analyzed_path(&context, Path::new(path)), "{path}");
        }

        let settings = lua_settings(&context);

        assert_eq!(settings["runtime"]["version"], "Lua 5.4");
        assert_eq!(settings["diagnostics"]["libraryFiles"], "Disable");

        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn formatting_edits_validate_unicode_and_overlapping_ranges() {
        let edit = |start, end, replacement| json!({"range": {"start": {"line": 0, "character": start}, "end": {"line": 0, "character": end}}, "newText": replacement});

        assert_eq!(
            apply_text_edits("a😀ç", &json!([edit(1, 3, "b")]), false).unwrap(),
            "abç"
        );

        assert!(apply_text_edits("a😀ç", &json!([edit(2, 3, "b")]), false).is_err());

        assert!(
            apply_text_edits("abcd", &json!([edit(0, 2, "x"), edit(1, 3, "y")]), true).is_err()
        );

        assert!(apply_text_edits("abc", &json!([edit(0, 99, "x")]), true).is_err());

        assert_eq!(
            edit_offset("a\r\n", &json!({"line": 1, "character": 0}), true).unwrap(),
            3
        );

        assert_eq!(apply_text_edits("abc", &Value::Null, true).unwrap(), "abc");
    }

    #[test]
    fn unicode_positions_and_uri_roundtrip() {
        let text = "a😀ç.x";

        assert_eq!(position_units(text, 0, 3, false), 4);
        assert_eq!(position_units(text, 0, 3, true), 7);
        assert_eq!(glyph_column(text, 0, 4, false), 3);

        let path = Path::new("/tmp/á b.lua");

        assert_eq!(uri_path(&file_uri(path)).as_deref(), Some(path));

        let mut reader = std::io::Cursor::new(b"Content-Length: 2\r\n\r\n{}");

        assert_eq!(read_message(&mut reader).unwrap(), Some(json!({})));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    #[test]
    fn real_server_completes_native_fields_and_resolves_module_definitions() {
        if !executable().is_file()
            && Command::new(executable())
                .arg("--version")
                .output()
                .is_err()
        {
            eprintln!("LuaLS integration unavailable: executable not installed.");

            return;
        }

        let root = std::env::temp_dir().join(format!("kinematic-luals-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(
            root.join("helper.lua"),
            "local M = {}\nfunction M.answer() return 42 end\nreturn M\n",
        )
        .unwrap();

        std::fs::write(root.join("kinematic.ron"), "(version:1,name:\"Test\",scenes:[\"intro.lua\"],resolution:(640,480),fps:60,margin:(0,0))").unwrap();

        let mut client = Client::start(
            ProjectContext::new(&root).unwrap(),
            &crate::scripting::definitions(),
        )
        .unwrap();
        let text = "\nreturn function(s)\nlocal ball = s:circle { id = 'ball' }\nball.\nend\n";
        let path = Path::new("intro.lua");

        client.sync(path, text, 1);
        client.request("textDocument/completion", path, text, 1, 3, 5);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut completed = false;
        let mut last_request = std::time::Instant::now();

        while std::time::Instant::now() < deadline {
            for reply in client.poll() {
                if reply.method == "textDocument/completion" {
                    let items = reply
                        .value
                        .as_array()
                        .or_else(|| reply.value.get("items").and_then(Value::as_array));

                    completed = items.is_some_and(|items| {
                        items
                            .iter()
                            .any(|item| item["label"].as_str() == Some("radius"))
                    });
                }
            }

            if completed {
                break;
            }

            if last_request.elapsed() > std::time::Duration::from_millis(500) {
                client.request("textDocument/completion", path, text, 1, 3, 5);

                last_request = std::time::Instant::now();
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert!(
            completed,
            "LuaLS must complete native properties: {:?}",
            client.messages
        );

        let text = "local serviço = require('helper')\nserviço.answer()\n";

        client.sync(path, text, 2);
        client.request("textDocument/definition", path, text, 2, 1, 10);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut found = false;

        while std::time::Instant::now() < deadline {
            for reply in client.poll() {
                if reply.method == "textDocument/definition" {
                    found = reply.value.to_string().contains("helper.lua");
                }
            }

            if found {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert!(
            found,
            "LuaLS must navigate across project modules with Unicode."
        );

        let text = "\nlocal point = k.vec2(1, 2)\n";

        client.sync(path, text, 3);
        client.request("textDocument/hover", path, text, 3, 1, 17);
        client.request("textDocument/signatureHelp", path, text, 3, 1, 23);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let (mut hover, mut signature) = (false, false);

        while std::time::Instant::now() < deadline && !(hover && signature) {
            for reply in client.poll() {
                if reply.method == "textDocument/hover" {
                    hover = reply.value.to_string().contains("vec2");
                } else if reply.method == "textDocument/signatureHelp" {
                    signature = reply.value["signatures"]
                        .as_array()
                        .is_some_and(|values| !values.is_empty());
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert!(
            hover && signature,
            "LuaLS must provide hover and function signatures."
        );

        let unformatted = "local value={1,2}  \nif value then\nreturn value\nend";
        let formatted = client.format(path, unformatted, 4, &json!({
            "tabSize": 2, "insertSpaces": true, "trimTrailingWhitespace": true, "insertFinalNewline": true,
        }), &mut Vec::new()).unwrap();

        assert_ne!(formatted, unformatted);
        assert!(formatted.contains("\n  return value\n"), "{formatted:?}");
        assert!(!formatted.contains("  \n"), "{formatted:?}");
        assert!(formatted.ends_with('\n'), "{formatted:?}");
        client.sync(Path::new("notes.txt"), "This is not Lua.", 1);

        client.request(
            "textDocument/completion",
            Path::new("settings.ron"),
            "()",
            1,
            0,
            0,
        );

        assert!(!client.versions.contains_key(Path::new("notes.txt")));
        assert!(!client.versions.contains_key(Path::new("settings.ron")));

        let text = "\nlocal ribbon = k.object { bounds = function() return {x = 0, y = 0, width = 10, height = 10} end }\nreturn function(s)\n  local shape = s:object(ribbon, {id = 'shape'})\n  shape.opacity:set(0)\n  shape:to {opacity = 1}:play(0.5)\n  local ball = s:circle { radius = 10 }\n  ball.radius:set(5)\n  ball.opacity:set(0)\n  ball.opacity:set('Invalid')\n  missing_name()\nend\n";

        client.sync(path, text, 5);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut diagnostics = None;

        while std::time::Instant::now() < deadline {
            for reply in client.poll() {
                assert!(analyzed_path(&client.root, &reply.path));

                if reply.method == "textDocument/publishDiagnostics"
                    && reply.path == path
                    && reply.version == 5
                {
                    diagnostics = Some(reply.value);
                }
            }

            if diagnostics.as_ref().is_some_and(|items| {
                items.as_array().is_some_and(|items| {
                    items.iter().any(|item| item["code"] == "undefined-global")
                })
            }) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        let diagnostics = diagnostics.expect("LuaLS must publish project diagnostics.");
        let items = diagnostics.as_array().unwrap();

        assert!(
            items.iter().any(|item| item["code"] == "undefined-global"),
            "{diagnostics}"
        );

        assert!(
            items
                .iter()
                .any(|item| item["code"] == "param-type-mismatch"
                    && item["range"]["start"]["line"] == 9),
            "{diagnostics}"
        );

        assert!(
            !items.iter().any(|item| item["code"] == "undefined-field"
                || (item["code"] == "param-type-mismatch" && item["range"]["start"]["line"] != 9)),
            "{diagnostics}"
        );

        let component_path = Path::new("components.lua");
        let component_text = "\nlocal motion = k.component { speed = {default = 40}, mode = {default = 'Idle', choices = {'Idle', 'Moving'}} }\nreturn { motion = motion }\n";

        std::fs::write(root.join(component_path), component_text).unwrap();
        client.sync(component_path, component_text, 1);

        let text = r#"local components = require('components')
local extra = k.component { caption = {default = 'Hello'} }
local definition = k.object { components = {components.motion, extra} }
local volume = k.object { dimension = 3 }
local counter = k.object { methods = { increment = function(self,amount) return self:by {value=amount} end } }
return function(s)
  local ball = s:circle { radius = 10 }
  local center = ball.position:get()
  ball:signal(function(obj, frame)
    obj.position:set(center + k.vec2(0, math.sin(frame.time * 3) * 40))
    obj.size:set(k.vec2(10, 10))
  end)
  local prism = s:prism {}
  prism.size:set(k.vec3(10, 20, 30))
  prism:signal(function(obj) obj.size:set(k.vec3(1, 2, 3)) end)
  local custom = s:object(definition, {})
  custom.speed:set(20)
  custom.caption:set('World')
  custom.mode:set('Moving')
  custom:signal(function(obj) obj.speed:set(30); obj.position:set(k.vec2(0, 0)) end)
  custom.sped:set(20)
  custom.speed:set('Invalid')
  custom.mode:set('Invalid')
  local three = s:object(volume, {})
  local text = s:text_2d {}
  text:fade('New'):play(1)
  local count = s:object(counter,{})
  count:increment(10):play(2)
  three.position:set(k.vec3(1, 2, 3))
  three.missing:set(1)
  missing_name()
end
"#;

        client.sync(path, text, 6);

        let diagnostics = wait_diagnostics(&mut client, path, 6);
        let items = diagnostics.as_array().unwrap();
        let line = |fragment: &str| {
            text.lines()
                .position(|line| line.contains(fragment))
                .unwrap() as u64
        };

        for fragment in ["obj.size:set(k.vec2", "custom.sped", "three.missing"] {
            assert!(
                items.iter().any(|item| item["code"] == "undefined-field"
                    && item["range"]["start"]["line"] == line(fragment)),
                "{diagnostics}"
            );
        }

        for fragment in ["custom.speed:set('Invalid')", "custom.mode:set('Invalid')"] {
            assert!(
                items
                    .iter()
                    .any(|item| item["code"] == "param-type-mismatch"
                        && item["range"]["start"]["line"] == line(fragment)),
                "{diagnostics}"
            );
        }

        let invalid_lines = [
            "obj.size:set(k.vec2",
            "custom.sped",
            "custom.speed:set('Invalid')",
            "custom.mode:set('Invalid')",
            "three.missing",
            "missing_name()",
        ]
        .map(line);

        assert!(
            items
                .iter()
                .all(|item| invalid_lines
                    .contains(&item["range"]["start"]["line"].as_u64().unwrap())),
            "{diagnostics}"
        );

        let component_text = component_text.replace("speed =", "velocity =");

        client.sync(component_path, &component_text, 2);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut updated = false;

        while std::time::Instant::now() < deadline && !updated {
            for reply in client.poll() {
                if reply.method == "textDocument/publishDiagnostics" && reply.path == path {
                    updated = reply.value.as_array().is_some_and(|items| {
                        items.iter().any(|item| {
                            item["code"] == "undefined-field"
                                && item["range"]["start"]["line"] == line("custom.speed:set(20)")
                        })
                    });
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert!(
            updated,
            "Component edits must update fields in importing scenes."
        );

        let formatted = client
            .format(
                path,
                text,
                6,
                &json!({"tabSize": 2, "insertSpaces": true}),
                &mut Vec::new(),
            )
            .unwrap();

        assert!(
            !formatted.contains("---@class") && !formatted.contains("@as Kinematic.Custom_"),
            "Formatting must preserve original project sources."
        );

        let text = "return function(s)\n  local ball = s:circle {radius=24}\n  ball.radius:get()\n  local label = s:text_2d {text='Hi'}\n  label:fade('Bye'):play(1)\n  local point = k.vec2(1, 2)\n  local center = k.rect(0, 0, 10, 10):middle()\nend\n";

        client.sync(path, text, 7);

        for (line, token, expected) in [
            (1, "circle", "Built-in circular scene object."),
            (2, "radius", "Radius in local coordinates."),
            (2, "get", "Reads the current value"),
            (4, "fade", "Replaces the text and fades"),
            (5, "vec2", "Creates a two-dimensional value"),
            (6, "middle", "point of this rectangle"),
        ] {
            let column = text.lines().nth(line).unwrap().find(token).unwrap();

            client.request("textDocument/hover", path, text, 7, line, column);

            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let mut contents = None;

            while std::time::Instant::now() < deadline && contents.is_none() {
                for reply in client.poll() {
                    if reply.method == "textDocument/hover"
                        && reply.position == Some((line, column))
                    {
                        contents = Some(reply.value.to_string());
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(20));
            }

            let contents = contents.expect("LuaLS must return documentation.");

            assert!(contents.contains(expected), "{token}: {contents}");
        }

        let column = text.lines().nth(4).unwrap().find("fade").unwrap();

        client.request("textDocument/definition", path, text, 7, 4, column);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut location = None;

        while std::time::Instant::now() < deadline && location.is_none() {
            for reply in client.poll() {
                if reply.method == "textDocument/definition" {
                    location = Some(reply.value);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        let location = location.unwrap();
        let location = location.as_array().unwrap().first().unwrap();

        assert!(location["uri"].as_str().unwrap().ends_with("kinematic.lua"));

        let line = location["range"]["start"]["line"].as_u64().unwrap() as usize;
        let definitions = crate::scripting::definitions();

        assert!(definitions.lines().nth(line).unwrap().contains("fade"));

        assert!(
            definitions
                .lines()
                .skip(line.saturating_sub(8))
                .take(8)
                .any(|line| line.contains("Replaces the text and fades"))
        );

        drop(client);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn real_server_completes_specific_values_and_custom_methods() {
        if Command::new(executable())
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("LuaLS integration unavailable: executable not installed.");

            return;
        }

        let root =
            std::env::temp_dir().join(format!("kinematic-completion-{}", std::process::id()));

        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("kinematic.ron"), "(version:1,name:\"Test\",scenes:[\"intro.lua\"],resolution:(640,480),fps:60,margin:(0,0))").unwrap();

        std::fs::write(
            root.join("custom.lua"),
            r#"
local function origin() return k.vec2(1, 2) end
local function heading(object) return object.velocity:get():normalized() end
local motion = k.component { velocity = {default = k.vec2(10, 20)}, speed = {default = 40}, offset = {default = origin()}, multiplier = {default = 2 * 3} }
return k.object {
    components = {motion},
    methods = {
        heading = heading,
        increment = function(self, amount) return self:by {speed = amount} end,
    },
}
"#,
        )
        .unwrap();

        let mut client = Client::start(
            ProjectContext::new(&root).unwrap(),
            &crate::scripting::definitions(),
        )
        .unwrap();
        let path = Path::new("intro.lua");

        for (index, (expression, expected, excluded)) in [
            ("k.vec2(1, 2).", "x", "r"),
            ("k.vec2(1, 2):", "length", "cross"),
            ("k.vec3(1, 2, 3):", "cross", "r"),
            ("k.color(1, 0, 0).", "r", "x"),
            ("ball.position:", "get", "cross"),
            ("ball.position:get():", "normalized", "cross"),
            ("ball.radius.", "get", "x"),
            ("custom.", "velocity", "radius"),
            ("custom:", "heading", "fade"),
            ("custom:heading():", "length", "cross"),
            ("custom.offset:get():", "length", "cross"),
            ("custom.multiplier.", "get", "x"),
            ("custom:increment(1):", "play", "length"),
            ("s:circle { ra", "radius", "velocity"),
        ]
        .into_iter()
        .enumerate()
        {
            let text = format!(
                "local definition = require('custom')\nreturn function(s)\nlocal ball = s:circle {{}}\nlocal custom = s:object(definition, {{}})\n{expression}\nend\n"
            );
            let version = index as u64 + 1;
            let column = expression.chars().count();

            client.sync(path, &text, version);

            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            let mut labels = Vec::new();
            let mut completion = Value::Null;

            while std::time::Instant::now() < deadline {
                client.request("textDocument/completion", path, &text, version, 4, column);

                let until = std::time::Instant::now() + std::time::Duration::from_millis(200);

                while std::time::Instant::now() < until {
                    for reply in client.poll() {
                        if reply.method == "textDocument/completion" && reply.version == version {
                            completion = reply.value.clone();

                            labels = reply
                                .value
                                .as_array()
                                .or_else(|| reply.value["items"].as_array())
                                .into_iter()
                                .flatten()
                                .filter_map(|item| item["label"].as_str().map(str::to_owned))
                                .collect();
                        }
                    }

                    if labels.iter().any(|label| label.starts_with(expected)) {
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                if labels.iter().any(|label| label.starts_with(expected)) {
                    break;
                }
            }

            assert!(
                labels.iter().any(|label| label.starts_with(expected)),
                "{expression} must suggest {expected}: {labels:?}; {:?}",
                client.messages
            );

            assert!(
                !labels
                    .iter()
                    .any(|label| label == excluded || label.starts_with(&format!("{excluded}("))),
                "{expression} must not suggest {excluded}: {labels:?}"
            );

            if expression == "custom:" {
                assert!(
                    labels.iter().any(|label| label.starts_with("increment(")
                        && label.contains("amount")
                        && !label.contains("...")),
                    "Custom signatures must preserve parameters: {labels:?}"
                );
            }

            if expression == "k.vec2(1, 2)." {
                let item = completion
                    .as_array()
                    .or_else(|| completion["items"].as_array())
                    .unwrap()
                    .iter()
                    .find(|item| item["label"] == "x")
                    .unwrap()
                    .clone();

                client.resolve_completion(path, &text, version, 4, column, item);

                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                let mut resolved = None;

                while std::time::Instant::now() < deadline && resolved.is_none() {
                    for reply in client.poll() {
                        if reply.method == "completionItem/resolve"
                            && reply.position == Some((4, column))
                        {
                            resolved = Some(reply.value);
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                let item = resolved.expect("LuaLS must resolve completion documentation.");

                assert!(
                    item["documentation"].to_string().contains("component"),
                    "{item}"
                );
            }
        }

        drop(client);

        std::fs::remove_dir_all(root).unwrap();
    }

    fn wait_diagnostics(client: &mut Client, path: &Path, version: u64) -> Value {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);

        while std::time::Instant::now() < deadline {
            for reply in client.poll() {
                if reply.method == "textDocument/publishDiagnostics"
                    && reply.path == path
                    && reply.version == version
                    && reply.value.as_array().is_some_and(|items| {
                        items.iter().any(|item| item["code"] == "undefined-global")
                    })
                {
                    return reply.value;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        panic!("LuaLS must publish updated field diagnostics.");
    }
}
