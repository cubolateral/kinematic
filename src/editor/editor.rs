use crate::{
    core::{Project, ProjectSettings, Scene, objects::ObjectHandler, types::Vector2},
    editor::{
        Canvas, Selection, Timeline,
        cache::{Camera2DCache, Camera3DCache, EditorCache, EditorMode},
    },
    renderer::{FrameResult, Renderer},
    utilities::FrameTimer,
};

#[derive(Default)]
pub(crate) struct Performance {
    pub update_ms: f32,
    pub render_ms: f32,
    pub avoided: u64,
    pub particles: usize,
}

struct EditorScene {
    scene: Scene,
    start: f32,
    end: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectedCanvas {
    None,
    Two(hecs::Entity),
    Three(hecs::Entity),
}

#[derive(Clone, Copy)]
struct EditorView2D {
    pan: [f32; 2],
    zoom: f32,
    correction: [f32; 2],
    canvas_view: bool,
    axes: [bool; 2],
}

#[derive(Clone, Copy)]
struct EditorView3D {
    position: glam::Vec3,
    yaw: f32,
    pitch: f32,
    canvas_view: bool,
    axes: [bool; 3],
}

impl Default for EditorView3D {
    fn default() -> Self {
        Self {
            position: glam::vec3(4.0, 3.0, 6.0),
            yaw: 0.588,
            pitch: -0.395,
            canvas_view: false,
            axes: [true; 3],
        }
    }
}

impl Default for EditorView2D {
    fn default() -> Self {
        Self {
            pan: [0.0; 2],
            zoom: 1.0,
            correction: [1.0; 2],
            canvas_view: false,
            axes: [true; 2],
        }
    }
}

pub(crate) struct Editor {
    project: Project,
    scenes: Vec<EditorScene>,
    active_scene: usize,
    selection: Selection,
    timeline: Timeline,
    preview: Canvas,
    editor_2d: Canvas,
    editor_view_2d: EditorView2D,
    editor_rendered: Option<((u64, u64), hecs::Entity, [u32; 6])>,
    editor_3d: Canvas,
    editor_view_3d: EditorView3D,
    editor_3d_rendered: Option<((u64, u64), hecs::Entity, Option<hecs::Entity>, [u32; 11])>,
    pending_editor_3d_size: Option<(u32, u32)>,
    pending_mouse_warp: Option<[f32; 2]>,
    suppress_editor_mouse_delta: bool,
    canvases: crate::renderer::canvases::Canvases,
    render_error: Option<String>,
    renderer: Renderer,
    pending_export_time: Option<f32>,
    pending_screenshot: bool,
    is_exporting: bool,
    accumulator: f32,
    window_timer: FrameTimer,
    canvas_timer: FrameTimer,
    rendered: Option<(u64, u64)>,
    evaluated: Option<(u64, f32)>,
    pub(crate) performance: Performance,
    pending_project_settings: Option<ProjectSettings>,
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

        let scenes = create_scenes(
            &project.scenes,
            project.settings.resolution,
            project.settings.fps,
        );
        let duration = scenes.last().map_or(0.0, |scene| scene.end);
        let cache = EditorCache::load();
        let mut timeline = Timeline::new(duration, project.settings.fps);
        let timeline_time = cache.timeline_time(duration);
        timeline.go_to(timeline_time);

        let preview = Canvas::new(
            project.settings.resolution,
            imgui_renderer,
            skia_context,
            gl,
        );
        let editor_2d = Canvas::new(
            project.settings.resolution,
            imgui_renderer,
            skia_context,
            gl,
        );
        let editor_3d = Canvas::new_3d(
            project.settings.resolution,
            imgui_renderer,
            skia_context,
            gl,
        );
        let renderer = Renderer::new(project.settings.resolution);

