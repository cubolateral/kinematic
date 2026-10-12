use crate::{
    core::{
        Project, ProjectSettings, Scene, TrackInfo, TrackValue, objects::ObjectHandler,
        types::Vector2,
    },
    editor::{
        Canvas, Selection, Timeline,
        cache::{Camera2DCache, Camera3DCache, EditorCache, EditorMode},
    },
    renderer::{FrameResult, Renderer},
    utilities::FrameTimer,
};

#[derive(Default)]
pub(crate) struct Performance {
    pub prepare_ms: f64,
    pub replay_ms: f64,
    pub apply_ms: f64,
    pub update_ms: f32,
    pub render_ms: f32,
    pub avoided: u64,
}

struct EditorScene {
    scene: SceneInstance,
    start: f32,
    end: f32,
    inspector_edits: InspectorEdits,
}

enum SceneInstance {
    Empty(Scene),
    Lua(crate::scripting::LuaScene),
}
impl std::ops::Deref for SceneInstance {
    type Target = Scene;
    fn deref(&self) -> &Scene {
        match self {
            Self::Empty(scene) => scene,
            Self::Lua(scene) => &scene.scene,
        }
    }
}
impl std::ops::DerefMut for SceneInstance {
    fn deref_mut(&mut self) -> &mut Scene {
        match self {
            Self::Empty(scene) => scene,
            Self::Lua(scene) => &mut scene.scene,
        }
    }
}
impl SceneInstance {
    fn evaluate(&self, time: f32) -> Result<(), String> {
        match self {
            Self::Empty(scene) => {
                scene.update(time);
                Ok(())
            }
            Self::Lua(scene) => scene.update(time).map_err(|e| e.to_string()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct InspectorTrack {
    entity: hecs::Entity,
    target: InspectorTarget,
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum InspectorTarget {
    Property {
        component: std::any::TypeId,
        track_id: crate::core::TrackId,
    },
    Uniform(String),
}

impl InspectorTrack {
    pub(crate) fn new(
        entity: hecs::Entity,
        component: std::any::TypeId,
        track: &'static TrackInfo,
    ) -> Self {
        Self {
            entity,
            target: InspectorTarget::Property {
                component,
                track_id: track.id,
            },
        }
    }

    pub(crate) fn uniform(entity: hecs::Entity, name: impl Into<String>) -> Self {
        Self {
            entity,
            target: InspectorTarget::Uniform(name.into()),
        }
    }
}

struct InspectorEdit {
    key: InspectorTrack,
    track: Option<&'static TrackInfo>,
    original: TrackValue,
    current: TrackValue,
    animated: bool,
}

#[derive(Default)]
pub(crate) struct InspectorEdits(Vec<InspectorEdit>);

impl InspectorEdits {
    pub(crate) fn contains(&self, key: &InspectorTrack) -> bool {
        self.0.iter().any(|edit| edit.key == *key)
    }

    pub(crate) fn record(
        &mut self,
        key: InspectorTrack,
        track: &'static TrackInfo,
        before: TrackValue,
        value: TrackValue,
        animated: bool,
    ) {
        if let Some(edit) = self.0.iter_mut().find(|edit| edit.key == key) {
            edit.current = value;
            if edit.current == edit.original {
                self.0.retain(|edit| edit.key != key);
            }
        } else if before != value {
            self.0.push(InspectorEdit {
                key,
                track: Some(track),
                original: before,
                current: value,
                animated,
            });
        }
    }

    pub(crate) fn record_uniform(
        &mut self,
        key: InspectorTrack,
        before: TrackValue,
        value: TrackValue,
        animated: bool,
    ) {
        if let Some(edit) = self.0.iter_mut().find(|edit| edit.key == key) {
            edit.current = value;
            if edit.current == edit.original {
                self.0.retain(|edit| edit.key != key);
            }
        } else if before != value {
            self.0.push(InspectorEdit {
                key,
                track: None,
                original: before,
                current: value,
                animated,
            });
        }
    }

    pub(crate) fn reset(&mut self, scene: &Scene, key: InspectorTrack) -> bool {
        let Some(index) = self.0.iter().position(|edit| edit.key == key) else {
            return false;
        };
        let edit = self.0.remove(index);
        Self::restore(scene, std::slice::from_ref(&edit));
        scene.invalidate();
        true
    }

    fn reset_animated(&mut self, scene: &Scene) {
        let (animated, remaining) = std::mem::take(&mut self.0)
            .into_iter()
            .partition(|edit| edit.animated);
        self.0 = remaining;
        Self::restore(scene, &animated);
    }

    fn restore(scene: &Scene, saved: &[InspectorEdit]) {
        if saved.is_empty() {
            return;
        }
        let world = scene.world();
        for saved in saved {
            match (&saved.key.target, saved.track) {
                (InspectorTarget::Property { .. }, Some(track)) => {
                    (track.set)(&world, saved.key.entity, saved.original.clone());
                }
                (InspectorTarget::Uniform(name), None) => {
                    crate::core::objects::set_shader_uniform(
                        &world,
                        saved.key.entity,
                        name,
                        saved.original.clone(),
                    )
                    .unwrap_or_else(|error| panic!("{error}"));
                }
                _ => unreachable!(),
            }
        }
        scene.invalidate();
    }
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

struct PendingLua {
    save_manifest: bool,
    revision: u64,
    seek: Option<(usize, f32)>,
    candidates: Vec<(usize, crate::scripting::LuaScene)>,
    manifest: Option<(crate::scripting::Manifest, Vec<usize>)>,
}

pub(crate) struct Editor {
    project: Project,
    pub(crate) lua_project: Option<crate::scripting::LuaProject>,
    pub(crate) documents: Option<crate::editor::documents::Documents>,
    pending_lua: Option<PendingLua>,
    scenes: Vec<EditorScene>,
    active_scene: usize,
    focused_scene: Option<usize>,
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
    margin_visible: bool,
    timeline_tracks_expanded: bool,
}

impl Editor {
    pub(crate) fn new_lua(
        project: crate::scripting::LuaProject,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
        three_context: three_d::Context,
    ) -> Result<Self, String> {
        let mut documents = crate::editor::documents::Documents::new(
            project.context.clone(),
            &crate::scripting::definitions(),
        )?;
        for path in &project.manifest.scenes {
            if let Err(error) = documents.open(path) {
                documents.diagnostics.push(error);
            }
        }
        let sources = documents.sources();
        let scenes = project
            .manifest
            .scenes
            .iter()
            .map(|path| {
                let scene = match crate::scripting::LuaScene::load(
                    project.context.clone(),
                    path,
                    sources.clone(),
                    project.manifest.resolution,
                    project.manifest.fps,
                    project.manifest.margin,
                ) {
                    Ok(scene) => SceneInstance::Lua(scene),
                    Err(error) => {
                        documents.diagnostics.push(error.to_string());
                        documents.status =
                            "Scene failed. Correct the script to build its preview.".into();
                        SceneInstance::Empty(Scene::new_candidate(
                            &path.display().to_string(),
                            project.manifest.resolution,
                        ))
                    }
                };
                EditorScene {
                    scene,
                    start: 0.0,
                    end: 0.0,
                    inspector_edits: InspectorEdits::default(),
                }
            })
            .collect::<Vec<_>>();
        let metadata = Project {
            name: project.manifest.name.clone(),
            context: project.context.clone(),
            settings: project.manifest.settings(),
        };
        let mut editor = Self::with_scenes(
            metadata,
            scenes,
            imgui_renderer,
            skia_context,
            gl,
            three_context,
        );
        editor.documents = Some(documents);
        editor.lua_project = Some(project);
        editor.update_active_scene(editor.timeline.time());
        Ok(editor)
    }

    fn with_scenes(
        project: Project,
        mut scenes: Vec<EditorScene>,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
        three_context: three_d::Context,
    ) -> Self {
        recalculate_scene_ranges(&mut scenes);
        let cache = EditorCache::load(&project.context);
        let focused_scene = (scenes.len() > 1)
            .then(|| {
                cache
                    .focused_scene
                    .as_deref()
                    .and_then(|name| scenes.iter().position(|scene| scene.scene.name() == name))
            })
            .flatten();
        let duration = focused_scene.map_or_else(
            || scenes.last().map_or(0.0, |scene| scene.end),
            |index| scenes[index].end - scenes[index].start,
        );
        let mut timeline = Timeline::new(duration, project.settings.fps);
        let timeline_time = cache.timeline_time(duration);
        timeline.set_loop(cache.loop_enabled);
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
        let mut renderer = Renderer::new(project.settings.resolution);
        renderer.set_project_root(project.context.root());

        Self {
            project,
            lua_project: None,
            documents: None,
            pending_lua: None,
            scenes,
            active_scene: focused_scene.unwrap_or(0),
            focused_scene,
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
            margin_visible: cache.margin_visible,
            timeline_tracks_expanded: cache.timeline_tracks_expanded,
        }
    }

    fn poll_scripts(&mut self) {
        let Some(documents) = &mut self.documents else {
            return;
        };
        if !documents.poll() {
            return;
        }
        if self.is_exporting {
            documents.status = "Edits pending until export completes.".into();
            return;
        }
        let started = std::time::Instant::now();
        self.performance.replay_ms = 0.0;
        let result = (|| {
            let sources = documents.sources();
            let project = self.lua_project.as_ref().unwrap();
            let manifest_changed = documents
                .changed
                .contains(std::path::Path::new("kinematic.ron"));
            let manifest = if manifest_changed {
                let text = sources
                    .get(std::path::Path::new("kinematic.ron"))
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(|| {
                        std::fs::read_to_string(project.context.root().join("kinematic.ron"))
                    })
                    .map_err(|e| e.to_string())?;
                let manifest: crate::scripting::Manifest =
                    ron::from_str(&text).map_err(|e| e.to_string())?;
                manifest.validate(&project.context)?;
                manifest
            } else {
                project.manifest.clone()
            };
            let settings_changed = manifest.settings() != project.manifest.settings();
            let mut candidates = Vec::new();
            let mut order = Vec::new();
            let mut next_index = self.scenes.len();
            for path in &manifest.scenes {
                let index = project
                    .manifest
                    .scenes
                    .iter()
                    .position(|old| old == path)
                    .unwrap_or_else(|| {
                        let index = next_index;
                        next_index += 1;
                        index
                    });
                order.push(index);
                if let Some(EditorScene {
                    scene: SceneInstance::Lua(scene),
                    ..
                }) = self.scenes.get(index)
                {
                    if settings_changed
                        || documents
                            .changed
                            .iter()
                            .any(|path| scene.dependencies().contains(path))
                    {
                        let candidate = scene
                            .prepare_settings_pending(
                                sources.clone(),
                                manifest.resolution,
                                manifest.fps,
                                manifest.margin,
                            )
                            .map_err(|e| e.to_string())?;
                        candidates.push((index, candidate));
                    }
                } else {
                    let candidate = crate::scripting::LuaScene::load(
                        project.context.clone(),
                        path,
                        sources.clone(),
                        manifest.resolution,
                        manifest.fps,
                        manifest.margin,
                    )
                    .map_err(|e| e.to_string())?;
                    candidates.push((index, candidate));
                }
            }
            Ok::<_, String>(PendingLua {
                save_manifest: false,
                revision: documents.revision,
                seek: None,
                candidates,
                manifest: manifest_changed.then_some((manifest, order)),
            })
        })();
        self.performance.prepare_ms = started.elapsed().as_secs_f64() * 1000.0;
        documents.applied = documents.revision;
        match result {
            Ok(pending) => {
                documents.replaying = true;
                self.pending_lua = Some(pending);
                documents.status = "Validating drawing and resources.".into();
            }
            Err(error) => {
                documents.status = "Edits failed; showing the last valid revision.".into();
                documents.diagnostics = vec![error];
            }
        }
    }

    pub fn update(&mut self) {
        self.poll_scripts();
        self.window_timer.tick();

        if self.pending_lua.is_some() && !self.is_exporting {
            self.accumulator = 0.0;
            return;
        }

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
                self.reset_animated_inspector_edits();
                self.update_active_scene(time);
            }

            self.canvas_timer.tick();
            self.accumulator -= delta;
        }
    }

    fn restore_seek_time(&mut self, seek: Option<(usize, f32)>) {
        if let Some((index, _)) = seek {
            let scene = &self.scenes[index];
            self.timeline.go_to(
                scene.scene.time()
                    + if self.focused_scene.is_some() {
                        0.0
                    } else {
                        scene.start
                    },
            );
            self.timeline.pause();
        }
    }

    pub(crate) fn apply_pending_lua(
        &mut self,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) {
        if self.is_exporting {
            return;
        }
        let Some(mut pending) = self.pending_lua.take() else {
            return;
        };
        if self
            .documents
            .as_ref()
            .is_none_or(|docs| docs.revision != pending.revision)
        {
            self.documents.as_mut().unwrap().replaying = false;
            self.restore_seek_time(pending.seek);
            return;
        }
        for (index, candidate) in &pending.candidates {
            let time = pending
                .seek
                .filter(|(target, _)| target == index)
                .map_or_else(
                    || {
                        self.scenes
                            .get(*index)
                            .map_or(0.0, |scene| scene.scene.time())
                    },
                    |(_, time)| time,
                )
                .min(candidate.duration());
            let started = std::time::Instant::now();
            let result = candidate.update_slice(time);
            self.performance.replay_ms += started.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(true) => {}
                Ok(false) => {
                    self.documents.as_mut().unwrap().status = format!(
                        "Replaying simulations at {time:.2} s: {} steps. Editing or Cancel discards this candidate.",
                        candidate.simulation_steps()
                    );
                    self.pending_lua = Some(pending);
                    return;
                }
                Err(error) => {
                    self.restore_seek_time(pending.seek);
                    let docs = self.documents.as_mut().unwrap();
                    docs.replaying = false;
                    docs.status = "Replay failed. Showing the last valid revision.".into();
                    docs.diagnostics.push(error.to_string());
                    return;
                }
            }
        }
        let gpu_started = std::time::Instant::now();
        let uploads = self.canvases.geometry_uploads;
        let upload_ms = self.canvases.geometry_upload_ms;
        let result = pending
            .candidates
            .iter()
            .try_for_each(|(_, scene)| self.canvases.validate(scene.scene(), skia));
        let validation_ms = gpu_started.elapsed().as_secs_f64() * 1000.0;
        let applied_at = std::time::Instant::now();
        if result.is_err() {
            self.restore_seek_time(pending.seek);
        }
        let documents = self.documents.as_mut().unwrap();
        if let Err(error) = result {
            documents.replaying = false;
            documents.status = "Drawing failed. Showing the last valid revision.".into();
            documents.diagnostics.push(error);
            return;
        }
        if pending.save_manifest {
            let project = self.lua_project.as_mut().unwrap();
            let previous = project.manifest.clone();
            project.manifest = pending.manifest.as_ref().unwrap().0.clone();
            if let Err(error) = project.save() {
                project.manifest = previous;
                documents.replaying = false;
                documents.status =
                    "Manifest save conflicted. Showing the last valid revision.".into();
                documents.diagnostics.push(error);
                return;
            }
        }
        for (_, candidate) in &mut pending.candidates {
            if let Err(error) = candidate.persist_prepared() {
                documents.replaying = false;
                documents.status = "Save conflicted. Showing the last valid revision.".into();
                documents.diagnostics.push(error.to_string());
                return;
            }
        }
        documents.replaying = false;
        if pending.seek.is_none() {
            documents.diagnostics.clear();
            documents.changed.clear();
        }
        for (index, candidate) in pending.candidates {
            if index >= self.scenes.len() {
                self.scenes.push(EditorScene {
                    scene: SceneInstance::Lua(candidate),
                    start: 0.0,
                    end: 0.0,
                    inspector_edits: InspectorEdits::default(),
                });
            } else if let SceneInstance::Lua(scene) = &mut self.scenes[index].scene {
                scene.apply_reload(candidate);
                documents.console.extend(scene.console());
                documents.diagnostics.extend(scene.warnings());
            } else {
                self.scenes[index].scene = SceneInstance::Lua(candidate);
            }
        }
        documents
            .console
            .drain(..documents.console.len().saturating_sub(500));
        documents.status = if documents.changed.is_empty() {
            "Preview is current."
        } else {
            "Showing the active revision; source edits remain pending."
        }
        .into();
        let local_time = self.scenes[self.active_scene].scene.time();
        let mut preserved_time = None;
        if let Some((manifest, order)) = pending.manifest {
            let local_time = self.scenes[self.active_scene].scene.time();
            let next_active = order.iter().position(|index| *index == self.active_scene);
            self.focused_scene = self
                .focused_scene
                .and_then(|focus| order.iter().position(|index| *index == focus));
            if let Some((selected, entity)) = self.selection.get() {
                self.selection.clear();
                if let Some(index) = order.iter().position(|old| *old == selected) {
                    self.selection.select(index, entity);
                }
            }
            let mut previous = std::mem::take(&mut self.scenes)
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>();
            self.scenes = order
                .iter()
                .map(|index| previous[*index].take().unwrap())
                .collect();
            self.project.name = manifest.name.clone();
            let settings = manifest.settings();
            if settings.resolution != self.project.settings.resolution {
                self.resize_project_targets(&settings, imgui_renderer, skia, gl);
            }
            self.timeline.set_fps(settings.fps);
            self.project.settings = settings;
            self.lua_project.as_mut().unwrap().accept_manifest(manifest);
            let duration = recalculate_scene_ranges(&mut self.scenes);
            if let Some(active) = next_active {
                self.active_scene = active;
                preserved_time = Some(if self.focused_scene.is_some() {
                    local_time
                } else {
                    self.scenes[active].start + local_time.min(self.scenes[active].scene.duration())
                });
            } else {
                self.active_scene = 0;
                preserved_time = Some(0.0_f32.min(duration));
            }
        }
        let duration = recalculate_scene_ranges(&mut self.scenes);
        let preserved_time = preserved_time.unwrap_or_else(|| {
            if self.focused_scene.is_some() {
                local_time
            } else {
                self.scenes[self.active_scene].start + local_time
            }
        });
        self.timeline
            .set_duration(self.focused_scene.map_or(duration, |index| {
                self.scenes[index].end - self.scenes[index].start
            }));
        self.evaluated = None;
        self.rendered = None;
        self.editor_rendered = None;
        self.editor_3d_rendered = None;
        self.render_error = None;
        if let Some((index, entity)) = self.selection.get() {
            if !self.scenes[index].scene.world().contains(entity) {
                self.selection.clear();
            }
        }
        self.timeline.go_to(preserved_time);
        self.update_active_scene(self.timeline.time());
        self.performance.apply_ms = applied_at.elapsed().as_secs_f64() * 1000.0;
        if pending.seek.is_none() {
            let stats = &self.performance;
            let documents = self.documents.as_mut().unwrap();
            documents.console.push(format!(
                "Reload: Lua {:.2} ms; replay {:.2} ms; apply {:.2} ms; GPU validation {:.2} ms; mesh uploads {} ({:.2} ms); previous frame render {:.2} ms.",
                stats.prepare_ms, stats.replay_ms, stats.apply_ms, validation_ms,
                self.canvases.geometry_uploads - uploads, self.canvases.geometry_upload_ms - upload_ms, stats.render_ms,
            ));
            documents
                .console
                .drain(..documents.console.len().saturating_sub(500));
        }
    }

    pub fn draw(
        &mut self,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &glow::Context,
        window_size: (u32, u32),
        mode: EditorMode,
    ) {
        if self.render_error.is_some() {
            return;
        }
        let scene = &self.scenes[self.active_scene].scene;
        let key = scene.render_key();
        if self.rendered == Some(key) {
            self.performance.avoided += 1;
            if self.is_exporting {
                self.process_export_frame(gl);
            }
        } else {
            let started = std::time::Instant::now();
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
                self.timeline.pause();
                return;
            }
            self.render_error = None;
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
                &self.project.name,
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

        self.clear_scene_focus();
        self.start_export(silent);
    }

    pub fn export_scene(&mut self, silent: bool) {
        if self.is_exporting {
            return;
        }

        self.focus_scene(self.active_scene, 0.0);
        self.start_export(silent);
    }

    fn start_export(&mut self, silent: bool) {
        let started = self.renderer.start(
            &self.project.name,
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
        self.reset_animated_inspector_edits();
        self.evaluated = None;
        self.is_exporting = true;
        self.update_active_scene(0.0);
        if self.render_error.is_some() {
            self.renderer.cancel();
            self.is_exporting = false;
            return;
        }
        self.pending_export_time = None;
        self.pending_project_settings = None;
        self.is_exporting = true;
        self.accumulator = 0.0;
    }

    pub(crate) fn export_succeeded(&self) -> bool {
        self.renderer.succeeded()
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
            margin_visible: self.margin_visible,
            loop_enabled: self.timeline.loop_enabled(),
            timeline_tracks_expanded: self.timeline_tracks_expanded,
            focused_scene: self
                .focused_scene
                .map(|index| self.scenes[index].scene.name().to_owned()),
        }
        .save(&self.project.context);
        self.renderer.shutdown(gl);
    }

    pub(crate) fn margin_visible(&self) -> bool {
        self.margin_visible
    }

    pub(crate) fn toggle_margin(&mut self) {
        self.margin_visible = !self.margin_visible;
    }

    pub(crate) fn margin_size(&self) -> (u32, u32) {
        self.project.settings.margin
    }

    pub(crate) fn context(&self) -> &crate::core::ProjectContext {
        &self.project.context
    }
    pub(crate) fn cached_view(&self) -> (EditorMode, bool) {
        let cache = EditorCache::load(&self.project.context);
        (cache.mode, cache.fullscreen)
    }

    pub(crate) fn project_info(&self) -> (&str, ProjectSettings) {
        (&self.project.name, self.project.settings)
    }

    pub(crate) fn request_project_settings(&mut self, settings: ProjectSettings) {
        if self.is_exporting {
            return;
        }
        self.pending_project_settings = Some(settings);
    }

    pub(crate) fn take_pending_project_settings(&mut self) -> Option<ProjectSettings> {
        self.pending_project_settings.take()
    }

    pub(crate) fn apply_project_settings(&mut self, settings: ProjectSettings) {
        if self.is_exporting || settings == self.project.settings {
            return;
        }

        if let Some(project) = &self.lua_project {
            let documents = self.documents.as_mut().unwrap();
            if self.pending_lua.is_some() || !documents.changed.is_empty() {
                documents.status =
                    "Apply or resolve source edits before changing project settings.".into();
                return;
            }
            let started = std::time::Instant::now();
            let prepared = (|| {
                let mut manifest = project.manifest.clone();
                manifest.resolution = settings.resolution;
                manifest.fps = settings.fps;
                manifest.margin = settings.margin;
                manifest.validate(&project.context)?;
                let sources = documents.sources();
                let mut candidates = Vec::new();
                for (index, entry) in self.scenes.iter().enumerate() {
                    let scene = match &entry.scene {
                        SceneInstance::Lua(scene) => scene.prepare_settings_pending(
                            sources.clone(),
                            settings.resolution,
                            settings.fps,
                            settings.margin,
                        ),
                        SceneInstance::Empty(_) => crate::scripting::LuaScene::load(
                            project.context.clone(),
                            &manifest.scenes[index],
                            sources.clone(),
                            settings.resolution,
                            settings.fps,
                            settings.margin,
                        ),
                    }
                    .map_err(|error| error.to_string())?;
                    candidates.push((index, scene));
                }
                Ok::<_, String>((manifest, candidates))
            })();
            self.performance.prepare_ms = started.elapsed().as_secs_f64() * 1000.0;
            self.performance.replay_ms = 0.0;
            match prepared {
                Ok((manifest, candidates)) => {
                    documents.revision += 1;
                    documents.applied = documents.revision;
                    documents.replaying = true;
                    documents.status = "Preparing project settings before saving.".into();
                    self.pending_lua = Some(PendingLua {
                        save_manifest: true,
                        revision: documents.revision,
                        seek: None,
                        candidates,
                        manifest: Some((manifest, (0..self.scenes.len()).collect())),
                    });
                }
                Err(error) => documents.diagnostics.push(error),
            }
            return;
        }
    }

    fn resize_project_targets(
        &mut self,
        settings: &ProjectSettings,
        imgui_renderer: &mut dear_imgui_glow::GlowRenderer,
        skia_context: &mut skia_safe::gpu::DirectContext,
        gl: &std::rc::Rc<glow::Context>,
    ) {
        self.renderer.shutdown(gl);
        let preview = Canvas::new(settings.resolution, imgui_renderer, skia_context, gl);
        let editor_2d = Canvas::new(settings.resolution, imgui_renderer, skia_context, gl);
        let editor_3d = Canvas::new_3d(settings.resolution, imgui_renderer, skia_context, gl);
        self.preview.unregister(imgui_renderer);
        self.editor_2d.unregister(imgui_renderer);
        self.editor_3d.unregister(imgui_renderer);
        self.preview = preview;
        self.editor_2d = editor_2d;
        self.editor_rendered = None;
        self.editor_3d = editor_3d;
        self.editor_3d_rendered = None;
        self.pending_editor_3d_size = None;
        self.renderer = Renderer::new(settings.resolution);
        self.renderer.set_project_root(self.project.context.root());
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scenes[self.active_scene].scene
    }

    pub(crate) fn scene(&self, index: usize) -> &Scene {
        &self.scenes[index].scene
    }

    pub(crate) fn scene_range(&self, index: usize) -> [f32; 2] {
        let scene = &self.scenes[index];
        if self.focused_scene.is_some() {
            [0.0, scene.end - scene.start]
        } else {
            [scene.start, scene.end]
        }
    }

    pub(crate) fn scenes(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            &str,
            [f32; 2],
            &[crate::core::scene_file::ScheduledEvent],
        ),
    > + '_ {
        self.scenes
            .iter()
            .enumerate()
            .filter(|(index, _)| self.focused_scene.map_or(true, |focused| focused == *index))
            .map(|(index, scene)| {
                let range = if self.focused_scene.is_some() {
                    [0.0, scene.end - scene.start]
                } else {
                    [scene.start, scene.end]
                };
                (index, scene.scene.name(), range, scene.scene.events())
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

        if let SceneInstance::Lua(scene) = &self.scenes[scene_index].scene {
            if self.pending_lua.is_some()
                || self
                    .documents
                    .as_ref()
                    .is_some_and(|docs| !docs.changed.is_empty())
            {
                self.documents.as_mut().unwrap().status =
                    "Apply or resolve source edits before changing event durations.".into();
                return;
            }
            match scene.prepare_event_duration(event_index, duration) {
                Ok(candidate) => self.queue_lua_change(scene_index, candidate),
                Err(error) => self
                    .documents
                    .as_mut()
                    .unwrap()
                    .diagnostics
                    .push(error.to_string()),
            }
            return;
        }
    }

    pub(crate) fn focused_scene_index(&self) -> Option<usize> {
        self.focused_scene
    }

    pub(crate) fn focus_scene(&mut self, index: usize, time: f32) {
        if self.scenes.len() <= 1 || index >= self.scenes.len() {
            return;
        }

        self.timeline.pause();
        self.focused_scene = Some(index);
        self.selection.clear();
        let duration = self.scenes[index].end - self.scenes[index].start;
        self.timeline.set_duration(duration);
        self.timeline.go_to(time.clamp(0.0, duration));
        self.evaluated = None;
        self.update_active_scene(self.timeline.time());
    }

    pub(crate) fn clear_scene_focus(&mut self) {
        let Some(index) = self.focused_scene.take() else {
            return;
        };

        let time = self.scenes[index].start + self.timeline.time();
        self.timeline
            .set_duration(self.scenes.last().map_or(0.0, |scene| scene.end));
        self.timeline.go_to(time);
        self.evaluated = None;
        self.update_active_scene(time);
    }
    pub fn active_scene_index(&self) -> usize {
        self.active_scene
    }

    pub(crate) fn timeline_tracks_expanded(&self) -> bool {
        self.timeline_tracks_expanded
    }

    pub(crate) fn toggle_timeline_tracks(&mut self) {
        self.timeline_tracks_expanded = !self.timeline_tracks_expanded;
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

    pub(crate) fn lua_inspector_values(
        &self,
        index: usize,
        entity: hecs::Entity,
    ) -> Option<(
        bool,
        Vec<(
            crate::core::TrackTarget,
            TrackValue,
            Option<TrackValue>,
            TrackValue,
        )>,
    )> {
        match &self.scenes[index].scene {
            SceneInstance::Lua(scene) => scene.inspector_values(entity),
            _ => None,
        }
    }
    pub(crate) fn set_lua_override(
        &mut self,
        index: usize,
        entity: hecs::Entity,
        target: &crate::core::TrackTarget,
        value: Option<TrackValue>,
    ) {
        if self.is_exporting {
            return;
        }
        if self.pending_lua.is_some()
            || self
                .documents
                .as_ref()
                .is_some_and(|docs| !docs.changed.is_empty())
        {
            self.documents.as_mut().unwrap().status =
                "Apply or resolve source edits before changing base values.".into();
            return;
        }
        if let SceneInstance::Lua(scene) = &self.scenes[index].scene {
            match scene.prepare_override(entity, target, value) {
                Ok(Some(candidate)) => self.queue_lua_change(index, candidate),
                Ok(None) => {
                    self.rendered = None;
                    self.editor_rendered = None;
                    self.editor_3d_rendered = None;
                }
                Err(error) => self
                    .documents
                    .as_mut()
                    .unwrap()
                    .diagnostics
                    .push(error.to_string()),
            }
        }
    }

    fn queue_lua_change(&mut self, index: usize, candidate: crate::scripting::LuaScene) {
        let documents = self.documents.as_mut().unwrap();
        documents.revision += 1;
        documents.applied = documents.revision;
        documents.replaying = true;
        documents.status = "Validating the edited base and drawing before saving.".into();
        self.pending_lua = Some(PendingLua {
            save_manifest: false,
            revision: documents.revision,
            seek: None,
            candidates: vec![(index, candidate)],
            manifest: None,
        });
    }

    pub(crate) fn inspector_scene_mut(
        &mut self,
        index: usize,
    ) -> (&mut Scene, &mut InspectorEdits) {
        let scene = &mut self.scenes[index];
        (&mut scene.scene, &mut scene.inspector_edits)
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
                camera.camera_fov = match camera.camera_mode {
                    crate::core::components::Camera3DMode::Perspective => {
                        ((camera.camera_fov * 0.5).tan() * canvas_aspect / editor_aspect).atan()
                            * 2.0
                    }
                    crate::core::components::Camera3DMode::Orthogonal => {
                        camera.camera_fov * canvas_aspect / editor_aspect
                    }
                };
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

    pub(crate) fn editor_3d_mut(&mut self) -> &mut Canvas {
        &mut self.editor_3d
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
        self.editor_3d.unregister(imgui_renderer);
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
        let active_scene = self
            .focused_scene
            .unwrap_or_else(|| active_scene_at(&self.scenes, time));

        self.active_scene = active_scene;

        let scene = &self.scenes[self.active_scene];
        let local_time = if self.focused_scene.is_some() {
            time
        } else {
            time - scene.start
        }
        .clamp(0.0, scene.end - scene.start);
        let identity = scene.scene.render_key().0;
        if self.evaluated != Some((identity, local_time)) {
            if !self.is_exporting
                && self.pending_lua.is_none()
                && (scene.scene.time() - local_time).abs() > 4.0 / self.project.settings.fps as f32
            {
                if let SceneInstance::Lua(lua) = &scene.scene {
                    if lua.has_simulations() {
                        match lua.prepare_seek() {
                            Ok(candidate) => {
                                let documents = self.documents.as_mut().unwrap();
                                documents.replaying = true;
                                documents.status = "Replaying the active revision. Cancel keeps the previous preview.".into();
                                self.pending_lua = Some(PendingLua {
                                    save_manifest: false,
                                    revision: documents.revision,
                                    candidates: vec![(active_scene, candidate)],
                                    manifest: None,
                                    seek: Some((active_scene, local_time)),
                                });
                            }
                            Err(error) => self
                                .documents
                                .as_mut()
                                .unwrap()
                                .diagnostics
                                .push(error.to_string()),
                        }
                        return;
                    }
                }
            }
            let started = std::time::Instant::now();
            if let Err(error) = scene.scene.evaluate(local_time) {
                self.timeline.pause();
                self.render_error = Some(error.clone());
                if self.is_exporting {
                    self.renderer.cancel();
                    self.is_exporting = false;
                    self.pending_export_time = None;
                }
                if let Some(documents) = &mut self.documents {
                    if documents.diagnostics.last() != Some(&error) {
                        documents.diagnostics.push(error);
                    }
                }
                return;
            }
            if let SceneInstance::Lua(scene) = &scene.scene {
                if let Some(documents) = &mut self.documents {
                    documents.console.extend(scene.console());
                    documents
                        .console
                        .drain(..documents.console.len().saturating_sub(500));
                }
            }
            self.render_error = None;
            self.performance.update_ms = started.elapsed().as_secs_f32() * 1000.0;
            self.evaluated = Some((identity, local_time));
        }
    }

    fn reset_animated_inspector_edits(&mut self) {
        for scene in &mut self.scenes {
            scene.inspector_edits.reset_animated(&scene.scene);
        }
    }
}

fn active_scene_at(scenes: &[EditorScene], time: f32) -> usize {
    scenes
        .iter()
        .position(|scene| time < scene.end)
        .unwrap_or(scenes.len() - 1)
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
    use crate::test_support::{ImageShader, rect};

    #[test]
    fn scenes_advance_at_the_end_of_each_range() {
        let scenes = vec![
            EditorScene {
                scene: SceneInstance::Empty(Scene::new()),
                start: 0.0,
                end: 2.0,
                inspector_edits: InspectorEdits::default(),
            },
            EditorScene {
                scene: SceneInstance::Empty(Scene::new()),
                start: 2.0,
                end: 5.0,
                inspector_edits: InspectorEdits::default(),
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
                scene: SceneInstance::Empty(Scene::new()),
                start: 10.0,
                end: 12.0,
                inspector_edits: InspectorEdits::default(),
            },
            EditorScene {
                scene: SceneInstance::Empty(Scene::new()),
                start: 12.0,
                end: 15.0,
                inspector_edits: InspectorEdits::default(),
            },
        ];
        scenes[0].scene.wait(4.0);
        scenes[1].scene.wait(2.0);

        let duration = recalculate_scene_ranges(&mut scenes);

        assert_eq!([scenes[0].start, scenes[0].end], [0.0, 4.0]);
        assert_eq!([scenes[1].start, scenes[1].end], [4.0, 6.0]);
        assert_eq!(duration, 6.0);
    }

    #[test]
    fn timeline_updates_reset_only_animated_inspector_edits() {
        use crate::core::{TrackChoices, TrackLimits};

        struct Values([f32; 2]);

        fn get<const INDEX: usize>(world: &hecs::World, entity: hecs::Entity) -> TrackValue {
            TrackValue::F32(world.get::<&Values>(entity).unwrap().0[INDEX])
        }

        fn set<const INDEX: usize>(world: &hecs::World, entity: hecs::Entity, value: TrackValue) {
            let TrackValue::F32(value) = value else {
                panic!("Test track requires an f32 value.");
            };
            world.get::<&mut Values>(entity).unwrap().0[INDEX] = value;
        }

        static TRACKS: [TrackInfo; 2] = [
            TrackInfo {
                id: 0,
                name: "animated",
                limits: TrackLimits::None,
                choices: TrackChoices::None,
                get: get::<0>,
                set: set::<0>,
            },
            TrackInfo {
                id: 1,
                name: "static",
                limits: TrackLimits::None,
                choices: TrackChoices::None,
                get: get::<1>,
                set: set::<1>,
            },
        ];

        let scene = Scene::new();
        let entity = scene.world_mut().spawn((Values([10.0, 20.0]),));
        let component = std::any::TypeId::of::<Values>();
        let animated = InspectorTrack::new(entity, component, &TRACKS[0]);
        let static_track = InspectorTrack::new(entity, component, &TRACKS[1]);
        let mut edits = InspectorEdits::default();
        edits.record(
            animated.clone(),
            &TRACKS[0],
            TrackValue::F32(1.0),
            TrackValue::F32(10.0),
            true,
        );
        edits.record(
            static_track.clone(),
            &TRACKS[1],
            TrackValue::F32(2.0),
            TrackValue::F32(20.0),
            false,
        );

        edits.reset_animated(&scene);

        assert_eq!(
            (TRACKS[0].get)(&scene.world(), entity),
            TrackValue::F32(1.0)
        );
        assert_eq!(
            (TRACKS[1].get)(&scene.world(), entity),
            TrackValue::F32(20.0)
        );
        assert!(!edits.contains(&animated));
        assert!(edits.contains(&static_track));
    }

    #[test]
    fn timeline_updates_restore_animated_uniform_inspector_edits() {
        let shader = ImageShader::new("#version 330 core\nvoid main() {}");
        let mut scene = Scene::new();
        let object = rect()
            .shader(&shader)
            .uniform("amount", 0.25_f32)
            .build(&mut scene);
        object.set_uniform("amount", 0.75_f32);
        let key = InspectorTrack::uniform(object.entity(), "amount");
        let mut edits = InspectorEdits::default();
        edits.record_uniform(
            key.clone(),
            TrackValue::F32(0.25),
            TrackValue::F32(0.75),
            true,
        );

        edits.reset_animated(&scene);

        assert_eq!(object.get_uniform::<f32>("amount"), 0.25);
        assert!(!edits.contains(&key));
    }

    #[test]
    fn inspector_reset_invalidates_the_rendered_scene() {
        use crate::core::components::Camera3D;

        let scene = Scene::new();
        let entity = scene.world_3d().entity();
        let track = <Camera3D as crate::core::Trackable>::track(0);
        let key = InspectorTrack::new(entity, std::any::TypeId::of::<Camera3D>(), track);
        let mut edits = InspectorEdits::default();
        let original = TrackValue::Enum("Perspective");
        let current = TrackValue::Enum("Orthogonal");
        (track.set)(&scene.world(), entity, current.clone());
        edits.record(key.clone(), track, original.clone(), current, false);
        let render_key = scene.render_key();

        assert!(edits.reset(&scene, key));
        assert_ne!(scene.render_key(), render_key);
        assert_eq!((track.get)(&scene.world(), entity), original);
    }
}
