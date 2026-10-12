use super::{Sources, extension, native, value};
use crate::core::objects::Object;
use crate::core::{
    AnimatorHandle, Easing, ProjectContext, Scene, SceneWorld, Task, TrackChoices, TrackTarget,
    TrackValue,
    components::{Name, TreeNode},
    objects::{ObjectHandler, attach_child, deactivate_subtree},
};
use mlua::{Function, Lua, MetaMethod, Result, Table, UserData, UserDataMethods, Value};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq)]
struct Definition {
    kind: String,
    entity: hecs::Entity,
    fingerprint: String,
}

pub(crate) struct Runtime {
    pub(super) lua: Lua,
    pub(super) context: ProjectContext,
    pub(super) sources: Sources,
    pub(super) dependencies: RefCell<BTreeSet<PathBuf>>,
    pub(super) scene: RefCell<Option<Scene>>,
    pub(super) world: RefCell<SceneWorld>,
    pub(super) animator: RefCell<AnimatorHandle>,
    definitions: RefCell<BTreeMap<String, Definition>>,
    previous: BTreeMap<String, Definition>,
    extensions: Vec<native::NativeObject>,
    pub(super) anonymous: Cell<u64>,
    pub(super) building: Cell<bool>,
    drawing: Cell<bool>,
    pub(super) assets: RefCell<Vec<Box<dyn std::any::Any>>>,
    pub(super) deadline: Rc<Cell<Instant>>,
    pub(super) errors: RefCell<Vec<String>>,
    pub(super) console: RefCell<Vec<String>>,
    last_print: RefCell<Option<String>>,
    pub(super) writes: RefCell<Option<Vec<(hecs::Entity, TrackTarget, TrackValue)>>>,
    pub(super) id: u64,
    pub(super) callbacks: RefCell<BTreeMap<hecs::Entity, extension::Callbacks>>,
    pub(super) simulation_steps: Cell<u64>,
    pub(super) mutations: RefCell<Vec<Function>>,
    overrides: super::overrides::Overrides,
    path: PathBuf,
    fps: u32,
    script_values: RefCell<BTreeMap<hecs::Entity, BTreeMap<String, TrackValue>>>,
    warnings: RefCell<Vec<String>>,
    pub(super) operation_keys: RefCell<BTreeMap<hecs::Entity, Vec<String>>>,
}

/// A compiled Lua scene and the isolated VM that owns its callbacks.
pub struct LuaScene {
    pub(crate) scene: Scene,
    runtime: Rc<Runtime>,
    path: PathBuf,
    resolution: (u32, u32),
    fps: u32,
    margin: (u32, u32),
    revision: u64,
    pending_write: Option<(PathBuf, Vec<u8>, Option<Vec<u8>>)>,
}

pub(super) fn engine<T>(f: impl FnOnce() -> T) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "Engine operation failed.".to_owned());
        mlua::Error::runtime(message)
    })
}

impl LuaScene {
    pub(crate) fn simulation_steps(&self) -> u64 {
        self.runtime.simulation_steps.get()
    }

    /// Compiles one scene without writing files or changing any active revision.
    pub fn load(
        context: ProjectContext,
        path: impl AsRef<Path>,
        sources: Sources,
        resolution: (u32, u32),
        fps: u32,
        margin: (u32, u32),
    ) -> Result<Self> {
        Self::load_registered(context, path, sources, resolution, fps, margin, Vec::new())
    }

    pub(crate) fn load_registered(
        context: ProjectContext,
        path: impl AsRef<Path>,
        sources: Sources,
        resolution: (u32, u32),
        fps: u32,
        margin: (u32, u32),
        extensions: Vec<native::NativeObject>,
    ) -> Result<Self> {
        Self::prepare(
            context,
            path.as_ref().to_owned(),
            sources,
            resolution,
            fps,
            margin,
            BTreeMap::new(),
            Vec::new(),
            None,
            None,
            extensions,
        )
    }

    fn prepare(
        context: ProjectContext,
        path: PathBuf,
        sources: Sources,
        resolution: (u32, u32),
        fps: u32,
        margin: (u32, u32),
        previous: BTreeMap<String, Definition>,
        allocated: Vec<hecs::Entity>,
        identity: Option<u64>,
        overrides: Option<super::overrides::Overrides>,
        extensions: Vec<native::NativeObject>,
    ) -> Result<Self> {
        if fps == 0
            || fps > 1000
            || resolution.0 == 0
            || resolution.1 == 0
            || resolution.0 > 8192
            || resolution.1 > 8192
        {
            return Err(mlua::Error::runtime(
                "Resolution must be 1–8192 pixels per dimension; FPS must be 1–1000.",
            ));
        }
        let lua = Lua::new_with(
            mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH | mlua::StdLib::UTF8,
            mlua::LuaOptions::default(),
        )?;
        lua.set_memory_limit(64 * 1024 * 1024)?;
        lua.globals()
            .get::<Table>("math")?
            .get::<Function>("randomseed")?
            .call::<()>((0_i64, 0_i64))?;
        for name in ["dofile", "loadfile", "load", "collectgarbage"] {
            lua.globals().set(name, Value::Nil)?;
        }

        let deadline = Rc::new(Cell::new(Instant::now() + Duration::from_secs(2)));
        let limit = deadline.clone();
        lua.set_hook(
            mlua::HookTriggers::new().every_nth_instruction(1000),
            move |_, _| {
                if Instant::now() > limit.get() {
                    Err(mlua::Error::runtime("Lua execution limit exceeded."))
                } else {
                    Ok(mlua::VmState::Continue)
                }
            },
        )?;
        let mut scene = Scene::new_candidate("Lua", resolution);
        scene.set_fps(fps);
        scene.set_margin(margin);
        if let Some(identity) = identity {
            scene.set_identity(identity);
        }
        for entity in allocated {
            if !scene.world().contains(entity) {
                scene.world_mut().spawn_at(entity, ());
            }
        }
        let overrides = overrides
            .map(Ok)
            .unwrap_or_else(|| super::overrides::Overrides::read(&context))
            .map_err(mlua::Error::runtime)?;
        let runtime = Rc::new(Runtime {
            overrides,
            path: path.clone(),
            fps,
            script_values: RefCell::new(BTreeMap::new()),
            warnings: RefCell::new(Vec::new()),
            id: extension::next_runtime(),
            callbacks: RefCell::new(BTreeMap::new()),
            simulation_steps: Cell::new(0),
            mutations: RefCell::new(Vec::new()),
            operation_keys: RefCell::new(BTreeMap::new()),
            world: RefCell::new(scene.world_handle()),
            animator: RefCell::new(scene.animator_handle()),
            scene: RefCell::new(Some(scene)),
            lua,
            context,
            sources,
            dependencies: RefCell::new(BTreeSet::new()),
            definitions: RefCell::new(BTreeMap::new()),
            previous,
            extensions,
            anonymous: Cell::new(0),
            building: Cell::new(true),
            drawing: Cell::new(false),
            assets: RefCell::new(Vec::new()),
            deadline,
            errors: RefCell::new(Vec::new()),
            console: RefCell::new(Vec::new()),
            last_print: RefCell::new(None),
            writes: RefCell::new(None),
        });
        extension::register_runtime(&runtime);
        runtime.register()?;
        let events = event_path(&path);
        runtime.dependencies.borrow_mut().insert(events.clone());
        runtime
            .dependencies
            .borrow_mut()
            .insert(PathBuf::from("kinematic/overrides.ron"));
        let file = if let Some(text) = runtime.sources.get(&events) {
            ron::from_str(text).map_err(mlua::Error::external)?
        } else {
            crate::core::scene_file::SceneFile::read(
                &runtime
                    .context
                    .resolve(&events)
                    .map_err(mlua::Error::external)?,
            )
            .map_err(mlua::Error::external)?
        };
        runtime
            .scene
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set_event_definitions(file);
        let text = runtime.source(&path)?;
        let definition: Table = runtime
            .lua
            .load(&text)
            .set_name(format!("@{}", path.display()))
            .eval()?;
        let build: Function = definition.get("build")?;
        let name: String = definition.get("name")?;
        runtime.scene.borrow_mut().as_mut().unwrap().set_name(name);
        build.call::<()>(SceneHandle(Rc::downgrade(&runtime)))?;
        if let Some(error) = runtime.errors.borrow().first() {
            return Err(mlua::Error::runtime(error.clone()));
        }
        let mut scene = runtime.scene.borrow_mut().take().unwrap();
        engine(|| scene.finish_build())?;
        runtime.building.set(false);
        {
            let world = scene.world();
            for definition in runtime.definitions.borrow_mut().values_mut() {
                definition.fingerprint = fingerprint(&world, definition.entity);
                if let Some(callback) = runtime.callbacks.borrow().get(&definition.entity) {
                    definition.fingerprint.push_str(&callback.visual_key);
                }
            }
        }
        extension::validate_dependencies(&runtime)?;
        let simulation_keys = runtime
            .callbacks
            .borrow()
            .keys()
            .copied()
            .filter_map(|entity| runtime.simulation_key(entity).map(|key| (entity, key)))
            .collect::<Vec<_>>();
        for (entity, key) in simulation_keys {
            runtime
                .callbacks
                .borrow_mut()
                .get_mut(&entity)
                .unwrap()
                .history_key = Some(key);
        }
        if let Some(overrides) = runtime.overrides.values.get(&path) {
            for id in overrides.keys() {
                if !runtime.definitions.borrow().contains_key(id) {
                    runtime
                        .warnings
                        .borrow_mut()
                        .push(format!("Orphan override: {} / {id}.", path.display()));
                }
            }
        }
        scene.keep_runtime(runtime.clone());
        let result = Self {
            scene,
            runtime,
            path,
            resolution,
            fps,
            margin,
            revision: 1,
            pending_write: None,
        };
        if result.runtime.previous.is_empty() {
            result.update(0.0)?;
        }
        Ok(result)
    }

