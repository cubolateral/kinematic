use std::collections::HashSet;

use crate::{
    core::{
        Scene,
        components::{Draw2D, Draw3D, Inspection, Name, TreeNode},
        objects::ObjectHandler,
    },
    editor::Editor,
};

use super::{
    controls,
    icons::{EYE, EYE_SLASH, PENCIL},
    widgets::{hide_single_window_tab, text_size},
};

const ROW_HEIGHT: f32 = 24.0;
const INDENT: f32 = 16.0;
const NAME_SPACING: f32 = 4.0;
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
    let _text_align = ui.push_style_var(dear_imgui_rs::StyleVar::SelectableTextAlign([0.0, 0.5]));

    hide_single_window_tab(ui);
    ui.window(WINDOW_NAME).build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let spacing = unsafe { ui.style().item_spacing() };
        let _spacing = ui.push_style_var(dear_imgui_rs::StyleVar::ItemSpacing([spacing[0], 0.0]));
        let root_name = world
            .get::<&Name>(root)
            .expect("Root must contain a Name component.");
        let position = ui.cursor_screen_pos();
        let root_clicked = selectable_row(
            ui,
            format!("##scene_tree_{}", root.to_bits()),
            [0.0, ROW_HEIGHT],
        );
        let draw_list = ui.get_window_draw_list();

        draw_list.add_text(
            [
                position[0] + INDENT + NAME_SPACING,
                position[1] + (ROW_HEIGHT - text_size(ui, root_name.get())[1]) * 0.5,
            ],
            ui.get_color_u32(if selected == Some(root) {
                dear_imgui_rs::StyleColor::CheckMark
            } else {
                dear_imgui_rs::StyleColor::Text
            }),
            root_name.get(),
        );
        if state.expanded.contains(&root) && !active_children(&world, root).is_empty() {
            let x = position[0] + INDENT * 0.5;
            draw_list
                .add_line(
                    [x, position[1] + ROW_HEIGHT * 0.5 + 6.0],
                    [x, position[1] + ROW_HEIGHT],
                    ui.get_color_u32(dear_imgui_rs::StyleColor::TreeLines),
                )
                .build();
        }
        drop(draw_list);

        let root_toggle = draw_arrow(ui, position, state.expanded.contains(&root), root_clicked);
        if root_toggle {
            state.toggle(root);
        } else if root_clicked {
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
            ui.text_disabled("   No objects.");
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
    let is_highlighted = row.ancestor_selected || selected == Some(entity);
    let name = world
        .get::<&Name>(entity)
        .expect("Scene tree object must contain a Name component.");
    let has_children = !active_children(world, entity).is_empty();
    let position = ui.cursor_screen_pos();
    let row_width = ui.content_region_avail_width();
    let visibility = object_visibility(world, entity);
    let effective_visibility = row.ancestor_visible && visibility.unwrap_or(true);
    let row_id = format!("##scene_tree_{}", entity.to_bits());
    let control_count = 1.0 + f32::from(visibility.is_some());
    let selectable_width = (row_width - ROW_HEIGHT * control_count).max(1.0);
    let was_clicked = selectable_row(ui, row_id, [selectable_width, ROW_HEIGHT]);
    if ui.is_item_hovered() {
        if let Ok(inspection) = world.get::<&Inspection>(entity) {
            ui.tooltip_text(format!("Type: {}", inspection.object_name));
        }
    }

    let text_y = position[1] + (ROW_HEIGHT - text_size(ui, name.get())[1]) * 0.5;
    let draw_list = ui.get_window_draw_list();
    let text_clip = draw_list.push_clip_rect(
        position,
        [position[0] + selectable_width, position[1] + ROW_HEIGHT],
        true,
    );

    let line_color = ui.get_color_u32(dear_imgui_rs::StyleColor::TreeLines);
    let middle = position[1] + ROW_HEIGHT * 0.5;
    for (depth, continues) in row.branches.iter().enumerate() {
        if *continues {
            let x = position[0] + INDENT * (depth as f32 + 0.5);
            draw_list
                .add_line([x, position[1]], [x, position[1] + ROW_HEIGHT], line_color)
                .build();
        }
    }
    let branch_x = position[0] + INDENT * (row.branches.len() as f32 + 0.5);
    let arrow_x = branch_x + INDENT * 0.5;
    draw_list
        .add_line(
            [branch_x, position[1]],
            [
                branch_x,
                if row.is_last {
                    middle
                } else {
                    position[1] + ROW_HEIGHT
                },
            ],
            line_color,
        )
        .build();
    draw_list
        .add_line([branch_x, middle], [arrow_x, middle], line_color)
        .build();
    if has_children && state.expanded.contains(&entity) {
        let x = arrow_x + INDENT * 0.5;
        draw_list
            .add_line([x, middle + 6.0], [x, position[1] + ROW_HEIGHT], line_color)
            .build();
    }
    draw_list.add_text(
        [arrow_x + INDENT + NAME_SPACING, text_y],
        ui.get_color_u32(if is_highlighted {
            dear_imgui_rs::StyleColor::CheckMark
        } else {
            dear_imgui_rs::StyleColor::Text
        }),
        name.get(),
    );
    drop(text_clip);
    drop(draw_list);
    drop(name);
    let toggled = has_children
        && draw_arrow(
            ui,
            [arrow_x, position[1]],
            state.expanded.contains(&entity),
            was_clicked && ui.io().mouse_pos()[0] < position[0] + selectable_width,
        );
    if toggled {
        state.toggle(entity);
    }
    ui.get_window_draw_list()
        .add_rect(
            [position[0] + selectable_width, position[1]],
            [position[0] + row_width, position[1] + ROW_HEIGHT],
            ui.get_color_u32(dear_imgui_rs::StyleColor::WindowBg),
        )
        .filled(true)
        .build();
    ui.same_line_with_spacing(0.0, 0.0);
    if edit_button(ui, entity) {
        *edit_clicked = Some(entity);
    }
    if let Some(visibility) = visibility {
        ui.same_line_with_spacing(0.0, 0.0);
        *visibility_changed |=
            visibility_button(world, ui, entity, visibility, effective_visibility);
    }

    if was_clicked && !toggled {
        *clicked = Some(entity);
    }
}

