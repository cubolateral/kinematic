use super::runtime::{ObjectHandle, PropertyHandle, duration, engine, parse_easing};
use crate::core::{Task, TrackValue};
use mlua::{Lua, Result, Table, UserData, UserDataMethods, Value};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Operation {
    To,
    By,
    From,
}

#[derive(Clone)]
struct Destination {
    property: PropertyHandle,
    start: Option<TrackValue>,
    finish: TrackValue,
    operation: Operation,
    mask: u8,
    instant: bool,
    rotation: Option<(glam::Vec3, f32)>,
}

#[derive(Clone)]
pub(crate) struct AnimationDefinition {
    pub(super) object: ObjectHandle,
    destinations: Vec<Destination>,
}

pub(super) fn channel(value: &TrackValue, name: &str) -> Result<u8> {
    let names: &[&str] = match value {
        TrackValue::Vector2(_) => &["x", "y"],
        TrackValue::Vector3(_) => &["x", "y", "z"],

        TrackValue::Color(_) => &["r", "g", "b", "a"],
        TrackValue::Quad(_) => &["a", "b", "c", "d"],
        _ => &[],
    };
    names
        .iter()
        .position(|v| *v == name)
        .map(|i| i as u8)
        .ok_or_else(|| {
            mlua::Error::runtime(format!("Unknown channel '{name}' for this value type."))
        })
}

// Resolve declarations before applying them, so overlaps never depend on table order.
fn properties(
    object: &ObjectHandle,
    table: Table,
    uniforms: bool,
) -> Result<Vec<(PropertyHandle, Value)>> {
    let mut result = Vec::new();
    for pair in table.pairs::<String, Value>() {
        let (name, value) = pair?;
        if !uniforms && name == "uniforms" {
            let Value::Table(table) = value else {
                return Err(mlua::Error::runtime("Uniforms must be a table."));
            };
            result.extend(properties(object, table, true)?);
            continue;
        }
        let property = if uniforms {
            let runtime = object.runtime()?;
            let target = crate::core::TrackTarget::uniform(name.clone());
            if !crate::core::objects::shader_uniforms(
                &runtime.world.borrow().borrow(),
                object.entity,
            )
            .iter()
            .any(|(key, _)| key == &name)
            {
                return Err(mlua::Error::runtime(format!("Unknown uniform '{name}'.")));
            }
            PropertyHandle {
                object: object.clone(),
                target,
                channel: None,
            }
        } else {
            object.property(&name)?
        };
        if let Value::Table(table) = &value
            && property.channel.is_none()
            && table
                .clone()
                .pairs::<Value, Value>()
                .any(|entry| matches!(entry, Ok((Value::String(_), _))))
        {
            for pair in table.clone().pairs::<String, Value>() {
                let (name, value) = pair?;
                let channel = channel(&property.value()?, &name)?;
                result.push((
                    PropertyHandle {
                        channel: Some(channel),
                        ..property.clone()
                    },
                    value,
                ));
            }
        } else {
            result.push((property, value));
        }
    }
    let mut masks: Vec<(crate::core::TrackTarget, u8)> = Vec::new();
    for (property, _) in &result {
        let mask = property
            .channel
            .map_or((1 << property.value()?.channels()) - 1, |c| 1 << c);
        if masks
            .iter()
            .any(|(target, previous)| target.same_field(&property.target) && previous & mask != 0)
        {
            return Err(mlua::Error::runtime(
                "Overlapping property or channel declarations.",
            ));
        }
        masks.push((property.target.clone(), mask));
    }
    Ok(result)
}

pub(super) fn configure(lua: &Lua, object: &ObjectHandle, options: &Table) -> Result<()> {
    let fields = lua.create_table()?;
    for pair in options.clone().pairs::<String, Value>() {
        let (name, value) = pair?;
        if name != "id" && name != "name" && !super::resources::reserved(&name) {
            fields.set(name, value)?;
        }
    }
    let resolved = properties(object, fields, false)?;
    for (property, value) in resolved {
        property.set(lua, value)?;
    }
    let runtime = object.runtime()?;
    let world = runtime.world.borrow();
    let world = world.borrow();
    if let Ok(camera) = world.get::<&crate::core::components::Camera3D>(object.entity) {
        camera.validate().map_err(mlua::Error::runtime)?;
    }
    Ok(())
}