    /// Accesses the native scene for drawing and inspection.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    /// Evaluates the same compiled animation tracks used by native projects.
    pub fn update(&self, time: f32) -> Result<()> {
        if !time.is_finite() || time < 0.0 {
            return Err(mlua::Error::runtime(
                "Scene time must be finite and nonnegative.",
            ));
        }
        self.runtime.errors.borrow_mut().clear();
        self.runtime
            .deadline
            .set(Instant::now() + Duration::from_secs(2));
        let saved = self
            .scene
            .world()
            .query::<(hecs::Entity, &crate::core::components::Simulation)>()
            .iter()
            .map(|(entity, state)| (entity, state.clone()))
            .collect::<Vec<_>>();
        let previous_time = self.scene.time();
        let signals = self.scene.animator_handle().signals();
        let overrides = signals.checkpoint();
        let values = {
            let world = self.scene.world();
            let mut values = signals.snapshot_values(&world);
            for (entity, animation) in world
                .query::<(hecs::Entity, &crate::core::components::Animation)>()
                .iter()
            {
                for track in &animation.tracks {
                    let target = &track.track.target;
                    values.push((entity, target.clone(), target.get(&world, entity)));
                }
            }
            values
        };
        let result = engine(|| self.scene.try_update(time))
            .and_then(|r| r.map_err(mlua::Error::runtime))
            .and_then(|()| {
                self.runtime
                    .errors
                    .borrow()
                    .first()
                    .map_or(Ok(()), |error| Err(mlua::Error::runtime(error.clone())))
            });
        if let Err(error) = result {
            let mut world = self.scene.world_mut();
            signals.restore_overrides(&world);
            for (entity, state) in saved {
                world
                    .insert_one(entity, state)
                    .map_err(mlua::Error::external)?;
            }
            for (entity, target, value) in values {
                target.set(&world, entity, value);
            }
            drop(world);
            signals.restore_checkpoint(overrides);
            self.scene.restore_time(previous_time);
            return Err(error);
        }
        Ok(())
    }
    pub(crate) fn update_slice(&self, time: f32) -> Result<bool> {
        self.runtime.errors.borrow_mut().clear();
        self.runtime
            .deadline
            .set(Instant::now() + Duration::from_millis(100));
        let complete = engine(|| {
            self.scene
                .update_slice(time, Some(Instant::now() + Duration::from_millis(8)))
        })?
        .map_err(mlua::Error::runtime)?;
        if let Some(error) = self.runtime.errors.borrow().first() {
            return Err(mlua::Error::runtime(error.clone()));
        }
        Ok(complete)
    }

