use crate::core::{
    TrackChoices, TrackValue,
    types::{Color, Quad},
};
use mlua::{FromLua, IntoLua, Lua, MetaMethod, Result, UserData, UserDataMethods, Value};

#[derive(Clone, Debug)]
pub(crate) struct ValueHandle(pub TrackValue);

impl UserData for ValueHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| Ok(this.0.to_string()));
        methods.add_meta_method(MetaMethod::Index, |_, this, name: String| {
            let index = match name.as_str() {
                "x" | "r" | "a" if !matches!(this.0, TrackValue::Color(_)) => 0,
                "r" => 0,
                "y" | "g" | "b" if !matches!(this.0, TrackValue::Color(_)) => 1,
                "g" => 1,
                "z" | "b" | "c" => 2,
                "w" | "a" | "d" => 3,
                _ => return Err(mlua::Error::runtime("Unknown value component.")),
            };
            let channels = match &this.0 {
                TrackValue::Vector2(v) => vec![v.x, v.y],
                TrackValue::Vector3(v) => vec![v.x, v.y, v.z],
                TrackValue::Quaternion(v) => vec![v.x, v.y, v.z, v.w],
                TrackValue::Color(v) => v.rgba().to_vec(),
                TrackValue::Quad(v) => vec![v.a, v.b, v.c, v.d],
                _ => Vec::new(),
            };
            channels
                .get(index)
                .copied()
                .ok_or_else(|| mlua::Error::runtime("Value component is out of range."))
        });
        for (op, subtract) in [(MetaMethod::Add, false), (MetaMethod::Sub, true)] {
            methods.add_meta_method(op, move |_, this, rhs: mlua::UserDataRef<ValueHandle>| {
                let sign = if subtract { -1.0 } else { 1.0 };
                let value = match (&this.0, &rhs.0) {
                    (TrackValue::Vector2(a), TrackValue::Vector2(b)) => {
                        TrackValue::Vector2(*a + *b * sign)
                    }
                    (TrackValue::Vector3(a), TrackValue::Vector3(b)) => {
                        TrackValue::Vector3(*a + *b * sign)
                    }
                    (TrackValue::Color(a), TrackValue::Color(b)) => {
                        TrackValue::Color(Color::from(std::array::from_fn::<_, 4, _>(|i| {
                            a.rgba()[i] + b.rgba()[i] * sign
                        })))
                    }
                    _ => {
                        return Err(mlua::Error::runtime(
                            "Arithmetic requires matching vectors or colors.",
                        ));
                    }
                };
                Ok(Self(value))
            });
        }
        methods.add_meta_method(MetaMethod::Mul, |_, this, rhs: Value| {
            let result = if let Value::Number(n) = rhs {
                scale(&this.0, n as f32)?
            } else if let Value::Integer(n) = rhs {
                scale(&this.0, n as f32)?
            } else if let (TrackValue::Quaternion(a), Value::UserData(rhs)) = (&this.0, rhs) {
                match &rhs.borrow::<Self>()?.0 {
                    TrackValue::Quaternion(b) => TrackValue::Quaternion(*a * *b),
                    TrackValue::Vector3(b) => TrackValue::Vector3(*a * *b),
                    _ => {
                        return Err(mlua::Error::runtime(
                            "Rotation requires a quaternion or vec3.",
                        ));
                    }
                }
            } else {
                return Err(mlua::Error::runtime("Expected a scalar multiplier."));
            };
            result.validate().map_err(mlua::Error::runtime)?;
            Ok(Self(result))
        });
        methods.add_meta_method(MetaMethod::Div, |_, this, n: f32| {
            Ok(Self(scale(&this.0, 1.0 / n)?))
        });
        methods.add_meta_method(MetaMethod::Unm, |_, this, ()| {
            Ok(Self(scale(&this.0, -1.0)?))
        });
        methods.add_method("dot", |_, this, other: mlua::UserDataRef<Self>| {
            match (&this.0, &other.0) {
                (TrackValue::Vector2(a), TrackValue::Vector2(b)) => Ok(a.dot(*b)),
                (TrackValue::Vector3(a), TrackValue::Vector3(b)) => Ok(a.dot(*b)),
                _ => Err(mlua::Error::runtime("Dot product needs matching vectors.")),
            }
        });
        methods.add_method("cross", |_, this, other: mlua::UserDataRef<Self>| {
            match (&this.0, &other.0) {
                (TrackValue::Vector3(a), TrackValue::Vector3(b)) => {
                    Ok(Self(TrackValue::Vector3(a.cross(*b))))
                }
                _ => Err(mlua::Error::runtime("Cross product needs vec3 values.")),
            }
        });
        methods.add_method("length", |_, this, ()| match this.0 {
            TrackValue::Vector2(v) => Ok(v.length()),
            TrackValue::Vector3(v) => Ok(v.length()),
            _ => Err(mlua::Error::runtime("Expected a vector.")),
        });
        methods.add_method("normalized", |_, this, ()| match this.0 {
            TrackValue::Vector2(v) => Ok(Self(TrackValue::Vector2(v.normalize_or_zero()))),
            TrackValue::Vector3(v) => Ok(Self(TrackValue::Vector3(v.normalize_or_zero()))),
            TrackValue::Quaternion(v) => Ok(Self(TrackValue::Quaternion(v.normalize()))),
            _ => Err(mlua::Error::runtime("Expected a vector or quaternion.")),
        });
        methods.add_method(
            "lerp",
            |_, this, (other, t): (mlua::UserDataRef<Self>, f32)| {
                if std::mem::discriminant(&this.0) != std::mem::discriminant(&other.0)
                    || !t.is_finite()
                {
                    return Err(mlua::Error::runtime(
                        "Interpolation needs compatible finite values.",
                    ));
                }
                Ok(Self(this.0.lerp(&other.0, t)))
            },
        );
    }
}

