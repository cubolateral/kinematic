use crate::core::TrackableInfo;

/// Concrete Rust type of a spawned scene object.
pub(crate) struct ObjectType(pub(crate) std::any::TypeId);

/// Entity-level metadata used by the inspector UI.
#[derive(Clone)]
pub struct Inspection {
    /// Object name for the entity.
    pub object_name: &'static str,

    /// Returns the static set of trackable components for this entity type.
    pub get: fn(&hecs::World, hecs::Entity) -> &'static [TrackableInfo],
}

impl Inspection {
    #[doc(hidden)]
    pub fn new(
        object_name: &'static str,
        get: fn(&hecs::World, hecs::Entity) -> &'static [TrackableInfo],
    ) -> Self {
        Self { object_name, get }
    }

    /// Returns native fields, dynamic fields and uniforms through one track contract.
    pub(crate) fn properties(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
    ) -> Vec<crate::core::TrackTarget> {
        let mut fields = Vec::new();

        for component in self.trackables(world, entity) {
            for info in (component.get)() {
                fields.push(crate::core::TrackTarget::property(
                    (component.type_id)(),
                    info,
                ));
            }
        }

        if let Ok(properties) = world.get::<&crate::core::Properties>(entity) {
            fields.extend(
                properties
                    .fields
                    .values()
                    .map(|(schema, _)| crate::core::TrackTarget::Dynamic(schema.clone())),
            );
        }

        fields.extend(
            crate::core::objects::shader_uniforms(world, entity)
                .into_iter()
                .map(|(name, _)| crate::core::TrackTarget::uniform(name)),
        );

        fields
    }

    /// Groups editable fields by their declaring component.
    pub(crate) fn property_groups(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
    ) -> Vec<(String, Vec<crate::core::TrackTarget>)> {
        let mut groups = Vec::new();

        for component in self.trackables(world, entity) {
            let fields = (component.get)()
                .iter()
                .map(|info| crate::core::TrackTarget::property((component.type_id)(), info))
                .collect();

            groups.push((component.name.to_owned(), fields));
        }

        if let Ok(properties) = world.get::<&crate::core::Properties>(entity) {
            for (name, fields) in &properties.components {
                groups.push((
                    name.clone(),
                    fields
                        .iter()
                        .map(|name| {
                            crate::core::TrackTarget::Dynamic(properties.fields[name].0.clone())
                        })
                        .collect(),
                ));
            }
        }

        let uniforms = crate::core::objects::shader_uniforms(world, entity)
            .into_iter()
            .map(|(name, _)| crate::core::TrackTarget::uniform(name))
            .collect::<Vec<_>>();

        if !uniforms.is_empty() {
            groups.push(("Shader Uniforms".into(), uniforms));
        }

        groups
    }

    /// Iterates over the object's trackable components.
    pub(crate) fn trackables<'a>(
        &'a self,
        world: &hecs::World,
        entity: hecs::Entity,
    ) -> impl Iterator<Item = &'a TrackableInfo> {
        (self.get)(world, entity).iter()
    }
}

impl Default for Inspection {
    fn default() -> Self {
        Self::new("Object", |_world, _entity| &[])
    }
}
