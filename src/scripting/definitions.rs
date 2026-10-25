use super::native;
use crate::core::{Easing, Scene, TrackLimits, TrackTarget, TrackValue};

/// Generates LuaLS definitions from the same object metadata used by the runtime.
pub fn definitions() -> String {
    let mut output = include_str!("api.lua").to_owned();

    output.push('\n');
    output.push_str(include_str!("property.lua"));

    value_definitions(&mut output);
    output.push_str("--- Named easing curves accepted by Animation:play. The default is cubic_in_out.\n---@class Kinematic.Easing\n");

    for easing in Easing::ALL {
        let name = super::runtime::easing_name(easing);
        let description = if name == "linear" {
            "Linear easing with constant speed.".to_owned()
        } else {
            let (family, behavior) = if let Some(family) = name.strip_suffix("_in_out") {
                (
                    family,
                    "accelerates at the start and decelerates at the end",
                )
            } else if let Some(family) = name.strip_suffix("_in") {
                (family, "accelerates at the start")
            } else {
                (name.strip_suffix("_out").unwrap(), "decelerates at the end")
            };

            format!("Native {family} curve that {behavior}.")
        };

        output.push_str(&format!("---@field {name} string {description}\n"));
    }

    output.push_str("k.ease = {}\n\n");

    let mut classes = std::collections::BTreeSet::new();
    let objects = native::objects()
        .into_iter()
        .map(|object| (object, false))
        .chain([
            (native::native::<super::extension::Script2D>(), true),
            (native::native::<super::extension::Script3D>(), true),
        ]);

    for (object, custom) in objects {
        let mut scene = Scene::new();
        let entity = scene.world_mut().spawn(());

        (object.spawn)(&mut scene, entity);

        let world = scene.world();
        let inspection = world
            .get::<&crate::core::components::Inspection>(entity)
            .unwrap();
        let class = inspection.object_name;
        let fields = native::properties(&world, entity);
        let new_class = classes.insert(class);

        if new_class {
            documentation(&mut output, object.documentation);
            output.push_str(&format!("---@class Kinematic.{class}: Kinematic.Object\n"));
            output.push_str(&format!("---@field signal fun(self:Kinematic.{class}, callback:fun(object:Kinematic.{class}, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.\n"));

            for field in &fields {
                output.push_str(&format!(
                    "---@field {} {} {}\n",
                    field.name(),
                    property_type(field, &world, entity),
                    property_documentation(field, &world, entity),
                ));
            }

            output.push_str(&format!("local {class} = {{}}\n\n"));

            for method in object.methods {
                documentation(&mut output, method.documentation);
                output.push_str(&method.annotation.replace("Native:", &format!("{class}:")));
                output.push('\n');
            }
        }

        let options_class = if new_class {
            format!("{class}Options")
        } else {
            format!("{class}Options_{}", object.name)
        };

        output.push_str(&format!(
            "--- Initial properties and resources for {}.\n---@class Kinematic.{options_class}: Kinematic.ResourceOptions\n---@field id? string Stable identifier used to preserve object identity across reloads.\n---@field name? string Display name in the scene tree.\n", object.name
        ));

        for field in &fields {
            output.push_str(&format!(
                "---@field {}? {} {}\n",
                field.name(),
                builder_type(field, &world, entity),
                property_documentation(field, &world, entity),
            ));
        }

        output.push('\n');

        if !custom {
            documentation(&mut output, object.documentation);
            output.push_str("--- Creates the object with the supplied initial properties at the current schedule time.\n");
            output.push_str(&format!("---@param options Kinematic.{options_class} Initial properties, name, identifier, and resources.\n---@return Kinematic.{class} object Handle exposing animatable properties and object methods.\nfunction Scene:{}(options) end\n\n",object.name));
        }
    }

    for &(name, _, _, _) in super::adapters::ANCHORS {
        output.push_str(&format!("--- Returns the {name} anchor of the bounding box in parent coordinates.\n---@return Kinematic.Value\nfunction Object:{name}() end\n\n--- Returns the local position that places the {name} anchor at position.\n---@param position Kinematic.Value Target position in parent coordinates.\n---@return Kinematic.Value\nfunction Object:{name}_to(position) end\n\n"));
    }

    output.push_str("return k\n");

    output
}
fn value_definitions(output: &mut String) {
    for (class, channels) in [
        ("Vec2", "xy"),
        ("Vec3", "xyz"),
        ("Color", "rgba"),
        ("Quaternion", "xyzw"),
        ("Quad", "abcd"),
    ] {
        output.push_str(&format!(
            "--- Immutable {class} value.\n---@class Kinematic.{class}\n"
        ));

        for channel in channels.chars() {
            output.push_str(&format!(
                "---@field {channel} number The {channel} component.\n"
            ));
        }

        if matches!(class, "Vec2" | "Vec3" | "Color") {
            output.push_str(&format!("---@operator add(Kinematic.{class}): Kinematic.{class}\n---@operator sub(Kinematic.{class}): Kinematic.{class}\n"));
        }

        if class == "Quaternion" {
            output.push_str("---@operator mul(Kinematic.Quaternion): Kinematic.Quaternion\n---@operator mul(Kinematic.Vec3): Kinematic.Vec3\n");
        } else {
            output.push_str(&format!("---@operator mul(number): Kinematic.{class}\n---@operator div(number): Kinematic.{class}\n---@operator unm: Kinematic.{class}\n"));
        }

        output.push_str(&format!("local {class} = {{}}\n\n"));
        output.push_str(&format!("--- Interpolates between compatible values.\n---@param other Kinematic.{class}\n---@param t number Interpolation factor.\n---@return Kinematic.{class}\nfunction {class}:lerp(other, t) end\n\n"));

        if matches!(class, "Vec2" | "Vec3") {
            output.push_str(&format!("--- Returns the dot product.\n---@param other Kinematic.{class}\n---@return number\nfunction {class}:dot(other) end\n\n--- Returns the vector magnitude.\n---@return number\nfunction {class}:length() end\n\n"));
        }

        if matches!(class, "Vec2" | "Vec3" | "Quaternion") {
            output.push_str(&format!("--- Returns a normalized value. Zero vectors remain zero.\n---@return Kinematic.{class}\nfunction {class}:normalized() end\n\n"));
        }

        if class == "Vec3" {
            output.push_str("--- Returns the cross product.\n---@param other Kinematic.Vec3\n---@return Kinematic.Vec3\nfunction Vec3:cross(other) end\n\n");
        }

        output.push_str(&format!("--- Animatable {class} property.\n---@class Kinematic.{class}Property: Kinematic.Property<Kinematic.{class}>\n"));

        if class != "Quaternion" {
            for channel in channels.chars() {
                output.push_str(&format!(
                    "---@field {channel} Kinematic.Property<number> Animatable {channel} component.\n"
                ));
            }
        }

        let methods = include_str!("property.lua");

        output.push_str(&format!("local {class}Property = {{}}\n\n"));

        output.push_str(
            &methods
                .replace("---@generic T\n", "")
                .replace(
                    "Kinematic.Property<T>",
                    &format!("Kinematic.{class}Property"),
                )
                .replace(" T\n", &format!(" Kinematic.{class}\n"))
                .replace(" T[]\n", &format!(" Kinematic.{class}[]\n"))
                .replace("function Property:", &format!("function {class}Property:")),
        );

        if class == "Quaternion" {
            output.push_str("--- Creates a quaternion rotation preserving full turns. Call :play to schedule it.\n---@param axis Kinematic.Vec3\n---@param radians number\n---@return Kinematic.Animation\nfunction QuaternionProperty:rotate(axis, radians) end\n");
        }

        output.push('\n');
    }
}

