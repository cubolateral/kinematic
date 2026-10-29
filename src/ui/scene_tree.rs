use kinematic_ui::widgets;
use std::collections::HashSet;

use crate::{
    core::{
        Scene,
        components::{Draw2D, Draw3D, Inspection, Name, TreeNode},
        objects::ObjectHandler,
    },
    editor::Editor,
};

pub(super) const WINDOW_NAME: &str = "Scene Tree";

#[derive(Default)]
pub(super) struct State {
    expanded: HashSet<hecs::Entity>,
    scene_key: Option<u64>,
}

impl State {
    fn sync(&mut self, scene: &Scene) {
        let scene_key = scene.render_key().0;

        if self.scene_key != Some(scene_key) {
            self.expanded = [
                scene.root().entity(),
                scene.world_2d().entity(),
                scene.world_3d().entity(),
            ]
            .into();

            self.scene_key = Some(scene_key);
        }
    }

    fn toggle(&mut self, entity: hecs::Entity) {
        if !self.expanded.remove(&entity) {
            self.expanded.insert(entity);
        }
    }
}

struct ObjectRow {
    entity: hecs::Entity,
    branches: Vec<bool>,
    is_last: bool,
    ancestor_selected: bool,
    ancestor_visible: bool,
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) -> bool {
    let is_exporting = editor.is_exporting();
    let selected = editor.selected_entity();

    state.sync(editor.scene_mut());

    let root = editor.scene_mut().root().entity();
    let world = editor.scene_mut().world();
    let mut clicked = None;
    let mut edit_clicked = None;
    let mut empty_clicked = false;
    let mut visibility_changed = false;

    kinematic_ui::windows::panel(ui, WINDOW_NAME, || {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let root_name = world
            .get::<&Name>(root)
            .expect("Root must contain a Name component.");
        let root_response = kinematic_ui::tree::root(
            ui,
            root_name.get(),
            state.expanded.contains(&root),
            !active_children(&world, root).is_empty(),
            selected == Some(root),
        );
        if root_response.toggled {
            state.toggle(root);
        } else if root_response.clicked {
            clicked = Some(root);
        }

        let root_visibility = object_visibility(&world, root).unwrap_or(true);
        let mut rows = vec![];

        collect_rows(
            &world,
            &state.expanded,
            root,
            &mut vec![],
            selected,
            selected == Some(root),
            root_visibility,
            &mut rows,
        );

        if rows.is_empty() && state.expanded.contains(&root) {
            widgets::muted(ui, "   No objects.");
        } else {
            for index in dear_imgui_rs::ListClipper::new(rows.len()).begin(ui).iter() {
                draw_row(
                    &world,
                    ui,
                    &rows[index],
                    state,
                    selected,
                    &mut clicked,
                    &mut edit_clicked,
                    &mut visibility_changed,
                );
            }
        }

        empty_clicked = !is_exporting
            && ui.is_window_hovered()
            && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left)
            && !ui.is_any_item_hovered();
    });

    drop(world);

    if visibility_changed {
        editor.scene_mut().invalidate();
    }

    if let Some(entity) = edit_clicked {
        editor.select_entity(entity);

        true
    } else if let Some(entity) = clicked {
        editor.select_entity(entity);

        false
    } else if empty_clicked {
        editor.clear_selection();

        false
    } else {
        false
    }
}

fn draw_row(
    world: &hecs::World,
    ui: &dear_imgui_rs::Ui,
    row: &ObjectRow,
    state: &mut State,
    selected: Option<hecs::Entity>,
    clicked: &mut Option<hecs::Entity>,
    edit_clicked: &mut Option<hecs::Entity>,
    visibility_changed: &mut bool,
) {
    let entity = row.entity;
    let _id = ui.push_id(&format!("scene_tree_{}", entity.to_bits()));
    let name = world
        .get::<&Name>(entity)
        .expect("Scene tree object must contain a Name component.");
    let visibility = object_visibility(world, entity);
    let response = kinematic_ui::tree::row(
        ui,
        kinematic_ui::tree::Row {
            label: name.get(),
            branches: &row.branches,
            is_last: row.is_last,
            expanded: state.expanded.contains(&entity),
            has_children: !active_children(world, entity).is_empty(),
            highlighted: row.ancestor_selected || selected == Some(entity),
            visibility,
            effective_visibility: row.ancestor_visible && visibility.unwrap_or(true),
        },
    );
    if response.hovered {
        if let Ok(inspection) = world.get::<&Inspection>(entity) {
            ui.tooltip_text(format!("Type: {}", inspection.object_name));
        }
    }
    if response.toggled {
        state.toggle(entity);
    }
    if response.clicked {
        *clicked = Some(entity);
    }
    if response.edit {
        *edit_clicked = Some(entity);
    }
    if response.visibility {
        if let Some(visibility) = visibility {
            if let Ok(mut draw) = world.get::<&mut Draw2D>(entity) {
                draw.visibility = !visibility;
                *visibility_changed = true;
            } else if let Ok(mut draw) = world.get::<&mut Draw3D>(entity) {
                draw.visibility = !visibility;
                *visibility_changed = true;
            }
        }
    }
}