    /// Returns the entity of an explicit scene-local ID.
    pub fn entity(&self, id: &str) -> Option<hecs::Entity> {
        self.runtime.definitions.borrow().get(id).map(|d| d.entity)
    }
    pub(crate) fn prepare_event_duration(&self, index: usize, seconds: f32) -> Result<Self> {
        let seconds = duration(seconds)?;
        let mut file = self.scene.event_definitions();
        let event = self
            .scene
            .events()
            .get(index)
            .ok_or_else(|| mlua::Error::runtime("Event no longer exists."))?;
        file.events[event.file_index].duration = seconds;
        let path = event_path(&self.path);
        let full = self
            .runtime
            .context
            .resolve(&path)
            .map_err(mlua::Error::external)?;
        let expected = std::fs::read(&full).ok();
        let text = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default())
            .map_err(mlua::Error::external)?;
        let mut sources = self.runtime.sources.clone();
        sources.insert(path, text.clone());
        let mut candidate = self.prepare_reload_pending(sources)?;
        candidate.pending_write = Some((full, text.into_bytes(), expected));
        Ok(candidate)
    }
    pub(crate) fn inspector_values(
        &self,
        entity: hecs::Entity,
    ) -> Option<(
        bool,
        Vec<(TrackTarget, TrackValue, Option<TrackValue>, TrackValue)>,
    )> {
        let id = self
            .runtime
            .definitions
            .borrow()
            .iter()
            .find(|(_, d)| d.entity == entity)
            .map(|(id, _)| id.clone())?;
        let scripts = self.runtime.script_values.borrow();
        let scripts = scripts.get(&entity)?;
        let world = self.scene.world();
        let fields = native::properties(&world, entity)
            .into_iter()
            .filter_map(|target| {
                let script = scripts.get(target.name())?.clone();
                let saved = self
                    .runtime
                    .overrides
                    .values
                    .get(&self.path)
                    .and_then(|s| s.get(&id))
                    .and_then(|s| s.get(target.name()))
                    .and_then(|v| v.restore(&target, &script).ok());
                let evaluated = target.get(&world, entity);
                Some((target, script, saved, evaluated))
            })
            .collect();
        Some((!id.starts_with('@'), fields))
    }
    pub(crate) fn prepare_override(
        &self,
        entity: hecs::Entity,
        target: &TrackTarget,
        value: Option<TrackValue>,
    ) -> Result<Option<Self>> {
        let id = self
            .runtime
            .definitions
            .borrow()
            .iter()
            .find(|(_, d)| d.entity == entity)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| mlua::Error::runtime("Object no longer exists."))?;
        if id.starts_with('@') {
            if let Some(value) = value {
                target.set(&self.scene.world(), entity, value);
                self.scene.invalidate();
            }
            return Ok(None);
        }
        let mut overrides = super::overrides::Overrides::read(&self.runtime.context)
            .map_err(mlua::Error::runtime)?;
        let properties = overrides
            .values
            .entry(self.path.clone())
            .or_default()
            .entry(id)
            .or_default();
        if let Some(value) = value {
            properties.insert(
                target.name().to_owned(),
                super::overrides::SavedValue::capture(&value),
            );
        } else {
            properties.remove(target.name());
        }
        let mut candidate = self.prepare_configured(
            self.runtime.sources.clone(),
            self.resolution,
            self.fps,
            self.margin,
            Some(overrides.clone()),
        )?;
        let bytes =
            ron::ser::to_string_pretty(&overrides.values, ron::ser::PrettyConfig::default())
                .map_err(mlua::Error::external)?
                .into_bytes();
        candidate.pending_write = Some((
            self.runtime
                .context
                .resolve(super::overrides::PATH)
                .map_err(mlua::Error::external)?,
            bytes,
            overrides.saved,
        ));
        Ok(Some(candidate))
    }

    pub(crate) fn persist_prepared(&mut self) -> Result<()> {
        if let Some((path, bytes, expected)) = &self.pending_write {
            crate::core::atomic_write(path, bytes, expected.as_deref())
                .map_err(mlua::Error::external)?;
        }
        self.pending_write = None;
        Ok(())
    }
    pub(crate) fn warnings(&self) -> Vec<String> {
        self.runtime.warnings.borrow().clone()
    }
    /// Returns the compiled timeline duration.
    pub fn duration(&self) -> f32 {
        self.scene.duration()
    }
    /// Returns the active source revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Drains bounded script print output.
    pub fn console(&self) -> Vec<String> {
        self.runtime.console.take()
    }
    /// Lists modules loaded by this scene, relative to the project root.
    pub fn dependencies(&self) -> BTreeSet<PathBuf> {
        self.runtime.dependencies.borrow().clone()
    }

    /// Prepares and validates a revision, then reconciles entities at the current time.
    pub fn reload(&mut self, sources: Sources) -> Result<()> {
        let candidate = self.prepare_reload(sources)?;
        self.apply_reload(candidate);
        Ok(())
    }

    pub(crate) fn prepare_seek(&self) -> Result<Self> {
        let mut sources = self.runtime.sources.clone();
        sources.insert(
            event_path(&self.path),
            ron::to_string(&self.scene.event_definitions()).map_err(mlua::Error::external)?,
        );
        self.prepare_configured(
            sources,
            self.resolution,
            self.fps,
            self.margin,
            Some(self.runtime.overrides.clone()),
        )
    }

    pub(crate) fn has_simulations(&self) -> bool {
        self.runtime
            .callbacks
            .borrow()
            .iter()
            .any(|(entity, callback)| {
                callback.update.is_some()
                    || self.runtime.operation_keys.borrow().contains_key(entity)
            })
    }

    pub(crate) fn prepare_reload(&self, sources: Sources) -> Result<Self> {
        let candidate = self.prepare_reload_pending(sources)?;
        candidate.update(self.scene.time().min(candidate.duration()))?;
        Ok(candidate)
    }
    pub(crate) fn prepare_reload_pending(&self, sources: Sources) -> Result<Self> {
        self.prepare_configured(sources, self.resolution, self.fps, self.margin, None)
    }

    pub(crate) fn prepare_settings_pending(
        &self,
        sources: Sources,
        resolution: (u32, u32),
        fps: u32,
        margin: (u32, u32),
    ) -> Result<Self> {
        self.prepare_configured(sources, resolution, fps, margin, None)
    }

    fn prepare_configured(
        &self,
        sources: Sources,
        resolution: (u32, u32),
        fps: u32,
        margin: (u32, u32),
        overrides: Option<super::overrides::Overrides>,
    ) -> Result<Self> {
        let previous = self.runtime.definitions.borrow().clone();
        let allocated = self.scene.world().iter().map(|e| e.entity()).collect();
        let candidate = Self::prepare(
            self.runtime.context.clone(),
            self.path.clone(),
            sources,
            resolution,
            fps,
            margin,
            previous,
            allocated,
            Some(self.scene.render_key().0),
            overrides,
            self.runtime.extensions.clone(),
        )?;
        {
            let previous = self.scene.world();
            let mut next = candidate.scene.world_mut();
            for entity in previous.iter().map(|entity| entity.entity()) {
                if next.contains(entity) {
                    crate::core::objects::reuse_image_texture(&previous, &mut next, entity);
                }
            }
        }
        for entity in candidate.runtime.callbacks.borrow().keys() {
            let old_key = self.runtime.simulation_key(*entity);
            let new_key = candidate.runtime.simulation_key(*entity);
            if old_key.is_some() && old_key == new_key {
                if let Ok(simulation) = self
                    .scene
                    .world()
                    .get::<&crate::core::components::Simulation>(*entity)
                {
                    let world = candidate.scene.world();
                    let mut next = world
                        .get::<&mut crate::core::components::Simulation>(*entity)
                        .map_err(mlua::Error::external)?;
                    next.copy_history(&simulation);
                    next.map_states::<extension::ScriptState>(|state| {
                        state.runtime = candidate.runtime.id
                    });
                }
            }
        }
        candidate.scene.revision_after(&self.scene);
        Ok(candidate)
    }

    pub(crate) fn apply_reload(&mut self, mut candidate: Self) {
        let previous = self.runtime.definitions.borrow().clone();
        let next = candidate.runtime.definitions.borrow().clone();
        let mut changed = next
            .iter()
            .filter(|(id, definition)| previous.get(*id) != Some(*definition))
            .map(|(_, d)| d.entity)
            .collect::<BTreeSet<_>>();
        for entity in candidate.runtime.callbacks.borrow().keys() {
            changed.insert(*entity);
        }
        let removed = previous
            .values()
            .filter(|old| !next.values().any(|new| new.entity == old.entity))
            .map(|d| d.entity)
            .collect::<Vec<_>>();
        self.scene
            .reconcile(&mut candidate.scene, &changed, &removed);
        candidate.runtime.world.replace(self.scene.world_handle());
        self.resolution = candidate.resolution;
        self.fps = candidate.fps;
        self.margin = candidate.margin;
        self.runtime = candidate.runtime;
        self.revision += 1;
        drop(candidate.scene);
        super::release_unused();
    }
}

fn fingerprint(world: &hecs::World, entity: hecs::Entity) -> String {
    let properties = native::properties(world, entity)
        .into_iter()
        .map(|target| (target.name().to_owned(), target.get(world, entity)))
        .collect::<Vec<_>>();
    let node = world.get::<&TreeNode>(entity).ok();
    let tracks = world
        .get::<&crate::core::components::Animation>(entity)
        .ok();
    let resources = world
        .get::<&super::resources::ResourceStamp>(entity)
        .ok()
        .map(|stamp| stamp.0.clone());
    format!(
        "{properties:?}|{node:?}|{resources:?}|{:?}",
        tracks
            .as_ref()
            .map(|a| a.tracks.iter().map(|t| &t.track).collect::<Vec<_>>())
    )
}

impl Runtime {
    fn spawn_native(
        self: &Rc<Self>,
        lua: &Lua,
        native: native::NativeObject,
        options: Table,
    ) -> Result<ObjectHandle> {
        self.construction()?;
        let (key, entity) = self.allocate(&options, native.name)?;
        engine(|| (native.spawn)(self.scene.borrow_mut().as_mut().unwrap(), entity))?;
        let handle = ObjectHandle {
            runtime: Rc::downgrade(self),
            entity,
        };
        super::resources::configure(lua, self, &handle, &options)?;
        for pair in options.pairs::<String, Value>() {
            let (name, value) = pair?;
            if name == "id" || super::resources::reserved(&name) {
                continue;
            }
            if name == "name" {
                self.world
                    .borrow()
                    .borrow()
                    .get::<&mut Name>(entity)
                    .unwrap()
                    .set(&String::from_lua(value, lua)?);
            } else {
                handle.property(&name)?.set(lua, value)?;
            }
        }
        self.apply_base(&key, entity)?;
        crate::core::objects::initialize_object_snapshots(&self.world.borrow(), entity);
        self.definitions.borrow_mut().insert(
            key,
            Definition {
                kind: native.name.to_owned(),
                entity,
                fingerprint: String::new(),
            },
        );
        Ok(handle)
    }

    pub(super) fn resource(&self, path: &Path) -> Result<Vec<u8>> {
        self.dependencies.borrow_mut().insert(path.to_owned());
        self.sources
            .read(&self.context, path)
            .map_err(mlua::Error::external)
    }