fn pair(value: Value) -> Result<(Value, Value)> {
    let Value::Table(table) = value else {
        return Err(mlua::Error::runtime(
            "From requires a pair {start, finish}.",
        ));
    };
    let entries = table
        .clone()
        .pairs::<Value, Value>()
        .collect::<Result<Vec<_>>>()?;
    if entries.len() != 2
        || entries
            .iter()
            .any(|(k, _)| !matches!(k, Value::Integer(1 | 2)))
    {
        return Err(mlua::Error::runtime(
            "From requires exactly two indexed values.",
        ));
    }
    let start = table.raw_get::<Value>(1)?;
    let finish = table.raw_get::<Value>(2)?;
    if start.is_nil() || finish.is_nil() {
        return Err(mlua::Error::runtime("From pairs cannot omit an endpoint."));
    }
    Ok((start, finish))
}

impl AnimationDefinition {
    pub(super) fn new(object: ObjectHandle) -> Self {
        Self {
            object,
            destinations: Vec::new(),
        }
    }

    pub(super) fn append(&self, lua: &Lua, table: Table, operation: Operation) -> Result<Self> {
        self.object.runtime()?.construction()?;
        let mut result = self.clone();
        for (property, value) in properties(&self.object, table, false)? {
            let mask = property
                .channel
                .map_or((1 << property.value()?.channels()) - 1, |c| 1 << c);
            if result
                .destinations
                .iter()
                .any(|d| d.property.target.same_field(&property.target) && d.mask & mask != 0)
            {
                return Err(mlua::Error::runtime(
                    "Overlapping destinations in the same tween.",
                ));
            }
            let (start, mut finish) = if operation == Operation::From {
                let (start, finish) = pair(value)?;
                (
                    Some(property.parse_value(lua, start, true)?),
                    property.parse_value(lua, finish, true)?,
                )
            } else {
                (
                    None,
                    property.parse_value(lua, value, operation != Operation::By)?,
                )
            };
            if operation == Operation::By
                && !matches!(
                    finish,
                    TrackValue::F32(_)
                        | TrackValue::I32(_)
                        | TrackValue::U32(_)
                        | TrackValue::Vector2(_)
                        | TrackValue::Vector3(_)
                        | TrackValue::Color(_)
                        | TrackValue::Quad(_)
                )
            {
                return Err(mlua::Error::runtime(
                    "By requires a numeric property or channel.",
                ));
            }
            if operation == Operation::By && property.channel.is_some() {
                let selected = property.channel.unwrap();
                let zero = match &finish {
                    TrackValue::Vector2(_) => TrackValue::Vector2(glam::Vec2::ZERO),
                    TrackValue::Vector3(_) => TrackValue::Vector3(glam::Vec3::ZERO),
                    TrackValue::Color(_) => TrackValue::Color([0.0; 4].into()),
                    TrackValue::Quad(_) => {
                        TrackValue::Quad(crate::core::types::Quad::new(0.0, 0.0, 0.0, 0.0))
                    }
                    _ => return Err(mlua::Error::runtime("Unsupported relative channel.")),
                };
                finish = zero.with_channel(&finish, selected);
            }
            result.destinations.push(Destination {
                property,
                start,
                finish,
                operation,
                mask,
                instant: false,
                rotation: None,
            });
        }
        Ok(result)
    }

