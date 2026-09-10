use crate::{
    core::{Project, Scene, types::Vector2},
    editor::{Canvas, Selection, Timeline},
    renderer::{FrameResult, Renderer},
    utilities::FrameTimer,
};

struct EditorScene {
    scene: Scene,
    start: f32,
    end: f32,
}

pub(crate) struct Editor {
    project: Project,
    scenes: Vec<EditorScene>,
    active_scene: usize,
    selection: Selection,
    timeline: Timeline,
    preview: Canvas,
    canvases: crate::renderer::canvases::Canvases,
    render_error: Option<String>,
    renderer: Renderer,
    pending_export_time: Option<f32>,
    is_exporting: bool,
    accumulator: f32,
    window_timer: FrameTimer,
    canvas_timer: FrameTimer,
    preview_scale: f32,
}

impl Editor {
    pub fn new(
        project: Project,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
        three_context: three_d::Context,
    ) -> Self {
        println!("Project initialized: {}", project.name);
        project.validate();

        let scenes = create_scenes(&project.scenes, project.resolution);
        let duration = scenes.last().map_or(0.0, |scene| scene.end);
        let timeline = Timeline::new(duration, project.fps);

        let preview = Canvas::new(project.resolution, imgui_renderer, skia_context, gl);
        let renderer = Renderer::new(project.resolution);

        let mut editor = Self {
            project,
            scenes,
            active_scene: 0,
            selection: Selection::default(),
            timeline,
            preview,
            canvases: crate::renderer::canvases::Canvases::new(three_context, gl),
            render_error: None,
            renderer,
            pending_export_time: None,
            is_exporting: false,
            accumulator: 0.0,
            window_timer: FrameTimer::new(),
            canvas_timer: FrameTimer::new(),
            preview_scale: 1.0,
        };
        editor.update_active_scene(0.0);
        editor
    }