        let mut editor = Self {
            project,
            scenes,
            active_scene: 0,
            selection: Selection::default(),
            timeline,
            preview,
            editor_2d,
            editor_view_2d: EditorView2D {
                pan: cache.camera_2d.pan,
                zoom: cache.camera_2d.zoom,
                canvas_view: cache.camera_2d.camera_view,
                axes: cache.camera_2d.axes,
                ..EditorView2D::default()
            },
            editor_rendered: None,
            editor_3d,
            editor_view_3d: EditorView3D {
                position: glam::Vec3::from_array(cache.camera_3d.position),
                yaw: cache.camera_3d.yaw,
                pitch: cache.camera_3d.pitch,
                canvas_view: cache.camera_3d.camera_view,
                axes: cache.camera_3d.axes,
                ..EditorView3D::default()
            },
            editor_3d_rendered: None,
            pending_editor_3d_size: None,
            pending_mouse_warp: None,
            suppress_editor_mouse_delta: false,
            canvases: crate::renderer::canvases::Canvases::new(three_context, gl),
            render_error: None,
            renderer,
            pending_export_time: None,
            pending_screenshot: false,
            is_exporting: false,
            accumulator: 0.0,
            window_timer: FrameTimer::new(),
            canvas_timer: FrameTimer::new(),
            rendered: None,
            evaluated: None,
            performance: Performance::default(),
            pending_project_settings: None,
        };
        editor.update_active_scene(timeline_time);
        editor
    }

    pub fn update(&mut self) {
        self.window_timer.tick();

        if self.pending_screenshot {
            self.canvas_timer.tick();
            self.accumulator = 0.0;
            return;
        }

        if self.is_exporting {
            if let Some(time) = self.pending_export_time.take() {
                self.timeline.go_to(time);
                self.update_active_scene(time);
            }

            self.canvas_timer.tick();
            self.accumulator = 0.0;
            return;
        }

        self.accumulator += self.window_timer.delta_time();

        let delta = 1.0 / self.project.settings.fps.max(1) as f32;

        while self.accumulator >= delta {
            if let Some(time) = self.timeline.update(delta) {
                self.update_active_scene(time);
            }

            self.canvas_timer.tick();
            self.accumulator -= delta;
        }
    }

