use glow::HasContext;

use crate::{core::Project, editor::Editor, ui::Ui};

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
}

impl App {
    pub fn new() -> Self {
        let sdl = sdl3::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let gl_attributes = video_subsystem.gl_attr();
        gl_attributes.set_context_version(3, 3);
        gl_attributes.set_context_profile(sdl3::video::GLProfile::Core);

        let window = video_subsystem
            .window("Kinematic", 1280, 720)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        window.gl_make_current(&gl_context).unwrap();

        // Initialize glow.
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                video_subsystem
                    .gl_get_proc_address(s)
                    .map(|f| f as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            })
        };

        // Initialize imgui.
        let mut imgui = dear_imgui_rs::Context::create();

        let ui = Ui::new(&mut imgui);

        let imgui_sdl = dear_imgui_sdl3::Sdl3PlatformBackend::init_platform_for_opengl(
            &mut imgui,
            &window,
            &gl_context,
        )
        .unwrap();

        let imgui_renderer = dear_imgui_glow::GlowRenderer::new(gl, &mut imgui).unwrap();
        let gl = imgui_renderer.gl_context().unwrap().clone();

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
        }
    }

    pub fn run(&mut self, project: Project) {
        let mut editor = Editor::new(
            project,
            &mut self.imgui_renderer,
            &mut self.skia_context,
            &self.gl,
        );

        let mut events = self.sdl.event_pump().unwrap();
        let mut last_frame = std::time::Instant::now();

        'running: loop {
            for event in events.poll_iter() {
                if let Some(raw) = event.to_ll() {
                    self.imgui_sdl.process_event(&mut self.imgui, &raw);
                }

                match event {
                    sdl3::event::Event::Quit { .. }
                    | sdl3::event::Event::KeyDown {
                        keycode: Some(sdl3::keyboard::Keycode::Escape),
                        ..
                    } => break 'running,

                    _ => {}
                }
            }

            // Calculate delta time.
            let now = std::time::Instant::now();
            let dt = now.duration_since(last_frame).as_secs_f32();
            last_frame = now;

            editor.update(dt);

            unsafe {
                self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }

            editor.draw(&mut self.skia_context, &self.gl, self.window.size());

            self.ui.apply_scale(&mut self.imgui);
            self.imgui_sdl.new_frame(&mut self.imgui);

            self.ui.draw(&mut editor, self.imgui.frame());

            self.imgui_renderer.render(self.imgui.render()).unwrap();
            self.window.gl_swap_window();
        }
    }
}
