use super::{
    runtime::{ObjectHandle, PropertyHandle, Runtime},
    snapshot::{RandomHandle, Snapshot},
    value::{self, ValueHandle},
};
use crate::core::{
    Properties, PropertySchema, TrackLimits, TrackTarget, TrackValue, components::*,
};
use kinematic_macros::Object;
use mlua::{Function, Lua, Result, Table, UserData, Value};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

thread_local! { static RUNTIMES: RefCell<BTreeMap<u64,Weak<Runtime>>> = RefCell::new(BTreeMap::new()); }
static NEXT_RUNTIME: AtomicU64 = AtomicU64::new(1);

pub(super) fn register_runtime(runtime: &Rc<Runtime>) {
    RUNTIMES.with(|r| r.borrow_mut().insert(runtime.id, Rc::downgrade(runtime)));
}
pub(super) fn release_runtime(id: u64) {
    RUNTIMES.with(|r| r.borrow_mut().remove(&id));
}
pub(super) fn next_runtime() -> u64 {
    NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed)
}
pub(super) fn runtime(id: u64) -> Result<Rc<Runtime>> {
    RUNTIMES
        .with(|r| r.borrow().get(&id).and_then(Weak::upgrade))
        .ok_or_else(|| mlua::Error::runtime("Callback belongs to a released scene revision."))
}

#[derive(Clone)]
pub(super) struct Component(Vec<Arc<PropertySchema>>);
impl UserData for Component {}
#[derive(Clone)]
pub(super) struct ObjectDefinition {
    pub table: Table,
    pub properties: Properties,
    pub dimension: u8,
}
impl UserData for ObjectDefinition {}
#[derive(Clone)]
pub(super) struct Callbacks {
    pub draw: Option<Function>,
    pub bounds: Option<Function>,
    pub update: Option<Function>,
    pub inputs: Vec<(String, hecs::Entity, TrackTarget, Option<u8>)>,
    pub methods: Option<Table>,
    pub states: Vec<(String, hecs::Entity)>,
    pub simulation_key: String,
    pub visual_key: String,
    pub history_key: Option<String>,
}
#[derive(Clone, Debug)]
pub(super) struct ScriptState {
    pub runtime: u64,
    pub entity: hecs::Entity,
    pub data: Snapshot,
}
impl SimulationState for ScriptState {
    fn on_update(&mut self, context: &SimulationContext<'_>) {
        let _ = self.try_on_update(context);
    }

    fn try_on_update(
        &mut self,
        context: &SimulationContext<'_>,
    ) -> std::result::Result<(), String> {
        let result = (|| {
            let runtime = runtime(self.runtime)?;
            let callbacks = runtime.callbacks.borrow();
            let callbacks = callbacks
                .get(&self.entity)
                .ok_or_else(|| mlua::Error::runtime("Simulation callback is unavailable."))?;

            let Some(update) = &callbacks.update else {
                return Ok(());
            };

            let lua = &runtime.lua;
            let frame = lua.create_table_with_capacity(0, 3)?;

            frame.set("time", context.time)?;
            frame.set("index", context.frame)?;
            frame.set("dt", context.dt)?;

            let inputs =
                lua.create_table_with_capacity(0, callbacks.inputs.len() + callbacks.states.len())?;

            for (name, entity, target, channel) in &callbacks.inputs {
                inputs.set(name.as_str(), {
                    let sampled = context.sample(*entity, target);

                    if let Some(channel) = channel {
                        Value::Number(sampled.channel(*channel) as f64)
                    } else {
                        value::to_lua(lua, sampled)?
                    }
                })?;
            }

            for (name, entity) in &callbacks.states {
                let state = context
                    .state::<ScriptState>(*entity)
                    .map_err(mlua::Error::runtime)?;

                inputs.set(name.as_str(), state.data.value(lua)?)?;
            }

            let state = self.data.value(lua)?;
            let returned = update.call::<Value>((state.clone(), frame, inputs))?;
            let next = Snapshot::read(if returned.is_nil() { state } else { returned })?;

            self.data = next;

            runtime
                .simulation_steps
                .set(runtime.simulation_steps.get() + 1);

            Ok(())
        })();

        result.map_err(|error: mlua::Error| {
            runtime(self.runtime).map_or_else(
                |_| format!("Update {:?}: {error}", self.entity),
                |runtime| runtime.diagnostic("Update", self.entity, &error),
            )
        })
    }
}