    pub(super) fn property(
        lua: &Lua,
        property: PropertyHandle,
        value: Value,
        operation: Operation,
    ) -> Result<Self> {
        property.object.runtime()?.construction()?;
        let (start, finish) = if operation == Operation::From {
            let (start, finish) = pair(value)?;
            (
                Some(property.parse_value(lua, start, true)?),
                property.parse_value(lua, finish, true)?,
            )
        } else {
            (
                None,
                property.parse_value(lua, value, operation != Operation::By)?,
            )
        };
        if operation == Operation::By
            && !matches!(
                finish,
                TrackValue::F32(_) | TrackValue::Vector2(_) | TrackValue::Vector3(_)
            )
        {
            return Err(mlua::Error::runtime(
                "Property increments require numbers or vectors.",
            ));
        }
        let mask = property
            .channel
            .map_or((1 << finish.channels()) - 1, |c| 1 << c);
        Ok(Self {
            object: property.object.clone(),
            destinations: vec![Destination {
                property,
                start,
                finish,
                operation,
                mask,
                instant: false,
                rotation: None,
            }],
        })
    }

    pub(super) fn play(&self, seconds: Option<f32>, ease: Option<String>) -> Result<()> {
        let runtime = self.object.runtime()?;
        runtime.construction()?;
        let seconds = duration(seconds.unwrap_or(0.0))?;
        let easing = parse_easing(ease)?;
        let tasks = self
            .destinations
            .iter()
            .map(|d| {
                let current = engine(|| {
                    runtime.animator.borrow().sample_scheduled(
                        &runtime.world.borrow().borrow(),
                        self.object.entity,
                        &d.property.target,
                    )
                })?;
                let from = d.start.clone().unwrap_or(current);
                let mut to = d.finish.clone();
                if d.operation == Operation::By {
                    let sum = add(&from, &d.finish)?;
                    to = if from.channels() == 1 {
                        sum
                    } else {
                        let mut value = from.clone();
                        for channel in 0..from.channels() {
                            if d.mask & (1 << channel) != 0 {
                                value = value.with_channel(&sum, channel);
                            }
                        }
                        value
                    };
                    d.property.validate(&to)?;
                }
                Ok(Task::RuntimeTween {
                    entity: self.object.entity,
                    target: d.property.target.clone(),
                    from,
                    to,
                    rotation: d.rotation,
                    duration: if d.instant { 0.0 } else { seconds },
                    easing,
                    mask: d.mask,
                    relative: if d.operation == Operation::By {
                        d.mask
                    } else {
                        0
                    },
                    implicit: d.start.is_none(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let camera = runtime
            .world
            .borrow()
            .borrow()
            .get::<&crate::core::components::Camera3D>(self.object.entity)
            .map(|camera| (*camera).clone())
            .ok();
        if let Some(mut camera) = camera {
            for name in [
                "camera_mode",
                "camera_position",
                "camera_rotation",
                "camera_fov",
                "camera_near",
                "camera_far",
            ] {
                let property = self.object.property(name)?;
                let value = engine(|| {
                    runtime.animator.borrow().sample_scheduled(
                        &runtime.world.borrow().borrow(),
                        self.object.entity,
                        &property.target,
                    )
                })?;
                camera_value(&mut camera, name, &value);
            }
            let mut initial = camera.clone();
            for task in &tasks {
                if let Task::RuntimeTween {
                    target, from, to, ..
                } = task
                {
                    camera_value(&mut initial, target.name(), from);
                    camera_value(&mut camera, target.name(), to);
                }
            }
            initial.validate().map_err(mlua::Error::runtime)?;
            camera.validate().map_err(mlua::Error::runtime)?;
        }
        engine(|| {
            runtime
                .scene
                .borrow_mut()
                .as_mut()
                .unwrap()
                .play(Task::All(tasks))
        })
    }

    pub(crate) fn target(
        object: ObjectHandle,
        target: crate::core::TrackTarget,
        finish: TrackValue,
    ) -> Self {
        let mask = (1 << finish.channels()) - 1;
        Self {
            object: object.clone(),
            destinations: vec![Destination {
                property: PropertyHandle {
                    object,
                    target,
                    channel: None,
                },
                start: None,
                finish,
                operation: Operation::To,
                mask,
                instant: false,
                rotation: None,
            }],
        }
    }

    pub(super) fn from_task(object: ObjectHandle, task: Task) -> Result<Self> {
        let mut result = Self::new(object.clone());
        match task {
            Task::All(tasks) => {
                for task in tasks {
                    result
                        .destinations
                        .extend(Self::from_task(object.clone(), task)?.destinations);
                }
            }
            Task::RuntimeTween {
                target,
                from,
                to,
                implicit,
                mask,
                rotation,
                duration,
                ..
            } => {
                result.destinations.push(Destination {
                    property: PropertyHandle {
                        object,
                        target,
                        channel: None,
                    },
                    start: if implicit { None } else { Some(from) },
                    finish: to,
                    operation: Operation::To,
                    mask,
                    instant: duration == 0.0,
                    rotation,
                });
            }
            _ => {
                return Err(mlua::Error::runtime(
                    "Native Lua methods must return runtime tween tasks.",
                ));
            }
        }
        Ok(result)
    }

    pub(super) fn destinations(
        object: ObjectHandle,
        values: Vec<(crate::core::TrackTarget, TrackValue)>,
    ) -> Self {
        let mut result = Self::new(object.clone());
        for (target, finish) in values {
            result
                .destinations
                .extend(Self::target(object.clone(), target, finish).destinations);
        }
        result
    }
}

impl UserData for AnimationDefinition {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        for (name, op) in [
            ("to", Operation::To),
            ("by", Operation::By),
            ("from", Operation::From),
        ] {
            methods.add_function(
                name,
                move |lua, (this, table): (mlua::AnyUserData, Table)| {
                    let appended = this.borrow::<Self>()?.append(lua, table, op)?;
                    *this.borrow_mut::<Self>()? = appended;
                    Ok(this)
                },
            );
        }
        methods.add_method(
            "play",
            |_, this, (seconds, ease): (Option<f32>, Option<String>)| this.play(seconds, ease),
        );
    }
}

fn add(from: &TrackValue, delta: &TrackValue) -> Result<TrackValue> {
    Ok(match (from, delta) {
        (TrackValue::F32(a), TrackValue::F32(b)) => TrackValue::F32(a + b),
        (TrackValue::I32(a), TrackValue::I32(b)) => TrackValue::I32(
            a.checked_add(*b)
                .ok_or_else(|| mlua::Error::runtime("Integer increment overflow."))?,
        ),
        (TrackValue::U32(a), TrackValue::U32(b)) => TrackValue::U32(
            a.checked_add(*b)
                .ok_or_else(|| mlua::Error::runtime("Integer increment overflow."))?,
        ),
        (TrackValue::Vector2(a), TrackValue::Vector2(b)) => TrackValue::Vector2(*a + *b),
        (TrackValue::Vector3(a), TrackValue::Vector3(b)) => TrackValue::Vector3(*a + *b),
        (TrackValue::Color(a), TrackValue::Color(b)) => {
            TrackValue::Color(std::array::from_fn::<_, 4, _>(|i| a.rgba()[i] + b.rgba()[i]).into())
        }
        (TrackValue::Quad(a), TrackValue::Quad(b)) => TrackValue::Quad(
            crate::core::types::Quad::new(a.a + b.a, a.b + b.b, a.c + b.c, a.d + b.d),
        ),
        _ => return Err(mlua::Error::runtime("Incompatible increment type.")),
    })
}

fn camera_value(camera: &mut crate::core::components::Camera3D, name: &str, value: &TrackValue) {
    use crate::core::components::Camera3DMode;
    match (name, value) {
        ("camera_mode", TrackValue::Enum("Perspective")) => {
            camera.camera_mode = Camera3DMode::Perspective
        }
        ("camera_mode", TrackValue::Enum("Orthogonal")) => {
            camera.camera_mode = Camera3DMode::Orthogonal
        }
        ("camera_fov", TrackValue::F32(value)) => camera.camera_fov = *value,
        ("camera_near", TrackValue::F32(value)) => camera.camera_near = *value,
        ("camera_far", TrackValue::F32(value)) => camera.camera_far = *value,
        ("camera_position", TrackValue::Vector3(value)) => camera.camera_position = *value,
        ("camera_rotation", TrackValue::Quaternion(value)) => camera.camera_rotation = *value,
        _ => (),
    }
}
