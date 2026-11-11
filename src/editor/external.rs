use crate::core::ProjectContext;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub profile: usize,
    pub executable: String,
    pub project_arguments: String,
    pub file_arguments: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self::profile(0)
    }
}

impl Settings {
    pub(crate) fn profile(profile: usize) -> Self {
        Self {
            profile,
            executable: default_executable(profile),
            project_arguments: r#"["{project}"]"#.into(),
            file_arguments: if profile == 2 {
                r#"["{file}"]"#
            } else {
                r#"["{project}", "--goto", "{file}:{line}:{column}"]"#
            }
            .into(),
        }
    }

    fn command(
        &self,
        root: &ProjectContext,
        file: Option<(&Path, usize, usize)>,
    ) -> Result<Command, String> {
        if self.executable.trim().is_empty() {
            return Err("Choose an external editor executable.".into());
        }
        let executable = Path::new(&self.executable);
        #[cfg(windows)]
        if executable.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("cmd") || extension.eq_ignore_ascii_case("bat")
        }) {
            return Err(
                "Choose the graphical editor executable (.exe), rather than a batch launcher."
                    .into(),
            );
        }
        let executable = if executable.is_relative() && executable.components().count() > 1 {
            root.root().join(executable)
        } else {
            executable.to_owned()
        };
        let arguments: Vec<String> = serde_json::from_str(if file.is_some() {
            &self.file_arguments
        } else {
            &self.project_arguments
        })
        .map_err(|error| format!("Arguments must be a JSON array of strings: {error}."))?;
        let location = file
            .map(|(path, line, column)| {
                let path = if path.is_absolute() {
                    std::fs::canonicalize(path).and_then(|path| {
                        let relative = path.strip_prefix(root.root()).map_err(|_| {
                            std::io::Error::other("Source file is outside the project.")
                        })?;
                        root.resolve(relative)
                    })
                } else {
                    root.resolve(path)
                };
                path.and_then(|path| {
                    if !path.is_file() {
                        return Err(std::io::Error::other(
                            "Source file does not exist or is not a regular file.",
                        ));
                    }
                    std::fs::canonicalize(path)
                })
                .map(|path| (path, line.max(1).to_string(), column.max(1).to_string()))
                .map_err(|error| format!("Could not open source file: {error}."))
            })
            .transpose()?;
        let mut command = Command::new(executable);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // Keep CLI launchers from opening a console window.
            command.creation_flags(0x08000000);
        }
        command
            .current_dir(root.root())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for argument in arguments {
            let mut expanded = OsString::new();
            let mut rest = argument.as_str();
            while let Some(start) = rest.find('{') {
                expanded.push(&rest[..start]);
                let end = rest[start..]
                    .find('}')
                    .map(|end| start + end)
                    .ok_or("Unclosed argument placeholder.")?;
                match &rest[start..=end] {
                    "{project}" => expanded.push(root.root()),
                    "{file}" => expanded.push(
                        &location
                            .as_ref()
                            .ok_or("File placeholder requires a file.")?
                            .0,
                    ),
                    "{line}" => expanded.push(
                        &location
                            .as_ref()
                            .ok_or("Line placeholder requires a file.")?
                            .1,
                    ),
                    "{column}" => expanded.push(
                        &location
                            .as_ref()
                            .ok_or("Column placeholder requires a file.")?
                            .2,
                    ),
                    _ => {
                        return Err(format!(
                            "Unknown argument placeholder: {}.",
                            &rest[start..=end]
                        ));
                    }
                }
                rest = &rest[end + 1..];
            }
            expanded.push(rest);
            command.arg(expanded);
        }
        Ok(command)
    }
}

fn default_executable(profile: usize) -> String {
    #[cfg(windows)]
    if profile != 2 {
        let (folder, executable) = if profile == 1 {
            ("VSCodium", "VSCodium.exe")
        } else {
            ("Microsoft VS Code", "Code.exe")
        };
        for (variable, prefix) in [("LOCALAPPDATA", "Programs"), ("ProgramFiles", "")] {
            if let Some(base) = std::env::var_os(variable) {
                let path = std::path::PathBuf::from(base)
                    .join(prefix)
                    .join(folder)
                    .join(executable);
                if path.is_file() {
                    return path.to_string_lossy().into_owned();
                }
            }
        }
        return executable.into();
    }
    match profile {
        1 => "codium",
        2 => "",
        _ => "code",
    }
    .into()
}