/// Scripted 2D object with local drawing, tracked properties, and fixed-step simulation.
#[derive(Object, Default)]
#[object(spatial = "2d", builder = "script_2d")]
pub(super) struct Script2D {
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub filter: Filter,
    #[trackable]
    pub simulation: Simulation,
}
/// Scripted 3D object with mesh drawing, tracked properties, and fixed-step simulation.
#[derive(Object, Default)]
#[object(spatial = "3d", builder = "script_3d")]
pub(super) struct Script3D {
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
    #[trackable]
    pub material: Material,
    #[trackable]
    pub simulation: Simulation,
}
#[derive(Clone, Debug)]
pub(super) struct ScriptObject {
    pub runtime: u64,
    pub name: String,
}

pub(super) fn register(lua: &Lua, api: &Table) -> Result<()> {
    api.set(
        "random",
        lua.create_function(|_, seed: u64| Ok(RandomHandle(crate::core::Random::new(seed))))?,
    )?;

    api.set(
        "component",
        lua.create_function(|lua, fields: Table| {
            let mut properties = Vec::new();

            for field in fields.pairs::<String, Table>() {
                let (name, definition) = field?;

                if name.is_empty() {
                    return Err(mlua::Error::runtime("Property name cannot be empty."));
                }

                let default = infer(definition.get("default")?)?;
                let min = definition.get::<Option<f32>>("min")?;
                let max = definition.get::<Option<f32>>("max")?;

                if min.is_some_and(|v| !v.is_finite())
                    || max.is_some_and(|v| !v.is_finite())
                    || min.zip(max).is_some_and(|(a, b)| a > b)
                {
                    return Err(mlua::Error::runtime(format!(
                        "Invalid limits for '{name}'."
                    )));
                }

                if (min.is_some() || max.is_some()) && !matches!(default, TrackValue::F32(_)) {
                    return Err(mlua::Error::runtime(
                        "Numeric limits require a numeric property.",
                    ));
                }

                let choices = definition
                    .get::<Option<Vec<String>>>("choices")?
                    .unwrap_or_default();
                let schema = PropertySchema {
                    name,
                    default,
                    limits: TrackLimits::F32 { min, max },
                    choices,
                };

                schema
                    .validate(&schema.default)
                    .map_err(mlua::Error::runtime)?;

                properties.push(Arc::new(schema));
            }

            let _ = lua;

            Ok(Component(properties))
        })?,
    )?;

    api.set(
        "object",
        lua.create_function(|_, definition: Table| {
            if definition
                .get::<Option<String>>("name")?
                .is_some_and(|name| name.is_empty())
            {
                return Err(mlua::Error::runtime("Object type name cannot be empty."));
            }

            let dimension = definition.get::<Option<u8>>("dimension")?.unwrap_or(2);

            if dimension != 2 && dimension != 3 {
                return Err(mlua::Error::runtime("Object dimension must be 2 or 3."));
            }

            let mut properties = Properties::default();

            if let Some(components) = definition.get::<Option<Table>>("components")? {
                for component in components.sequence_values::<mlua::UserDataRef<Component>>() {
                    for schema in &component?.0 {
                        if properties
                            .0
                            .insert(
                                schema.name.clone(),
                                (schema.clone(), schema.default.clone()),
                            )
                            .is_some()
                        {
                            return Err(mlua::Error::runtime(format!(
                                "Duplicate property '{}'.",
                                schema.name
                            )));
                        }
                    }
                }
            }

            if let Some(methods) = definition.get::<Option<Table>>("methods")? {
                for entry in methods.pairs::<String, Function>() {
                    let (name, _) = entry?;

                    if super::runtime::OBJECT_METHODS.contains(&name.as_str())
                        || super::adapters::ANCHORS.iter().any(|(anchor, _, _, _)| {
                            name == *anchor || name == format!("{anchor}_to")
                        })
                    {
                        return Err(mlua::Error::runtime(format!(
                            "Cannot replace fundamental object method '{name}'."
                        )));
                    }
                }
            }

            Ok(ObjectDefinition {
                table: definition,
                properties,
                dimension,
            })
        })?,
    )?;

    Ok(())
}
pub(super) fn infer(value: Value) -> Result<TrackValue> {
    let value = match value {
        Value::Boolean(v) => TrackValue::Bool(v),
        Value::Integer(v) => TrackValue::F32(v as f32),
        Value::Number(v) => TrackValue::F32(v as f32),
        Value::String(v) => TrackValue::String(v.to_str()?.to_string()),
        Value::UserData(v) => v.borrow::<ValueHandle>()?.0.clone(),
        _ => {
            return Err(mlua::Error::runtime(
                "Property defaults must be scalar or Kinematic values.",
            ));
        }
    };

    value.validate().map_err(mlua::Error::runtime)?;

    Ok(value)
}

