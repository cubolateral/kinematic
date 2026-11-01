// Typed runtime helpers are also used by internal regression tests.
#[allow(dead_code)]
mod core;
mod editor;
mod renderer;
mod scripting;
#[cfg(test)]
mod test_support;
mod ui;
mod utilities;

mod app;
mod audio;
mod storage;
mod window_cache;

use crate::scripting::LuaProject;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");

        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.first().is_some_and(|arg| arg == "--help") {
        println!(
            "Kinematic [PROJECT]\nKinematic --new DIRECTORY NAME [--3d]\nKinematic --check PROJECT\nKinematic --render PROJECT"
        );

        return Ok(());
    }

    if args.first().is_some_and(|arg| arg == "--new") {
        let root = args.get(1).ok_or("Missing project directory.")?;
        let name = args.get(2).ok_or("Missing project name.")?;

        LuaProject::create(root, name, args.iter().any(|a| a == "--3d"))?;
        println!("Created {}.", root);

        return Ok(());
    }

    if args.first().is_some_and(|arg| arg == "--check") {
        let project = LuaProject::open(args.get(1).ok_or("Missing project directory.")?)?;
        let scenes = project.load_scenes(&Default::default())?;

        for scene in scenes {
            scene.update(scene.duration()).map_err(|e| e.to_string())?;
            println!("Validated scene ({} seconds).", scene.duration());
        }

        return Ok(());
    }

    if args.first().is_some_and(|arg| arg == "--render") {
        let root = args.get(1).ok_or("Missing project directory.")?;

        LuaProject::open(root)?;

        return app::app().open_project(root).run_export();
    }

    let mut app = app::app();
    if let Some(root) = args.first() {
        app = app.open_project(root);
    } else if let Some(root) = storage::current_project()
        .unwrap_or_else(|error| {
            eprintln!("Could not restore current project: {error}.");
            None
        })
        .filter(|root| root.is_dir())
    {
        app = app.open_project(root);
    }

    app.run();

    Ok(())
}

#[cfg(test)]
mod tests;
