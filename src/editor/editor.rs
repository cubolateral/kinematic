use crate::{
    core::{Project, Scene},
    editor::Canvas,
};

pub(crate) struct Editor {
    project: Project,
    scene: Scene,
    preview: Canvas,
}

impl Editor {
    pub fn new(
        mut project: Project,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
    ) -> Self {
        println!("Project initialized: {}", project.name);

        let mut scene = Scene::new();
        scene.build(project.scene.as_mut());

        let preview = Canvas::new(project.resolution, vg, imgui_renderer);

        Self {
            project,
            scene,
            preview,
        }
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
        let (width, height) = (width as f32, height as f32);

        self.preview.draw(window_size, gl, vg, |vg| {
            vg.clear_rect(0, 0, width as u32, height as u32, femtovg::Color::black());
            vg.save();
            vg.translate(width * 0.5, height * 0.5);
            self.scene.draw(vg);
            vg.restore();
        });

        ui.window("Preview").build(|| {
            ui.text(format!(
                "[INFO] Project name: {} / Project resolution: {}x{}",
                self.project.name, self.project.resolution.0, self.project.resolution.1,
            ));
            ui.separator();

            let available = ui.content_region_avail();

            let aspect = (available[0] / width).min(available[1] / height);
            let (image_width, image_height) = (width * aspect, height * aspect);

            // Centralize preview image.
            ui.set_cursor_pos_x(ui.cursor_pos_x() + (available[0] - image_width) * 0.5);
            ui.set_cursor_pos_y(ui.cursor_pos_y() + (available[1] - image_height) * 0.5);

            // Draw preview image.
            ui.image_config(
                self.preview.get_imgui_texture_id(),
                [image_width, image_height],
            )
            .uv0([0.0, 1.0])
            .uv1([1.0, 0.0])
            .build();
        });
    }
}
