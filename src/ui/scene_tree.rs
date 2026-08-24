use crate::{
    core::{
        components::{Name, Node},
        objects::ObjectHandler,
    },
    editor::Editor,
};

const ROW_HEIGHT: f32 = 24.0;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    let selected = editor.get_selected_entity();
    let root = editor.get_scene().get_root().get_id();
    let world = editor.get_scene().get_world();
    let mut clicked = None;
    let _text_align = ui.push_style_var(dear_imgui_rs::StyleVar::SelectableTextAlign([0.0, 0.5]));

    ui.window("Scene Tree").build(|| {
        let root_name = world
            .get::<&Name>(root)
            .expect("Root group must contain a Name component.");
        let root_label = format!("{}##scene_tree_{}", root_name.get(), root.to_bits());

        let root_clicked = if selected == Some(root) {
            let mut accent = ui.style_color(dear_imgui_rs::StyleColor::CheckMark);
            accent[3] = 0.24;
            let _header = ui.push_style_color(dear_imgui_rs::StyleColor::Header, accent);
            let _header_hovered =
                ui.push_style_color(dear_imgui_rs::StyleColor::HeaderHovered, accent);
            let _header_active =
                ui.push_style_color(dear_imgui_rs::StyleColor::HeaderActive, accent);

            ui.selectable_config(root_label)
                .selected(true)
                .size([0.0, ROW_HEIGHT])
                .build()
        } else {
            ui.selectable_config(root_label)
                .selected(false)
                .size([0.0, ROW_HEIGHT])
                .build()
        };

        if root_clicked {
            clicked = Some(root);
        }

        let children = group_children(&world, root);
        if children.is_empty() {
            ui.text_disabled("   No objects.");
            return;
        }

        draw_children(&world, ui, &children, &mut vec![], selected, &mut clicked);
    });

    drop(world);

    if let Some(entity) = clicked {
        editor.select_entity(entity);
    }
}

fn draw_children(
    world: &hecs::World,
    ui: &dear_imgui_rs::Ui,
    children: &[hecs::Entity],
    branches: &mut Vec<bool>,
    selected: Option<hecs::Entity>,
    clicked: &mut Option<hecs::Entity>,
) {
    for (index, entity) in children.iter().copied().enumerate() {
        let is_last = index + 1 == children.len();
        let entity_children = group_children(world, entity);
        let name = world
            .get::<&Name>(entity)
            .expect("Scene tree object must contain a Name component.");
        let label = row_label(branches, is_last, name.get(), entity);
        let was_clicked = {
            if selected == Some(entity) {
                let mut accent = ui.style_color(dear_imgui_rs::StyleColor::CheckMark);
                accent[3] = 0.24;
                let _header = ui.push_style_color(dear_imgui_rs::StyleColor::Header, accent);
                let _header_hovered =
                    ui.push_style_color(dear_imgui_rs::StyleColor::HeaderHovered, accent);
                let _header_active =
                    ui.push_style_color(dear_imgui_rs::StyleColor::HeaderActive, accent);

                ui.selectable_config(label)
                    .selected(true)
                    .size([0.0, ROW_HEIGHT])
                    .build()
            } else {
                ui.selectable_config(label)
                    .selected(false)
                    .size([0.0, ROW_HEIGHT])
                    .build()
            }
        };

        if was_clicked {
            *clicked = Some(entity);
        }

        if entity_children.is_empty() {
            continue;
        }

        branches.push(!is_last);
        draw_children(world, ui, &entity_children, branches, selected, clicked);
        branches.pop();
    }
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

fn row_label(branches: &[bool], is_last: bool, name: &str, entity: hecs::Entity) -> String {
    let mut label = String::new();

    for continues in branches {
        label.push_str(if *continues { "│  " } else { "   " });
    }

    label.push_str(if is_last { "└─ " } else { "├─ " });
    label.push_str(name);
    label.push_str(&format!("##scene_tree_{}", entity.to_bits()));
    label
}

#[cfg(test)]
mod tests {
    use crate::core::{Scene, objects::*};

    use super::*;

    #[test]
    fn labels_preserve_tree_connectors_and_entity_identity() {
        let label = row_label(&[true, false], false, "Group", hecs::Entity::DANGLING);

        assert!(label.starts_with("│     ├─ Group##scene_tree_"));
    }

    #[test]
    fn labels_do_not_require_object_type_markers() {
        let label = row_label(&[], true, "Circle", hecs::Entity::DANGLING);

        assert!(label.starts_with("└─ Circle##scene_tree_"));
    }

    #[test]
    fn inactive_objects_are_omitted_from_the_tree() {
        let mut scene = Scene::new();
        let circle = scene.create::<Circle>().build();
        let root = scene.get_root();

        root.add(&circle);
        assert_eq!(
            group_children(&scene.get_world(), root.get_id()),
            [circle.get_id()]
        );

        root.remove(&circle);
        assert!(group_children(&scene.get_world(), root.get_id()).is_empty());
    }
}
