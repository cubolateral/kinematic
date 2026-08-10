pub struct App {
    sdl: sdl3::Sdl,
    canvas: sdl3::render::Canvas<sdl3::video::Window>,
}

impl App {
    pub fn new() -> Self {
        let sdl = sdl3::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video
            .window("Kinematic", 1280, 720)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas();

        Self { sdl, canvas }
    }

    pub fn run(mut self) {
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

            self.canvas.clear();
            self.canvas.present();
        }
    }
}
