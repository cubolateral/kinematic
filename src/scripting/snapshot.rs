use super::value::ValueHandle;
use crate::core::Random;
use mlua::{Lua, Result, UserData, UserDataMethods, Value};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(crate) enum Snapshot {
    Nil,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Value(crate::core::TrackValue),
    Random(Random),
    Image(super::resources::ImageHandle),
    Font(super::resources::FontHandle),
    Mesh(super::drawing::MeshHandle),
    Matrix(super::drawing::Matrix),
    Table(Vec<(Snapshot, Snapshot)>),
}
impl Snapshot {
    pub fn read(value: Value) -> Result<Self> {
        Self::capture(value, "state", &mut BTreeSet::new(), 0)
    }
    fn capture(value: Value, path: &str, seen: &mut BTreeSet<usize>, depth: usize) -> Result<Self> {
        if depth > 64 {
            return Err(mlua::Error::runtime(format!(
                "Snapshot nesting exceeds 64 at {path}."
            )));
        }
        Ok(match value {
            Value::Nil => Self::Nil,
            Value::Boolean(v) => Self::Bool(v),
            Value::Integer(v) => Self::Integer(v),
            Value::Number(v) if v.is_finite() => Self::Number(v),
            Value::String(v) => Self::String(v.to_str()?.to_owned()),
            Value::UserData(v) if v.is::<ValueHandle>() => {
                Self::Value(v.borrow::<ValueHandle>()?.0.clone())
            }
            Value::UserData(v) if v.is::<RandomHandle>() => {
                Self::Random(v.borrow::<RandomHandle>()?.0.clone())
            }
            Value::UserData(v) if v.is::<super::resources::ImageHandle>() => {
                Self::Image(v.borrow::<super::resources::ImageHandle>()?.clone())
            }
            Value::UserData(v) if v.is::<super::resources::FontHandle>() => {
                Self::Font(v.borrow::<super::resources::FontHandle>()?.clone())
            }
            Value::UserData(v) if v.is::<super::drawing::MeshHandle>() => {
                Self::Mesh(v.borrow::<super::drawing::MeshHandle>()?.clone())
            }
            Value::UserData(v) if v.is::<super::drawing::Matrix>() => {
                Self::Matrix(v.borrow::<super::drawing::Matrix>()?.clone())
            }
            Value::Table(table) => {
                if !seen.insert(table.to_pointer() as usize) {
                    return Err(mlua::Error::runtime(format!(
                        "Cycles and shared table references are not supported at {path}."
                    )));
                }
                if table.metatable().is_some() {
                    return Err(mlua::Error::runtime(format!(
                        "Snapshot table at {path} cannot have a metatable."
                    )));
                }
                let mut entries = Vec::new();
                for pair in table.pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    let key_name = match &key {
                        Value::String(v) => v.to_str()?.to_string(),
                        Value::Integer(v) => v.to_string(),
                        _ => {
                            return Err(mlua::Error::runtime(format!(
                                "Snapshot key at {path} must be a string or integer."
                            )));
                        }
                    };
                    entries.push((
                        Self::capture(key, path, seen, depth + 1)?,
                        Self::capture(value, &format!("{path}.{key_name}"), seen, depth + 1)?,
                    ));
                }
                entries.sort_by_key(|(key, _)| format!("{key:?}"));
                Self::Table(entries)
            }
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "Unsupported snapshot value at {path}."
                )));
            }
        })
    }
    pub fn value(&self, lua: &Lua) -> Result<Value> {
        use mlua::IntoLua;
        match self {
            Self::Nil => Ok(Value::Nil),
            Self::Bool(v) => v.into_lua(lua),
            Self::Integer(v) => v.into_lua(lua),
            Self::Number(v) => v.into_lua(lua),
            Self::String(v) => v.as_str().into_lua(lua),
            Self::Value(v) => ValueHandle(v.clone()).into_lua(lua),
            Self::Random(v) => RandomHandle(v.clone()).into_lua(lua),
            Self::Image(v) => v.clone().into_lua(lua),
            Self::Font(v) => v.clone().into_lua(lua),
            Self::Mesh(v) => v.clone().into_lua(lua),
            Self::Matrix(v) => v.clone().into_lua(lua),
            Self::Table(entries) => {
                let table = lua.create_table()?;
                for (key, value) in entries {
                    table.set(key.value(lua)?, value.value(lua)?)?;
                }
                Ok(Value::Table(table))
            }
        }
    }
}
#[derive(Clone)]
pub(crate) struct RandomHandle(pub Random);
impl UserData for RandomHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("number", |_, this, ()| Ok(this.0.f32()));
        methods.add_method_mut("integer", |_, this, ()| Ok(this.0.u32()));
        methods.add_method_mut("boolean", |_, this, ()| Ok(this.0.bool()));
        methods.add_method_mut("range", |_, this, (min, max): (f64, f64)| {
            super::runtime::engine(|| this.0.range_f64(min..max))
        });
        methods.add_method_mut("range_integer", |_, this, (min, max): (i64, i64)| {
            super::runtime::engine(|| this.0.range_i64(min..max))
        });
        methods.add_method_mut("chance", |_, this, probability: f64| {
            super::runtime::engine(|| this.0.chance(probability))
        });
        methods.add_method_mut("normal", |_, this, (mean, deviation): (f64, f64)| {
            super::runtime::engine(|| this.0.normal(mean, deviation))
        });
        methods.add_method_mut(
            "choose",
            |_, this, (values, weights): (Vec<Value>, Option<Vec<f64>>)| {
                super::runtime::engine(|| match weights {
                    Some(weights) => this.0.choose_weighted(&values, &weights).cloned(),
                    None => this.0.choose(&values).cloned(),
                })
            },
        );
        methods.add_method_mut(
            "choose_multiple",
            |_, this, (values, amount): (Vec<Value>, usize)| {
                super::runtime::engine(|| {
                    this.0
                        .choose_multiple(&values, amount)
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>()
                })
            },
        );
        methods.add_method_mut("shuffle", |_, this, values: mlua::Table| {
            let mut items = values
                .clone()
                .sequence_values::<Value>()
                .collect::<Result<Vec<_>>>()?;
            this.0.shuffle(&mut items);
            for (index, value) in items.into_iter().enumerate() {
                values.raw_set(index + 1, value)?;
            }
            Ok(values)
        });
        methods.add_method("fork", |_, this, key: u64| Ok(Self(this.0.fork(key))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_are_deep_and_reject_hidden_state() {
        let lua = Lua::new();
        let original: Value = lua
            .load("return { count=1, nested={value=2} }")
            .eval()
            .unwrap();
        let saved = Snapshot::read(original.clone()).unwrap();
        original.as_table().unwrap().set("count", 99).unwrap();
        assert_eq!(
            saved
                .value(&lua)
                .unwrap()
                .as_table()
                .unwrap()
                .get::<i64>("count")
                .unwrap(),
            1
        );
        let cyclic: Value = lua.load("local t={};t.self=t;return t").eval().unwrap();
        assert!(
            Snapshot::read(cyclic)
                .unwrap_err()
                .to_string()
                .contains("state.self")
        );
    }
}
