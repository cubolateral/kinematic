use crate::core::{Scene, TrackTarget, components::Inspection, objects::*};

#[derive(Clone, Copy)]
pub(crate) struct NativeObject {
    pub name: &'static str,
    pub object_name: &'static str,
    pub documentation: &'static str,
    pub methods: &'static [super::methods::LuaMethod],
    pub spawn: fn(&mut Scene, hecs::Entity),
}

pub(super) fn native<T: Object + Default>() -> NativeObject {
    NativeObject {
        name: T::BUILDER_NAME,
        object_name: T::inspection().object_name,
        documentation: T::DOCUMENTATION,
        methods: &[],
        spawn: |scene, entity| {
            T::spawn_at(
                scene.world_handle(),
                scene.animator_handle(),
                T::default(),
                crate::core::components::Name::new(T::inspection().object_name),
                Some(entity),
            );
        },
    }
}

pub(crate) fn objects() -> Vec<NativeObject> {
    let mut objects = vec![
        native::<Circle>(),
        native::<Ellipse>(),
        native::<Rect>(),
        native::<RegularPolygon2D>(),
        native::<Line2D>(),
        native::<Image2D>(),
        NativeObject {
            methods: Text2DHandler::lua_methods(),
            ..native::<Text2D>()
        },
        native::<Latex2D>(),
        native::<Group2D>(),
        native::<Container2D>(),
        native::<Canvas2D>(),
        native::<Projection2D>(),
        native::<Sphere>(),
        native::<Torus>(),
        native::<Prism>(),
        native::<Pyramid>(),
        native::<Plane>(),
        native::<RegularPolygon3D>(),
        native::<Line3D>(),
        native::<Image3D>(),
        native::<Text3D>(),
        native::<Latex3D>(),
        native::<Group3D>(),
        native::<Container3D>(),
        native::<Canvas3D>(),
        native::<Projection3D>(),
        native::<DirectionalLight3D>(),
        native::<PointLight3D>(),
        native::<SpotLight3D>(),
    ];

    macro_rules! preset {
        ($name:literal, $object:ty, $sides:expr) => {
            objects.push(NativeObject {
                name: $name,
                object_name: <$object>::inspection().object_name,
                documentation: <$object>::DOCUMENTATION,
                methods: &[],
                spawn: |scene, entity| {
                    let mut object = <$object>::default();
                    object.shape.sides = $sides;
                    <$object>::spawn_at(
                        scene.world_handle(),
                        scene.animator_handle(),
                        object,
                        crate::core::components::Name::new(<$object>::inspection().object_name),
                        Some(entity),
                    );
                },
            });
        };
    }

    preset!("triangle", RegularPolygon2D, 3);
    preset!("square", RegularPolygon2D, 4);
    preset!("triangle_3d", RegularPolygon3D, 3);
    preset!("square_3d", RegularPolygon3D, 4);
    preset!("circle_3d", RegularPolygon3D, 32);
    preset!("tetrahedron", Prism, 3);
    preset!("cube", Prism, 4);
    preset!("cylinder", Prism, 32);
    preset!("cone", Pyramid, 32);

    objects
}

pub(crate) fn properties(world: &hecs::World, entity: hecs::Entity) -> Vec<TrackTarget> {
    world
        .get::<&Inspection>(entity)
        .map(|inspection| inspection.properties(world, entity))
        .unwrap_or_default()
}

pub(super) fn property(
    world: &hecs::World,
    entity: hecs::Entity,
    name: &str,
) -> Option<TrackTarget> {
    let inspection = world.get::<&Inspection>(entity).ok()?;

    for component in inspection.trackables(world, entity) {
        if let Some(info) = (component.get)().iter().find(|info| info.name == name) {
            return Some(TrackTarget::property((component.type_id)(), info));
        }
    }

    if let Ok(properties) = world.get::<&crate::core::Properties>(entity)
        && let Some((schema, _)) = properties.0.get(name)
    {
        return Some(TrackTarget::Dynamic(schema.clone()));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_matches_enumerated_properties() {
        let mut scene = Scene::new();
        let circle = circle().build(&mut scene);
        let world = scene.world();

        for target in properties(&world, circle.entity()) {
            assert!(
                property(&world, circle.entity(), target.name())
                    .unwrap()
                    .same(&target)
            );
        }

        assert!(property(&world, circle.entity(), "missing").is_none());
    }
}
