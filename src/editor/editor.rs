use crate::{
    core::{Project, Scene, types::Vector2},
    editor::{Canvas, Selection, Timeline},
    renderer::{FrameResult, Renderer},
    utilities::FrameTimer,
};

pub(crate) struct Editor {
    project: Project,
    scene: Scene,
    selection: Selection,
    timeline: Timeline,
    preview: Canvas,
    renderer: Renderer,
    pending_export_time: Option<f32>,
    is_exporting: bool,
    accumulator: f32,
    window_timer: FrameTimer,
    canvas_timer: FrameTimer,
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

        let timeline = Timeline::new(scene.build(project.scene.as_mut()), project.fps);

        let preview = Canvas::new(project.resolution, imgui_renderer, skia_context, gl);
        let renderer = Renderer::new(project.resolution);

        Self {
            project,
            scene,
            selection: Selection::default(),
            timeline,
            preview,
            renderer,
            pending_export_time: None,
            is_exporting: false,
            accumulator: 0.0,
            window_timer: FrameTimer::new(),
            canvas_timer: FrameTimer::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        self.window_timer.tick();

        if self.is_exporting {
            if let Some(time) = self.pending_export_time.take() {
                self.timeline.go_to(time);
                self.scene.update(time);
            }

            self.canvas_timer.tick();
            self.accumulator = 0.0;
            return true;
        }

        self.accumulator += self.window_timer.get_delta_time();

        let delta = 1.0 / self.project.fps.max(1) as f32;
        let mut update_canvas = false;

        while self.accumulator >= delta {
            if let Some(time) = self.timeline.update(delta) {
                self.scene.update(time);
            }

            self.canvas_timer.tick();
            self.accumulator -= delta;
            update_canvas = true;
        }

        update_canvas
    }

    pub fn draw(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        update_canvas: bool,
    ) {
        if !update_canvas {
            return;
        }

        let (width, height) = self.preview.get_size();
        let selected = (!self.is_exporting).then(|| self.selection.get()).flatten();

        self.preview.draw(skia_context, gl, window_size, |canvas| {
            canvas.clear(skia_safe::colors::BLACK);

            let save_count = canvas.save();

            canvas.translate((width as f32 * 0.5, height as f32 * 0.5));
            self.scene.draw(canvas);

            if let Some(entity) = selected {
                self.scene.draw_outline(entity, canvas);
            }

            canvas.restore_to_count(save_count);
        });

        if self.is_exporting {
            self.process_export_frame(gl);
        }
    }

    pub fn toggle_export(&mut self, silent: bool) {
        if self.is_exporting {
            self.renderer.cancel();
            self.timeline.pause();
            self.pending_export_time = None;
            self.is_exporting = false;
            self.accumulator = 0.0;
            return;
        }

        let started = self.renderer.start(
            self.project.name,
            self.project.resolution,
            self.project.fps,
            self.timeline.get_duration(),
            silent,
        );
        if !started {
            return;
        }

        self.timeline.pause();
        self.timeline.go_to_start();
        self.scene.update(0.0);
        self.pending_export_time = None;
        self.is_exporting = true;
        self.accumulator = 0.0;
    }

    pub fn is_exporting(&self) -> bool {
        self.is_exporting
    }

    pub fn get_export_progress(&self) -> f32 {
        self.renderer.progress()
    }

    pub fn get_export_message(&self) -> Option<&str> {
        self.renderer.message()
    }

    pub fn shutdown(&mut self, gl: &glow::Context) {
        self.renderer.shutdown(gl);
    }

    pub fn get_project(&mut self) -> &mut Project {
        &mut self.project
    }

    pub fn get_scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn get_selected_entity(&self) -> Option<hecs::Entity> {
        self.selection.get()
    }

    pub fn select_entity(&mut self, entity: hecs::Entity) {
        assert!(
            self.scene.get_world().contains(entity),
            "Selected object must belong to this scene."
        );
        self.selection.select(entity);
    }

    pub fn select_at(&mut self, point: Vector2) {
        match self.scene.pick(point) {
            Some(entity) => self.selection.select(entity),
            None => self.selection.clear(),
        }
    }

    pub fn get_timeline(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn get_preview(&mut self) -> &mut Canvas {
        &mut self.preview
    }

    pub fn get_preview_fps(&self) -> f32 {
        self.canvas_timer.get_fps()
    }

    fn process_export_frame(&mut self, gl: &glow::Context) {
        let result = self.renderer.process_frame(
            gl,
            self.preview.get_framebuffer(),
            self.project.resolution,
        );

        match result {
            Ok(FrameResult::Continue(time)) => self.pending_export_time = Some(time),
            Ok(FrameResult::Finished) => {
                self.pending_export_time = None;
                self.is_exporting = false;
                self.accumulator = 0.0;
            }
            Err(error) => {
                self.renderer.fail(&error);
                self.pending_export_time = None;
                self.is_exporting = false;
                self.accumulator = 0.0;
            }
        }
    }
}
