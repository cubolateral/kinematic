use glow::HasContext;

pub struct App {
    gl: glow::Context,
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

        Self {
            sdl,
            window,
            _gl_context: gl_context,
            gl,
        }
    }

    pub fn run(&mut self) {
        let mut events = self.sdl.event_pump().unwrap();

        'running: loop {
            for event in events.poll_iter() {
                match event {
                    sdl3::event::Event::Quit { .. }
                    | sdl3::event::Event::KeyDown {
                        keycode: Some(sdl3::keyboard::Keycode::Escape),
                        ..
                    } => break 'running,

                    _ => {}
                }
            }

            unsafe {
                self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }

            self.window.gl_swap_window();
        }
    }
}
