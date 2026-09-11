use crate::core::{
    Scene, SceneIdentity,
    components::{Draw3D, Node},
    objects::{CanvasSettings, ProjectionSource, validate_canvas},
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

pub(crate) fn visible_subtree_3d(
    world: &hecs::World,
    root: hecs::Entity,
    result: &mut Vec<hecs::Entity>,
) {
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
        for child in crate::core::objects::child_iter(world, entity) {
            visit(world, child, result);
        }
    }

    result.clear();
    visit(world, root, result);
}

pub(crate) fn active_subtree(world: &hecs::World, root: hecs::Entity) -> Vec<hecs::Entity> {
    fn visit(world: &hecs::World, entity: hecs::Entity, result: &mut Vec<hecs::Entity>) {
        if !world.get::<&Node>(entity).is_ok_and(|n| n.is_activated) {
            return;
        }
        result.push(entity);
        for child in crate::core::objects::child_iter(world, entity) {
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
            .ok_or("Canvas dependency is missing or inactive.")?;
        reachable.insert(entity, dependencies.clone());
        for dependency in dependencies {
            collect(*dependency, graph, reachable)?;
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

#[derive(Default)]
pub(crate) struct PlanCache {
    entries: HashMap<u64, CachedPlan>,
}

struct CachedPlan {
    revision: u64,
    output: hecs::Entity,
    last_used: u64,
    plan: RenderPlan,
}

pub(crate) struct RenderPlan {
    pub order: Vec<hecs::Entity>,
    pub sources: HashMap<hecs::Entity, Vec<crate::core::objects::CanvasTexture>>,
}

impl PlanCache {
    pub fn get(&mut self, scene: &Scene, frame: u64) -> Result<&RenderPlan, String> {
        let identity = scene.render_key().0;
        let revision = scene.plan_revision();
        let output = scene.get_view().entity;
        if self
            .entries
            .get(&identity)
            .is_none_or(|entry| entry.revision != revision || entry.output != output)
        {
            let order = canvas_order(scene)?;
            let world = scene.get_world();
            let mut sources = HashMap::new();
            for entity in &order {
                let mut textures = Vec::new();
                for child in active_subtree(&world, *entity) {
                    if let Some(source) = world
                        .get::<&ProjectionSource>(child)
                        .ok()
                        .and_then(|source| source.0)
                        && !textures.contains(&source)
                    {
                        textures.push(source);
                    }
                }
                sources.insert(*entity, textures);
            }
            self.entries.insert(
                identity,
                CachedPlan {
                    revision,
                    output,
                    last_used: frame,
                    plan: RenderPlan { order, sources },
                },
            );
        }
        if self.entries.len() > 32 {
            if let Some(oldest) = self
                .entries
                .iter()
                .filter(|(key, _)| **key != identity)
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            {
                self.entries.remove(&oldest);
            }
        }
        let entry = self.entries.get_mut(&identity).unwrap();
        entry.last_used = frame;
        Ok(&entry.plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn plans_survive_frames_and_scene_switches_and_follow_lifetimes() {
        let mut first = Scene::new();
        first.wait(1.0);
        let projection = projection_2d()
            .source(&first.get_world_3d())
            .build(&mut first);
        first.get_world_2d().add(&projection);
        first.update(0.0);
        let second = Scene::new();
        second.update(0.0);
        let mut cache = PlanCache::default();
        let initial = cache.get(&first, 1).unwrap();
        assert_eq!(initial.order, vec![first.get_world_2d().get_id()]);
        let allocation = initial.order.as_ptr();
        cache.get(&second, 2).unwrap();
        first.update(0.5);
        assert_eq!(cache.get(&first, 3).unwrap().order.as_ptr(), allocation);
        assert_eq!(cache.entries.len(), 2);
        first.update(1.0);
        assert_eq!(
            cache.get(&first, 4).unwrap().order,
            vec![first.get_world_3d().get_id(), first.get_world_2d().get_id()]
        );
        first.update(0.0);
        assert_eq!(
            cache.get(&first, 5).unwrap().order,
            vec![first.get_world_2d().get_id()]
        );
        first.invalidate();
        cache.get(&first, 6).unwrap();
        assert_eq!(
            cache.entries[&first.render_key().0].revision,
            first.plan_revision()
        );
    }
}