#[derive(Default)]
pub(crate) struct Launcher {
    pending: Vec<mpsc::Receiver<Result<(), String>>>,
}
impl Launcher {
    pub fn open(
        &mut self,
        settings: &Settings,
        root: &ProjectContext,
        file: Option<(&Path, usize, usize)>,
    ) -> Result<(), String> {
        let mut command = settings.command(root, file)?;
        let executable = settings.executable.clone();
        let (send, receive) = mpsc::channel();
        std::thread::Builder::new()
            .name("External editor".into())
            .spawn(move || {
                let result = command
                    .spawn()
                    .and_then(|mut child| child.wait())
                    .map_err(|error| format!("Could not launch {executable}: {error}."))
                    .and_then(|status| {
                        if status.success() {
                            Ok(())
                        } else {
                            Err(format!("External editor exited with {status}."))
                        }
                    });
                let _ = send.send(result);
            })
            .map_err(|error| error.to_string())?;
        self.pending.push(receive);
        Ok(())
    }
    pub fn poll(&mut self) -> Option<Result<(), String>> {
        for index in 0..self.pending.len() {
            match self.pending[index].try_recv() {
                Ok(result) => {
                    self.pending.swap_remove(index);
                    return Some(result);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending.swap_remove(index);
                    return Some(Err("External editor worker stopped.".into()));
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        None
    }
}

pub(crate) fn prepare_luals(
    root: &ProjectContext,
    definitions: &str,
) -> Result<Option<String>, String> {
    use serde_json::json;
    let settings = json!({
        "runtime.version": "Lua 5.4",
        "runtime.plugin": "kinematic/cache/luals/components.lua",
        "workspace.library": ["kinematic/cache/luals"],
        "workspace.checkThirdParty": "Disable",
        "workspace.ignoreDir": ["kinematic", "output", "target", ".git"],
        "diagnostics.libraryFiles": "Disable"
    });
    for (name, contents) in [
        ("kinematic.lua", definitions.to_owned()),
        (
            "components.lua",
            include_str!("../scripting/luals.lua").to_owned(),
        ),
        (
            "luarc.json",
            serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
        ),
    ] {
        let path = root
            .resolve(format!("kinematic/cache/luals/{name}"))
            .map_err(|e| e.to_string())?;
        let previous = std::fs::read(&path).ok();
        if previous.as_deref() != Some(contents.as_bytes()) {
            crate::core::atomic_write(&path, contents.as_bytes(), previous.as_deref())
                .map_err(|e| e.to_string())?;
        }
    }
    let path = root.resolve(".luarc.json").map_err(|e| e.to_string())?;
    let contents = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    if !path.exists()
        && !root.root().join(".luarc.jsonc").exists()
        && !root.root().join(".vscode/settings.json").exists()
    {
        match crate::core::atomic_write(&path, &contents, None) {
            Ok(()) => return Ok(None),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    if std::fs::read(&path).ok().as_deref() == Some(contents.as_slice()) {
        return Ok(None);
    }
    Ok(Some("Warning: Existing LuaLS configuration preserved. Merge kinematic/cache/luals/luarc.json into your LuaLS settings to enable the Kinematic API and custom components.".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{Duration, Instant},
    };

    #[test]
    fn arguments_keep_paths_and_shell_characters_in_separate_arguments() {
        let directory =
            std::env::temp_dir().join(format!("kinematic-external-{} space ç", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let root = ProjectContext::new(&directory).unwrap();
        let name = "scene ' $(echo unsafe) ç.lua";
        fs::write(directory.join(name), "return function() end").unwrap();
        let settings = Settings::default();
        let command = settings
            .command(&root, Some((Path::new(name), 12, 4)))
            .unwrap();
        let arguments: Vec<_> = command.get_args().map(OsString::from).collect();

        assert_eq!(
            arguments,
            vec![
                root.root().as_os_str().to_owned(),
                OsString::from("--goto"),
                OsString::from(format!("{}:12:4", root.root().join(name).display()))
            ]
        );
        assert_eq!(command.get_current_dir(), Some(root.root()));
        assert!(
            settings
                .command(&root, Some((Path::new("../escape.lua"), 1, 1)))
                .is_err()
        );
        assert!(
            settings
                .command(&root, Some((Path::new("missing.lua"), 1, 1)))
                .is_err()
        );
        assert!(
            settings
                .command(&root, Some((&root.root().join(name), 1, 1)))
                .is_ok()
        );
        for arguments in [
            "Invalid.",
            r#"["{unknown}"]"#,
            r#"["{file}"]"#,
            r#"["{project"]"#,
        ] {
            let settings = Settings {
                project_arguments: arguments.into(),
                ..Settings::default()
            };
            assert!(settings.command(&root, None).is_err());
        }
        #[cfg(not(windows))]
        assert_eq!(Settings::profile(1).executable, "codium");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn luals_definitions_refresh_without_overwriting_user_configuration() {
        let directory =
            std::env::temp_dir().join(format!("kinematic-external-luals-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let root = ProjectContext::new(&directory).unwrap();
        assert!(prepare_luals(&root, "First.").unwrap().is_none());
        let path = directory.join(".luarc.json");
        let config: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(config["runtime.version"], "Lua 5.4");
        let plugin =
            fs::read_to_string(directory.join(config["runtime.plugin"].as_str().unwrap())).unwrap();
        assert!(plugin.contains("function OnSetText"));
        fs::write(&path, "User settings, including invalid JSON.").unwrap();
        assert!(prepare_luals(&root, "Second.").unwrap().is_some());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "User settings, including invalid JSON."
        );
        assert_eq!(
            fs::read_to_string(directory.join("kinematic/cache/luals/kinematic.lua")).unwrap(),
            "Second."
        );
        fs::remove_file(&path).unwrap();
        fs::write(directory.join(".luarc.jsonc"), "// Existing settings.\n{}").unwrap();
        assert!(prepare_luals(&root, "Third.").unwrap().is_some());
        assert!(!path.exists());
        fs::remove_file(directory.join(".luarc.jsonc")).unwrap();
        fs::create_dir(directory.join(".vscode")).unwrap();
        fs::write(
            directory.join(".vscode/settings.json"),
            "// Existing settings.\n{}",
        )
        .unwrap();
        assert!(prepare_luals(&root, "Fourth.").unwrap().is_some());
        assert!(!path.exists());
        assert_eq!(
            fs::read_to_string(directory.join(".vscode/settings.json")).unwrap(),
            "// Existing settings.\n{}"
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn launcher_runs_asynchronously_and_reports_spawn_and_exit_failures() {
        use std::os::unix::fs::PermissionsExt;
        let directory =
            std::env::temp_dir().join(format!("kinematic-editor-process-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let root = ProjectContext::new(&directory).unwrap();
        let executable = directory.join("Fake editor");
        fs::write(&executable, "#!/bin/sh\nprintf '%s\\n' \"$@\" > arguments\nwhile [ ! -f release ]; do sleep 0.01; done\nexit 7\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        let mut launcher = Launcher::default();
        let settings = Settings {
            executable: executable.to_str().unwrap().into(),
            ..Settings::default()
        };
        launcher.open(&settings, &root, None).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !directory.join("arguments").exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(launcher.poll().is_none());
        assert_eq!(
            fs::read_to_string(directory.join("arguments")).unwrap(),
            format!("{}\n", root.root().display())
        );
        fs::write(directory.join("release"), "").unwrap();
        let wait = |launcher: &mut Launcher| loop {
            if let Some(result) = launcher.poll() {
                break result;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(wait(&mut launcher).unwrap_err().contains('7'));
        launcher
            .open(
                &Settings {
                    executable: directory.join("Missing").to_str().unwrap().into(),
                    ..Settings::default()
                },
                &root,
                None,
            )
            .unwrap();
        assert!(
            wait(&mut launcher)
                .unwrap_err()
                .contains("Could not launch")
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
