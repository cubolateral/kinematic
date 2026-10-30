use super::*;
use crate::core::{components::Name, objects::CircleShape};
use crate::scripting::LuaProject;
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn frame(app: &mut App, editor: &mut Editor) {
    frame_input(app, editor, |_| {});
}

fn frame_input(
    app: &mut App,
    editor: &mut Editor,
    input: impl FnOnce(&mut dear_imgui_rs::Context),
) {
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
    input(&mut app.imgui);

    let ui = app.imgui.frame();

    app.ui.draw(editor, ui);
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
    editor.shutdown(
        &app.gl,
        app.ui.editor_mode(),
        app.ui.is_fullscreen(),
        app.ui.preview_view(),
    );
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
    let _ = std::fs::remove_dir_all(&root);

    std::fs::create_dir_all(&root).unwrap();

    let mut app = App::new();

    app.window.hide();

    let three = app.three_context();

    for name in ["First", "Second"] {
        let mut project = LuaProject::create(root.join(name), name, false).unwrap();

        project.manifest.resolution = (96, 64);
        project.manifest.fps = 12;

        project.save().unwrap();

        let source = "return function(s) local a=s:circle{id='ball',name='Ball',radius=4}; s:world_2d():add(a); s:wait(0.5) end";
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
            4.0
        );

        assert_eq!(
            std::fs::read_to_string(editor.context().root().join(path)).unwrap(),
            source
        );

        editor.documents.as_mut().unwrap().save(path).unwrap();
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

        editor
            .documents
            .as_mut()
            .unwrap()
            .edit(path, "Invalid Lua ???".into());
        editor.documents.as_mut().unwrap().save(path).unwrap();

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
            8.0
        );

        editor.documents.as_mut().unwrap().save_all().unwrap();

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

        let mut settings = editor.project_info().1;

        settings.resolution = (100, 80);

        editor.apply_project_settings(settings);

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
            "return function(s) s:wait(1) end",
        )
        .unwrap();

        let mut manifest = editor.lua_project.as_ref().unwrap().manifest.clone();

        manifest.scenes.insert(0, other.to_owned());

        let docs = editor.documents.as_mut().unwrap();

        docs.open(Path::new("kinematic/project.ron")).unwrap();

        docs.edit(
            Path::new("kinematic/project.ron"),
            ron::to_string(&manifest).unwrap(),
        );
        docs.save(Path::new("kinematic/project.ron")).unwrap();

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

        editor.shutdown(
            &app.gl,
            app.ui.editor_mode(),
            app.ui.is_fullscreen(),
            app.ui.preview_view(),
        );
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

