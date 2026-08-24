use super::{Layout, SCRUBBER_HEIGHT, State, TRACK_SPACING, TimeRange, text_size};
use crate::{
    core::components::{Name, Node},
    core::objects::ObjectHandler,
    editor::Editor,
};

const ENTITY_HEIGHT: f32 = 24.0;
const SELECTED_ACCENT_ALPHA: f32 = 0.35;

struct EntityRow {
    entity: hecs::Entity,
    lifetime: [f32; 2],
    name: String,
    is_group: bool,
    branches: Vec<bool>,
    is_last: bool,
}

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    state: &mut State,
) {
    ui.set_cursor_screen_pos([layout.timeline_left, layout.top + SCRUBBER_HEIGHT]);
    ui.separator();
    ui.dummy([0.0, TRACK_SPACING]);

    let origin = ui.cursor_screen_pos();
    let selected = editor.get_selected_entity();
    let root = editor.get_scene().get_root().get_id();
    let world = editor.get_scene().get_world();
    let mut entities = vec![];

    collect_rows(&world, root, &mut vec![], &mut entities);

    let height = entities.len() as f32 * (ENTITY_HEIGHT + TRACK_SPACING);

    ui.dummy([layout.timeline_width, height.max(1.0)]);

    let clip = draw_list.push_clip_rect(
        [layout.timeline_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );

    let mut hovered_entity = None;

    for (row, entity_row) in entities.into_iter().enumerate() {
        let start = entity_row.lifetime[0].max(time.start);
        let end = entity_row.lifetime[1].min(time.end);

        if end <= start {
            continue;
        }

        let top = origin[1] + row as f32 * (ENTITY_HEIGHT + TRACK_SPACING);
        let min = [time.x(layout, start), top];
        let max = [time.x(layout, end), top + ENTITY_HEIGHT];
        let hovered = ui.is_window_hovered() && ui.is_mouse_hovering_rect(min, max);
        let is_selected = selected == Some(entity_row.entity);

        if hovered {
            hovered_entity = Some(entity_row.entity);
        }

        let color = if is_selected {
            ui.get_color_u32_with_alpha(dear_imgui_rs::StyleColor::CheckMark, SELECTED_ACCENT_ALPHA)
        } else if entity_row.is_group {
            if hovered {
                ui.get_color_u32(dear_imgui_rs::StyleColor::HeaderActive)
            } else {
                ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBgHovered)
            }
        } else if hovered {
            ui.get_color_u32(dear_imgui_rs::StyleColor::ButtonHovered)
        } else {
            ui.get_color_u32(dear_imgui_rs::StyleColor::Button)
        };

        draw_list
            .add_rect(min, max, color)
            .filled(true)
            .rounding(3.0)
            .build();
        draw_list
            .add_rect(
                min,
                max,
                ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
            )
            .rounding(3.0)
            .build();

        let name = tree_name(&entity_row.branches, entity_row.is_last, &entity_row.name);
        let name_size = text_size(ui, &name);

        let name_position = [min[0] + 6.0, top + (ENTITY_HEIGHT - name_size[1]) * 0.5];
        let text_clip = draw_list.push_clip_rect(min, max, true);

        draw_list.add_text(
            name_position,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            &name,
        );

        drop(text_clip);
    }

    drop(clip);
    drop(world);

    let left = dear_imgui_rs::MouseButton::Left;

    if ui.is_mouse_clicked(left) {
        state.press_entity(hovered_entity);
    }

    if let Some(entity) = ui
        .is_mouse_released(left)
        .then(|| state.release_entity(hovered_entity, ui.mouse_drag_delta(left)))
        .flatten()
    {
        editor.select_entity(entity);
    }
}

fn collect_rows(
    world: &hecs::World,
    group: hecs::Entity,
    branches: &mut Vec<bool>,
    rows: &mut Vec<EntityRow>,
) {
    let children = world
        .get::<&Vec<hecs::Entity>>(group)
        .map(|children| (*children).clone())
        .unwrap_or_default();

    for (index, entity) in children.iter().copied().enumerate() {
        let is_last = index + 1 == children.len();
        let node = world
            .get::<&Node>(entity)
            .expect("Timeline object must contain a Node component.");
        let name = world
            .get::<&Name>(entity)
            .expect("Timeline object must contain a Name component.");
        let is_group = world.get::<&Vec<hecs::Entity>>(entity).is_ok();

        if node.lifetime[0].is_finite() {
            rows.push(EntityRow {
                entity,
                lifetime: node.lifetime,
                name: name.get().to_owned(),
                is_group,
                branches: branches.clone(),
                is_last,
            });
        }

        if !is_group {
            continue;
        }

        branches.push(!is_last);
        collect_rows(world, entity, branches, rows);
        branches.pop();
    }
}

fn tree_name(branches: &[bool], is_last: bool, name: &str) -> String {
    let mut label = String::new();

    for continues in branches {
        label.push_str(if *continues { "│  " } else { "   " });
    }

    label.push_str(if is_last { "└─ " } else { "├─ " });
    label.push_str(name);
    label
}

#[cfg(test)]
mod tests {
    use crate::core::{Scene, objects::*};

    use super::*;

    #[test]
    fn rows_follow_the_scene_tree_preorder() {
        let mut scene = Scene::new();
        let group = scene.create::<Group>().name("Group").build();
        let child = scene.create::<Circle>().name("Child").build();
        let sibling = scene.create::<Rect>().name("Sibling").build();
        let root = scene.get_root();

        group.add(&child);
        root.add(&group);
        root.add(&sibling);

        let world = scene.get_world();
        let mut rows = vec![];

        collect_rows(&world, root.get_id(), &mut vec![], &mut rows);

        assert_eq!(
            rows.iter()
                .map(|row| (row.name.as_str(), row.branches.len()))
                .collect::<Vec<_>>(),
            [("Group", 0), ("Child", 1), ("Sibling", 0)]
        );
        assert_eq!(
            tree_name(&rows[1].branches, rows[1].is_last, "Child"),
            "│  └─ Child"
        );
    }
}
