use crate::{
    core::{
        components::{Name, Node},
        objects::ObjectHandler,
    },
    editor::Editor,
};

const ROW_HEIGHT: f32 = 24.0;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    let is_exporting = editor.is_exporting();
    let selected = editor.get_selected_entity();
    let root = editor.get_scene().get_root().get_id();
    let world = editor.get_scene().get_world();
    let mut clicked = None;
    let mut empty_clicked = false;
    let _text_align = ui.push_style_var(dear_imgui_rs::StyleVar::SelectableTextAlign([0.0, 0.5]));

    ui.window("Scene Tree").build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        let root_name = world
            .get::<&Name>(root)
            .expect("Root group must contain a Name component.");
        let position = ui.cursor_screen_pos();
        let root_clicked = selectable_row(ui, format!("##scene_tree_{}", root.to_bits()));
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

        let children = group_children(&world, root);
        if children.is_empty() {
            ui.text_disabled("   No objects.");
        } else {
            draw_children(
                &world,
                ui,
                &children,
                &mut vec![],
                selected,
                selected == Some(root),
                &mut clicked,
            );
        }

        empty_clicked = !is_exporting
            && ui.is_window_hovered()
            && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left)
            && !ui.is_any_item_hovered();
    });

    drop(world);

    if let Some(entity) = clicked {
        editor.select_entity(entity);
    } else if empty_clicked {
        editor.clear_selection();
    }
}

fn draw_children(
    world: &hecs::World,
    ui: &dear_imgui_rs::Ui,
    children: &[hecs::Entity],
    branches: &mut Vec<bool>,
    selected: Option<hecs::Entity>,
    ancestor_selected: bool,
    clicked: &mut Option<hecs::Entity>,
) {
    for (index, entity) in children.iter().copied().enumerate() {
        let is_last = index + 1 == children.len();
        let is_highlighted = ancestor_selected || selected == Some(entity);
        let entity_children = group_children(world, entity);
        let name = world
            .get::<&Name>(entity)
            .expect("Scene tree object must contain a Name component.");
        let tree = tree_chars(branches, is_last);
        let position = ui.cursor_screen_pos();
        let row_id = format!("##scene_tree_{}", entity.to_bits());
        let was_clicked = selectable_row(ui, row_id);

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

        if was_clicked {
            *clicked = Some(entity);
        }

        if entity_children.is_empty() {
            continue;
        }

        branches.push(!is_last);
        draw_children(
            world,
            ui,
            &entity_children,
            branches,
            selected,
            is_highlighted,
            clicked,
        );
        branches.pop();
    }
}

fn selectable_row(ui: &dear_imgui_rs::Ui, id: String) -> bool {
    let transparent = [0.0; 4];
    let _header = ui.push_style_color(dear_imgui_rs::StyleColor::Header, transparent);
    let _header_hovered =
        ui.push_style_color(dear_imgui_rs::StyleColor::HeaderHovered, transparent);
    let _header_active = ui.push_style_color(dear_imgui_rs::StyleColor::HeaderActive, transparent);

    ui.selectable_config(id)
        .selected(false)
        .size([0.0, ROW_HEIGHT])
        .build()
}

fn group_children(world: &hecs::World, entity: hecs::Entity) -> Vec<hecs::Entity> {
    world
        .get::<&Vec<hecs::Entity>>(entity)
        .map(|children| {
            children
                .iter()
                .copied()
                .filter(|child| {
                    world
                        .get::<&Node>(*child)
                        .expect("Scene tree object must contain a Node component.")
                        .is_activated
                })
                .collect()
        })
        .unwrap_or_default()
}

fn tree_chars(branches: &[bool], is_last: bool) -> String {
    let mut tree = String::new();

    for continues in branches {
        tree.push_str(if *continues { "│  " } else { "   " });
    }

    tree.push_str(if is_last { "└─ " } else { "├─ " });
    tree
}

fn text_size(ui: &dear_imgui_rs::Ui, text: &str) -> [f32; 2] {
    ui.current_font()
        .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, text)
}
