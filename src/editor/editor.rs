use crate::{core::Project, editor::Canvas};

pub(crate) struct Editor {
    project: Project,
    preview: Canvas,
}

impl Editor {
    pub fn new(
        project: Project,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
    ) -> Self {
        println!("Project initialized: {}", project.name);

        let preview = Canvas::new(project.resolution, vg, imgui_renderer);

        Self { project, preview }
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn draw(
        &mut self,
        window_size: (u32, u32),
        gl: &glow::Context,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        ui: &mut dear_imgui_rs::Ui,
    ) {
        let (width, height) = self.preview.get_size();

        self.preview.draw(window_size, gl, vg, |vg| {
            vg.clear_rect(0, 0, width, height, femtovg::Color::black());

            let mut path = femtovg::Path::new();
            path.circle(width as f32 / 2.0, height as f32 / 2.0, 64.0);
            vg.fill_path(&path, &femtovg::Paint::color(femtovg::Color::white()));
        });

        ui.window("Preview").build(|| {
            ui.text(format!(
                "[INFO] Project name: {} / Project resolution: {}x{}",
                self.project.name, self.project.resolution.0, self.project.resolution.1,
            ));
            ui.separator();

            let available = ui.content_region_avail();

            let (width, height) = (width as f32, height as f32);
            let aspect = (available[0] / width).min(available[1] / height);
            let (image_width, image_height) = (width * aspect, height * aspect);

            ui.set_cursor_pos_x(ui.cursor_pos_x() + (available[0] - image_width) * 0.5);
            ui.set_cursor_pos_y(ui.cursor_pos_y() + (available[1] - image_height) * 0.5);
            ui.image(
                self.preview.get_imgui_texture_id(),
                [image_width, image_height],
            );
        });
    }
}
