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
}
struct Request {
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
        let mut child = Command::new(executable())
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
        let uri = file_uri(root.root());
        tx.send(json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"processId":std::process::id(),"rootUri":uri,"workspaceFolders":[{"uri":uri,"name":"Kinematic"}],"capabilities":{"general":{"positionEncodings":["utf-8","utf-16"]},"textDocument":{"completion":{"completionItem":{"snippetSupport":false}},"publishDiagnostics":{"versionSupport":true}}},"initializationOptions":{"Lua":{"runtime":{"version":"Lua 5.4"},"workspace":{"library":[library],"checkThirdParty":false}}}}})).map_err(|e|e.to_string())?;
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
    pub fn sync(&mut self, path: &Path, text: &str, version: u64) {
        if self.versions.get(path) == Some(&version) {
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
        self.sync(path, text, version);
        let id = self.next;
        self.next += 1;
        self.requests.insert(
            id,
            Request {
                utf8_column: position_units(text, line, column, true),
                method: method.into(),
                path: path.to_owned(),
                version,
            },
        );
        let character = position_units(text, line, column, self.utf8);
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"textDocument":{"uri":file_uri(&self.root.root().join(path))},"position":{"line":line,"character":character}}}));
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
                let library = self.root.root().join("kinematic/cache/luals");
                let _=self.send.send(json!({"jsonrpc":"2.0","method":"workspace/didChangeConfiguration","params":{"settings":{"Lua":{"runtime":{"version":"Lua 5.4"},"workspace":{"library":[library],"checkThirdParty":false}}}}}));
                for mut message in self.queued.drain(..) {
                    if self.utf8 {
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
                        Value::Array(value.pointer("/params/items").and_then(Value::as_array).map_or(Vec::new(),|items|items.iter().map(|item| {
                        let settings=json!({"runtime":{"version":"Lua 5.4"},"workspace":{"library":[self.root.root().join("kinematic/cache/luals")],"checkThirdParty":false}});
                        match item["section"].as_str().unwrap_or("") { "Lua"=>settings, "Lua.workspace"=>settings["workspace"].clone(), "Lua.runtime"=>settings["runtime"].clone(), _=>Value::Null }
                    }).collect()))
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
                    let Some(version) = value.pointer("/params/version").and_then(Value::as_u64)
                    else {
                        continue;
                    };
                    if self.versions.get(&path) == Some(&version) {
                        replies.push(Reply {
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
        let mut client = Client::start(
            ProjectContext::new(&root).unwrap(),
            &crate::scripting::definitions(),
        )
        .unwrap();
        let text = "local k = require('kinematic')\nreturn k.scene('Test', function(s)\nlocal ball = s:circle { id = 'ball' }\nball.\nend)\n";
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
        let text = "local k = require('kinematic')\nlocal point = k.vec2(1, 2)\n";
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
        drop(client);
        std::fs::remove_dir_all(root).unwrap();
    }
}