fn property_type(field: &TrackTarget, world: &hecs::World, entity: hecs::Entity) -> String {
    let value = field.get(world, entity);
    let ty = field_type(field, world, entity);

    if value.channels() > 1 || matches!(value, TrackValue::Quaternion(_)) {
        format!("{ty}Property")
    } else {
        format!("Kinematic.Property<{ty}>")
    }
}

fn documentation(output: &mut String, text: &str) {
    for line in text.trim().lines() {
        output.push_str("--- ");
        output.push_str(line.trim());
        output.push('\n');
    }
}

fn property_documentation(
    field: &TrackTarget,
    world: &hecs::World,
    entity: hecs::Entity,
) -> String {
    let mut text = match field {
        TrackTarget::Property { info, .. } => info
            .documentation
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    };
    let default = match field.get(world, entity) {
        TrackValue::String(value) => format!("{value:?}"),
        TrackValue::Enum(value) => format!("{value:?}"),
        TrackValue::F32(value) => value.to_string(),
        TrackValue::Quaternion(value) => format!("{:?}", value.to_array()),
        value if value.channels() > 1 => format!(
            "[{}]",
            (0..value.channels())
                .map(|index| value.channel(index).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        value => value.to_string(),
    };

    text.push_str(&format!(" Default: {default}."));

    if let TrackTarget::Property { info, .. } = field {
        let (min, max) = match info.limits {
            TrackLimits::None => (None, None),
            TrackLimits::F32 { min, max } => {
                (min.map(|v| v.to_string()), max.map(|v| v.to_string()))
            }
            TrackLimits::I32 { min, max } => {
                (min.map(|v| v.to_string()), max.map(|v| v.to_string()))
            }
            TrackLimits::U32 { min, max } => {
                (min.map(|v| v.to_string()), max.map(|v| v.to_string()))
            }
        };

        if let Some(min) = min {
            text.push_str(&format!(" Minimum: {min}."));
        }

        if let Some(max) = max {
            text.push_str(&format!(" Maximum: {max}."));
        }
    }

    text
}

fn value_type(value: &TrackValue) -> &'static str {
    match value {
        TrackValue::Bool(_) => "boolean",
        TrackValue::U32(_) | TrackValue::I32(_) => "integer",
        TrackValue::F32(_) => "number",
        TrackValue::String(_) | TrackValue::Enum(_) => "string",
        TrackValue::Vector2(_) => "Kinematic.Vec2",
        TrackValue::Vector3(_) => "Kinematic.Vec3",
        TrackValue::Color(_) => "Kinematic.Color",
        TrackValue::Quaternion(_) => "Kinematic.Quaternion",
        TrackValue::Quad(_) => "Kinematic.Quad",
    }
}

fn field_type(
    field: &crate::core::TrackTarget,
    world: &hecs::World,
    entity: hecs::Entity,
) -> String {
    if let crate::core::TrackTarget::Property { info, .. } = field {
        if let crate::core::TrackChoices::Enum(choices) = info.choices {
            return choices
                .iter()
                .map(|choice| format!("'{choice}'"))
                .collect::<Vec<_>>()
                .join("|");
        }
    }

    value_type(&field.get(world, entity)).into()
}

fn builder_type(
    field: &crate::core::TrackTarget,
    world: &hecs::World,
    entity: hecs::Entity,
) -> String {
    let channels = match field.get(world, entity) {
        TrackValue::Vector2(_) => Some("{x?:number,y?:number}"),
        TrackValue::Vector3(_) => Some("{x?:number,y?:number,z?:number}"),
        TrackValue::Color(_) => Some("{r?:number,g?:number,b?:number,a?:number}"),
        TrackValue::Quad(_) => Some("{a?:number,b?:number,c?:number,d?:number}"),
        _ => None,
    };

    channels.map_or_else(
        || field_type(field, world, entity),
        |channels| format!("{}|{channels}", field_type(field, world, entity)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitions_document_registered_objects_tracks_and_methods() {
        let definitions = definitions();
        let lua = mlua::Lua::new();

        lua.load(&definitions).exec().unwrap();

        for object in native::objects().into_iter().chain([
            native::native::<super::super::extension::Script2D>(),
            native::native::<super::super::extension::Script3D>(),
        ]) {
            assert!(!object.documentation.trim().is_empty(), "{}", object.name);

            let mut scene = Scene::new();
            let entity = scene.world_mut().spawn(());

            (object.spawn)(&mut scene, entity);

            let world = scene.world();

            for field in native::properties(&world, entity) {
                if let TrackTarget::Property { info, .. } = &field {
                    assert!(
                        !info.documentation.trim().is_empty(),
                        "{}.{}",
                        object.name,
                        info.name
                    );

                    let documentation = property_documentation(&field, &world, entity);

                    assert!(definitions.contains(&documentation), "{documentation}");
                }
            }

            for method in object.methods {
                assert!(
                    !method.documentation.trim().is_empty(),
                    "{}.{}",
                    object.name,
                    method.name
                );

                assert!(definitions.contains(method.documentation.lines().next().unwrap().trim()));
            }
        }

        assert!(definitions.contains("Radius in local coordinates. Default: 128. Minimum: 0."));
        assert!(definitions.contains("Position of the entity."));
        for name in ["Text2D", "Text3D", "Latex2D", "Latex3D"] {
            assert!(definitions.contains(&format!("function {name}:morph(text) end\n\n")));
        }
        for name in ["Text2D", "Latex2D", "Latex3D"] {
            assert!(definitions.contains(&format!("function {name}:fade(text) end\n\n")));
        }
        assert!(!definitions.contains("Scene:native"));
        for name in ["to", "by", "from"] {
            assert!(!definitions.contains(&format!("function Animation:{name}(")));
        }

        for name in super::super::runtime::OBJECT_METHODS {
            assert!(
                definitions.contains(&format!("function Object:{name}("))
                    || definitions.contains(&format!("function Animation:{name}(")),
                "{name}"
            );
        }

        for easing in Easing::ALL {
            assert!(definitions.contains(&format!(
                "---@field {} string ",
                super::super::runtime::easing_name(easing)
            )));
        }

        for block in definitions
            .split("\n\n")
            .filter(|block| block.contains("function "))
        {
            assert!(
                block.lines().any(|line| line.starts_with("--- ")),
                "{block}"
            );

            assert_eq!(
                block
                    .lines()
                    .filter(|line| line.starts_with("function "))
                    .count(),
                1,
                "{block}"
            );
        }
    }
}
