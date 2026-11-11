use std::path::Path;

pub(crate) fn open_terminal(folder: &Path) -> Result<std::sync::mpsc::Receiver<String>, String> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    use std::process::Command;

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd.exe")
        .args(["/C", "start", "cmd.exe", "/K"])
        .current_dir(folder)
        .spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open")
        .args(["-a", "Terminal"])
        .arg(folder)
        .spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let result = [
        "xdg-terminal-exec",
        "x-terminal-emulator",
        "kitty",
        "alacritty",
        "ghostty",
        "foot",
        "wezterm",
        "kgx",
        "ptyxis",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "xterm",
    ]
    .iter()
    .find_map(|program| {
        let mut command = terminal_command(program, folder);

        command.spawn().ok()
    })
    .ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No supported terminal emulator found in PATH.",
        )
    });
    let child = result.map_err(|error| {
        format!(
            "Could not open terminal: {}.",
            error.to_string().trim_end_matches('.')
        )
    })?;
    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || match child.wait_with_output() {
        Ok(output) if !output.status.success() => {
            let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let _ = sender.send(if details.is_empty() {
                format!("Terminal exited with {}.", output.status)
            } else {
                format!("Could not open terminal: {details}")
            });
        }
        Err(error) => {
            let _ = sender.send(format!("Could not wait for terminal: {error}."));
        }
        _ => {}
    });

    Ok(receiver)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn terminal_command(program: &str, folder: &Path) -> std::process::Command {
    let mut command = std::process::Command::new(program);

    command
        .current_dir(folder)
        .stderr(std::process::Stdio::piped());

    match program {
        "kitty" => {
            command.arg("--directory").arg(folder);
        }
        "alacritty" | "gnome-terminal" | "xfce4-terminal" | "ghostty" => {
            command.arg("--working-directory").arg(folder);
        }
        "konsole" => {
            command.arg("--workdir").arg(folder);
        }
        "wezterm" => {
            command.args(["start", "--cwd"]).arg(folder);
        }
        "foot" | "kgx" | "ptyxis" => {
            command.arg("--working-directory").arg(folder);
        }
        _ => {}
    }

    command
}

#[cfg(test)]
mod tests {
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    use super::*;

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[test]
    fn terminal_launch_preserves_folder_arguments_and_working_directory() {
        let folder =
            std::env::temp_dir().join(format!("kinematic terminal {}", std::process::id()));

        std::fs::create_dir_all(&folder).unwrap();

        let kitty = terminal_command("kitty", &folder);

        assert_eq!(
            kitty.get_args().collect::<Vec<_>>(),
            vec![std::ffi::OsStr::new("--directory"), folder.as_os_str()]
        );

        let output = terminal_command("sh", &folder)
            .args(["-c", "pwd"])
            .output()
            .unwrap();

        assert!(output.status.success());
        assert_eq!(
            Path::new(String::from_utf8(output.stdout).unwrap().trim()),
            folder
        );

        std::fs::remove_dir(folder).unwrap();
    }
}
