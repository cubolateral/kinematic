use crate::core::{
    Scene, SceneIdentity,
    components::{Draw3D, Node},
    objects::{CanvasSettings, ProjectionSource, children, validate_canvas},
};
use std::collections::HashMap;

/// Orders canvas dependencies independently of their rendering backend.
pub(crate) fn order_dependencies(
    graph: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
) -> Result<Vec<hecs::Entity>, String> {
    fn visit(
        entity: hecs::Entity,
        graph: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
        visiting: &mut Vec<hecs::Entity>,
        done: &mut Vec<hecs::Entity>,
    ) -> Result<(), String> {
        if done.contains(&entity) {
            return Ok(());
        }
        if visiting.contains(&entity) {
            return Err("Canvas dependency cycle detected.".into());
        }
        let dependencies = graph
            .get(&entity)
            .ok_or("Canvas dependency is missing or inactive.")?;
        visiting.push(entity);
        for dependency in dependencies {
            visit(*dependency, graph, visiting, done)?;
        }
        visiting.pop();
        done.push(entity);
        Ok(())
    }
    let mut ordered = Vec::new();
    let mut keys: Vec<_> = graph.keys().copied().collect();
    keys.sort_by_key(|entity| entity.to_bits());
    for entity in keys {
        visit(entity, graph, &mut Vec::new(), &mut ordered)?;
    }
    Ok(ordered)
}

pub(crate) fn visible_subtree_3d(world: &hecs::World, root: hecs::Entity) -> Vec<hecs::Entity> {
    fn visit(world: &hecs::World, entity: hecs::Entity, result: &mut Vec<hecs::Entity>) {
        if !world
            .get::<&Node>(entity)
            .is_ok_and(|node| node.is_activated)
            || !world
                .get::<&Draw3D>(entity)
                .is_ok_and(|draw| draw.visibility)
        {
            return;
        }
        result.push(entity);
        for child in children(world, entity) {
            visit(world, child, result);
        }
    }

    let mut result = Vec::new();
    visit(world, root, &mut result);
    result
}

pub(crate) fn active_subtree(world: &hecs::World, root: hecs::Entity) -> Vec<hecs::Entity> {
    fn visit(world: &hecs::World, entity: hecs::Entity, result: &mut Vec<hecs::Entity>) {
        if !world.get::<&Node>(entity).is_ok_and(|n| n.is_activated) {
            return;
        }
        result.push(entity);
        for child in children(world, entity) {
            visit(world, child, result);
        }
    }
    let mut result = Vec::new();
    visit(world, root, &mut result);
    result
}

pub(crate) fn canvas_order(scene: &Scene) -> Result<Vec<hecs::Entity>, String> {
    let world = scene.get_world();
    let root = scene.get_root().get_id();
    let scene_id = world.get::<&SceneIdentity>(root).unwrap().0;
    let output = scene.get_view();
    let mut graph = HashMap::new();
    for entity in active_subtree(&world, root) {
        if world.get::<&CanvasSettings>(entity).is_err() {
            continue;
        }
        let mut dependencies = Vec::new();
        for child in active_subtree(&world, entity) {
            if let Ok(source) = world.get::<&ProjectionSource>(child) {
                let source = source.0.ok_or("Projection requires a canvas source.")?;
                if source.scene != scene_id {
                    return Err("Projection source belongs to another scene.".into());
                }
                if world.get::<&CanvasSettings>(source.entity).is_err() {
                    return Err("Projection requires a canvas source.".into());
                }
                if !dependencies.contains(&source.entity) {
                    dependencies.push(source.entity);
                }
            }
        }
        graph.insert(entity, dependencies);
    }

    fn collect(
        entity: hecs::Entity,
        graph: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
        reachable: &mut HashMap<hecs::Entity, Vec<hecs::Entity>>,
    ) -> Result<(), String> {
        if reachable.contains_key(&entity) {
            return Ok(());
        }
        let dependencies = graph
            .get(&entity)
            .ok_or("Canvas dependency is missing or inactive.")?
            .clone();
        reachable.insert(entity, dependencies.clone());
        for dependency in dependencies {
            collect(dependency, graph, reachable)?;
        }
        Ok(())
    }

    let mut reachable = HashMap::new();
    collect(output.entity, &graph, &mut reachable)?;
    for entity in reachable.keys() {
        validate_canvas(&world, *entity)?;
    }
    order_dependencies(&reachable)
}