    pub(super) fn source(&self, path: &Path) -> Result<String> {
        String::from_utf8(self.resource(path)?).map_err(mlua::Error::external)
    }
    pub(super) fn construction(&self) -> Result<()> {
        if self.building.get() && !self.drawing.get() {
            Ok(())
        } else {
            Err(mlua::Error::runtime(
                "This operation is only valid while building the scene.",
            ))
        }
    }
    fn register(self: &Rc<Self>) -> Result<()> {
        let lua = &self.lua;
        let api = lua.create_table()?;
        value::register(lua, &api)?;
        extension::register(lua, &api)?;
        super::drawing::register(lua, &api)?;
        super::resources::register(self, &api)?;
        api.set(
            "scene",
            lua.create_function(|lua, (name, build): (String, Function)| {
                let result = lua.create_table()?;
                result.set("name", name)?;
                result.set("build", build)?;
                Ok(result)
            })?,
        )?;
        let easing = lua.create_table()?;
        for ease in Easing::ALL {
            easing.set(easing_name(ease), format!("{ease:?}"))?;
        }
        api.set("ease", easing)?;
        let loaded = lua.create_table()?;
        loaded.set("kinematic", api)?;
        let weak = Rc::downgrade(self);
        let loading = Rc::new(RefCell::new(BTreeSet::new()));
        lua.globals().set(
            "require",
            lua.create_function(move |lua, name: String| {
                if let Value::Nil = loaded.get::<Value>(name.as_str())? {
                } else {
                    return loaded.get::<Value>(name.as_str());
                }
                let runtime = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
                if name.contains('/')
                    || name.contains('\\')
                    || name.starts_with('.')
                    || name.ends_with('.')
                {
                    return Err(mlua::Error::runtime("Invalid module name."));
                }
                if !loading.borrow_mut().insert(name.clone()) {
                    return Err(mlua::Error::runtime(format!(
                        "Cyclic module dependency: {name}."
                    )));
                }
                let module = name.replace('.', "/");
                let file = PathBuf::from(format!("{module}.lua"));
                let path = if runtime.sources.contains_key(&file)
                    || runtime
                        .context
                        .resolve(&file)
                        .is_ok_and(|path| path.is_file())
                {
                    file
                } else {
                    PathBuf::from(format!("{module}/init.lua"))
                };
                let result = runtime.source(&path).and_then(|text| {
                    lua.load(&text)
                        .set_name(format!("@{}", path.display()))
                        .eval::<Value>()
                });
                loading.borrow_mut().remove(&name);
                let result = result?;
                let result = if result.is_nil() {
                    Value::Boolean(true)
                } else {
                    result
                };
                loaded.set(name, result.clone())?;
                Ok(result)
            })?,
        )?;
        let weak = Rc::downgrade(self);
        let tostring: Function = lua.globals().get("tostring")?;
        lua.globals().set(
            "print",
            lua.create_function(move |_, args: mlua::Variadic<Value>| {
                let runtime = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
                let line = args
                    .into_iter()
                    .map(|value| tostring.call::<String>(value))
                    .collect::<Result<Vec<_>>>()?
                    .join("\t");
                let line: String = line.chars().take(8192).collect();
                if runtime.last_print.borrow().as_ref() == Some(&line) {
                    return Ok(());
                }
                runtime.last_print.replace(Some(line.clone()));
                let mut console = runtime.console.borrow_mut();
                if console.len() == 500 {
                    console.remove(0);
                }
                console.push(line);
                Ok(())
            })?,
        )?;
        extension::prepare_environment(lua)
    }
}

fn easing_name(ease: Easing) -> String {
    let name = format!("{ease:?}");
    if let Some(rest) = name.strip_prefix("InOut") {
        format!("{}_in_out", rest.to_lowercase())
    } else if let Some(rest) = name.strip_prefix("Out") {
        format!("{}_out", rest.to_lowercase())
    } else if let Some(rest) = name.strip_prefix("In") {
        format!("{}_in", rest.to_lowercase())
    } else {
        name.to_lowercase()
    }
}
pub(super) fn parse_easing(name: Option<String>) -> Result<Easing> {
    match name {
        None => Ok(Easing::default()),
        Some(name) => Easing::ALL
            .into_iter()
            .find(|ease| format!("{ease:?}") == name || easing_name(*ease) == name)
            .ok_or_else(|| mlua::Error::runtime("Unknown easing curve.")),
    }
}
pub(super) fn duration(value: f32) -> Result<f32> {
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(mlua::Error::runtime(
            "Duration must be finite and nonnegative.",
        ))
    }
}

#[derive(Clone)]
struct SceneHandle(Weak<Runtime>);
impl SceneHandle {
    fn runtime(&self) -> Result<Rc<Runtime>> {
        self.0
            .upgrade()
            .ok_or_else(|| mlua::Error::runtime("Scene has been released."))
    }
}

