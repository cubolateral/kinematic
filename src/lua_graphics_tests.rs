use super::*;
use crate::core::{components::Name, objects::CircleShape};
use crate::scripting::LuaProject;
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn frame(app: &mut App, editor: &mut Editor) {
    editor.update();
    editor.apply_pending_lua(&mut app.imgui_renderer, &mut app.skia_context, &app.gl);
    editor.draw(
        &mut app.skia_context,
        &app.gl,
        app.window.size(),
        app.ui.render_mode(),
    );
    app.ui.prepare_documents(&app.imgui, Some(editor));
    app.ui.apply_scale(&mut app.imgui);
    app.imgui_sdl.new_frame(&mut app.imgui).unwrap();
    let ui = app.imgui.frame();
    app.ui.draw(editor, ui, false);
    app.ui.draw_projects(Some(editor), ui);
    let frame = app
        .imgui
        .render(app.imgui_renderer.renderer_consumer().unwrap());
    app.imgui_renderer.render(frame).unwrap();
}

fn settle(app: &mut App, editor: &mut Editor) {
    let timeout = Instant::now() + Duration::from_secs(10);
    loop {
        frame(app, editor);
        let docs = editor.documents.as_ref().unwrap();
        if docs.applied == docs.revision && !docs.replaying {
            break;
        }
        assert!(
            Instant::now() < timeout,
            "{} {:?}",
            docs.status,
            docs.diagnostics
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "Requires SDL/OpenGL; measures simulation startup and editor frames."]
fn lua_simulation_startup_and_frames() {
    let root =
        std::env::temp_dir().join(format!("kinematic-simulation-bench-{}", std::process::id()));
    let mut project = LuaProject::create(&root, "Simulation benchmark", false).unwrap();
    project.manifest.resolution = (640, 480);
    project.manifest.fps = 60;
    project.save().unwrap();
    std::fs::write(
        root.join("scenes/intro.lua"),
        include_str!("../examples/lua/simulation/scenes/life.lua")
            .replace("s:wait(3)", "s:wait(10)"),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("kinematic/cache")).unwrap();
    std::fs::write(
        root.join("kinematic/cache/editor.ron"),
        "(timeline_time: 1.5)",
    )
    .unwrap();
    let started = Instant::now();
    let mut app = App::new();
    app.window.hide();
    let app_ready = started.elapsed();
    let three = app.three_context();
    let mut editor = Editor::new_lua(
        project,
        &mut app.imgui_renderer,
        &mut app.skia_context,
        &app.gl,
        three,
    )
    .unwrap();
    app.ui.restore_project_view(&editor);
    let constructed = started.elapsed();
    settle(&mut app, &mut editor);
    println!(
        "Simulation startup: app={:.3} ms, construction={:.3} ms, preview={:.3} ms.",
        app_ready.as_secs_f64() * 1000.0,
        constructed.as_secs_f64() * 1000.0,
        started.elapsed().as_secs_f64() * 1000.0
    );
    assert_eq!(editor.scene(0).time(), 1.5);
    let mut times = Vec::new();
    let mut updates = Vec::new();
    let mut renders = Vec::new();
    for index in 0..180 {
        let time = (90 + index) as f32 / 60.0;
        editor.timeline_mut().go_to(time);
        std::thread::sleep(Duration::from_millis(17));
        let started = Instant::now();
        frame(&mut app, &mut editor);
        app.skia_context.submit(skia_safe::gpu::SyncCpu::Yes);
        app.window.gl_swap_window();
        times.push(started.elapsed().as_secs_f64() * 1000.0);
        assert!((editor.scene(0).time() - time).abs() < 0.0001);
        updates.push(editor.performance.update_ms);
        renders.push(editor.performance.render_ms);
    }
    times.sort_by(f64::total_cmp);
    updates.sort_by(f32::total_cmp);
    renders.sort_by(f32::total_cmp);
    println!(
        "Simulation editor frames: median={:.3} ms, p95={:.3} ms, max={:.3} ms.",
        times[90], times[171], times[179]
    );
    println!(
        "Simulation stages: update={:.3} ms, render={:.3} ms (medians).",
        updates[90], renders[90]
    );
    assert!(editor.render_error().is_none());
    editor.shutdown(&app.gl, app.ui.editor_mode(), app.ui.is_fullscreen());
    editor.preview_mut().unregister(&mut app.imgui_renderer);
    editor.editor_2d_mut().unregister(&mut app.imgui_renderer);
    editor.editor_3d_mut().unregister(&mut app.imgui_renderer);
    drop(editor);
    app.imgui_renderer.shutdown(&mut app.imgui).unwrap();
    app.imgui_sdl.shutdown(&mut app.imgui).unwrap();
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "Requires SDL/OpenGL and FFmpeg; exercises the actual editor and CTE."]
fn lua_editor_reload_switch_and_frozen_export() {
    let root = std::env::temp_dir().join(format!("kinematic-editor-smoke-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut app = App::new();
    app.window.hide();
    let three = app.three_context();
    for name in ["First", "Second"] {
        let mut project = LuaProject::create(root.join(name), name, false).unwrap();
        project.manifest.resolution = (96, 64);
        project.manifest.fps = 12;
        project.save().unwrap();
        let source = "local k=require('kinematic'); return k.scene('Test', function(s) local a=s:circle{id='ball',name='Ball',radius=4}; s:world_2d():add(a); s:wait(0.5) end)";
        let path = Path::new("scenes/intro.lua");
        std::fs::write(project.context.root().join(path), source).unwrap();
        let mut editor = Editor::new_lua(
            project,
            &mut app.imgui_renderer,
            &mut app.skia_context,
            &app.gl,
            three.clone(),
        )
        .unwrap();
        app.ui.restore_project_view(&editor);
        settle(&mut app, &mut editor);
        let ball = editor
            .scene(0)
            .world()
            .query::<(hecs::Entity, &Name)>()
            .iter()
            .find(|(_, name)| name.get() == "Ball")
            .unwrap()
            .0;
        editor.select_entity(ball);
        editor
            .documents
            .as_mut()
            .unwrap()
            .edit(path, source.replace("radius=4", "radius=8"));
        settle(&mut app, &mut editor);
        assert_eq!(editor.selected_object(), Some((0, ball)));
        assert_eq!(
            editor
                .scene(0)
                .world()
                .get::<&CircleShape>(ball)
                .unwrap()
                .radius,
            8.0
        );
        assert_eq!(
            std::fs::read_to_string(editor.context().root().join(path)).unwrap(),
            source
        );
        editor
            .documents
            .as_mut()
            .unwrap()
            .edit(path, "Invalid Lua ???".into());
        settle(&mut app, &mut editor);
        assert_eq!(
            editor
                .scene(0)
                .world()
                .get::<&CircleShape>(ball)
                .unwrap()
                .radius,
            8.0
        );
        assert!(!editor.documents.as_ref().unwrap().diagnostics.is_empty());
        editor.export_scene(true);
        assert!(editor.is_exporting(), "{:?}", editor.export_message());
        editor
            .documents
            .as_mut()
            .unwrap()
            .edit(path, source.replace("radius=4", "radius=16"));
        let timeout = Instant::now() + Duration::from_secs(15);
        while editor.is_exporting() {
            frame(&mut app, &mut editor);
            assert_eq!(
                editor
                    .scene(0)
                    .world()
                    .get::<&CircleShape>(ball)
                    .unwrap()
                    .radius,
                8.0
            );
            assert!(Instant::now() < timeout, "Export timed out.");
        }
        assert!(editor.export_succeeded(), "{:?}", editor.export_message());
        let movie = editor.context().root().join(format!("output/{name}.mp4"));
        let probe = std::process::Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=nb_frames,width,height",
                "-of",
                "csv=p=0",
            ])
            .arg(&movie)
            .output()
            .unwrap();
        assert!(probe.status.success());
        assert_eq!(String::from_utf8(probe.stdout).unwrap().trim(), "96,64,6");
        settle(&mut app, &mut editor);
        assert_eq!(
            editor
                .scene(0)
                .world()
                .get::<&CircleShape>(ball)
                .unwrap()
                .radius,
            16.0
        );
        editor.documents.as_mut().unwrap().save_all().unwrap();
        settle(&mut app, &mut editor);
        let mut settings = editor.project_info().1;
        settings.resolution = (100, 80);
        editor.apply_project_settings(
            settings,
            &mut app.imgui_renderer,
            &mut app.skia_context,
            &app.gl,
        );
        settle(&mut app, &mut editor);
        assert_eq!(
            LuaProject::open(editor.context().root())
                .unwrap()
                .manifest
                .resolution,
            (100, 80)
        );
        assert_eq!(editor.selected_object(), Some((0, ball)));
        let other = Path::new("scenes/other.lua");
        std::fs::write(
            editor.context().root().join(other),
            "local k=require('kinematic'); return k.scene('Other',function(s) s:wait(1) end)",
        )
        .unwrap();
        let mut manifest = editor.lua_project.as_ref().unwrap().manifest.clone();
        manifest.scenes.insert(0, other.to_owned());
        let docs = editor.documents.as_mut().unwrap();
        docs.open(Path::new("kinematic.ron")).unwrap();
        docs.edit(
            Path::new("kinematic.ron"),
            ron::to_string(&manifest).unwrap(),
        );
        settle(&mut app, &mut editor);
        assert_eq!(editor.selected_object(), Some((1, ball)));
        assert_eq!(
            editor
                .scene(1)
                .world()
                .get::<&CircleShape>(ball)
                .unwrap()
                .radius,
            16.0
        );
        for line in &editor.documents.as_ref().unwrap().console {
            if line.starts_with("Reload:") {
                println!("{name}: {line}");
            }
        }
        editor.shutdown(&app.gl, app.ui.editor_mode(), app.ui.is_fullscreen());
        editor.preview_mut().unregister(&mut app.imgui_renderer);
        editor.editor_2d_mut().unregister(&mut app.imgui_renderer);
        editor.editor_3d_mut().unregister(&mut app.imgui_renderer);
        drop(editor);
        app.ui.reset_project();
    }
    app.imgui_renderer.shutdown(&mut app.imgui).unwrap();
    app.imgui_sdl.shutdown(&mut app.imgui).unwrap();
    drop(three);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
