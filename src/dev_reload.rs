use std::{
    env,
    path::PathBuf,
    process::Command,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub(crate) struct DevReload {
    _watcher: RecommendedWatcher,
    executable: PathBuf,
    restart: Receiver<()>,
}

impl DevReload {
    pub(crate) fn new() -> Option<Self> {
        let project_dir = env::current_dir().ok()?;
        let executable = env::current_exe().ok()?;
        if !project_dir.join("Cargo.toml").is_file() || !project_dir.join("src").is_dir() {
            return None;
        }

        let (changes_tx, changes_rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<Event>| {
                if result.is_ok_and(|event| reloadable(&event)) {
                    let _ = changes_tx.send(());
                }
            },
            notify::Config::default(),
        )
        .ok()?;
        watcher.watch(&project_dir, RecursiveMode::Recursive).ok()?;

        let (restart_tx, restart) = mpsc::channel();
        std::thread::spawn(move || {
            while changes_rx.recv().is_ok() {
                while changes_rx.recv_timeout(Duration::from_millis(150)).is_ok() {}

                println!("Change detected. Rebuilding...");
                let mut command = Command::new("cargo");
                command.arg("build").current_dir(&project_dir);
                if !cfg!(debug_assertions) {
                    command.arg("--release");
                }

                match command.status() {
                    Ok(status) if status.success() => {
                        println!("Build succeeded. Restarting...");
                        if restart_tx.send(()).is_err() {
                            return;
                        }
                    }
                    Ok(_) => println!("Build failed. Waiting for changes..."),
                    Err(error) => eprintln!("Could not run cargo build: {error}"),
                }
            }
        });

        Some(Self {
            _watcher: watcher,
            executable,
            restart,
        })
    }

    pub(crate) fn should_restart(&self) -> bool {
        self.restart.try_recv().is_ok()
    }

    pub(crate) fn restart(&self) -> ! {
        let args = env::args_os().skip(1);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            let error = Command::new(&self.executable).args(args).exec();
            panic!("Could not restart the application: {error}");
        }

        #[cfg(not(unix))]
        {
            Command::new(&self.executable)
                .args(args)
                .spawn()
                .expect("Could not restart the application.");
            std::process::exit(0);
        }
    }
}

fn reloadable(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event.paths.iter().any(|path| {
        !path
            .components()
            .any(|component| component.as_os_str() == "target")
            && (path.file_name().is_some_and(|name| name == "Cargo.toml")
                || path.extension().is_some_and(|extension| extension == "rs"))
    })
}