impl UserData for SceneHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "track",
            |lua, this, (initial, id): (Value, Option<String>)| {
                let runtime = this.runtime()?;
                runtime.construction()?;
                let options = lua.create_table()?;
                if let Some(id) = id {
                    options.set("id", id)?;
                }
                let (key, entity) = runtime.allocate(&options, "StandaloneTrack")?;
                let value = extension::infer(initial)?;
                let (entity, target) = engine(|| {
                    crate::core::standalone_value(
                        &runtime.world.borrow(),
                        &runtime.animator.borrow(),
                        value,
                        Some(entity),
                    )
                })?;
                runtime.definitions.borrow_mut().insert(
                    key,
                    Definition {
                        entity,
                        kind: "StandaloneTrack".into(),
                        fingerprint: String::new(),
                    },
                );
                Ok(PropertyHandle {
                    object: ObjectHandle {
                        runtime: this.0.clone(),
                        entity,
                    },
                    target,
                    channel: None,
                })
            },
        );
        methods.add_method("object", |lua, this, (definition, options): (mlua::UserDataRef<extension::ObjectDefinition>, Table)| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            let kind = if definition.dimension == 2 { "Lua2D" } else { "Lua3D" };
            let (key, entity) = runtime.allocate(&options, kind)?;
            let world = runtime.world.borrow().clone();
            let animator = runtime.animator.borrow().clone();
            engine(|| {
                if definition.dimension == 2 {
                    extension::Script2D::spawn_at(world.clone(), animator, extension::Script2D::default(), Name::new(kind), Some(entity));
                } else {
                    extension::Script3D::spawn_at(world.clone(), animator, extension::Script3D::default(), Name::new(kind), Some(entity));
                }
            })?;
            for name in definition.properties.0.keys() {
                if native::properties(&world.borrow(), entity).iter().any(|target| target.name() == name) {
                    return Err(mlua::Error::runtime(format!("Custom property '{name}' conflicts with a native property.")));
                }
            }
            world.borrow_mut().insert_one(entity, definition.properties.clone()).map_err(mlua::Error::external)?;
            let handle = ObjectHandle { runtime: this.0.clone(), entity };
            super::resources::configure(lua, &runtime, &handle, &options)?;
            for pair in options.pairs::<String, Value>() {
                let (name, value) = pair?;
                if name == "id" || super::resources::reserved(&name) { continue; }
                if name == "name" {
                    world.borrow().get::<&mut Name>(entity).unwrap().set(&String::from_lua(value, lua)?);
                } else {
                    handle.property(&name)?.set(lua, value)?;
                }
            }
            extension::configure(&runtime, &handle, &definition)?;
            runtime.apply_base(&key, entity)?;
            crate::core::objects::initialize_object_snapshots(&world, entity);
            runtime.definitions.borrow_mut().insert(key, Definition { kind: kind.into(), entity, fingerprint: String::new() });
            Ok(handle)
        });
        for native in native::objects() {
            methods.add_method(native.name, move |lua, this, options: Table| {
                this.runtime()?.spawn_native(lua, native, options)
            });
        }
        methods.add_method("native", |lua, this, (name, options): (String, Table)| {
            let runtime = this.runtime()?;
            let native = runtime
                .extensions
                .iter()
                .find(|object| object.name == name)
                .copied()
                .ok_or_else(|| {
                    mlua::Error::runtime(format!("Native builder '{name}' is not registered."))
                })?;
            runtime.spawn_native(lua, native, options)
        });
        for (name, which) in [("root", 0), ("world_2d", 1), ("world_3d", 2)] {
            methods.add_method(name, move |_, this, ()| {
                let runtime = this.runtime()?;
                runtime.construction()?;
                let scene = runtime.scene.borrow();
                let scene = scene.as_ref().unwrap();
                let entity = match which {
                    0 => scene.root().entity(),
                    1 => scene.world_2d().entity(),
                    _ => scene.world_3d().entity(),
                };
                Ok(ObjectHandle {
                    runtime: this.0.clone(),
                    entity,
                })
            });
        }
        methods.add_method("wait", |_, this, seconds: f32| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            let seconds = duration(seconds)?;
            engine(|| runtime.scene.borrow_mut().as_mut().unwrap().wait(seconds))
        });
        methods.add_method("event", |_, this, name: String| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            engine(|| runtime.scene.borrow_mut().as_mut().unwrap().event(&name))
        });
        methods.add_method("play", |_, this, task: mlua::UserDataRef<TaskHandle>| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            engine(|| {
                runtime
                    .scene
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .play(task.0.clone())
            })
        });
        for name in ["chain", "all", "parallel", "loop"] {
            methods.add_method(name, move |_, this, callback: Function| {
                let runtime = this.runtime()?;
                runtime.construction()?;
                let repeating = name == "loop";
                let saved = if repeating {
                    let world = runtime.world.borrow();
                    let world = world.borrow();
                    world
                        .iter()
                        .flat_map(|entity| {
                            native::properties(&world, entity.entity())
                                .into_iter()
                                .map(|target| {
                                    (
                                        entity.entity(),
                                        target.clone(),
                                        target.get(&world, entity.entity()),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                let scope = engine(|| {
                    runtime
                        .scene
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .begin_schedule(name == "all", repeating)
                })?;
                let result = callback.call::<()>(this.clone());
                let close = engine(|| {
                    runtime.scene.borrow_mut().as_mut().unwrap().end_schedule(
                        scope,
                        repeating,
                        name == "parallel",
                    )
                });
                for (entity, target, value) in saved {
                    target.set(&runtime.world.borrow().borrow(), entity, value);
                }
                result?;
                close
            });
        }
    }
}

#[derive(Clone)]
pub(super) struct ObjectHandle {
    pub runtime: Weak<Runtime>,
    pub entity: hecs::Entity,
}
impl ObjectHandle {
    pub(super) fn runtime(&self) -> Result<Rc<Runtime>> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
        if runtime
            .world
            .borrow()
            .borrow()
            .get::<&crate::core::components::Inspection>(self.entity)
            .is_err()
        {
            return Err(mlua::Error::runtime(
                "Object handle no longer belongs to this scene revision.",
            ));
        }
        Ok(runtime)
    }
    pub(super) fn property(&self, name: &str) -> Result<PropertyHandle> {
        let runtime = self.runtime()?;
        let target = native::property(&runtime.world.borrow().borrow(), self.entity, name)
            .ok_or_else(|| mlua::Error::runtime(format!("Unknown object property '{name}'.")))?;
        Ok(PropertyHandle {
            object: self.clone(),
            target,
            channel: None,
        })
    }
}
impl UserData for ObjectHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        super::adapters::object_methods(methods);
        methods.add_meta_method(MetaMethod::Index, |_, this, name: String| {
            this.property(&name)
        });
        methods.add_method("add", |_, this, child: mlua::UserDataRef<Self>| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            if !Weak::ptr_eq(&this.runtime, &child.runtime) {
                return Err(mlua::Error::runtime("Objects belong to different scenes."));
            }
            child.runtime()?;
            engine(|| {
                runtime.animator.borrow().assert_finite_scope();
                attach_child(
                    &runtime.world.borrow(),
                    this.entity,
                    child.entity,
                    runtime.animator.borrow().time(),
                )
            })
        });
        methods.add_method("remove", |_, this, ()| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            engine(|| {
                runtime.animator.borrow().assert_finite_scope();
                deactivate_subtree(
                    &runtime.world.borrow().borrow(),
                    this.entity,
                    runtime.animator.borrow().time(),
                )
            })
        });
        methods.add_method("texture", |_, this, ()| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            crate::core::objects::validate_canvas(&runtime.world.borrow().borrow(), this.entity)
                .map_err(mlua::Error::runtime)?;
            Ok(super::resources::TextureHandle {
                texture: crate::core::objects::CanvasTexture {
                    scene: crate::core::objects::scene_identity(&runtime.world.borrow()),
                    entity: this.entity,
                },
                runtime: this.runtime.clone(),
            })
        });
        methods.add_method("uniform", |_, this, name: String| {
            let property = this.property(&name)?;
            if !matches!(property.target, TrackTarget::Uniform(_)) {
                return Err(mlua::Error::runtime("Property is not a shader uniform."));
            }
            Ok(property)
        });
        methods.add_method("inputs", |_, this, inputs: Table| {
            extension::configure_inputs(this, inputs)
        });
        methods.add_method("state", |lua, this, ()| {
            let runtime = this.runtime()?;
            let world = runtime.world.borrow();
            let world = world.borrow();
            world
                .get::<&crate::core::components::Simulation>(this.entity)
                .map_err(mlua::Error::external)?
                .read_state::<extension::ScriptState, _>(|state| state.data.value(lua))
        });
        for (name, checkpoint) in [("update", false), ("checkpoint", true)] {
            methods.add_method(name, move |_, this, ()| {
                let runtime = this.runtime()?;
                runtime.construction()?;
                runtime.animator.borrow().assert_finite_scope();
                let world = runtime.world.borrow();
                let world = world.borrow();
                let mut simulation = world
                    .get::<&mut crate::core::components::Simulation>(this.entity)
                    .map_err(mlua::Error::external)?;
                let time = runtime.animator.borrow().time();
                if checkpoint {
                    simulation.schedule_checkpoint(time);
                } else {
                    simulation.schedule_update(time);
                }
                runtime
                    .operation_keys
                    .borrow_mut()
                    .entry(this.entity)
                    .or_default()
                    .push(format!("{name}:{time}"));
                Ok(())
            });
        }
        methods.add_method("mutate", |_, this, callback: Function| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            extension::validate_update(&runtime.lua, &callback)?;
            let key = runtime.callback_source(&callback)?;
            let callback_id = runtime.mutations.borrow().len();
            runtime.mutations.borrow_mut().push(callback);
            runtime
                .operation_keys
                .borrow_mut()
                .entry(this.entity)
                .or_default()
                .push(format!("{}:{key}", runtime.animator.borrow().time()));
            let entity = this.entity;
            runtime
                .world
                .borrow()
                .borrow()
                .get::<&mut crate::core::components::Simulation>(entity)
                .map_err(mlua::Error::external)?
                .schedule_fallible_mutation::<extension::ScriptState>(
                    runtime.animator.borrow().time(),
                    move |state| {
                        let result: Result<()> = (|| {
                            let runtime = extension::runtime(state.runtime)?;
                            let callback = runtime.mutations.borrow()[callback_id].clone();
                            let value = state.data.value(&runtime.lua)?;
                            callback.call::<()>(value.clone())?;
                            state.data = super::snapshot::Snapshot::read(value)?;
                            Ok(())
                        })();
                        result.map_err(|error| {
                            extension::runtime(state.runtime).map_or_else(
                                |_| format!("Mutation {entity:?}: {error}"),
                                |runtime| runtime.diagnostic("Mutation", entity, &error),
                            )
                        })
                    },
                );
            Ok(())
        });
        methods.add_method("signal", |_, this, callback: Function| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            let object = this.clone();
            let weak = this.runtime.clone();
            let signal = engine(|| {
                runtime.animator.borrow().signal(this.entity, move |frame| {
                    let Some(runtime) = weak.upgrade() else {
                        return;
                    };
                    runtime
                        .deadline
                        .set(Instant::now() + Duration::from_millis(100));
                    runtime.writes.replace(Some(Vec::new()));
                    let result = (|| {
                        let info = runtime.lua.create_table()?;
                        info.set("time", frame.time)?;
                        info.set("index", frame.index)?;
                        info.set("dt", frame.dt)?;
                        callback.call::<()>((object.clone(), info))
                    })();
                    let writes = runtime.writes.take().unwrap_or_default();
                    match result {
                        Ok(()) => {
                            let world = runtime.world.borrow();
                            let world = world.borrow();
                            for (entity, target, value) in writes {
                                let old = target.get(&world, entity);
                                runtime.animator.borrow().signals().record_target_override(
                                    entity,
                                    target.clone(),
                                    old,
                                );
                                target.set(&world, entity, value);
                            }
                        }
                        Err(error) => runtime.errors.borrow_mut().push(runtime.diagnostic(
                            "Signal",
                            object.entity,
                            error,
                        )),
                    }
                })
            })?;
            Ok(SignalHandle(signal))
        });
    }
}
struct SignalHandle(crate::core::SignalHandle);
impl UserData for SignalHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("stop", |_, this, ()| engine(|| this.0.stop()));
    }
}