fn draw_arrow(ui: &dear_imgui_rs::Ui, position: [f32; 2], expanded: bool, clicked: bool) -> bool {
    let center = [position[0] + INDENT * 0.5, position[1] + ROW_HEIGHT * 0.5];
    let (a, b, c) = if expanded {
        (
            [center[0] - 4.0, center[1] - 2.0],
            [center[0] + 4.0, center[1] - 2.0],
            [center[0], center[1] + 3.0],
        )
    } else {
        (
            [center[0] - 2.0, center[1] - 4.0],
            [center[0] + 3.0, center[1]],
            [center[0] - 2.0, center[1] + 4.0],
        )
    };
    ui.get_window_draw_list()
        .add_triangle(a, b, c, ui.get_color_u32(dear_imgui_rs::StyleColor::Text))
        .filled(true)
        .build();
    let mouse_x = ui.io().mouse_pos()[0];
    clicked && mouse_x >= position[0] && mouse_x < position[0] + INDENT
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

fn edit_button(ui: &dear_imgui_rs::Ui, entity: hecs::Entity) -> bool {
    let _id = ui.push_id(&format!("edit_{}", entity.to_bits()));
    let clicked = controls::text_button_colored(
        ui,
        PENCIL,
        [ROW_HEIGHT, ROW_HEIGHT],
        dear_imgui_rs::StyleColor::Text,
    );
    if ui.is_item_hovered() {
        ui.tooltip_text("Edit in the matching canvas");
    }
    clicked
}

fn visibility_button(
    world: &hecs::World,
    ui: &dear_imgui_rs::Ui,
    entity: hecs::Entity,
    visibility: bool,
    effective_visibility: bool,
) -> bool {
    let _id = ui.push_id(&format!("visibility_{}", entity.to_bits()));
    let clicked = controls::text_button_colored(
        ui,
        if visibility { EYE } else { EYE_SLASH },
        [ROW_HEIGHT, ROW_HEIGHT],
        if effective_visibility {
            dear_imgui_rs::StyleColor::Text
        } else {
            dear_imgui_rs::StyleColor::TextDisabled
        },
    );
    if ui.is_item_hovered() {
        ui.tooltip_text(if visibility { "Hide" } else { "Show" });
    }
    if clicked {
        if let Ok(mut draw) = world.get::<&mut Draw2D>(entity) {
            draw.visibility = !draw.visibility;
        } else if let Ok(mut draw) = world.get::<&mut Draw3D>(entity) {
            draw.visibility = !draw.visibility;
        }
    }
    clicked
}

fn object_visibility(world: &hecs::World, entity: hecs::Entity) -> Option<bool> {
    world
        .get::<&Draw2D>(entity)
        .map(|draw| draw.visibility)
        .or_else(|_| world.get::<&Draw3D>(entity).map(|draw| draw.visibility))
        .ok()
}

fn selectable_row(ui: &dear_imgui_rs::Ui, id: String, size: [f32; 2]) -> bool {
    let transparent = [0.0; 4];
    let _header = ui.push_style_color(dear_imgui_rs::StyleColor::Header, transparent);
    let _header_hovered =
        ui.push_style_color(dear_imgui_rs::StyleColor::HeaderHovered, transparent);
    let _header_active = ui.push_style_color(dear_imgui_rs::StyleColor::HeaderActive, transparent);

    ui.selectable_config(id).selected(false).size(size).build()
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