fn scale(value: &TrackValue, n: f32) -> Result<TrackValue> {
    let value = match value {
        TrackValue::Vector2(v) => TrackValue::Vector2(*v * n),
        TrackValue::Vector3(v) => TrackValue::Vector3(*v * n),
        TrackValue::Color(v) => TrackValue::Color(Color::from(v.rgba().map(|v| v * n))),
        TrackValue::Quad(v) => TrackValue::Quad(Quad::new(v.a * n, v.b * n, v.c * n, v.d * n)),
        _ => return Err(mlua::Error::runtime("Expected a vector, color or quad.")),
    };
    value.validate().map_err(mlua::Error::runtime)?;
    Ok(value)
}

pub(crate) fn to_lua(lua: &Lua, value: TrackValue) -> Result<Value> {
    match value {
        TrackValue::Bool(v) => v.into_lua(lua),
        TrackValue::U32(v) => v.into_lua(lua),
        TrackValue::I32(v) => v.into_lua(lua),
        TrackValue::F32(v) => v.into_lua(lua),
        TrackValue::String(v) => v.into_lua(lua),
        TrackValue::Enum(v) => v.into_lua(lua),
        other => ValueHandle(other).into_lua(lua),
    }
}

pub(crate) fn from_lua(
    lua: &Lua,
    value: Value,
    expected: &TrackValue,
    choices: TrackChoices,
) -> Result<TrackValue> {
    let compatible = match expected {
        TrackValue::Bool(_) => matches!(value, Value::Boolean(_)),
        TrackValue::String(_) | TrackValue::Enum(_) => matches!(value, Value::String(_)),
        TrackValue::F32(_) => matches!(value, Value::Integer(_) | Value::Number(_)),
        TrackValue::U32(_) | TrackValue::I32(_) => {
            matches!(value, Value::Integer(_))
                || matches!(value,Value::Number(v)if v.is_finite()&&v.fract()==0.0)
        }
        _ => matches!(value, Value::UserData(_)),
    };
    if !compatible {
        return Err(mlua::Error::runtime(
            "Property value has an incompatible Lua type.",
        ));
    }
    let result = match expected {
        TrackValue::Bool(_) => TrackValue::Bool(bool::from_lua(value, lua)?),
        TrackValue::U32(_) => TrackValue::U32(u32::from_lua(value, lua)?),
        TrackValue::I32(_) => TrackValue::I32(i32::from_lua(value, lua)?),
        TrackValue::F32(_) => TrackValue::F32(f32::from_lua(value, lua)?),
        TrackValue::String(_) => TrackValue::String(String::from_lua(value, lua)?),
        TrackValue::Enum(_) => {
            let name = String::from_lua(value, lua)?;
            let TrackChoices::Enum(choices) = choices else {
                return Err(mlua::Error::runtime("Missing enum choices."));
            };
            TrackValue::Enum(
                choices
                    .iter()
                    .copied()
                    .find(|choice| *choice == name)
                    .ok_or_else(|| mlua::Error::runtime(format!("Unknown enum value '{name}'.")))?,
            )
        }
        _ => {
            let Value::UserData(value) = value else {
                return Err(mlua::Error::runtime("Expected a Kinematic value."));
            };
            let actual = value.borrow::<ValueHandle>()?.0.clone();
            if std::mem::discriminant(&actual) != std::mem::discriminant(expected) {
                return Err(mlua::Error::runtime("Incompatible property value type."));
            }
            actual
        }
    };
    result.validate().map_err(mlua::Error::runtime)?;
    Ok(result)
}

