use crate::{
    core::components::{Draw2D, Draw3D, Inspection, Name, TreeNode},
    editor::Editor,
};

use super::{
    controls,
    icons::{EYE, EYE_SLASH, PENCIL},
    widgets::{hierarchy_prefix, text_size},
};

const ROW_HEIGHT: f32 = 24.0;
pub(super) const WINDOW_NAME: &str = "Scene Tree";

struct ObjectRow {
    entity: hecs::Entity,
    branches: Vec<bool>,
    is_last: bool,
    ancestor_selected: bool,
    ancestor_visible: bool,
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    let is_exporting = editor.is_exporting();
    let selected = editor.selected_entity();
    let root = editor.scene_mut().root().entity();
    let world = editor.scene_mut().world();
    let mut clicked = None;
    let mut edit_clicked = None;
    let mut empty_clicked = false;
    let mut visibility_changed = false;
    let _text_align = ui.push_style_var(dear_imgui_rs::StyleVar::SelectableTextAlign([0.0, 0.5]));

    ui.window(WINDOW_NAME).build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
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
                position[0],
                position[1] + (ROW_HEIGHT - text_size(ui, root_name.get())[1]) * 0.5,
            ],
            ui.get_color_u32(if selected == Some(root) {
                dear_imgui_rs::StyleColor::CheckMark
            } else {
                dear_imgui_rs::StyleColor::Text
            }),
            root_name.get(),
        );
        drop(draw_list);

        if root_clicked {
            clicked = Some(root);
        }

        let root_visibility = object_visibility(&world, root).unwrap_or(true);
        let mut rows = vec![];
        collect_rows(
            &world,
            root,
            &mut vec![],
            selected,
            selected == Some(root),
            root_visibility,
            &mut rows,
        );
        if rows.is_empty() {
            ui.text_disabled("   No objects.");
        } else {
            for index in dear_imgui_rs::ListClipper::new(rows.len()).begin(ui).iter() {
                draw_row(
                    &world,
                    ui,
                    &rows[index],
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
    let tree = hierarchy_prefix(&row.branches, row.is_last);
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

    draw_list.add_text(
        [position[0], text_y],
        ui.get_color_u32(if is_highlighted {
            dear_imgui_rs::StyleColor::CheckMark
        } else {
            dear_imgui_rs::StyleColor::TextDisabled
        }),
        &tree,
    );
    draw_list.add_text(
        [position[0] + text_size(ui, &tree)[0], text_y],
        ui.get_color_u32(if is_highlighted {
            dear_imgui_rs::StyleColor::CheckMark
        } else {
            dear_imgui_rs::StyleColor::Text
        }),
        name.get(),
    );
    drop(draw_list);
    drop(name);
    ui.same_line_with_spacing(0.0, 0.0);
    if edit_button(ui, entity) {
        *edit_clicked = Some(entity);
    }
    if let Some(visibility) = visibility {
        ui.same_line_with_spacing(0.0, 0.0);
        *visibility_changed |=
            visibility_button(world, ui, entity, visibility, effective_visibility);
    }

    if was_clicked {
        *clicked = Some(entity);
    }
}

fn collect_rows(
    world: &hecs::World,
    parent: hecs::Entity,
    branches: &mut Vec<bool>,
    selected: Option<hecs::Entity>,
    ancestor_selected: bool,
    ancestor_visible: bool,
    rows: &mut Vec<ObjectRow>,
) {
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
