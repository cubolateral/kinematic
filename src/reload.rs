use std::{
    env,
    path::PathBuf,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
};

pub(crate) struct Reload {
    building: Arc<AtomicBool>,
    executable: PathBuf,
    request: Sender<()>,
    restart: Receiver<()>,
}

impl Reload {
    pub(crate) fn new() -> Self {
        let project_dir = env::current_dir().expect("Current directory must be available.");
        let executable = env::current_exe().expect("Current executable must be available.");

        let building = Arc::new(AtomicBool::new(false));
        let worker_building = building.clone();
        let (request, requests) = mpsc::channel();
        let (restart_tx, restart) = mpsc::channel();
        std::thread::spawn(move || {
            while requests.recv().is_ok() {
                println!("Rebuilding...");
                let mut command = Command::new("cargo");
                command.arg("build").current_dir(&project_dir);
                if !cfg!(debug_assertions) {
                    command.arg("--release");
                }

                match command.status() {
                    Ok(status) if status.success() => {
                        println!("Build succeeded. Reloading...");
                        if restart_tx.send(()).is_err() {
                            return;
                        }
                    }
                    Ok(_) => {
                        println!("Build failed.");
                        worker_building.store(false, Ordering::Relaxed);
                    }
                    Err(error) => {
                        eprintln!("Could not run cargo build: {error}");
                        worker_building.store(false, Ordering::Relaxed);
                    }
                }
            }
        });

        Self {
            building,
            executable,
            request,
            restart,
        }
    }

    pub(crate) fn request(&self) {
        if !self.building.swap(true, Ordering::Relaxed) && self.request.send(()).is_err() {
            self.building.store(false, Ordering::Relaxed);
        }
    }

    pub(crate) fn is_building(&self) -> bool {
        self.building.load(Ordering::Relaxed)
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