    pub fn update(&mut self) -> bool {
        self.window_timer.tick();

        if self.is_exporting {
            if let Some(time) = self.pending_export_time.take() {
                self.timeline.go_to(time);
                self.update_active_scene(time);
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
                self.update_active_scene(time);
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
        show_selection: bool,
    ) {
        if !update_canvas && !(show_selection && self.selection.get().is_some()) {
            return;
        }

        let (width, height) = self.preview.get_size();
        let selected = (show_selection && !self.is_exporting)
            .then(|| self.selection.get())
            .flatten();
        let scene = &self.scenes[self.active_scene].scene;

        let result = self
            .canvases
            .render(scene, &mut self.preview.target, skia_context);
        if let Err(error) = result {
            self.render_error = Some(error);
            if self.is_exporting {
                self.renderer.cancel();
                self.is_exporting = false;
                self.pending_export_time = None;
            }
            self.preview.draw(skia_context, gl, window_size, |canvas| {
                canvas.clear(skia_safe::colors::BLACK);
            });
            return;
        }
        self.render_error = None;
        if let Some(entity) = selected {
            let output = scene.get_view();
            let output_size = scene
                .get_world()
                .get::<&crate::core::objects::CanvasSettings>(output.entity)
                .map(|settings| settings.resolution)
                .unwrap_or((width, height));
            let scale = (
                width as f32 / output_size.0.max(1) as f32,
                height as f32 / output_size.1.max(1) as f32,
            );
            self.preview.draw(skia_context, gl, window_size, |canvas| {
                let saved = canvas.save();
                canvas.translate((width as f32 * 0.5, height as f32 * 0.5));
                canvas.scale(scale);
                scene.draw_outline(
                    entity,
                    canvas,
                    2.0 / (self.preview_scale * scale.0).max(0.001),
                );
                canvas.restore_to_count(saved);
            });
        }
        crate::renderer::target::reset_gl(gl, window_size);
        skia_context.reset(None);

        if self.is_exporting {
            self.process_export_frame(gl);
        }
    }

    pub fn set_preview_scale(&mut self, scale: f32) {
        self.preview_scale = scale.max(0.001);
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
        self.update_active_scene(0.0);
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
        self.render_error
            .as_deref()
            .or_else(|| self.renderer.message())
    }

    pub fn get_render_error(&self) -> Option<&str> {
        self.render_error.as_deref()
    }

    pub fn shutdown(&mut self, gl: &glow::Context) {
        self.renderer.shutdown(gl);
    }

    pub fn get_project(&mut self) -> &mut Project {
        &mut self.project
    }

    pub fn get_scene(&mut self) -> &mut Scene {
        &mut self.scenes[self.active_scene].scene
    }

    pub fn get_scene_range(&self) -> [f32; 2] {
        let scene = &self.scenes[self.active_scene];
        [scene.start, scene.end]
    }

    pub(crate) fn get_scenes(
        &self,
    ) -> impl Iterator<
        Item = (
            &'static str,
            [f32; 2],
            &[crate::core::scene_file::ScheduledEvent],
        ),
    > + '_ {
        self.scenes.iter().map(|scene| {
            (
                scene.scene.get_name(),
                [scene.start, scene.end],
                scene.scene.get_events(),
            )
        })
    }

    pub(crate) fn set_event_duration(
        &mut self,
        scene_index: usize,
        event_index: usize,
        duration: f32,
    ) {
        self.scenes[scene_index]
            .scene
            .set_event_duration(event_index, duration);

        let factory = self.project.scenes[scene_index];
        let replacement = factory(self.project.resolution);
        self.scenes[scene_index].scene = replacement;

        let duration = recalculate_scene_ranges(&mut self.scenes);

        self.selection.clear();
        self.timeline.set_duration(duration);
        self.update_active_scene(self.timeline.get_time());
    }

    pub fn get_active_scene_index(&self) -> usize {
        self.active_scene
    }

    pub fn get_selected_entity(&self) -> Option<hecs::Entity> {
        self.selection.get()
    }

    pub fn select_entity(&mut self, entity: hecs::Entity) {
        assert!(
            self.get_scene().get_world().contains(entity),
            "Selected object must belong to this scene."
        );
        self.selection.select(entity);
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select_at(&mut self, point: Vector2) {
        let project_size = self.project.resolution;
        let output = self.get_scene().get_view();
        let source_size = self
            .get_scene()
            .get_world()
            .get::<&crate::core::objects::CanvasSettings>(output.entity)
            .map(|settings| settings.resolution)
            .unwrap_or(project_size);
        let point = point
            * Vector2::new(
                source_size.0 as f32 / project_size.0.max(1) as f32,
                source_size.1 as f32 / project_size.1.max(1) as f32,
            );
        match self.get_scene().pick(point) {
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

    fn update_active_scene(&mut self, time: f32) {
        let active_scene = active_scene_at(&self.scenes, time);

        if self.active_scene != active_scene {
            self.active_scene = active_scene;
            self.selection.clear();
        }

        let scene = &self.scenes[self.active_scene];
        let local_time = (time - scene.start).clamp(0.0, scene.end - scene.start);
        scene.scene.update(local_time);
    }
}

fn active_scene_at(scenes: &[EditorScene], time: f32) -> usize {
    scenes
        .iter()
        .position(|scene| time < scene.end)
        .unwrap_or(scenes.len() - 1)
}

fn create_scenes(
    factories: &[crate::core::SceneFactory],
    resolution: (u32, u32),
) -> Vec<EditorScene> {
    let mut start = 0.0;

    factories
        .iter()
        .map(|create_scene| {
            let scene = create_scene(resolution);
            let end = start + scene.get_duration();
            let editor_scene = EditorScene { scene, start, end };
            start = end;
            editor_scene
        })
        .collect()
}

fn recalculate_scene_ranges(scenes: &mut [EditorScene]) -> f32 {
    let mut start = 0.0;
    for scene in scenes {
        scene.start = start;
        scene.end = start + scene.scene.get_duration();
        start = scene.end;
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::ObjectHandler;

    #[crate::scene]
    fn opening(scene: &mut Scene) {
        scene.wait(2.0);
    }

    #[crate::scene]
    fn ending(scene: &mut Scene) {
        scene.wait(3.0);
    }

    #[test]
    fn scene_factories_create_ordered_project_ranges() {
        let factories: [crate::core::SceneFactory; 2] = [opening, ending];
        let scenes = create_scenes(&factories, (1280, 720));

        assert_eq!(scenes[0].scene.get_name(), "opening");
        assert_eq!([scenes[0].start, scenes[0].end], [0.0, 2.0]);
        assert_eq!(
            scenes[0]
                .scene
                .get_world()
                .get::<&crate::core::objects::CanvasSettings>(
                    scenes[0].scene.get_world_2d().get_id(),
                )
                .unwrap()
                .resolution,
            (1280, 720)
        );
        assert_eq!(scenes[1].scene.get_name(), "ending");
        assert_eq!([scenes[1].start, scenes[1].end], [2.0, 5.0]);
    }

    #[test]
    fn scenes_advance_at_the_end_of_each_range() {
        let scenes = vec![
            EditorScene {
                scene: Scene::new(),
                start: 0.0,
                end: 2.0,
            },
            EditorScene {
                scene: Scene::new(),
                start: 2.0,
                end: 5.0,
            },
        ];

        assert_eq!(active_scene_at(&scenes, 0.0), 0);
        assert_eq!(active_scene_at(&scenes, 1.999), 0);
        assert_eq!(active_scene_at(&scenes, 2.0), 1);
        assert_eq!(active_scene_at(&scenes, 5.0), 1);
    }

    #[test]
    fn recalculating_ranges_uses_rebuilt_scene_durations() {
        let mut scenes = vec![
            EditorScene {
                scene: Scene::new(),
                start: 10.0,
                end: 12.0,
            },
            EditorScene {
                scene: Scene::new(),
                start: 12.0,
                end: 15.0,
            },
        ];
        scenes[0].scene.wait(4.0);
        scenes[1].scene.wait(2.0);

        let duration = recalculate_scene_ranges(&mut scenes);

        assert_eq!([scenes[0].start, scenes[0].end], [0.0, 4.0]);
        assert_eq!([scenes[1].start, scenes[1].end], [4.0, 6.0]);
        assert_eq!(duration, 6.0);
    }
}