pub(crate) fn register(lua: &Lua, api: &mlua::Table) -> Result<()> {
    api.set(
        "rect",
        lua.create_function(|_, (x, y, width, height): (f32, f32, f32, f32)| {
            if ![x, y, width, height].iter().all(|v| v.is_finite()) || width < 0.0 || height < 0.0 {
                return Err(mlua::Error::runtime(
                    "Rectangle needs finite coordinates and nonnegative size.",
                ));
            }
            Ok(RectHandle(crate::core::types::Rect::new(
                x, y, width, height,
            )))
        })?,
    )?;
    api.set(
        "axis_angle",
        lua.create_function(|_, (axis, angle): (mlua::UserDataRef<ValueHandle>, f32)| {
            let TrackValue::Vector3(axis) = axis.0 else {
                return Err(mlua::Error::runtime("Rotation axis must be vec3."));
            };
            if !angle.is_finite() || !axis.is_finite() || axis.length_squared() < f32::EPSILON {
                return Err(mlua::Error::runtime(
                    "Rotation needs a finite angle and nonzero axis.",
                ));
            }
            Ok(ValueHandle(TrackValue::Quaternion(
                glam::Quat::from_axis_angle(axis.normalize(), angle),
            )))
        })?,
    )?;

    api.set(
        "vec2",
        lua.create_function(|_, (x, y): (f32, f32)| {
            Ok(ValueHandle(TrackValue::Vector2(glam::vec2(x, y))))
        })?,
    )?;
    api.set(
        "vec3",
        lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            Ok(ValueHandle(TrackValue::Vector3(glam::vec3(x, y, z))))
        })?,
    )?;
    api.set(
        "quat",
        lua.create_function(|_, (x, y, z, w): (f32, f32, f32, f32)| {
            let value = TrackValue::Quaternion(glam::Quat::from_xyzw(x, y, z, w));
            value.validate().map_err(mlua::Error::runtime)?;
            Ok(ValueHandle(value))
        })?,
    )?;
    api.set(
        "quad",
        lua.create_function(|_, (a, b, c, d): (f32, f32, f32, f32)| {
            Ok(ValueHandle(TrackValue::Quad(Quad::new(a, b, c, d))))
        })?,
    )?;
    api.set(
        "color",
        lua.create_function(|_, args: mlua::Variadic<Value>| {
            let color = if let Some(Value::String(hex)) = args.first() {
                let hex = hex.to_str()?;
                let hex = hex.strip_prefix('#').unwrap_or(&hex);
                if !matches!(hex.len(), 6 | 8) {
                    return Err(mlua::Error::runtime(
                        "Color needs six or eight hexadecimal digits.",
                    ));
                }
                let n = u32::from_str_radix(hex, 16).map_err(mlua::Error::external)?;
                let n = if hex.len() == 6 { (n << 8) | 255 } else { n };
                Color::from([
                    ((n >> 24) & 255) as f32 / 255.0,
                    ((n >> 16) & 255) as f32 / 255.0,
                    ((n >> 8) & 255) as f32 / 255.0,
                    (n & 255) as f32 / 255.0,
                ])
            } else {
                let mut rgba = [0.0, 0.0, 0.0, 1.0];
                if !(3..=4).contains(&args.len()) {
                    return Err(mlua::Error::runtime("Color needs RGB or RGBA components."));
                }
                for (i, v) in args.iter().enumerate() {
                    rgba[i] = v
                        .as_number()
                        .ok_or_else(|| mlua::Error::runtime("Expected a color number."))?
                        as f32;
                }
                Color::from(rgba)
            };
            Ok(ValueHandle(TrackValue::Color(color)))
        })?,
    )?;
    Ok(())
}

#[derive(Clone, Copy)]
struct RectHandle(crate::core::types::Rect);
impl UserData for RectHandle {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Index, |_, this, name: String| {
            match name.as_str() {
                "x" => Ok(this.0.x),
                "y" => Ok(this.0.y),
                "width" => Ok(this.0.width),
                "height" => Ok(this.0.height),
                _ => Err(mlua::Error::runtime("Unknown rectangle field.")),
            }
        });
        for (name, anchor) in [
            (
                "middle",
                crate::core::types::Rect::middle as fn(crate::core::types::Rect) -> glam::Vec2,
            ),
            ("top", crate::core::types::Rect::top),
            ("bottom", crate::core::types::Rect::bottom),
            ("left", crate::core::types::Rect::left),
            ("right", crate::core::types::Rect::right),
            ("top_left", crate::core::types::Rect::top_left),
            ("bottom_right", crate::core::types::Rect::bottom_right),
            ("top_right", crate::core::types::Rect::top_right),
            ("bottom_left", crate::core::types::Rect::bottom_left),
        ] {
            m.add_method(name, move |_, this, ()| {
                Ok(ValueHandle(TrackValue::Vector2(anchor(this.0))))
            });
        }
    }
}
