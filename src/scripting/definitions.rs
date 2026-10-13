use super::native;
use crate::core::{Scene, TrackValue};

/// Generates LuaLS definitions from the same object metadata used by the runtime.
pub fn definitions() -> String {
    let mut output = include_str!("api.lua").to_owned();
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
        if classes.insert(class) {
            output.push_str(&format!("---@class Kinematic.{class}: Kinematic.Object\n"));
            output.push_str(&format!("---@field signal fun(self:Kinematic.{class}, callback:fun(object:Kinematic.{class}, frame:Kinematic.Frame)):Kinematic.Signal\n"));
            for field in &fields {
                output.push_str(&format!(
                    "---@field {} Kinematic.Property<{}>\n",
                    field.name(),
                    field_type(field, &world, entity)
                ));
            }
            for method in object.methods {
                output.push_str(&format!(
                    "local Native = {{}} --[[@as Kinematic.{class}]]\n{}",
                    method.annotation
                ));
            }
            output.push_str(&format!(
                "---@class Kinematic.{class}Options: Kinematic.ResourceOptions\n---@field id? string\n---@field name? string\n"
            ));
            for field in &fields {
                output.push_str(&format!(
                    "---@field {}? {}\n",
                    field.name(),
                    builder_type(field, &world, entity)
                ));
            }
        }
        if !custom {
            output.push_str(&format!("---@param options Kinematic.{class}Options\n---@return Kinematic.{class}\nfunction Scene:{}(options) end\n",object.name));
        }
    }
    for &(name, _, _, _) in super::adapters::ANCHORS {
        output.push_str(&format!("---@return Kinematic.Value\nfunction Object:{name}() end\n---@param position Kinematic.Value\n---@return Kinematic.Value\nfunction Object:{name}_to(position) end\n"));
    }
    output.push_str("return k\n");
    output
}
fn value_type(value: &TrackValue) -> &'static str {
    match value {
        TrackValue::Bool(_) => "boolean",
        TrackValue::U32(_) | TrackValue::I32(_) => "integer",
        TrackValue::F32(_) => "number",
        TrackValue::String(_) | TrackValue::Enum(_) => "string",
        _ => "Kinematic.Value",
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
        |channels| format!("Kinematic.Value|{channels}"),
    )
}