#[test]
#[ignore = "Requires SDL/OpenGL and FFmpeg; exercises UI audio persistence and playback."]
fn ui_audio_clips_save_mix_and_rebuild() {
    let root =
        std::env::temp_dir().join(format!("kinematic-ui-audio-editor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut project = LuaProject::create(&root, "Audio UI", false).unwrap();
    project.manifest.resolution = (96, 64);
    project.manifest.scenes.push("scenes/other.lua".into());
    project.save().unwrap();
    std::fs::write(
        root.join("scenes/other.lua"),
        "return function(s) s:wait(0.1) end",
    )
    .unwrap();
    std::fs::write(
        root.join("scenes/intro.lua"),
        "return function(s) local ball=s:circle{id='ball',name='Ball'}; s:world_2d():add(ball); s:wait(0.05) end",
    )
    .unwrap();
    let source = include_bytes!("../examples/lua/audio/assets/chime.wav");
    std::fs::write(root.join("assets/chime.wav"), source).unwrap();
    std::fs::create_dir_all(root.join("kinematic/cache")).unwrap();
    std::fs::write(
        root.join("kinematic/cache/editor.ron"),
        "(preview_pan: (20.0, -10.0), preview_zoom: 1.5)",
    )
    .unwrap();
    project.manifest.legacy_audio.insert(
        "scenes/other.lua".into(),
        vec![crate::scripting::AudioClip {
            id: 77,
            lane: 0,
            file: "assets/chime.wav".into(),
            at: 0.02,
            offset: 0.0,
            duration: 0.02,
            gain: 1.0,
            pan: 0.0,
        }],
    );
    project.save().unwrap();
    let mut app = App::new();
    app.window.hide();
    app.window.set_size(1280, 1200).unwrap();
    let three = app.three_context();
    let mut editor = Editor::new_lua(
        project,
        &mut app.imgui_renderer,
        &mut app.skia_context,
        &app.gl,
        three,
    )
    .unwrap();
    // Navigation releases camera following from its current transform.
    let canvas_2d = editor.editor_2d_canvas();
    let canvas_3d = editor.editor_3d_canvas();
    let camera_2d = crate::core::components::Camera2D {
        camera_position: crate::core::types::vec2(10.0, 20.0),
        camera_zoom: 1.5,
        camera_rotation: 0.3,
    };
    let camera_3d = crate::core::components::Camera3D {
        camera_position: glam::vec3(3.0, 4.0, 5.0),
        camera_rotation: glam::Quat::from_euler(glam::EulerRot::YXZ, 0.4, -0.2, 0.3),
        camera_mode: crate::core::components::Camera3DMode::Orthogonal,
        camera_fov: 10.0,
        ..Default::default()
    };
    {
        let world = editor.scene(0).world();
        *world
            .get::<&mut crate::core::components::Camera2D>(canvas_2d)
            .unwrap() = camera_2d.clone();
        *world
            .get::<&mut crate::core::components::Camera3D>(canvas_3d)
            .unwrap() = camera_3d.clone();
    }
    editor.toggle_editor_2d_camera_view();
    editor.pan_editor_2d([0.0; 2]);
    editor.zoom_editor_2d_at(0.0, [0.0; 2]);
    assert!(editor.editor_2d_camera_view());
    editor.release_editor_2d_camera(2.0);
    editor.pan_editor_2d([5.0, -3.0]);
    assert!(!editor.editor_2d_camera_view());
    let (pan, zoom, rotation) = editor.editor_2d_view();
    let expected =
        -(glam::Mat2::from_angle(-0.3) * camera_2d.camera_position) * 3.0 + glam::vec2(5.0, -3.0);
    assert!((glam::Vec2::from_array(pan) - expected).length() < 1e-4);
    assert_eq!(zoom, 3.0);
    assert_eq!(rotation, -0.3);
    editor.toggle_editor_3d_camera_view();
    let followed = editor.editor_3d_camera();
    editor.control_editor_3d([0.0; 2], [0.0; 3], 0.1, false, false);
    assert!(editor.editor_3d_camera_view());
    editor.control_editor_3d([2.0, 0.0], [0.0; 3], 0.1, false, false);
    assert!(!editor.editor_3d_camera_view());
    let released = editor.editor_3d_camera();
    assert_eq!(released.camera_position, followed.camera_position);
    assert_eq!(released.camera_mode, followed.camera_mode);
    assert_eq!(released.camera_fov, followed.camera_fov);
    let expected = glam::Quat::from_euler(glam::EulerRot::YXZ, 0.394, -0.2, 0.3);
    assert!(released.camera_rotation.abs_diff_eq(expected, 1e-5));
    {
        let world = editor.scene(0).world();
        *world
            .get::<&mut crate::core::components::Camera2D>(canvas_2d)
            .unwrap() = Default::default();
        *world
            .get::<&mut crate::core::components::Camera3D>(canvas_3d)
            .unwrap() = Default::default();
    }
    editor.reset_editor_2d_view();
    editor.reset_editor_3d_view();
    let migrated = LuaProject::open(&root).unwrap();
    assert!(migrated.manifest.legacy_audio.is_empty());
    assert!((migrated.manifest.audio[0].at - 0.07).abs() < 1e-6);
    editor.commit_audio(Vec::new());
    app.ui.restore_project_view(&editor);
    assert_eq!(app.ui.preview_view(), ([20.0, -10.0], 1.5));
    editor.import_audio("assets/chime.wav".into());
    editor.import_audio("assets/chime.wav".into());
    let mut clips = editor.audio_clips().to_vec();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[1].at, clips[0].duration);
    assert!(
        (editor.timeline_mut().duration() as f64 - 2.0 * clips[0].duration).abs() < 1.0 / 48_000.0
    );
    clips[0].offset = 0.02;
    clips[0].duration = 0.1;
    clips[0].gain = 0.5;
    clips[0].pan = -0.2;
    let id = clips[0].id;
    editor.commit_audio(clips);
    let entity = editor
        .scene(0)
        .world()
        .query::<(hecs::Entity, &Name)>()
        .iter()
        .find(|(_, name)| name.get() == "Ball")
        .unwrap()
        .0;
    editor.select_entity(entity);
    editor.select_audio(id);
    assert_eq!(editor.selected_object(), Some((0, entity)));
    editor.select_entity(entity);
    assert_eq!(editor.selected_audio().unwrap().id, id);
    settle(&mut app, &mut editor);

    fn pointer(
        app: &mut App,
        editor: &mut Editor,
        point: [f32; 2],
        button: dear_imgui_rs::MouseButton,
        down: bool,
    ) {
        frame_input(app, editor, |context| {
            context.io_mut().add_mouse_pos_event(point);
            context.io_mut().add_mouse_button_event(button, down);
        });
    }
    fn drag(app: &mut App, editor: &mut Editor, from: [f32; 2], to: [f32; 2]) {
        let left = dear_imgui_rs::MouseButton::Left;
        pointer(app, editor, from, left, false);
        pointer(app, editor, from, left, false);
        pointer(app, editor, from, left, true);
        let manifest = editor.context().root().join(crate::scripting::MANIFEST);
        let saved = std::fs::read(&manifest).unwrap();
        let time = editor.timeline_mut().time();
        pointer(app, editor, to, left, true);
        assert_eq!(std::fs::read(&manifest).unwrap(), saved);
        assert_eq!(editor.timeline_mut().time(), time);
        pointer(app, editor, to, left, false);
        assert_eq!(std::fs::read(&manifest).unwrap(), saved);
        settle(app, editor);
    }
    // Let the scrollbar layout settle before measuring clip edges.
    frame(&mut app, &mut editor);
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let pixels_per_second = (max[0] - min[0]) / 0.1;
    let y = (min[1] + max[1]) * 0.5;
    let original = editor.audio_clips()[0].clone();
    let body = [(min[0] + max[0]) * 0.5, y];
    drag(&mut app, &mut editor, body, [body[0] + 3.0, y]);
    assert_eq!(editor.audio_clips()[0], original);
    let right = [max[0] - 2.0, y];
    drag(
        &mut app,
        &mut editor,
        right,
        [right[0] - pixels_per_second / 60.0, y],
    );
    assert!((editor.audio_clips()[0].duration - 5.0 / 60.0).abs() < 1e-6);
    let [_, max] = app.ui.selected_audio_rect().unwrap();
    let right = [max[0] - 2.0, y];
    drag(
        &mut app,
        &mut editor,
        right,
        [right[0] + pixels_per_second / 60.0, y],
    );
    assert!((editor.audio_clips()[0].duration - 0.1).abs() < 1e-6);
    let [min, _] = app.ui.selected_audio_rect().unwrap();
    drag(
        &mut app,
        &mut editor,
        [min[0] + 2.0, y],
        [min[0] + 2.0 + pixels_per_second / 60.0, y],
    );
    assert!((editor.audio_clips()[0].offset - (0.02 + 1.0 / 60.0)).abs() < 1e-6);
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, y];
    drag(
        &mut app,
        &mut editor,
        body,
        [body[0] - pixels_per_second / 60.0, y],
    );
    assert!(editor.audio_clips()[0].at < 1e-6);
    assert_eq!(editor.selected_object(), Some((0, entity)));

    let height = kinematic_ui::timeline::metrics::AUDIO_HEIGHT;
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    drag(
        &mut app,
        &mut editor,
        body,
        [body[0] + pixels_per_second * 0.5, body[1]],
    );
    let clips = editor.audio_clips();
    assert!((clips[0].at + clips[0].duration - clips[1].at).abs() < 1e-6);
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    drag(&mut app, &mut editor, body, [body[0], body[1] + height]);
    assert_eq!(editor.audio_clips()[0].lane, 1);
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    drag(
        &mut app,
        &mut editor,
        body,
        [body[0] + pixels_per_second / 60.0, body[1]],
    );
    let clips = editor.audio_clips();
    assert!(clips[0].at + clips[0].duration > clips[1].at);
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    drag(
        &mut app,
        &mut editor,
        body,
        [body[0] - pixels_per_second * 0.5, body[1]],
    );
    assert!(editor.audio_clips()[0].at < 1e-6);

    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let right = dear_imgui_rs::MouseButton::Right;
    pointer(&mut app, &mut editor, body, right, false);
    pointer(&mut app, &mut editor, body, right, true);
    pointer(
        &mut app,
        &mut editor,
        [body[0] + 20.0, body[1]],
        right,
        true,
    );
    assert!(editor.timeline_mut().is_controlling);
    pointer(&mut app, &mut editor, body, right, true);
    pointer(&mut app, &mut editor, body, right, false);
    frame(&mut app, &mut editor);
    let [min, max] = app.ui.selected_audio_rect().unwrap();
    let body = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let right = dear_imgui_rs::MouseButton::Right;
    let time = editor.timeline_mut().time();
    pointer(&mut app, &mut editor, body, right, false);
    pointer(&mut app, &mut editor, body, right, true);
    pointer(&mut app, &mut editor, body, right, false);
    assert_eq!(editor.timeline_mut().time(), time);
    let [min, _] = app.ui.selected_audio_rect().unwrap();
    let header = [
        min[0] + 35.0,
        min[1] - height * editor.audio_clips()[0].lane as f32 - 16.0,
    ];
    let left = dear_imgui_rs::MouseButton::Left;
    pointer(&mut app, &mut editor, header, left, false);
    pointer(&mut app, &mut editor, header, left, false);
    // Dismiss the context menu before clicking the header itself.
    pointer(&mut app, &mut editor, header, left, true);
    pointer(&mut app, &mut editor, header, left, false);
    assert!(app.ui.selected_audio_rect().is_some());
    pointer(&mut app, &mut editor, header, left, true);
    pointer(&mut app, &mut editor, header, left, false);
    assert!(app.ui.selected_audio_rect().is_none());
    pointer(&mut app, &mut editor, header, left, true);
    pointer(&mut app, &mut editor, header, left, false);
    assert!(editor.selected_audio().is_none());
    editor.select_audio(id);
    frame(&mut app, &mut editor);
    assert!(app.ui.selected_audio_rect().is_some());
    let blank = [(min[0] + max[0]) * 0.5, max[1] + 1.0];
    for point in [blank, body] {
        pointer(&mut app, &mut editor, point, left, false);
        editor.timeline_mut().play();
        pointer(&mut app, &mut editor, point, left, true);
        assert!(editor.timeline_mut().is_playing());
        pointer(&mut app, &mut editor, point, left, false);
        assert!(editor.timeline_mut().is_playing());
        editor.timeline_mut().pause();
    }
    pointer(&mut app, &mut editor, blank, right, false);
    editor.timeline_mut().play();
    pointer(&mut app, &mut editor, blank, right, true);
    assert!(!editor.timeline_mut().is_controlling);
    assert!(editor.timeline_mut().is_playing());
    assert!(editor.selected_audio().is_none());
    pointer(&mut app, &mut editor, blank, right, true);
    assert!(editor.timeline_mut().is_playing());
    pointer(
        &mut app,
        &mut editor,
        [blank[0] + 15.0, blank[1]],
        right,
        true,
    );
    assert!(editor.timeline_mut().is_controlling);
    assert!(editor.timeline_mut().is_playing());
    pointer(&mut app, &mut editor, blank, right, false);
    assert!(editor.timeline_mut().is_playing());
    editor.timeline_mut().pause();
    editor.timeline_mut().go_to(time);

    let expected = editor.mix_audio(0, 48_000);
    editor.select_audio(id);
    editor.timeline_mut().go_to(0.05);
    editor.split_audio();
    assert_eq!(editor.audio_clips().len(), 3);
    assert_eq!(
        editor
            .audio_clips()
            .iter()
            .filter(|clip| clip.lane == 1)
            .count(),
        2
    );
    assert_eq!(editor.mix_audio(0, 48_000), expected);
    settle(&mut app, &mut editor);
    assert_eq!(editor.mix_audio(0, 48_000), expected);
    editor
        .documents
        .as_mut()
        .unwrap()
        .request("scenes/intro.lua".into());
    settle(&mut app, &mut editor);
    assert_eq!(editor.mix_audio(0, 48_000), expected);
    let persisted = LuaProject::open(&root).unwrap().manifest.audio;
    let draft = editor.audio_clips().to_vec();
    let mut settings = editor.project_info().1;
    settings.margin = (1, 1);
    editor.apply_project_settings(settings);
    settle(&mut app, &mut editor);
    assert_eq!(LuaProject::open(&root).unwrap().manifest.audio, persisted);
    assert_eq!(editor.audio_clips(), draft);
    assert!(editor.documents.as_ref().unwrap().project_dirty);
    assert!(editor.documents.as_ref().unwrap().dirty());
    assert_ne!(
        LuaProject::open(&root).unwrap().manifest.audio,
        editor.audio_clips()
    );
    frame_input(&mut app, &mut editor, |context| {
        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::ModCtrl, true);
        context.io_mut().add_key_event(dear_imgui_rs::Key::S, true);
    });
    frame_input(&mut app, &mut editor, |context| {
        context.io_mut().add_key_event(dear_imgui_rs::Key::S, false);
        context
            .io_mut()
            .add_key_event(dear_imgui_rs::Key::ModCtrl, false);
    });
    assert!(!editor.documents.as_ref().unwrap().project_dirty);
    let saved = LuaProject::open(&root).unwrap();
    assert_eq!(saved.manifest.audio, editor.audio_clips());
    let absolute_clips = editor.audio_clips().to_vec();
    let global_duration = editor.project_duration();
    assert!((editor.scene_range(0)[1] - 0.05).abs() < 1e-6);
    editor.focus_scene(1, 0.0);
    assert_eq!(editor.audio_clips(), absolute_clips);
    assert_eq!(editor.project_duration(), global_duration);
    assert!((editor.absolute_time() - 0.05).abs() < 1e-6);
    assert_eq!(editor.mix_audio(0, 4800), expected[4800..14400]);
    editor.select_audio(id);
    frame(&mut app, &mut editor);
    assert!(app.ui.selected_audio_rect().is_none());
    editor.select_audio(editor.audio_clips()[2].id);
    frame(&mut app, &mut editor);
    assert!(app.ui.selected_audio_rect().is_some());
    editor.clear_scene_focus();
    frame(&mut app, &mut editor);
    assert_eq!(editor.audio_clips(), absolute_clips);
    editor.select_audio(id);
    editor.delete_audio();
    assert_eq!(editor.audio_clips().len(), 2);
    let previous = editor.audio_clips().to_vec();
    std::fs::write(root.join(crate::scripting::MANIFEST), "External changes.").unwrap();
    assert!(editor.save_project().is_err());
    assert!(editor.documents.as_ref().unwrap().project_dirty);
    assert_eq!(editor.audio_clips(), previous);
    assert_eq!(
        std::fs::read(root.join("assets/chime.wav")).unwrap(),
        source
    );
    assert_eq!(
        std::fs::read_to_string(root.join(crate::scripting::MANIFEST)).unwrap(),
        "External changes."
    );
    editor.shutdown(
        &app.gl,
        app.ui.editor_mode(),
        app.ui.is_fullscreen(),
        app.ui.preview_view(),
    );
    let cache = std::fs::read_to_string(root.join("kinematic/cache/editor.ron")).unwrap();
    assert!(cache.contains("preview_zoom: 1.5"));
    app.ui.restore_project_view(&editor);
    assert_eq!(app.ui.preview_view(), ([20.0, -10.0], 1.5));
    editor.preview_mut().unregister(&mut app.imgui_renderer);
    editor.editor_2d_mut().unregister(&mut app.imgui_renderer);
    editor.editor_3d_mut().unregister(&mut app.imgui_renderer);
    drop(editor);
    app.imgui_renderer.shutdown(&mut app.imgui).unwrap();
    app.imgui_sdl.shutdown(&mut app.imgui).unwrap();
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