fn collect_rows(
    world: &hecs::World,
    expanded: &HashSet<hecs::Entity>,
    parent: hecs::Entity,
    branches: &mut Vec<bool>,
    selected: Option<hecs::Entity>,
    ancestor_selected: bool,
    ancestor_visible: bool,
    rows: &mut Vec<ObjectRow>,
) {
    if !expanded.contains(&parent) {
        return;
    }

    let children = active_children(world, parent);

    for (index, entity) in children.iter().copied().enumerate() {
        let is_last = index + 1 == children.len();
        let is_selected = ancestor_selected || selected == Some(entity);
        let visibility = object_visibility(world, entity).unwrap_or(true);

        rows.push(ObjectRow {
            entity,
            branches: branches.clone(),
            is_last,
            ancestor_selected,
            ancestor_visible,
        });

        branches.push(!is_last);

        collect_rows(
            world,
            expanded,
            entity,
            branches,
            selected,
            is_selected,
            ancestor_visible && visibility,
            rows,
        );

        branches.pop();
    }
}

fn object_visibility(world: &hecs::World, entity: hecs::Entity) -> Option<bool> {
    world
        .get::<&Draw2D>(entity)
        .map(|draw| draw.visibility)
        .or_else(|_| world.get::<&Draw3D>(entity).map(|draw| draw.visibility))
        .ok()
}

fn active_children(world: &hecs::World, entity: hecs::Entity) -> Vec<hecs::Entity> {
    world
        .get::<&TreeNode>(entity)
        .map(|node| {
            node.children
                .as_ref()
                .into_iter()
                .flatten()
                .copied()
                .filter(|child| {
                    world
                        .get::<&TreeNode>(*child)
                        .expect("Scene tree object must contain a Node component.")
                        .is_activated
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_root_and_builtin_worlds_start_expanded() {
        let scene = Scene::new();
        let mut state = State::default();

        state.sync(&scene);

        assert_eq!(
            state.expanded,
            [
                scene.root().entity(),
                scene.world_2d().entity(),
                scene.world_3d().entity()
            ]
            .into()
        );

        state.toggle(scene.world_2d().entity());
        state.sync(&scene);
        assert!(!state.expanded.contains(&scene.world_2d().entity()));

        let next_scene = Scene::new();

        state.sync(&next_scene);

        assert_eq!(
            state.expanded,
            [
                next_scene.root().entity(),
                next_scene.world_2d().entity(),
                next_scene.world_3d().entity()
            ]
            .into()
        );
    }

    #[test]
    fn collapsed_nodes_hide_descendants_and_preserve_sibling_branches() {
        let mut world = hecs::World::new();
        let inactive = world.spawn((TreeNode::default(),));
        let child = world.spawn((TreeNode {
            is_activated: true,
            ..TreeNode::default()
        },));
        let node = world.spawn((TreeNode {
            is_activated: true,
            children: Some(vec![child, inactive]),
            ..TreeNode::default()
        },));
        let sibling = world.spawn((TreeNode {
            is_activated: true,
            ..TreeNode::default()
        },));
        let root = world.spawn((TreeNode {
            is_activated: true,
            children: Some(vec![node, sibling]),
            ..TreeNode::default()
        },));
        let mut state = State {
            expanded: [root, node].into(),
            ..State::default()
        };
        let mut rows = vec![];

        collect_rows(
            &world,
            &state.expanded,
            root,
            &mut vec![],
            Some(node),
            false,
            true,
            &mut rows,
        );

        assert_eq!(
            rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
            vec![node, child, sibling]
        );

        assert_eq!(rows[1].branches, vec![true]);
        assert!(rows[1].is_last);
        assert!(rows[1].ancestor_selected);
        assert!(rows[2].is_last);

        state.toggle(node);
        rows.clear();

        collect_rows(
            &world,
            &state.expanded,
            root,
            &mut vec![],
            Some(node),
            false,
            true,
            &mut rows,
        );

        assert_eq!(
            rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
            vec![node, sibling]
        );

        state.toggle(node);
        assert_eq!(state.expanded, [root, node].into());
        state.toggle(root);
        rows.clear();

        collect_rows(
            &world,
            &state.expanded,
            root,
            &mut vec![],
            None,
            false,
            true,
            &mut rows,
        );

        assert!(rows.is_empty());
    }
}
