use glow::HasContext;

use crate::{
    core::{Project, SceneFactory},
    editor::Editor,
    ui::Ui,
    window_cache::WindowCache,
};

pub struct App {
    ui: Ui,
    imgui_renderer: dear_imgui_glow::GlowRenderer,
    imgui_sdl: dear_imgui_sdl3::Sdl3PlatformBackend,
    imgui: dear_imgui_rs::Context,
    gl: std::rc::Rc<glow::Context>,
    skia_context: skia_safe::gpu::DirectContext,
    _gl_context: sdl3::video::GLContext,
    window: sdl3::video::Window,
    sdl: sdl3::Sdl,
    project: Option<Project>,
    lua_project: Option<Result<crate::scripting::LuaProject, String>>,
    window_cache: WindowCache,
    export_only: bool,
    export_result: Option<Result<(), String>>,
}

/// Creates a new application.
pub fn app() -> App {
    App::new()
}

impl App {
    pub fn new() -> Self {
        let sdl = sdl3::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let gl_attributes = video_subsystem.gl_attr();
        gl_attributes.set_context_version(3, 3);
        gl_attributes.set_context_profile(sdl3::video::GLProfile::Core);

        let window_cache = WindowCache::load();
        let mut window_builder =
            video_subsystem.window("Kinematic", window_cache.size.0, window_cache.size.1);
        if let Some((x, y)) = window_cache.position {
            window_builder.position(x, y);
        } else {
            window_builder.position_centered();
        }
        if window_cache.maximized {
            window_builder.maximized();
        }
        let window = window_builder.resizable().opengl().build().unwrap();

        let gl_context = window.gl_create_context().unwrap();
        window.gl_make_current(&gl_context).unwrap();

        // Initialize glow.
        let gl = unsafe {
            glow_imgui::Context::from_loader_function(|s| {
                video_subsystem
                    .gl_get_proc_address(s)
                    .map(|f| f as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            })
        };

        // Initialize imgui.
        let mut imgui = dear_imgui_rs::Context::create();
        imgui.set_ini_filename(None::<std::path::PathBuf>).unwrap();

        let ui = Ui::new(&mut imgui);
        if let Ok(path) = crate::storage::global_path("cache/layout.ini") {
            if let Ok(text) = std::fs::read_to_string(path) {
                imgui.load_ini_settings(&text);
            }
        }

        // The window and current GL context outlive backend teardown.
        let imgui_sdl = unsafe {
            dear_imgui_sdl3::Sdl3PlatformBackend::init_platform_for_opengl(
                &mut imgui,
                &window,
                &gl_context,
            )
        }
        .unwrap();

        let imgui_renderer = dear_imgui_glow::GlowRenderer::new(gl, &mut imgui).unwrap();
        let gl = std::rc::Rc::new(unsafe {
            glow::Context::from_loader_function(|name| {
                video_subsystem
                    .gl_get_proc_address(name)
                    .map(|f| f as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            })
        });

        // Initialize Skia against the current OpenGL context.
        let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }

            video_subsystem
                .gl_get_proc_address(name)
                .map(|f| f as *const std::ffi::c_void)
                .unwrap_or(std::ptr::null())
        })
        .expect("Skia OpenGL interface must be created.");
        let skia_context = skia_safe::gpu::direct_contexts::make_gl(interface, None)
            .expect("Skia OpenGL context must be created.");