pub(super) fn configure(
    runtime: &Rc<Runtime>,
    handle: &ObjectHandle,
    definition: &ObjectDefinition,
) -> Result<()> {
    let entity = handle.entity;
    let update = definition.table.get::<Option<Function>>("update")?;

    if let Some(update) = &update {
        validate_update(&runtime.lua, update)?;
    }

    let state = Snapshot::read(definition.table.get("state")?)?;
    let draw = definition.table.get::<Option<Function>>("draw")?;
    let bounds = definition.table.get::<Option<Function>>("bounds")?;

    if draw.is_some() && bounds.is_none() {
        return Err(mlua::Error::runtime(
            "Custom drawing requires local bounds.",
        ));
    }

    let callback_key = |callback: &Option<Function>| {
        callback
            .as_ref()
            .map(|f| runtime.callback_source(f))
            .transpose()
            .map(|v| v.unwrap_or_default())
    };
    let simulation_key = format!("{}|{state:?}", callback_key(&update)?);
    let visual_key = format!("{}|{}", callback_key(&draw)?, callback_key(&bounds)?);
    let callbacks = Callbacks {
        draw,
        bounds,
        update,
        inputs: Vec::new(),
        methods: definition.table.get("methods")?,
        states: Vec::new(),
        simulation_key,
        visual_key,
        history_key: None,
    };

    runtime.callbacks.borrow_mut().insert(entity, callbacks);

    let world_handle = runtime.world.borrow();
    let mut world = world_handle.borrow_mut();

    world
        .insert_one(
            entity,
            ScriptObject {
                runtime: runtime.id,
                name: definition
                    .table
                    .get::<Option<String>>("name")?
                    .unwrap_or_else(|| {
                        if definition.dimension == 2 {
                            "Lua2D".into()
                        } else {
                            "Lua3D".into()
                        }
                    }),
            },
        )
        .map_err(mlua::Error::external)?;

    let auto_update = world
        .get::<&Simulation>(entity)
        .map_or(true, |s| s.auto_update);

    world
        .insert_one(
            entity,
            Simulation::new(ScriptState {
                runtime: runtime.id,
                entity,
                data: state,
            }),
        )
        .map_err(mlua::Error::external)?;

    world
        .get::<&mut Simulation>(entity)
        .map_err(mlua::Error::external)?
        .auto_update = auto_update;

    if definition.dimension == 2 {
        let mut draw = world.get::<&mut Draw2D>(entity).unwrap();

        draw.on_draw = super::drawing::draw_2d;

        draw.box_size = |w, e| {
            let b = super::drawing::bounds_2d(w, e);

            glam::vec2(b.width(), b.height())
        };

        draw.logical_bounds = Some(super::drawing::bounds_2d);
        draw.visual_bounds = super::drawing::bounds_2d;
    } else {
        let mut draw = world.get::<&mut Draw3D>(entity).unwrap();

        draw.on_draw = super::drawing::draw_3d;

        draw.box_size = |w, e| {
            let (min, max) = super::drawing::bounds_3d(w, e);

            max - min
        };

        draw.logical_bounds = Some(super::drawing::bounds_3d);
    }

    drop(world);

    drop(world_handle);

    let inputs = if let Some(inputs) = definition.table.get::<Option<Table>>("inputs")? {
        inputs
    } else {
        let inputs = runtime.lua.create_table()?;

        for target in super::native::properties(&runtime.world.borrow().borrow(), entity) {
            if target.uniform_name().is_some() {
                continue;
            }

            inputs.set(target.name(), handle.property(target.name())?)?;
        }

        inputs
    };

    configure_inputs(handle, inputs)?;

    Ok(())
}