    pub fn draw(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        mode: EditorMode,
    ) {
        let scene = &self.scenes[self.active_scene].scene;
        let key = scene.render_key();
        if self.rendered == Some(key) {
            self.performance.avoided += 1;
            if self.is_exporting {
                self.process_export_frame(gl);
            }
        } else {
            let started = std::time::Instant::now();
            crate::core::objects::particle::DRAW_COUNT.set(0);
            let result = self
                .canvases
                .render(scene, &mut self.preview.target, skia_context);
            if let Err(error) = result {
                self.rendered = None;
                self.render_error = Some(error);
                self.pending_screenshot = false;
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
            self.performance.particles = crate::core::objects::particle::DRAW_COUNT.get();
            self.rendered = Some(key);
            self.performance.render_ms = started.elapsed().as_secs_f32() * 1000.0;
            crate::renderer::target::reset_gl(gl, window_size);
            skia_context.reset(None);

            if self.is_exporting {
                self.process_export_frame(gl);
            }
        }

        if self.pending_screenshot {
            self.renderer.screenshot(
                gl,
                self.preview.framebuffer(),
                self.project.name,
                self.project.settings.resolution,
            );
            self.pending_screenshot = false;
        }

        if !self.is_exporting {
            match mode {
                EditorMode::Preview => {}
                EditorMode::Two => self.draw_editor_2d(skia_context, gl, window_size, key),
                EditorMode::Three => self.draw_editor_3d(skia_context, gl, window_size, key),
            }
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
            self.project.settings.resolution,
            self.project.settings.fps,
            self.timeline.duration(),
            silent,
        );
        if !started {
            return;
        }

        self.timeline.pause();
        self.timeline.go_to_start();
        self.evaluated = None;
        self.update_active_scene(0.0);
        self.pending_export_time = None;
        self.pending_project_settings = None;
        self.is_exporting = true;
        self.accumulator = 0.0;
    }

    pub fn is_exporting(&self) -> bool {
        self.is_exporting
    }

    pub fn request_screenshot(&mut self) {
        if !self.is_exporting {
            self.pending_screenshot = true;
        }
    }

    pub fn export_progress(&self) -> f32 {
        self.renderer.progress()
    }

    pub fn export_message(&self) -> Option<&str> {
        self.render_error
            .as_deref()
            .or_else(|| self.renderer.message())
    }

    pub fn render_error(&self) -> Option<&str> {
        self.render_error.as_deref()
    }

    pub fn shutdown(&mut self, gl: &glow::Context, mode: EditorMode, fullscreen: bool) {
        EditorCache {
            camera_2d: Camera2DCache {
                pan: self.editor_view_2d.pan,
                zoom: self.editor_view_2d.zoom,
                camera_view: self.editor_view_2d.canvas_view,
                axes: self.editor_view_2d.axes,
            },
            camera_3d: Camera3DCache {
                position: self.editor_view_3d.position.to_array(),
                yaw: self.editor_view_3d.yaw,
                pitch: self.editor_view_3d.pitch,
                camera_view: self.editor_view_3d.canvas_view,
                axes: self.editor_view_3d.axes,
            },
            timeline_time: self.timeline.time(),
            mode,
            fullscreen,
        }
        .save();
        self.renderer.shutdown(gl);
    }

    pub(crate) fn project_info(&self) -> (&'static str, ProjectSettings) {
        (self.project.name, self.project.settings)
    }

    pub(crate) fn request_project_settings(&mut self, settings: ProjectSettings) {
        if self.is_exporting {
            return;
        }
        settings.save();
        self.pending_project_settings = Some(settings);
    }

    pub(crate) fn take_pending_project_settings(&mut self) -> Option<ProjectSettings> {
        self.pending_project_settings.take()
    }

    pub(crate) fn apply_project_settings(
        &mut self,
        settings: ProjectSettings,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) {
        if self.is_exporting || settings == self.project.settings {
            return;
        }

        self.renderer.shutdown(gl);
        let preview = Canvas::new(settings.resolution, imgui_renderer, skia_context, gl);
        let editor_2d = Canvas::new(settings.resolution, imgui_renderer, skia_context, gl);
        let editor_3d = Canvas::new_3d(settings.resolution, imgui_renderer, skia_context, gl);
        imgui_renderer
            .texture_map_mut()
            .remove(self.preview.imgui_texture_id());
        imgui_renderer
            .texture_map_mut()
            .remove(self.editor_2d.imgui_texture_id());
        imgui_renderer
            .texture_map_mut()
            .remove(self.editor_3d.imgui_texture_id());
        self.preview = preview;
        self.editor_2d = editor_2d;
        self.editor_view_2d = EditorView2D::default();
        self.editor_rendered = None;
        self.editor_3d = editor_3d;
        self.editor_view_3d = EditorView3D::default();
        self.editor_3d_rendered = None;
        self.pending_editor_3d_size = None;
        self.renderer = Renderer::new(settings.resolution);
        self.scenes = create_scenes(&self.project.scenes, settings.resolution, settings.fps);
        let duration = self.scenes.last().map_or(0.0, |scene| scene.end);
        self.timeline = Timeline::new(duration, settings.fps);
        self.project.settings = settings;
        self.active_scene = 0;
        self.selection.clear();
        self.render_error = None;
        self.pending_export_time = None;
        self.pending_screenshot = false;
        self.is_exporting = false;
        self.accumulator = 0.0;
        self.update_active_scene(0.0);
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scenes[self.active_scene].scene
    }

    pub fn scene_range(&self) -> [f32; 2] {
        let scene = &self.scenes[self.active_scene];
        [scene.start, scene.end]
    }

    pub(crate) fn scenes(
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
                scene.scene.name(),
                [scene.start, scene.end],
                scene.scene.events(),
            )
        })
    }

    pub(crate) fn set_event_duration(
        &mut self,
        scene_index: usize,
        event_index: usize,
        duration: f32,
    ) {
        if self.is_exporting {
            return;
        }

        self.scenes[scene_index]
            .scene
            .set_event_duration(event_index, duration);

        let factory = self.project.scenes[scene_index];
        let mut replacement = factory(self.project.settings.resolution);
        replacement.set_fps(self.project.settings.fps);
        self.scenes[scene_index].scene = replacement;

        let duration = recalculate_scene_ranges(&mut self.scenes);

        self.selection.clear();
        self.timeline.set_duration(duration);
        self.update_active_scene(self.timeline.time());
    }

    pub fn active_scene_index(&self) -> usize {
        self.active_scene
    }

    pub fn selected_entity(&self) -> Option<hecs::Entity> {
        self.selection
            .get()
            .filter(|(scene, _)| *scene == self.active_scene)
            .map(|(_, entity)| entity)
    }

    pub(crate) fn selected_object(&self) -> Option<(usize, hecs::Entity)> {
        self.selection.get()
    }

    pub(crate) fn scene_at_mut(&mut self, index: usize) -> &mut Scene {
        &mut self.scenes[index].scene
    }

    pub fn select_entity(&mut self, entity: hecs::Entity) {
        assert!(
            self.scene_mut().world().contains(entity),
            "Selected object must belong to this scene."
        );
        self.selection.select(self.active_scene, entity);
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub(crate) fn selected_canvas(&self) -> SelectedCanvas {
        let Some(entity) = self.selected_entity() else {
            return SelectedCanvas::None;
        };
        match self.scenes[self.active_scene].scene.nearest_canvas(entity) {
            Some((entity, crate::core::objects::CanvasDimension::Two)) => {
                SelectedCanvas::Two(entity)
            }
            Some((entity, crate::core::objects::CanvasDimension::Three)) => {
                SelectedCanvas::Three(entity)
            }
            None => SelectedCanvas::None,
        }
    }

    pub(crate) fn editor_2d_canvas(&self) -> hecs::Entity {
        match self.selected_canvas() {
            SelectedCanvas::Two(entity) => entity,
            SelectedCanvas::None | SelectedCanvas::Three(_) => {
                self.scenes[self.active_scene].scene.world_2d().entity()
            }
        }
    }

    pub(crate) fn editor_3d_canvas(&self) -> hecs::Entity {
        match self.selected_canvas() {
            SelectedCanvas::Three(entity) => entity,
            SelectedCanvas::None | SelectedCanvas::Two(_) => {
                self.scenes[self.active_scene].scene.world_3d().entity()
            }
        }
    }

    pub(crate) fn reset_editor_3d_view(&mut self) {
        self.editor_view_3d = EditorView3D::default();
        self.editor_3d_rendered = None;
    }

    pub(crate) fn reset_editor_3d_camera_transform(&mut self) {
        let default = EditorView3D::default();
        self.editor_view_3d.position = default.position;
        self.editor_view_3d.yaw = default.yaw;
        self.editor_view_3d.pitch = default.pitch;
        self.editor_3d_rendered = None;
    }

    pub(crate) fn toggle_editor_3d_camera_view(&mut self) {
        self.editor_view_3d.canvas_view = !self.editor_view_3d.canvas_view;
        self.editor_3d_rendered = None;
    }

    pub(crate) fn editor_3d_camera_view(&self) -> bool {
        self.editor_view_3d.canvas_view
    }

    pub(crate) fn toggle_editor_3d_axis(&mut self, axis: usize) {
        self.editor_view_3d.axes[axis] = !self.editor_view_3d.axes[axis];
        self.editor_3d_rendered = None;
    }

    pub(crate) fn editor_3d_axes(&self) -> [bool; 3] {
        self.editor_view_3d.axes
    }

    pub(crate) fn control_editor_3d(
        &mut self,
        look: [f32; 2],
        movement: [f32; 3],
        delta: f32,
        fast: bool,
    ) {
        if self.editor_view_3d.canvas_view {
            return;
        }
        let view = &mut self.editor_view_3d;
        view.yaw -= look[0] * 0.003;
        view.pitch = (view.pitch - look[1] * 0.003).clamp(-1.55, 1.55);
        let rotation =
            glam::Quat::from_rotation_y(view.yaw) * glam::Quat::from_rotation_x(view.pitch);
        let speed = if fast { 12.0 } else { 3.0 } * delta.min(0.1);
        view.position += rotation
            * glam::vec3(movement[0], movement[1], -movement[2]).normalize_or_zero()
            * speed;
        self.editor_3d_rendered = None;
    }

    pub(crate) fn editor_3d_camera(&self) -> crate::core::components::Camera3D {
        let canvas = self.editor_3d_canvas();
        if self.editor_view_3d.canvas_view {
            let mut camera = self.scenes[self.active_scene]
                .scene
                .world()
                .get::<&crate::core::components::Camera3D>(canvas)
                .map(|camera| (*camera).clone())
                .unwrap_or_default();
            let (_, canvas_size) = self.editor_3d_canvas_camera();
            let editor_size = self.editor_3d.size();
            let canvas_aspect = canvas_size.0.max(1) as f32 / canvas_size.1.max(1) as f32;
            let editor_aspect = editor_size.0.max(1) as f32 / editor_size.1.max(1) as f32;
            if editor_aspect < canvas_aspect {
                camera.camera_fov =
                    ((camera.camera_fov * 0.5).tan() * canvas_aspect / editor_aspect).atan() * 2.0;
            }
            return camera;
        }
        crate::core::components::Camera3D {
            camera_position: self.editor_view_3d.position,
            camera_rotation: glam::Quat::from_rotation_y(self.editor_view_3d.yaw)
                * glam::Quat::from_rotation_x(self.editor_view_3d.pitch),
            ..Default::default()
        }
    }

    pub(crate) fn editor_3d_canvas_camera(
        &self,
    ) -> (crate::core::components::Camera3D, (u32, u32)) {
        let canvas = self.editor_3d_canvas();
        let world = self.scenes[self.active_scene].scene.world();
        let camera = world
            .get::<&crate::core::components::Camera3D>(canvas)
            .map(|camera| (*camera).clone())
            .unwrap_or_default();
        let size = world
            .get::<&crate::core::objects::CanvasSettings>(canvas)
            .map(|settings| settings.resolution())
            .unwrap_or((1, 1));
        (camera, size)
    }

    pub(crate) fn select_at_editor_3d(&mut self, point: Vector2) {
        let canvas = self.editor_3d_canvas();
        let camera = self.editor_3d_camera();
        let size = self.editor_3d.size();
        match self.scenes[self.active_scene]
            .scene
            .pick_editor_3d(canvas, &camera, size, point)
        {
            Some(entity) => self.selection.select(self.active_scene, entity),
            None => self.clear_selection(),
        }
    }

    pub(crate) fn reset_editor_2d_view(&mut self) {
        self.editor_view_2d.pan = [0.0; 2];
        self.editor_view_2d.zoom = 1.0;
        self.editor_view_2d.canvas_view = false;
        self.editor_rendered = None;
    }

    pub(crate) fn reset_editor_2d_camera_transform(&mut self) {
        self.editor_view_2d.pan = [0.0; 2];
        self.editor_view_2d.zoom = 1.0;
        self.editor_rendered = None;
    }

    pub(crate) fn toggle_editor_2d_camera_view(&mut self) {
        self.editor_view_2d.canvas_view = !self.editor_view_2d.canvas_view;
        self.editor_rendered = None;
    }

    pub(crate) fn editor_2d_camera_view(&self) -> bool {
        self.editor_view_2d.canvas_view
    }

    pub(crate) fn toggle_editor_2d_axis(&mut self, axis: usize) {
        self.editor_view_2d.axes[axis] = !self.editor_view_2d.axes[axis];
    }

    pub(crate) fn editor_2d_axes(&self) -> [bool; 2] {
        self.editor_view_2d.axes
    }

    pub(crate) fn editor_2d_canvas_size(&self) -> (u32, u32) {
        let canvas = self.editor_2d_canvas();
        self.scenes[self.active_scene]
            .scene
            .world()
            .get::<&crate::core::objects::CanvasSettings>(canvas)
            .map(|settings| settings.resolution())
            .unwrap_or((1, 1))
    }

    pub(crate) fn set_editor_2d_viewport(&mut self, display_size: [f32; 2]) {
        let target_size = self.editor_2d.size();
        let content_size = if self.editor_view_2d.canvas_view {
            self.editor_2d_canvas_size()
        } else {
            target_size
        };
        let display_x = display_size[0].max(1.0) / target_size.0.max(1) as f32;
        let display_y = display_size[1].max(1.0) / target_size.1.max(1) as f32;
        let content_scale = (display_size[0].max(1.0) / content_size.0.max(1) as f32)
            .min(display_size[1].max(1.0) / content_size.1.max(1) as f32);
        let correction = [content_scale / display_x, content_scale / display_y];
        if self.editor_view_2d.correction != correction {
            self.editor_view_2d.correction = correction;
            self.editor_rendered = None;
        }
    }

    pub(crate) fn pan_editor_2d(&mut self, delta: [f32; 2]) {
        if self.editor_view_2d.canvas_view {
            return;
        }
        self.editor_view_2d.pan[0] += delta[0];
        self.editor_view_2d.pan[1] += delta[1];
        self.editor_rendered = None;
    }

    pub(crate) fn zoom_editor_2d_at(&mut self, wheel: f32, anchor: [f32; 2]) {
        if wheel == 0.0 || self.editor_view_2d.canvas_view {
            return;
        }
        let old_zoom = self.editor_view_2d.zoom;
        let zoom = (old_zoom * (wheel * 0.15).exp()).clamp(0.01, 100.0);
        let ratio = zoom / old_zoom;
        for axis in 0..2 {
            self.editor_view_2d.pan[axis] =
                anchor[axis] + (self.editor_view_2d.pan[axis] - anchor[axis]) * ratio;
        }
        self.editor_view_2d.zoom = zoom;
        self.editor_rendered = None;
    }

    pub(crate) fn editor_2d_view(&self) -> ([f32; 2], f32) {
        (self.editor_view_2d.pan, self.editor_view_2d.zoom)
    }

    pub(crate) fn select_at_editor_2d(&mut self, point: Vector2) {
        let canvas = self.editor_2d_canvas();
        match self.scenes[self.active_scene].scene.pick_editor_2d(
            canvas,
            point,
            self.editor_view_2d.canvas_view,
        ) {
            Some(entity) => self.selection.select(self.active_scene, entity),
            None => self.clear_selection(),
        }
    }

    pub(crate) fn editor_2d_camera_outline(&self) -> Option<[skia_safe::Point; 4]> {
        let canvas = self.editor_2d_canvas();
        self.scenes[self.active_scene]
            .scene
            .editor_2d_camera_outline(canvas, self.editor_view_2d.canvas_view)
    }

    pub(crate) fn editor_2d_project_outline(&self) -> [skia_safe::Point; 4] {
        let (width, height) = self.editor_2d_canvas_size();
        let half_width = width as f32 * 0.5;
        let half_height = height as f32 * 0.5;
        [
            skia_safe::Point::new(-half_width, -half_height),
            skia_safe::Point::new(half_width, -half_height),
            skia_safe::Point::new(half_width, half_height),
            skia_safe::Point::new(-half_width, half_height),
        ]
    }

    pub(crate) fn editor_2d_selection_outline(&self) -> Option<[skia_safe::Point; 4]> {
        let entity = self.selected_entity()?;
        let SelectedCanvas::Two(canvas) = self.selected_canvas() else {
            return None;
        };
        self.scenes[self.active_scene]
            .scene
            .editor_2d_selection_outline(canvas, entity, self.editor_view_2d.canvas_view)
    }

    pub(crate) fn editor_2d_background(&self) -> [f32; 4] {
        use crate::core::types::Color;

        let canvas = self.editor_2d_canvas();
        self.scenes[self.active_scene]
            .scene
            .world()
            .get::<&crate::core::objects::CanvasSettings>(canvas)
            .map_or(Color::TRANSPARENT.rgba(), |settings| settings.clear.rgba())
    }

    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn preview_mut(&mut self) -> &mut Canvas {
        &mut self.preview
    }

    pub(crate) fn editor_2d_mut(&mut self) -> &mut Canvas {
        &mut self.editor_2d
    }

    pub(crate) fn editor_2d_texture_id(&self) -> dear_imgui_rs::TextureId {
        self.editor_2d.imgui_texture_id()
    }

    pub(crate) fn editor_3d_mut(&mut self) -> &mut Canvas {
        &mut self.editor_3d
    }

    pub(crate) fn editor_3d_texture_id(&self) -> dear_imgui_rs::TextureId {
        self.editor_3d.imgui_texture_id()
    }

    pub(crate) fn request_editor_3d_size(&mut self, size: [f32; 2]) {
        let size = (
            size[0].max(1.0).round() as u32,
            size[1].max(1.0).round() as u32,
        );
        if self.editor_3d.size() != size {
            self.pending_editor_3d_size = Some(size);
        }
    }

    pub(crate) fn take_pending_editor_3d_size(&mut self) -> Option<(u32, u32)> {
        self.pending_editor_3d_size.take()
    }

    pub(crate) fn request_editor_mouse_warp(&mut self, position: [f32; 2]) {
        self.pending_mouse_warp = Some(position);
        self.suppress_editor_mouse_delta = true;
    }

    pub(crate) fn take_pending_mouse_warp(&mut self) -> Option<[f32; 2]> {
        self.pending_mouse_warp.take()
    }

    pub(crate) fn editor_mouse_delta(&mut self, delta: [f32; 2]) -> [f32; 2] {
        if std::mem::take(&mut self.suppress_editor_mouse_delta) {
            [0.0; 2]
        } else {
            delta
        }
    }

    pub(crate) fn resize_editor_3d(
        &mut self,
        size: (u32, u32),
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) {
        if self.editor_3d.size() == size {
            return;
        }
        let replacement = Canvas::new_3d(size, imgui_renderer, skia_context, gl);
        imgui_renderer
            .texture_map_mut()
            .remove(self.editor_3d.imgui_texture_id());
        self.editor_3d = replacement;
        self.editor_3d_rendered = None;
    }

    pub fn preview_fps(&self) -> f32 {
        self.canvas_timer.fps()
    }

    fn process_export_frame(&mut self, gl: &glow::Context) {
        let result = self.renderer.process_frame(
            gl,
            self.preview.framebuffer(),
            self.project.settings.resolution,
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

    fn draw_editor_2d(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        scene_key: (u64, u64),
    ) {
        let canvas = self.editor_2d_canvas();
        let view = self.editor_view_2d;
        let key = (
            scene_key,
            canvas,
            [
                view.pan[0].to_bits(),
                view.pan[1].to_bits(),
                view.zoom.to_bits(),
                view.correction[0].to_bits(),
                view.correction[1].to_bits(),
                u32::from(view.canvas_view),
            ],
        );
        if self.editor_rendered == Some(key) {
            return;
        }
        let scene = &self.scenes[self.active_scene].scene;
        if let Err(error) = self.canvases.render_editor_2d(
            scene,
            canvas,
            &mut self.editor_2d.target,
            skia_context,
            view.pan,
            view.zoom,
            view.correction,
            view.canvas_view,
        ) {
            self.render_error = Some(error);
            return;
        }
        self.editor_rendered = Some(key);
        crate::renderer::target::reset_gl(gl, window_size);
        skia_context.reset(None);
    }

    fn draw_editor_3d(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        scene_key: (u64, u64),
    ) {
        let canvas = self.editor_3d_canvas();
        let camera = self.editor_3d_camera();
        let size = self.editor_3d.size();
        let view = self.editor_view_3d;
        let selection = self.selected_entity();
        let key = (
            scene_key,
            canvas,
            selection,
            [
                view.position.x.to_bits(),
                view.position.y.to_bits(),
                view.position.z.to_bits(),
                view.yaw.to_bits(),
                view.pitch.to_bits(),
                size.0,
                size.1,
                u32::from(view.canvas_view),
                u32::from(view.axes[0]),
                u32::from(view.axes[1]),
                u32::from(view.axes[2]),
            ],
        );
        if self.editor_3d_rendered == Some(key) {
            return;
        }
        let scene = &self.scenes[self.active_scene].scene;
        let selection =
            selection.and_then(|target| scene.editor_3d_selection_segments(canvas, target));
        let (canvas_camera, canvas_size) = self.editor_3d_canvas_camera();
        let guides = crate::renderer::editor_guides::EditorGuides3D {
            axes: view.axes,
            canvas_camera: (!view.canvas_view).then_some((canvas_camera, canvas_size)),
            selection,
        };
        if let Err(error) = self.canvases.render_editor_3d(
            scene,
            canvas,
            &mut self.editor_3d.target,
            skia_context,
            &camera,
            &guides,
        ) {
            self.render_error = Some(error);
            return;
        }
        self.editor_3d_rendered = Some(key);
        crate::renderer::target::reset_gl(gl, window_size);
        skia_context.reset(None);
    }

    fn update_active_scene(&mut self, time: f32) {
        let active_scene = active_scene_at(&self.scenes, time);

        self.active_scene = active_scene;

        let scene = &self.scenes[self.active_scene];
        let local_time = (time - scene.start).clamp(0.0, scene.end - scene.start);
        let identity = scene.scene.render_key().0;
        if self.evaluated != Some((identity, local_time)) {
            let started = std::time::Instant::now();
            scene.scene.update(local_time);
            self.performance.update_ms = started.elapsed().as_secs_f32() * 1000.0;
            self.evaluated = Some((identity, local_time));
        }
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
    fps: u32,
) -> Vec<EditorScene> {
    let mut start = 0.0;

    factories
        .iter()
        .map(|create_scene| {
            let mut scene = create_scene(resolution);
            scene.set_fps(fps);
            let end = start + scene.duration();
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
        scene.end = start + scene.scene.duration();
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
        let scenes = create_scenes(&factories, (1280, 720), 60);

        assert_eq!(scenes[0].scene.name(), "opening");
        assert_eq!([scenes[0].start, scenes[0].end], [0.0, 2.0]);
        assert_eq!(
            scenes[0]
                .scene
                .world()
                .get::<&crate::core::objects::CanvasSettings>(scenes[0].scene.world_2d().entity(),)
                .unwrap()
                .resolution,
            (1280, 720)
        );
        assert_eq!(scenes[1].scene.name(), "ending");
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
