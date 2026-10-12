use crate::core::{Scene, TrackTarget, components::Inspection, objects::*};

#[derive(Clone, Copy)]
pub(crate) struct NativeObject {
    pub name: &'static str,
    pub spawn: fn(&mut Scene, hecs::Entity),
}

pub(super) fn native<T: Object + Default>() -> NativeObject {
    NativeObject {
        name: T::BUILDER_NAME,
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
        native::<Text2D>(),
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