#[derive(Clone)]
pub(super) struct PropertyHandle {
    pub object: ObjectHandle,
    pub target: TrackTarget,
    pub channel: Option<u8>,
}
impl PropertyHandle {
    fn value(&self) -> Result<TrackValue> {
        let runtime = self.object.runtime()?;
        if let Some(writes) = runtime.writes.borrow().as_ref() {
            if let Some((_, _, value)) = writes.iter().rev().find(|(entity, target, _)| {
                *entity == self.object.entity && target.same(&self.target)
            }) {
                return Ok(value.clone());
            }
        }
        let value = self
            .target
            .get(&runtime.world.borrow().borrow(), self.object.entity);
        Ok(value)
    }
    fn convert(&self, lua: &Lua, value: Value) -> Result<TrackValue> {
        self.parse_value(lua, value, true)
    }
    fn parse_value(&self, lua: &Lua, value: Value, validate: bool) -> Result<TrackValue> {
        let expected = self.value()?;
        if let Some(channel) = self.channel {
            let number = f32::from_lua(value, lua)?;
            if !number.is_finite() {
                return Err(mlua::Error::runtime("Property values must be finite."));
            }
            let mut result = expected.clone();
            match &mut result {
                TrackValue::Vector2(v) => v[channel as usize] = number,
                TrackValue::Vector3(v) => v[channel as usize] = number,
                TrackValue::Color(v) => {
                    let mut rgba = v.rgba();
                    rgba[channel as usize] = number;
                    *v = rgba.into();
                }
                TrackValue::Quad(v) => {
                    let mut channels = [v.a, v.b, v.c, v.d];
                    channels[channel as usize] = number;
                    *v = channels.into();
                }
                _ => return Err(mlua::Error::runtime("Unsupported subchannel.")),
            };
            return Ok(result);
        }
        let choices = match &self.target {
            TrackTarget::Property { info, .. } => info.choices,
            _ => TrackChoices::None,
        };
        let value = value::from_lua(lua, value, &expected, choices)?;
        if !validate {
            return Ok(value);
        }
        if let TrackTarget::Dynamic(schema) = &self.target {
            schema.validate(&value).map_err(mlua::Error::runtime)?;
        } else if self.target.clamp(value.clone()) != value {
            return Err(mlua::Error::runtime(
                "Property is outside its allowed range.",
            ));
        }
        Ok(value)
    }
    fn set(&self, lua: &Lua, value: Value) -> Result<()> {
        let runtime = self.object.runtime()?;
        let value = self.convert(lua, value)?;
        if let Some(writes) = runtime.writes.borrow_mut().as_mut() {
            writes.push((self.object.entity, self.target.clone(), value));
            return Ok(());
        }
        runtime.construction()?;
        self.target
            .set(&runtime.world.borrow().borrow(), self.object.entity, value);
        Ok(())
    }
}
use mlua::FromLua;
impl UserData for PropertyHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("signal", |lua, this, callback: Function| {
            use mlua::ObjectLike;
            let property = this.clone();
            let callback = lua.create_function(move |_, (_, frame): (Value, Table)| {
                callback.call::<()>((property.clone(), frame))
            })?;
            lua.create_userdata(this.object.clone())?
                .call_method::<Value>("signal", callback)
        });
        methods.add_method("get", |lua, this, ()| {
            let value = this.value()?;
            if let Some(channel) = this.channel {
                return Ok(Value::Number(value.channel(channel) as f64));
            }
            value::to_lua(lua, value)
        });
        methods.add_method("set", |lua, this, value: Value| this.set(lua, value));
        methods.add_meta_method(MetaMethod::Index, |_, this, name: String| {
            let channel = match name.as_str() {
                "x" | "r" => 0,
                "y" | "g" => 1,
                "z" | "b" => 2,
                "w" | "a" => 3,
                _ => return Err(mlua::Error::runtime("Unknown property subchannel.")),
            };
            if this.value()?.channels() == 1 || this.value()?.channels() <= channel {
                return Err(mlua::Error::runtime("Property has no such subchannel."));
            }
            Ok(Self {
                channel: Some(channel),
                ..this.clone()
            })
        });
        methods.add_method(
            "rotate",
            |_,
             this,
             (axis, angle, seconds, ease): (
                mlua::UserDataRef<value::ValueHandle>,
                f32,
                Option<f32>,
                Option<String>,
            )| {
                let runtime = this.object.runtime()?;
                runtime.construction()?;
                let (TrackValue::Quaternion(from), TrackValue::Vector3(axis)) =
                    (this.value()?, &axis.0)
                else {
                    return Err(mlua::Error::runtime(
                        "Rotation needs a quaternion property and vec3 axis.",
                    ));
                };
                if !axis.is_finite() || axis.length_squared() <= f32::EPSILON || !angle.is_finite()
                {
                    return Err(mlua::Error::runtime(
                        "Rotation needs a nonzero finite axis and finite angle.",
                    ));
                }
                let axis = axis.normalize();
                let from = crate::core::normalized_quaternion(from);
                let to = TrackValue::Quaternion(crate::core::normalized_quaternion(
                    from * glam::Quat::from_axis_angle(axis, angle),
                ));
                let task = Task::RuntimeTween {
                    entity: this.object.entity,
                    target: this.target.clone(),
                    from: TrackValue::Quaternion(from),
                    to: to.clone(),
                    rotation: Some((axis, angle)),
                    duration: duration(seconds.unwrap_or(1.0))?,
                    easing: parse_easing(ease)?,
                    mask: 1,
                    relative: 0,
                    implicit: true,
                };
                this.target
                    .set(&runtime.world.borrow().borrow(), this.object.entity, to);
                Ok(TaskHandle(task))
            },
        );
        methods.add_method("from", |lua, this, (start, finish, seconds, ease): (Value, Value, Option<f32>, Option<String>)| {
            let runtime = this.object.runtime()?;
            runtime.construction()?;
            let from = this.convert(lua, start)?;
            let to = this.convert(lua, finish)?;
            let mask = this.channel.map_or((1 << from.channels()) - 1, |channel| 1 << channel);
            let task = Task::RuntimeTween {
                    rotation: None,
                entity: this.object.entity, target: this.target.clone(), from, to: to.clone(),
                duration: duration(seconds.unwrap_or(1.0))?, easing: parse_easing(ease)?,
                mask, relative: 0, implicit: false,
            };
            this.target.set(&runtime.world.borrow().borrow(), this.object.entity, to);
            Ok(TaskHandle(task))
        });
        for relative in [false, true] {
            methods.add_method(
                if relative { "by" } else { "to" },
                move |lua, this, (value, seconds, ease): (Value, Option<f32>, Option<String>)| {
                    let runtime = this.object.runtime()?;
                    runtime.construction()?;
                    let from = this.value()?;
                    let channel_delta = if relative && this.channel.is_some() {
                        Some(f32::from_lua(value.clone(), lua)?)
                    } else {
                        None
                    };
                    let mut to = if let (Some(channel), Some(delta)) = (this.channel, channel_delta)
                    {
                        this.convert(lua, Value::Number((from.channel(channel) + delta) as f64))?
                    } else {
                        this.parse_value(lua, value, !relative)?
                    };
                    if relative && this.channel.is_none() {
                        to = match (&from, &to) {
                            (TrackValue::F32(a), TrackValue::F32(b)) => TrackValue::F32(a + b),
                            (TrackValue::Vector2(a), TrackValue::Vector2(b)) => {
                                TrackValue::Vector2(*a + *b)
                            }
                            (TrackValue::Vector3(a), TrackValue::Vector3(b)) => {
                                TrackValue::Vector3(*a + *b)
                            }
                            _ => {
                                return Err(mlua::Error::runtime(
                                    "Relative tween requires a number or vector.",
                                ));
                            }
                        };
                    }
                    to.validate().map_err(mlua::Error::runtime)?;
                    if let TrackTarget::Dynamic(schema) = &this.target {
                        schema.validate(&to).map_err(mlua::Error::runtime)?;
                    } else if this.target.clamp(to.clone()) != to {
                        return Err(mlua::Error::runtime(
                            "Tween endpoint is outside property limits.",
                        ));
                    }
                    let mask = this
                        .channel
                        .map_or(from.changed_channels(&to), |channel| 1 << channel);
                    let task = Task::RuntimeTween {
                        rotation: None,
                        entity: this.object.entity,
                        target: this.target.clone(),
                        from,
                        to: to.clone(),
                        duration: duration(seconds.unwrap_or(1.0))?,
                        easing: parse_easing(ease)?,
                        mask,
                        relative: if relative { mask } else { 0 },
                        implicit: true,
                    };
                    this.target
                        .set(&runtime.world.borrow().borrow(), this.object.entity, to);
                    Ok(TaskHandle(task))
                },
            );
        }
    }
}
#[derive(Clone)]
pub(super) struct TaskHandle(pub Task);
impl UserData for TaskHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("delay", |_, this, seconds: f32| {
            Ok(Self(this.0.clone().delay(duration(seconds)?)))
        });
    }
}

