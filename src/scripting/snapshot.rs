use super::value::ValueHandle;
use crate::core::Random;
use mlua::{Lua, Result, UserData, UserDataMethods, Value};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Debug)]
pub(super) enum Snapshot {
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
    Table(Arc<[(Snapshot, Snapshot)]>),
}
impl Snapshot {
    pub fn read(value: Value) -> Result<Self> {
        Self::capture(value, &mut Vec::new(), &mut BTreeSet::new(), 0)
    }

    fn capture(
        value: Value,
        path: &mut Vec<Value>,
        seen: &mut BTreeSet<usize>,
        depth: usize,
    ) -> Result<Self> {
        if depth > 64 {
            return Err(Self::error("Snapshot nesting exceeds 64", path));
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
                    return Err(Self::error(
                        "Cycles and shared table references are not supported",
                        path,
                    ));
                }

                if table.metatable().is_some() {
                    return Err(Self::error("Snapshot table cannot have a metatable", path));
                }

                let mut entries = Vec::new();

                table.for_each::<Value, Value>(|key, value| {
                    if !matches!(key, Value::String(_) | Value::Integer(_)) {
                        return Err(Self::error(
                            "Snapshot key must be a string or integer",
                            path,
                        ));
                    }

                    let saved_key = Self::capture(key.clone(), path, seen, depth + 1)?;

                    path.push(key);

                    let saved_value = Self::capture(value, path, seen, depth + 1)?;

                    path.pop();
                    entries.push((saved_key, saved_value));

                    Ok(())
                })?;

                entries.sort_unstable_by(|(a, _), (b, _)| match (a, b) {
                    (Self::Integer(a), Self::Integer(b)) => a.cmp(b),
                    (Self::String(a), Self::String(b)) => a.cmp(b),
                    (Self::Integer(_), Self::String(_)) => std::cmp::Ordering::Less,
                    (Self::String(_), Self::Integer(_)) => std::cmp::Ordering::Greater,
                    _ => unreachable!("Snapshot keys must be strings or integers."),
                });

                Self::Table(entries.into())
            }
            _ => {
                return Err(Self::error("Unsupported snapshot value", path));
            }
        })
    }

    fn error(message: &str, keys: &[Value]) -> mlua::Error {
        use std::fmt::Write;

        let mut path = String::from("state");

        for key in keys {
            match key {
                Value::Integer(index) => {
                    let _ = write!(path, ".{index}");
                }
                Value::String(name) => {
                    let _ = write!(path, ".{}", name.to_string_lossy());
                }
                _ => unreachable!("Snapshot keys must be strings or integers."),
            }
        }

        mlua::Error::runtime(format!("{message} at {path}."))
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
                let table = if entries.iter().enumerate().all(|(index, (key, _))| {
                    matches!(key, Self::Integer(key) if *key == index as i64 + 1)
                }) {
                    lua.create_sequence_from(entries.iter().map(|(_, value)| value))?
                } else {
                    lua.create_table_from(entries.iter().map(|(key, value)| (key, value)))?
                };

                Ok(Value::Table(table))
            }
        }
    }
}
impl mlua::IntoLua for &Snapshot {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        self.value(lua)
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
    fn table_order_is_deterministic_for_mixed_keys() {
        let lua = Lua::new();
        let a = Snapshot::read(lua.load("return {[10]=1,[2]=2,z=3,a=4}").eval().unwrap()).unwrap();
        let b = Snapshot::read(lua.load("return {a=4,z=3,[2]=2,[10]=1}").eval().unwrap()).unwrap();

        assert_eq!(format!("{a:?}"), format!("{b:?}"));

        let table = a.value(&lua).unwrap();

        assert_eq!(table.as_table().unwrap().get::<i64>(10).unwrap(), 1);
        assert_eq!(table.as_table().unwrap().get::<i64>("a").unwrap(), 4);
    }
    #[test]
    fn dense_and_sparse_tables_round_trip() {
        let lua = Lua::new();

        for source in [
            "return {10, {20, 30}, false}",
            "return {[-1]=4, [2]=5, [100]=6, name='x'}",
        ] {
            let saved = Snapshot::read(lua.load(source).eval().unwrap()).unwrap();
            let restored = Snapshot::read(saved.value(&lua).unwrap()).unwrap();

            assert_eq!(format!("{saved:?}"), format!("{restored:?}"));
        }

        let invalid = lua
            .load("return {nested={values={function() end}}}")
            .eval()
            .unwrap();

        assert!(
            Snapshot::read(invalid)
                .unwrap_err()
                .to_string()
                .contains("state.nested.values.1")
        );
    }

    #[test]
    fn snapshots_are_deep_and_reject_hidden_state() {
        let lua = Lua::new();
        let original: Value = lua
            .load("return { count=1, nested={value=2} }")
            .eval()
            .unwrap();
        let saved = Snapshot::read(original.clone()).unwrap();
        let copy = saved.clone();

        if let (Snapshot::Table(a), Snapshot::Table(b)) = (&saved, &copy) {
            assert!(Arc::ptr_eq(a, b));
        } else {
            panic!("Expected table snapshots.");
        }

        let restored = copy.value(&lua).unwrap();

        restored.as_table().unwrap().set("count", 50).unwrap();
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