        Self {
            ui,
            imgui_renderer,
            imgui_sdl,
            imgui,
            sdl,
            window,
            _gl_context: gl_context,
            gl,
            skia_context,
            project: None,
            lua_project: None,
            window_cache,
            export_only: false,
            export_result: None,
        }
    }

    /// Selects the project name and ordered scene factories to run.
    pub fn project(mut self, name: &'static str, scenes: Vec<SceneFactory>) -> Self {
        self.project = Some(Project::new(name, scenes));
        self
    }

    /// Selects a native project with an explicit asset, settings and output root.
    pub fn project_in(
        mut self,
        context: crate::core::ProjectContext,
        name: &str,
        scenes: Vec<SceneFactory>,
    ) -> std::io::Result<Self> {
        self.project = Some(Project::new_in(context, name, scenes)?);
        Ok(self)
    }

    /// Opens a Lua project when the editor starts.
    pub fn open_project(mut self, root: impl Into<std::path::PathBuf>) -> Self {
        self.lua_project = Some(crate::scripting::LuaProject::open(root.into()));
        self
    }

    /// Opens a Lua project with any native object registrations supplied by the host.
    pub fn lua_project(mut self, project: crate::scripting::LuaProject) -> Self {
        self.lua_project = Some(Ok(project));
        self
    }

    fn save_layout(&mut self) {
        let mut text = String::new();
        self.imgui.save_ini_settings(&mut text);
        if let Ok(path) = crate::storage::global_path("cache/layout.ini") {
            let expected = std::fs::read(&path).ok();
            let _ = crate::core::atomic_write(&path, text.as_bytes(), expected.as_deref());
        }
    }

    fn restore_window(&mut self, context: &crate::core::ProjectContext) {
        if let Err(error) = crate::storage::migrate_globals(context) {
            eprintln!("Could not migrate global settings: {error}.");
        }
        self.ui.reload_appearance();
        if let Ok(path) = crate::storage::global_path("cache/layout.ini") {
            if let Ok(text) = std::fs::read_to_string(path) {
                self.imgui.load_ini_settings(&text);
            }
        }
        let cache = WindowCache::load();
        let _ = self.window.set_size(cache.size.0, cache.size.1);
        if let Some((x, y)) = cache.position {
            self.window.set_position(
                sdl3::video::WindowPos::Positioned(x),
                sdl3::video::WindowPos::Positioned(y),
            );
        }
        if cache.maximized {
            self.window.maximize();
        }
        self.window_cache = cache;
    }

    /// Exports the selected project through the existing application renderer and exits.
    pub fn run_export(&mut self) -> Result<(), String> {
        self.export_only = true;
        self.run();
        self.export_result
            .take()
            .unwrap_or_else(|| Err("Project could not be exported.".into()))
    }

    fn three_context(&self) -> three_d::Context {
        // Load another function table for the same SDL context; no additional GL context is created.
        let video = self.sdl.video().unwrap();
        let three_gl = unsafe {
            glow::Context::from_loader_function(|name| {
                video
                    .gl_get_proc_address(name)
                    .map(|f| f as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            })
        };
        three_d::Context::from_gl_context(std::sync::Arc::new(three_gl))
            .expect("Three-d must initialize against the SDL context.")
    }

    /// Runs the configured project editor.
    pub fn run(&mut self) {
        let three_context = self.three_context();
        let mut editor = self.project.take().map(|project| {
            Editor::new(
                project,
                &mut self.imgui_renderer,
                &mut self.skia_context,
                &self.gl,
                three_context.clone(),
            )
        });
        if let Some(project) = self.lua_project.take() {
            match project.and_then(|project| {
                Editor::new_lua(
                    project,
                    &mut self.imgui_renderer,
                    &mut self.skia_context,
                    &self.gl,
                    three_context.clone(),
                )
            }) {
                Ok(opened) => editor = Some(opened),
                Err(error) => self.ui.project_error(error),
            }
        }

        if let Some(editor) = &editor {
            self.ui.restore_project_view(editor);
            self.restore_window(editor.context());
        }
        if self.export_only {
            if let Some(editor) = &mut editor {
                editor.toggle_export(true);
            } else {
                self.export_result = Some(Err("Project could not be opened for export.".into()));
            }
        }
        let mut events = self.sdl.event_pump().unwrap();
        let reload = crate::reload::Reload::new();
        let mut restart = false;

        'running: loop {
            for event in events.poll_iter() {
                self.imgui_sdl
                    .process_event(&mut self.imgui, &event)
                    .unwrap();

                match event {
                    sdl3::event::Event::Quit { .. } => self.ui.request_exit(),

                    _ => {}
                }
            }

            if let Some(editor) = &mut editor {
                if let Some(settings) = editor.take_pending_project_settings() {
                    editor.apply_project_settings(
                        settings,
                        &mut self.imgui_renderer,
                        &mut self.skia_context,
                        &self.gl,
                    );
                }
                if let Some(size) = editor.take_pending_editor_3d_size() {
                    editor.resize_editor_3d(
                        size,
                        &mut self.imgui_renderer,
                        &mut self.skia_context,
                        &self.gl,
                    );
                }

                editor.update();
                editor.apply_pending_lua(
                    &mut self.imgui_renderer,
                    &mut self.skia_context,
                    &self.gl,
                );

                unsafe {
                    self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
                    self.gl.clear(glow::COLOR_BUFFER_BIT);
                }

                editor.draw(
                    &mut self.skia_context,
                    &self.gl,
                    self.window.size(),
                    self.ui.render_mode(),
                );
            }
            if self.export_only && editor.as_ref().is_none_or(|editor| !editor.is_exporting()) {
                self.export_result = Some(
                    if editor
                        .as_ref()
                        .is_some_and(|editor| editor.export_succeeded())
                    {
                        Ok(())
                    } else {
                        Err(editor
                            .as_ref()
                            .and_then(|editor| editor.export_message())
                            .unwrap_or("Export failed.")
                            .to_owned())
                    },
                );
                break 'running;
            }
            self.ui.prepare_documents(&self.imgui, editor.as_ref());
            self.ui.apply_scale(&mut self.imgui);
            self.imgui_sdl.new_frame(&mut self.imgui).unwrap();

            let ui = self.imgui.frame();
            if let Some(editor) = &mut editor {
                if self.ui.draw(
                    editor,
                    ui,
                    !reload.is_building() && editor.lua_project.is_none(),
                ) {
                    reload.request();
                }
                if let Some([x, y]) = editor.take_pending_mouse_warp() {
                    self.sdl.mouse().warp_mouse_in_window(&self.window, x, y);
                }
            } else {
                ui.dockspace()
                    .build()
                    .expect("Welcome workspace must be valid.");
            }
            self.ui.draw_projects(editor.as_mut(), ui);

            let frame = self
                .imgui
                .render(self.imgui_renderer.renderer_consumer().unwrap());
            self.imgui_renderer.render(frame).unwrap();
            self.window.gl_swap_window();
            self.window_cache.update(&self.window);

            if let Some(action) = self.ui.take_project_action() {
                let result = match action {
                    crate::ui::ProjectAction::Exit => break 'running,
                    crate::ui::ProjectAction::Close => Ok(None),
                    crate::ui::ProjectAction::Open(root) => {
                        crate::scripting::LuaProject::open(root).map(Some)
                    }
                    crate::ui::ProjectAction::Create(root, name, three) => {
                        crate::scripting::LuaProject::create(root, &name, three).map(Some)
                    }
                }
                .and_then(|project| {
                    project
                        .map(|project| {
                            Editor::new_lua(
                                project,
                                &mut self.imgui_renderer,
                                &mut self.skia_context,
                                &self.gl,
                                three_context.clone(),
                            )
                        })
                        .transpose()
                });
                match result {
                    Ok(next) => {
                        if let Some(mut previous) = editor.take() {
                            self.window_cache.save();
                            self.save_layout();
                            previous.shutdown(
                                &self.gl,
                                self.ui.editor_mode(),
                                self.ui.is_fullscreen(),
                            );
                            previous.preview_mut().unregister(&mut self.imgui_renderer);
                            previous
                                .editor_2d_mut()
                                .unregister(&mut self.imgui_renderer);
                            previous
                                .editor_3d_mut()
                                .unregister(&mut self.imgui_renderer);
                        }
                        crate::scripting::release_unused();
                        self.ui.reset_project();
                        editor = next;
                        if let Some(editor) = &editor {
                            self.ui.restore_project_view(editor);
                            self.restore_window(editor.context());
                        }
                    }
                    Err(error) => self.ui.project_error(error),
                }
            }

            if reload.should_restart() {
                restart = true;
                break 'running;
            }
        }

        self.ui.reset_project();
        if let Some(mut editor) = editor {
            editor.shutdown(&self.gl, self.ui.editor_mode(), self.ui.is_fullscreen());
            editor.preview_mut().unregister(&mut self.imgui_renderer);
            editor.editor_2d_mut().unregister(&mut self.imgui_renderer);
            editor.editor_3d_mut().unregister(&mut self.imgui_renderer);
        }
        crate::scripting::release_unused();
        self.window_cache.update(&self.window);
        self.window_cache.save();
        self.save_layout();
        self.imgui_renderer.shutdown(&mut self.imgui).unwrap();
        self.imgui_sdl.shutdown(&mut self.imgui).unwrap();

        if restart {
            reload.restart();
        }
    }
}

#[cfg(test)]
#[path = "lua_graphics_tests.rs"]
mod lua_graphics_tests;
