use crate::{
    core::{Project, Scene},
    editor::{Canvas, Timeline},
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
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
    ) -> Self {
        println!("Project initialized: {}", project.name);

        let mut scene = Scene::new();

        let timeline = Timeline::new(scene.build(project.scene.as_mut()));

        let preview = Canvas::new(project.resolution, imgui_renderer, skia_context, gl);

        Self {
            project,
            scene,
            timeline,
            preview,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(time) = self.timeline.update(dt) {
            self.scene.update(time);
        }
    }

    pub fn draw(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
    ) {
        let (width, height) = self.preview.get_size();
        self.preview.draw(skia_context, gl, window_size, |canvas| {
            canvas.clear(skia_safe::colors::BLACK);
            let save_count = canvas.save();
            canvas.translate((width as f32 * 0.5, height as f32 * 0.5));
            self.scene.draw(canvas);
            canvas.restore_to_count(save_count);
        });
    }

    pub fn get_project(&mut self) -> &mut Project {
        &mut self.project
    }

    pub fn get_scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn get_timeline(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn get_preview(&mut self) -> &mut Canvas {
        &mut self.preview
    }
}