impl crate::core::SceneRuntime for Runtime {
    fn check(&self) -> std::result::Result<(), String> {
        self.errors.borrow().first().cloned().map_or(Ok(()), Err)
    }
}
impl Runtime {
    pub(super) fn draw_call<T>(&self, f: impl FnOnce() -> Result<T>) -> Result<T> {
        self.deadline
            .set(Instant::now() + Duration::from_millis(100));
        struct Restore<'a>(&'a Cell<bool>, bool);
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.0.set(self.1);
            }
        }
        let _restore = Restore(&self.drawing, self.drawing.replace(true));
        f()
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        extension::release_runtime(self.id);
        self.assets.borrow_mut().clear();
        super::resources::release_unused();
    }
}
impl Runtime {
    fn apply_base(&self, id: &str, entity: hecs::Entity) -> Result<()> {
        let world = self.world.borrow();
        let world = world.borrow();
        let targets = native::properties(&world, entity);
        self.script_values.borrow_mut().insert(
            entity,
            targets
                .iter()
                .map(|target| (target.name().to_owned(), target.get(&world, entity)))
                .collect(),
        );
        if let Some(overrides) = self
            .overrides
            .values
            .get(&self.path)
            .and_then(|s| s.get(id))
        {
            for (name, saved) in overrides {
                if let Some(target) = targets.iter().find(|t| t.name() == name) {
                    match saved.restore(target, &target.get(&world, entity)) {
                        Ok(value) => target.set(&world, entity, value),
                        Err(error) => self
                            .warnings
                            .borrow_mut()
                            .push(format!("Override {id}.{name}: {error}")),
                    }
                } else {
                    self.warnings
                        .borrow_mut()
                        .push(format!("Orphan override: {id}.{name}."));
                }
            }
        }
        Ok(())
    }
    fn allocate(&self, options: &Table, kind: &str) -> Result<(String, hecs::Entity)> {
        let id = options.get::<Option<String>>("id")?;
        let key = if let Some(id) = &id {
            if id.is_empty() || id.starts_with('@') {
                return Err(mlua::Error::runtime(
                    "Explicit IDs must be nonempty and cannot start with @.",
                ));
            }
            id.clone()
        } else {
            let n = self.anonymous.get();
            self.anonymous.set(n + 1);
            format!("@anonymous-{n}")
        };
        if self.definitions.borrow().contains_key(&key) {
            return Err(mlua::Error::runtime(format!(
                "Duplicate object ID '{key}'."
            )));
        }
        let entity = id
            .as_ref()
            .and_then(|id| self.previous.get(id))
            .filter(|d| d.kind == kind)
            .map(|d| d.entity)
            .unwrap_or_else(|| self.world.borrow().borrow_mut().spawn(()));
        Ok((key, entity))
    }
    pub(super) fn diagnostic(
        &self,
        phase: &str,
        entity: hecs::Entity,
        error: impl std::fmt::Display,
    ) -> String {
        let id = self
            .definitions
            .borrow()
            .iter()
            .find(|(_, definition)| definition.entity == entity)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| format!("{entity:?}"));
        format!("{phase} object '{id}': {error}")
    }

    pub(super) fn callback_source(&self, callback: &Function) -> Result<String> {
        let info = callback.info();
        let path = info
            .source
            .as_deref()
            .and_then(|s| s.strip_prefix('@'))
            .ok_or_else(|| {
                mlua::Error::runtime("Callbacks must be defined in a project source file.")
            })?;
        self.source(Path::new(path))?;
        super::callback::key(callback)
    }
    fn simulation_key(&self, entity: hecs::Entity) -> Option<String> {
        let callbacks = self.callbacks.borrow();
        let callback = callbacks.get(&entity)?;
        if let Some(key) = &callback.history_key {
            return Some(key.clone());
        }
        let world = self.world.borrow();
        let world = world.borrow();
        let auto_update = TrackTarget::Property {
            type_id: std::any::TypeId::of::<crate::core::components::Simulation>(),
            info: <crate::core::components::Simulation as crate::core::Trackable>::track(0),
        };
        let own_input = [("auto_update".to_owned(), entity, auto_update)];
        let inputs = callback
            .inputs
            .iter()
            .chain(own_input.iter())
            .map(|(name, entity, target)| {
                let animation = world
                    .get::<&crate::core::components::Animation>(*entity)
                    .ok();
                let tracks = animation.as_ref().map(|a| {
                    a.tracks
                        .iter()
                        .filter(|t| t.track.target.same_field(target))
                        .map(|t| format!("{:?}", t.track))
                        .collect::<Vec<_>>()
                });
                format!(
                    "{name}:{entity:?}:{:?}:{tracks:?}",
                    target.get(&world, *entity)
                )
            })
            .collect::<Vec<_>>();
        let node = world.get::<&TreeNode>(entity).ok();
        let states = callback
            .states
            .iter()
            .map(|(name, entity)| (name, self.simulation_key(*entity)))
            .collect::<Vec<_>>();
        Some(format!(
            "{}|{}|{inputs:?}|{states:?}|{:?}|{:?}",
            self.fps,
            callback.simulation_key,
            node.as_ref().map(|n| n.lifetime),
            self.operation_keys.borrow().get(&entity)
        ))
    }
}
#[cfg(test)]
mod extension_tests {
    use super::*;
    const SOURCE: &str = r##"
local k=require('kinematic')
local settings=k.component {speed={default=2,min=0,max=20}}
local counter=k.object {
    components={settings}, state={total=0}, inputs={speed='speed'},
    update=function(state,frame,inputs)
        state.total=state.total+inputs.speed*frame.dt
    end,
    bounds=function(props,state) return {10,20,30,40} end,
    draw=function(ctx,props,state)
        ctx:rect(10,20,30,40,props.fill)
    end,
}
return k.scene('Counter',function(s)
    local counter=s:object(counter,{id='counter',fill=k.color('#ff0000')})
    s:world_2d():add(counter)
    s:play(counter.speed:to(8,3,k.ease.linear))
    s:wait(1)
end)
"##;
    fn sources(text: &str) -> Sources {
        [(PathBuf::from("counter.lua"), text.to_owned())].into()
    }
    fn load(text: &str) -> LuaScene {
        LuaScene::load(
            ProjectContext::new(".").unwrap(),
            "counter.lua",
            sources(text),
            (100, 100),
            60,
            (0, 0),
        )
        .unwrap()
    }
    fn total(scene: &LuaScene) -> f64 {
        let world = scene.scene.world();
        world
            .get::<&crate::core::components::Simulation>(scene.entity("counter").unwrap())
            .unwrap()
            .read_state::<extension::ScriptState, _>(|state| {
                state
                    .data
                    .value(&scene.runtime.lua)
                    .unwrap()
                    .as_table()
                    .unwrap()
                    .get("total")
                    .unwrap()
            })
    }

