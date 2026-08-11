use glow::HasContext;

use crate::{core::Project, editor::Editor};

pub struct App {
    imgui_renderer: dear_imgui_glow::GlowRenderer,
    imgui_sdl: dear_imgui_sdl3::Sdl3PlatformBackend,
    imgui: dear_imgui_rs::Context,
    vg: femtovg::Canvas<femtovg::renderer::OpenGl>,
    gl: std::rc::Rc<glow::Context>,
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

        // Initialize femtovg.
        let vg = femtovg::Canvas::new(
            unsafe {
                femtovg::renderer::OpenGl::new_from_function(|s| {
                    video_subsystem
                        .gl_get_proc_address(s)
                        .map(|f| f as *const std::ffi::c_void)
                        .unwrap_or(std::ptr::null())
                })
            }
            .unwrap(),
        )
        .unwrap();

        // Initialize imgui.
        let mut imgui = dear_imgui_rs::Context::create();

        let imgui_sdl = dear_imgui_sdl3::Sdl3PlatformBackend::init_platform_for_opengl(
            &mut imgui,
            &window,
            &gl_context,
        )
        .unwrap();

        let imgui_renderer = dear_imgui_glow::GlowRenderer::new(gl, &mut imgui).unwrap();
        let gl = imgui_renderer.gl_context().unwrap().clone();

        Self {
            imgui_renderer,
            imgui_sdl,
            imgui,
            vg,
            sdl,
            window,
            _gl_context: gl_context,
            gl,
        }
    }

    pub fn run(&mut self, project: Project) {
        let mut editor = Editor::new(project, &mut self.vg, &mut self.imgui_renderer);

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

            self.imgui_sdl.new_frame(&mut self.imgui);

            editor.draw(
                self.window.size(),
                &self.gl,
                &mut self.vg,
                self.imgui.frame(),
            );

            self.imgui_renderer.render(self.imgui.render()).unwrap();
            self.window.gl_swap_window();
        }
    }
}