pub(super) fn configure_inputs(handle: &ObjectHandle, inputs: Table) -> Result<()> {
    let runtime = handle.runtime()?;

    runtime.construction()?;

    let mut properties = Vec::new();
    let mut states = Vec::new();

    for pair in inputs.pairs::<String, Value>() {
        let (name, input) = pair?;

        if let Value::UserData(value) = &input {
            if let Ok(object) = value.borrow::<ObjectHandle>() {
                object.runtime()?;

                if !Weak::ptr_eq(&handle.runtime, &object.runtime)
                    || !runtime.callbacks.borrow().contains_key(&object.entity)
                {
                    return Err(mlua::Error::runtime(
                        "State inputs require a Lua simulation in the same scene.",
                    ));
                }

                states.push((name, object.entity));

                continue;
            }
        }

        let property = match input {
            Value::String(s) => handle.property(s.to_str()?.as_ref())?,
            Value::UserData(v) => v.borrow::<PropertyHandle>()?.clone(),
            _ => {
                return Err(mlua::Error::runtime(
                    "Inputs require properties or simulation objects.",
                ));
            }
        };

        if !Weak::ptr_eq(&handle.runtime, &property.object.runtime) {
            return Err(mlua::Error::runtime(
                "Inputs must belong to the same scene.",
            ));
        }

        properties.push((
            name,
            property.object.entity,
            property.target,
            property.channel,
        ));
    }

    properties.sort_by(|a, b| a.0.cmp(&b.0));
    states.sort_by(|a, b| a.0.cmp(&b.0));

    let mut callbacks = runtime.callbacks.borrow_mut();
    let callback = callbacks
        .get_mut(&handle.entity)
        .ok_or_else(|| mlua::Error::runtime("Inputs require a Lua simulation."))?;

    runtime
        .world
        .borrow()
        .borrow()
        .get::<&mut Simulation>(handle.entity)
        .map_err(mlua::Error::external)?
        .dependencies = states.iter().map(|(_, entity)| *entity).collect();

    callback.inputs = properties;
    callback.states = states;

    Ok(())
}

pub(super) fn validate_dependencies(runtime: &Runtime) -> Result<()> {
    fn visit(
        entity: hecs::Entity,
        callbacks: &BTreeMap<hecs::Entity, Callbacks>,
        visiting: &mut Vec<hecs::Entity>,
        done: &mut Vec<hecs::Entity>,
    ) -> Result<()> {
        if done.contains(&entity) {
            return Ok(());
        }

        if visiting.contains(&entity) {
            return Err(mlua::Error::runtime(format!(
                "Simulation dependency cycle at {entity:?}."
            )));
        }

        visiting.push(entity);

        for (_, dependency) in &callbacks[&entity].states {
            visit(*dependency, callbacks, visiting, done)?;
        }

        visiting.pop();
        done.push(entity);

        Ok(())
    }

    let callbacks = runtime.callbacks.borrow();
    let mut done = Vec::new();

    for entity in callbacks.keys() {
        visit(*entity, &callbacks, &mut Vec::new(), &mut done)?;
    }

    Ok(())
}

pub(super) fn validate_update(lua: &Lua, update: &Function) -> Result<()> {
    let info = update.info();
    let has_env = update.environment().is_some();

    if info.num_upvalues > u8::from(has_env) {
        return Err(mlua::Error::runtime(
            "Update cannot capture mutable upvalues. Put state in the state table and declare property inputs.",
        ));
    }

    update.set_environment(lua.named_registry_value::<Table>("kinematic.update_environment")?)?;

    Ok(())
}

pub(super) fn prepare_environment(runtime: &Rc<Runtime>) -> Result<()> {
    let lua = &runtime.lua;
    let globals = lua.globals();
    let env = lua.create_table()?;

    for name in [
        "assert", "error", "ipairs", "pairs", "next", "select", "tonumber", "tostring", "type",
        "print",
    ] {
        env.set(name, globals.get::<Value>(name)?)?;
    }

    let values = lua.create_table()?;

    value::register(lua, &values)?;
    super::drawing::register(lua, &values)?;

    values.set(
        "random",
        lua.create_function(|_, seed: u64| Ok(RandomHandle(crate::core::Random::new(seed))))?,
    )?;

    let module = readonly(lua, values)?;

    env.set("k", module.clone())?;

    let weak = Rc::downgrade(runtime);
    let loaded = lua.create_table()?;
    let loading = Rc::new(std::cell::RefCell::new(std::collections::BTreeSet::new()));

    env.set(
        "require",
        lua.create_function(move |lua, name: String| {
            if name == "kinematic" {
                return Ok(Value::Table(module.clone()));
            }

            if !loaded.get::<Value>(name.as_str())?.is_nil() {
                return loaded.get::<Value>(name.as_str());
            }

            if name.split('.').any(|part| {
                part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            }) {
                return Err(mlua::Error::runtime("Invalid simulation module name."));
            }

            if !loading.borrow_mut().insert(name.clone()) {
                return Err(mlua::Error::runtime("Cyclic simulation module dependency."));
            }

            let result: Result<Value> = (|| {
                let runtime = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::runtime("Scene has been released."))?;
                let module = name.replace('.', "/");
                let file = std::path::PathBuf::from(format!("{module}.lua"));
                let path = if runtime.sources.contains_key(&file)
                    || runtime.context.resolve(&file).is_ok_and(|p| p.is_file())
                {
                    file
                } else {
                    std::path::PathBuf::from(format!("{module}/init.lua"))
                };
                let source = runtime.source(&path)?;
                let value = lua
                    .load(&source)
                    .set_name(format!("@{}", path.display()))
                    .set_environment(
                        lua.named_registry_value::<Table>("kinematic.update_environment")?,
                    )
                    .eval::<Value>()?;
                let value =
                    immutable_module(lua, value, &mut std::collections::BTreeSet::new(), 0)?;

                runtime
                    .simulation_modules
                    .borrow_mut()
                    .insert(name.clone(), source);

                Ok(value)
            })();

            loading.borrow_mut().remove(&name);

            let value = result?;

            loaded.set(name, value.clone())?;

            Ok(value)
        })?,
    )?;

    for name in ["math", "string", "table", "utf8"] {
        let table = lua.create_table()?;

        for pair in globals.get::<Table>(name)?.pairs::<String, Value>() {
            let (k, v) = pair?;

            if k != "random" && k != "randomseed" {
                table.set(k, v)?;
            }
        }

        env.set(name, readonly(lua, table)?)?;
    }

    let proxy = lua.create_table()?;
    let meta = lua.create_table()?;

    meta.set("__index", env)?;

    meta.set(
        "__newindex",
        lua.create_function(|_, (_, name, _): (Value, String, Value)| {
            Err::<(), _>(mlua::Error::runtime(format!(
                "Update cannot write global '{name}'."
            )))
        })?,
    )?;

    meta.set("kinematic.readonly", true)?;
    meta.set("__metatable", false)?;
    proxy.set_metatable(Some(meta))?;
    lua.set_named_registry_value("kinematic.update_environment", proxy)?;

    Ok(())
}