    #[test]
    fn simulation_seek_visual_reload_and_logic_replay() {
        let mut scene = load(SOURCE);
        scene.update(3.0).unwrap();
        let expected = total(&scene);
        scene.update(0.5).unwrap();
        scene.update(3.0).unwrap();
        assert_eq!(total(&scene), expected);
        let entity = scene.entity("counter");
        scene
            .reload(sources(&SOURCE.replace("#ff0000", "#00ff00")))
            .unwrap();
        assert_eq!(scene.entity("counter"), entity);
        assert_eq!(total(&scene), expected);
        assert_eq!(
            scene.runtime.simulation_steps.get(),
            0,
            "Visual edits must reuse compatible history."
        );
        let source = SOURCE.replace("inputs.speed*frame.dt", "2*inputs.speed*frame.dt");
        scene.reload(sources(&source)).unwrap();
        let clean = load(&source);
        clean.update(3.0).unwrap();
        assert_eq!(total(&scene), total(&clean));
        assert!(scene.runtime.simulation_steps.get() > 0);
        let before = total(&scene);
        assert!(
            scene
                .reload(sources(&source.replace(
                    "state.total=state.total+2*inputs.speed*frame.dt",
                    "state.total=99; error('Update failed')"
                )))
                .is_err()
        );
        assert_eq!(total(&scene), before);
    }
    #[test]
    fn dependent_simulations_sample_each_step_and_replay_only_changed_dependencies() {
        let source = r#"
local k = require('kinematic')
return k.scene('Dependencies', function(s)
    local a = s:object(k.object{
        state={total=0},
        update=function(state, frame) state.total=state.total+1 end,
    }, {id='source'})
    local b = s:object(k.object{
        state={total=0}, inputs={source=a},
        update=function(state, frame, inputs) state.total=state.total+inputs.source.total end,
    }, {id='counter'})
    s:world_2d():add(a)
    s:world_2d():add(b)
    s:wait(4)
end)
"#;
        let mut scene = load(source);
        scene.update(3.0).unwrap();
        let expected = 181.0 * 182.0 / 2.0;
        assert_eq!(total(&scene), expected);
        scene.update(0.5).unwrap();
        scene.update(3.0).unwrap();
        assert_eq!(total(&scene), expected);
        scene
            .reload(sources(&format!("-- A comment.\n{source}")))
            .unwrap();
        assert_eq!(scene.runtime.simulation_steps.get(), 0);
        let changed = source.replace("state.total+1", "state.total+2");
        scene.reload(sources(&changed)).unwrap();
        let clean = load(&changed);
        clean.update(3.0).unwrap();
        assert_eq!(total(&scene), total(&clean));
        assert_eq!(total(&scene), expected * 2.0);
        let cycle = changed.replace("s:wait(4)", "a:inputs{source=b}; s:wait(4)");
        let error = scene.reload(sources(&cycle)).err().unwrap().to_string();
        assert!(error.contains("cycle"), "{error}");
        assert_eq!(total(&scene), expected * 2.0);
    }

    #[test]
    fn simulation_libraries_ignore_hidden_mutations_and_support_owned_values() {
        let source = r#"
local k=require('kinematic')
local hidden=0
math.sin=function() hidden=hidden+1; return hidden end
local counter=k.object{state={total=0},inputs={}, update=function(state)
    local values=require('kinematic')
    state.total=state.total+values.vec2(3,4):length()+math.sin(0)
end}
return k.scene('Pure',function(s)
    local a=s:object(counter,{id='counter'})
    s:world_2d():add(a)
    s:wait(1)
end)
"#;
        let scene = load(source);
        scene.update(1.0).unwrap();
        assert_eq!(total(&scene), 305.0);
        scene.update(0.0).unwrap();
        scene.update(1.0).unwrap();
        assert_eq!(total(&scene), 305.0);
    }

    #[test]
    fn life_and_manual_operations_reproduce_playback_seek_and_reload() {
        let source = include_str!("../../examples/lua/simulation/scenes/life.lua");
        let state = |scene: &LuaScene, id: &str| {
            scene
                .scene
                .world()
                .get::<&crate::core::components::Simulation>(scene.entity(id).unwrap())
                .unwrap()
                .read_state::<extension::ScriptState, _>(|state| format!("{:?}", state.data))
        };
        let mut scene = load(source);
        for frame in 0..=120 {
            scene.update(frame as f32 / 60.0).unwrap();
        }
        let expected = state(&scene, "life");
        scene.update(0.0).unwrap();
        scene.update(2.0).unwrap();
        assert_eq!(state(&scene, "life"), expected);
        scene
            .reload(sources(&format!("-- Visual edit.\n{source}")))
            .unwrap();
        assert_eq!(state(&scene, "life"), expected);
        assert_eq!(scene.runtime.simulation_steps.get(), 0);
        let changed = source.replace("% 6", "% 4");
        scene.reload(sources(&changed)).unwrap();
        let clean = load(&changed);
        clean.update(2.0).unwrap();
        assert_eq!(state(&scene, "life"), state(&clean, "life"));
        let manual = load(
            r#"
local k=require('kinematic')
local definition=k.object{state={total=0},inputs={},update=function(state) state.total=state.total+1 end}
return k.scene('Manual',function(s)
    local obj=s:object(definition,{id='counter',auto_update=false})
    s:world_2d():add(obj)
    s:wait(0.5)
    obj:mutate(function(state) state.total=state.total+5 end)
    obj:update()
    obj:checkpoint()
    s:wait(0.5)
end)
"#,
        );
        manual.update(1.0).unwrap();
        assert_eq!(total(&manual), 6.0);
        manual.update(0.0).unwrap();
        assert_eq!(total(&manual), 0.0);
        manual.update(1.0).unwrap();
        assert_eq!(total(&manual), 6.0);
    }

    #[test]
    fn saved_bases_precede_tweens_and_conflicts_leave_the_active_revision_untouched() {
        let root = std::env::temp_dir().join(format!("kinematic-override-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let context = ProjectContext::new(&root).unwrap();
        let source = r#"local k=require('kinematic'); return k.scene('Base',function(s)
            local a=s:circle{id='ball',radius=4}; s:world_2d():add(a)
            s:play(a.radius:to(a.radius:get()*2, 1, k.ease.linear))
        end)"#;
        let mut scene = LuaScene::load(
            context.clone(),
            "counter.lua",
            sources(source),
            (100, 100),
            60,
            (0, 0),
        )
        .unwrap();
        let ball = scene.entity("ball").unwrap();
        let target = scene
            .inspector_values(ball)
            .unwrap()
            .1
            .into_iter()
            .find(|(target, _, _, _)| target.name() == "radius")
            .unwrap()
            .0;
        let mut candidate = scene
            .prepare_override(ball, &target, Some(TrackValue::F32(10.0)))
            .unwrap()
            .unwrap();
        candidate.update(1.0).unwrap();
        assert!(!root.join(super::super::overrides::PATH).exists());
        candidate.persist_prepared().unwrap();
        scene.apply_reload(candidate);
        assert_eq!(
            target.get(&scene.scene.world(), ball),
            TrackValue::F32(20.0)
        );
        let reopened = LuaScene::load(
            context,
            "counter.lua",
            sources(source),
            (100, 100),
            60,
            (0, 0),
        )
        .unwrap();
        reopened.update(1.0).unwrap();
        assert_eq!(
            target.get(&reopened.scene.world(), reopened.entity("ball").unwrap()),
            TrackValue::F32(20.0)
        );
        let mut restore = scene
            .prepare_override(ball, &target, None)
            .unwrap()
            .unwrap();
        std::fs::write(root.join(super::super::overrides::PATH), "External change.").unwrap();
        assert!(restore.persist_prepared().is_err());
        assert_eq!(
            target.get(&scene.scene.world(), ball),
            TrackValue::F32(20.0)
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn custom_drawing_uses_displaced_bounds_and_commits_only_complete_callbacks() {
        let scene = load(SOURCE);
        let entity = scene.entity("counter").unwrap();
        let bounds = super::super::drawing::bounds_2d(&scene.scene.world(), entity);
        assert_eq!(bounds, skia_safe::Rect::from_xywh(10.0, 20.0, 30.0, 40.0));
        let mut surface = skia_safe::surfaces::raster_n32_premul((100, 100)).unwrap();
        scene.scene.draw(surface.canvas());
        assert_eq!(surface.peek_pixels().unwrap().get_color((15, 25)).r(), 255);
        let scene = load(&SOURCE.replace(
            "ctx:rect(10,20,30,40,props.fill)",
            "ctx:rect(10,20,30,40,props.fill); error('Draw failed')",
        ));
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        scene.scene.draw(surface.canvas());
        assert_eq!(surface.peek_pixels().unwrap().get_color((15, 25)).a(), 0);
        assert!(!scene.runtime.errors.borrow().is_empty());
    }
}

fn event_path(path: &Path) -> PathBuf {
    Path::new("kinematic/events")
        .join(path)
        .with_extension("ron")
}
