use glow::HasContext;

use crate::{editor::Editor, ui::Ui};

mod live_redraw;
mod window;

pub struct App {
    ui: Ui,
    imgui_renderer: dear_imgui_glow::GlowRenderer,
    imgui_sdl: dear_imgui_sdl3::Sdl3PlatformBackend,
    imgui: dear_imgui_rs::Context,
    gl: std::rc::Rc<glow::Context>,
    skia_context: skia_safe::gpu::DirectContext,
    _gl_context: sdl3::video::GLContext,
    window: sdl3::video::Window,
    window_chrome: window::Chrome,
    sdl: sdl3::Sdl,
    lua_project: Option<Result<crate::scripting::LuaProject, String>>,
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

        let mut window = video_subsystem
            .window("Kinematic", 1280, 720)
            .position_centered()
            .resizable()
            .opengl()
            .build()
            .unwrap();
        let icon = skia_safe::Image::from_encoded(skia_safe::Data::new_copy(include_bytes!(
            "../assets/brand/logo.png"
        )))
        .expect("Application icon must decode.");
        let info = skia_safe::ImageInfo::new(
            (icon.width(), icon.height()),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Unpremul,
            None,
        );
        let mut pixels = vec![0; info.compute_min_byte_size()];
        assert!(icon.read_pixels(
            &info,
            &mut pixels,
            info.min_row_bytes(),
            (0, 0),
            skia_safe::image::CachingHint::Allow,
        ));
        let surface = sdl3::surface::Surface::from_data(
            &mut pixels,
            icon.width() as u32,
            icon.height() as u32,
            info.min_row_bytes() as u32,
            sdl3::pixels::PixelFormat::RGBA32,
        )
        .expect("Application icon surface must be valid.");
        window.set_icon(surface);

        let window_chrome = window::Chrome::new(&mut window);
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

        match crate::storage::global_path("cache/imgui.ini").and_then(|path| {
            std::fs::create_dir_all(path.parent().unwrap())?;

            Ok(path)
        }) {
            Ok(path) => imgui.set_ini_filename(Some(path)).unwrap(),
            Err(error) => {
                eprintln!("Could not prepare ImGui cache: {error}.");

                imgui.set_ini_filename(None::<std::path::PathBuf>).unwrap();
            }
        }

