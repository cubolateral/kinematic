use crate::{
    core::{Project, Scene},
    editor::{Canvas, Timeline},
    ui::Ui,
};

pub(crate) struct Editor {
    project: Project,
    scene: Scene,
    timeline: Timeline,
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

        let timeline = Timeline::new(scene.build(project.scene.as_mut()));

        let preview = Canvas::new(project.resolution, vg, imgui_renderer);

        Self {
            project,
            scene,
            timeline,
            preview,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timeline.update(dt);
    }

    pub fn draw(
        &mut self,
        window_size: (u32, u32),
        gl: &glow::Context,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        ui: &mut dear_imgui_rs::Ui,
    ) {
        self.preview.draw(window_size, gl, vg, |vg| {
            let (width, height) = self.preview.get_size();

            vg.clear_rect(0, 0, width, height, femtovg::Color::black());
            vg.save();
            vg.translate(width as f32 * 0.5, height as f32 * 0.5);
            self.scene.draw(vg);
            vg.restore();
        });

        Ui::draw(self, ui);
    }

    pub fn get_project(&mut self) -> &mut Project {
        &mut self.project
    }

    pub fn get_timeline(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn get_preview(&mut self) -> &mut Canvas {
        &mut self.preview
    }
}