pub(super) fn readonly(lua: &Lua, table: Table) -> Result<Table> {
    let proxy = lua.create_table()?;
    let meta = lua.create_table()?;
    let next = lua.globals().get::<Function>("next")?;
    let contents = table.clone();

    meta.set(
        "__pairs",
        lua.create_function(move |lua, _: Value| {
            let next = next.clone();
            let contents = contents.clone();
            let iterator = lua.create_function(move |_, (_, key): (Value, Value)| {
                next.call::<(Value, Value)>((contents.clone(), key))
            })?;

            Ok((iterator, Value::Nil, Value::Nil))
        })?,
    )?;

    let length = table.raw_len();

    meta.set("__len", lua.create_function(move |_, _: Value| Ok(length))?)?;
    meta.set("__index", table)?;

    meta.set(
        "__newindex",
        lua.create_function(|_, _: mlua::MultiValue| {
            Err::<(), _>(mlua::Error::runtime(
                "Engine and simulation libraries are read-only.",
            ))
        })?,
    )?;

    meta.set("kinematic.readonly", true)?;
    meta.set("__metatable", false)?;
    proxy.set_metatable(Some(meta))?;

    Ok(proxy)
}

pub(super) fn method(object: &ObjectHandle, name: &str) -> Result<Option<Function>> {
    let runtime = object.runtime()?;
    let callbacks = runtime.callbacks.borrow();

    callbacks
        .get(&object.entity)
        .and_then(|c| c.methods.as_ref())
        .map(|methods| methods.get(name))
        .transpose()
        .map(|v| v.flatten())
}

fn immutable_module(
    lua: &Lua,
    value: Value,
    seen: &mut std::collections::BTreeSet<usize>,
    depth: usize,
) -> Result<Value> {
    match value {
        Value::Function(function) => {
            validate_update(lua, &function)?;

            Ok(Value::Function(function))
        }
        Value::Table(table) => {
            if depth > 64
                || !seen.insert(table.to_pointer() as usize)
                || table.metatable().is_some()
            {
                return Err(mlua::Error::runtime(
                    "Simulation modules require acyclic plain tables.",
                ));
            }

            let contents = lua.create_table()?;

            for entry in table.pairs::<Value, Value>() {
                let (key, value) = entry?;

                if !matches!(key, Value::String(_) | Value::Integer(_)) {
                    return Err(mlua::Error::runtime("Invalid simulation module key."));
                }

                contents.set(key, immutable_module(lua, value, seen, depth + 1)?)?;
            }

            Ok(Value::Table(readonly(lua, contents)?))
        }
        Value::Nil | Value::Boolean(_) | Value::Integer(_) | Value::String(_) => Ok(value),
        Value::Number(number) if number.is_finite() => Ok(value),
        Value::UserData(ref data)
            if data.is::<ValueHandle>() || data.is::<super::drawing::Matrix>() =>
        {
            Ok(value)
        }
        _ => Err(mlua::Error::runtime(
            "Simulation modules cannot export hidden mutable state.",
        )),
    }
}