        let ui = Ui::new(&mut imgui);

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
            window_chrome,
            _gl_context: gl_context,
            gl,
            skia_context,
            lua_project: None,
            export_only: false,
            export_result: None,
        }
    }

    /// Opens a Lua project when the editor starts.
    pub fn open_project(mut self, root: impl Into<std::path::PathBuf>) -> Self {
        self.lua_project = Some(crate::scripting::LuaProject::open(root.into()));

        self
    }

    fn remember_project(&mut self, editor: Option<&Editor>) {
        if !self.export_only {
            if let Some(editor) = editor {
                self.ui.remember_project(editor.context().root());
            }

            if let Err(error) =
                crate::storage::save_current_project(editor.map(|editor| editor.context().root()))
            {
                self.ui.project_error(error);
            }
        }
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

    fn draw_ui(
        &mut self,
        editor: &mut Option<Editor>,
        ui_motion: &mut kinematic_ui::motion::Overlays,
    ) {
        crate::renderer::target::reset_gl(&self.gl, self.window.size());

        unsafe {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.ui.prepare_documents(&self.imgui);
        self.ui.apply_scale(&mut self.imgui);
        self.imgui_sdl.new_frame(&mut self.imgui).unwrap();
        self.ui
            .set_window_controls(self.window_chrome.controls(&self.window));

        let ui = self.imgui.frame();

        if let Some(editor) = editor.as_mut() {
            self.ui.draw(editor, ui);

            if let Some([x, y]) = editor.take_pending_mouse_warp() {
                self.sdl.mouse().warp_mouse_in_window(&self.window, x, y);
            }
        }

        self.ui.draw_projects(editor.as_mut(), ui);

        ui_motion.draw(ui);

        self.window_chrome.update(
            &self.window,
            self.ui.window_drag_area(ui),
            ui.current_font_size() / 16.0,
        );

        let frame = self
            .imgui
            .render(self.imgui_renderer.renderer_consumer().unwrap());

        self.imgui_renderer.render(frame).unwrap();
        self.window.gl_swap_window();
    }

    /// Runs the configured project editor.
    pub fn run(&mut self) {
        let three_context = self.three_context();
        let mut editor = None;

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

        if editor.is_some() {
            self.remember_project(editor.as_ref());
        }

        if let Some(editor) = &editor {
            self.ui.restore_project_view(editor);
        }

        if self.export_only {
            if let Some(editor) = &mut editor {
                editor.toggle_export(true);
            } else {
                self.export_result = Some(Err("Project could not be opened for export.".into()));
            }
        }

        let mut audio = if self.export_only {
            None
        } else {
            match crate::audio::AudioPreview::new(&self.sdl) {
                Ok(audio) => Some(audio),
                Err(error) => {
                    eprintln!("Audio preview unavailable: {error}");

                    None
                }
            }
        };
        let mut events = self.sdl.event_pump().unwrap();
        let mut pending_events = Vec::new();
        let mut ui_motion = kinematic_ui::motion::Overlays::default();

        'running: loop {
            live_redraw::poll(
                self.window.id(),
                || self.draw_ui(&mut editor, &mut ui_motion),
                || pending_events.extend(events.poll_iter()),
            );

            for mut event in pending_events.drain(..) {
                normalize_key_event(&mut event);

                self.imgui_sdl
                    .process_event(&mut self.imgui, &event)
                    .unwrap();

                match event {
                    sdl3::event::Event::Quit { .. } => self.ui.request_exit(),

                    sdl3::event::Event::Window {
                        window_id,
                        win_event: sdl3::event::WindowEvent::CloseRequested,
                        ..
                    } if window_id == self.window.id() => self.ui.request_exit(),

                    _ => {}
                }
            }

            if let Some(preview) = &mut audio {
                if let Err(error) = preview.update(editor.as_ref(), self.ui.audio_volume()) {
                    eprintln!("Audio preview failed: {error}");

                    audio = None;
                }
            }

            if let Some(editor) = &mut editor {
                if let Some(settings) = editor.take_pending_project_settings() {
                    editor.apply_project_settings(settings);
                }

                if let Some(size) = editor.take_pending_editor_2d_size() {
                    editor.resize_editor_2d(
                        size,
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

                if let Some(preview) = &mut audio {
                    if let Err(error) = preview.update(Some(editor), self.ui.audio_volume()) {
                        eprintln!("Audio preview failed: {error}");

                        audio = None;
                    }
                }

                editor.apply_pending_lua(
                    &mut self.imgui_renderer,
                    &mut self.skia_context,
                    &self.gl,
                );

                crate::renderer::target::reset_gl(&self.gl, self.window.size());

                editor.set_viewport_qualities(self.ui.viewport_qualities());

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

            self.draw_ui(&mut editor, &mut ui_motion);

            if let Some(action) = self.ui.take_window_action() {
                if action == kinematic_ui::title_bar::Action::Close {
                    self.ui.request_exit();
                } else if let Err(error) = self.window_chrome.apply(&mut self.window, action) {
                    self.ui.project_error(error);
                }
            }

            if let Some(action) = self.ui.take_project_action() {
                let result = match action {
                    crate::ui::ProjectAction::Exit => break 'running,
                    crate::ui::ProjectAction::Close => Ok(None),
                    crate::ui::ProjectAction::Open(root) => {
                        crate::scripting::LuaProject::open(root).map(Some)
                    }
                    crate::ui::ProjectAction::Create(root, name, settings) => {
                        crate::scripting::LuaProject::create_with_settings(
                            root, &name, false, settings,
                        )
                        .map(Some)
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
                            self.ui.save_cache();

                            previous.shutdown(
                                &self.gl,
                                self.ui.editor_mode(),
                                self.ui.is_fullscreen(),
                                self.ui.preview_view(),
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

                        self.remember_project(editor.as_ref());

                        if let Some(editor) = &editor {
                            self.ui.restore_project_view(editor);
                        }
                    }

                    Err(error) => self.ui.project_error(error),
                }
            }
        }

        if editor.is_none() {
            self.remember_project(None);
        }

        self.ui.reset_project();

        if let Some(mut editor) = editor {
            editor.shutdown(
                &self.gl,
                self.ui.editor_mode(),
                self.ui.is_fullscreen(),
                self.ui.preview_view(),
            );
            editor.preview_mut().unregister(&mut self.imgui_renderer);
            editor.editor_2d_mut().unregister(&mut self.imgui_renderer);
            editor.editor_3d_mut().unregister(&mut self.imgui_renderer);
        }

        crate::scripting::release_unused();

        self.ui.save_cache();
        self.imgui_renderer.shutdown(&mut self.imgui).unwrap();
        self.imgui_sdl.shutdown(&mut self.imgui).unwrap();
    }
}

#[cfg(test)]
#[path = "lua_graphics_tests.rs"]
mod lua_graphics_tests;

fn normalize_key_event(event: &mut sdl3::event::Event) {
    use sdl3::{
        event::Event,
        keyboard::{Keycode, Scancode},
    };

    let (Event::KeyDown {
        keycode: Some(key),
        scancode,
        ..
    }
    | Event::KeyUp {
        keycode: Some(key),
        scancode,
        ..
    }) = event
    else {
        return;
    };

    // ImGui uses US scancodes for punctuation, including the unrecognized ABNT2 slash key.
    let mapped = match key {
        Keycode::Slash => Scancode::Slash,
        Keycode::LeftBracket => Scancode::LeftBracket,
        Keycode::RightBracket => Scancode::RightBracket,
        Keycode::Backslash => Scancode::Backslash,
        Keycode::Minus => Scancode::Minus,
        Keycode::Equals => Scancode::Equals,
        Keycode::Apostrophe => Scancode::Apostrophe,
        Keycode::Grave => Scancode::Grave,
        _ => return,
    };

    *scancode = Some(mapped);
}

#[cfg(test)]
mod keyboard_tests {
    use crate::app::normalize_key_event;
    use sdl3::{
        event::Event,
        keyboard::{Keycode, Mod, Scancode},
    };

    #[test]
    fn punctuation_uses_the_layout_symbol_for_press_and_release() {
        for (keycode, physical, expected) in [
            (Keycode::Slash, Scancode::International1, Scancode::Slash),
            (
                Keycode::LeftBracket,
                Scancode::RightBracket,
                Scancode::LeftBracket,
            ),
            (
                Keycode::RightBracket,
                Scancode::Backslash,
                Scancode::RightBracket,
            ),
            (
                Keycode::Backslash,
                Scancode::NonUsBackslash,
                Scancode::Backslash,
            ),
            (Keycode::A, Scancode::A, Scancode::A),
        ] {
            let mut down = Event::KeyDown {
                timestamp: 0,
                window_id: 1,
                keycode: Some(keycode),
                scancode: Some(physical),
                keymod: Mod::LCTRLMOD,
                repeat: false,
                which: 0,
                raw: 0,
            };
            let mut up = Event::KeyUp {
                timestamp: 0,
                window_id: 1,
                keycode: Some(keycode),
                scancode: Some(physical),
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            };

            normalize_key_event(&mut down);
            normalize_key_event(&mut up);

            assert!(
                matches!(down, Event::KeyDown { scancode: Some(code), keymod: Mod::LCTRLMOD, .. } if code == expected)
            );
            assert!(
                matches!(up, Event::KeyUp { scancode: Some(code), keymod: Mod::NOMOD, .. } if code == expected)
            );
        }
    }
}
