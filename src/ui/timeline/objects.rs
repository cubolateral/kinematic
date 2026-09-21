use super::{
    layout::{Layout, TimeRange},
    metrics::{PANEL_TEXT_PADDING, TRACK_SPACING, TRACK_TEXT_INDENT},
    state::State,
    tracks,
};
use crate::{
    core::components::{Name, TreeNode},
    editor::Editor,
    ui::widgets::{draw_panel_rect, hierarchy_prefix, text_size},
};

const OBJECT_HEIGHT: f32 = 24.0;
const OBJECT_SEPARATOR_GAP: f32 = 8.0;

struct ObjectRow {
    entity: hecs::Entity,
    lifetime: [f32; 2],
    branches: Vec<bool>,
    is_last: bool,
    is_highlighted: bool,
    tracks_height: f32,
}

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    state: &mut State,
    interactive: bool,
) {
    let origin = ui.cursor_screen_pos();
    let selected = editor.selected_entity();
    let scene_range = editor.scene_range();
    let root = editor.scene_mut().root().entity();
    let world = editor.scene_mut().world();
    let root_lifetime = {
        let root_node = world
            .get::<&TreeNode>(root)
            .expect("Timeline root must contain a Node component.");
        root_node.lifetime
    };
    let mut objects = vec![ObjectRow {
        entity: root,
        lifetime: root_lifetime,
        branches: vec![],
        is_last: true,
        is_highlighted: selected == Some(root),
        tracks_height: expanded_tracks_height(&world, state, root),
    }];

    collect_rows(
        &world,
        root,
        &mut vec![false],
        selected,
        selected == Some(root),
        state,
        &mut objects,
    );

    let height = objects
        .iter()
        .map(|object| OBJECT_HEIGHT + TRACK_SPACING + object.tracks_height)
        .sum::<f32>();

    ui.dummy([
        layout.timeline_right() - layout.content_left,
        height.max(1.0),
    ]);

    let clip = draw_list.push_clip_rect(
        [layout.content_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );

    let mut hovered_object = None;
    let mut top = origin[1];

    for object in objects {
        let tracks_height = object.tracks_height;
        let bottom = top + OBJECT_HEIGHT + tracks_height;
        if !vertically_visible(top, bottom, layout.viewport_top, layout.bottom) {
            top += OBJECT_HEIGHT + TRACK_SPACING + tracks_height;
            continue;
        }

        let lifetime = [
            scene_range[0] + object.lifetime[0],
            (scene_range[0] + object.lifetime[1]).min(scene_range[1]),
        ];
        let start = lifetime[0].max(time.start);
        let end = lifetime[1].min(time.end);
        let sidebar_min = [layout.content_left, top];
        let sidebar_max = [layout.divider_x, bottom];
        let view_hovered = ui.is_window_hovered();
        let sidebar_hovered = view_hovered && ui.is_mouse_hovering_rect(sidebar_min, sidebar_max);
        let timeline_min = [time.x(layout, start), top];
        let timeline_max = [time.x(layout, end), bottom];
        let timeline_hovered =
            end > start && view_hovered && ui.is_mouse_hovering_rect(timeline_min, timeline_max);
        let hovered = sidebar_hovered || timeline_hovered;
        let is_highlighted = object.is_highlighted;

        if hovered {
            hovered_object = Some(object.entity);
        }

        if end > start {
            let fill = if is_highlighted {
                Some(ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg))
            } else if timeline_hovered {
                Some(ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBgHovered))
            } else {
                Some(ui.get_color_u32(dear_imgui_rs::StyleColor::WindowBg))
            };

            let border = ui.get_color_u32(if is_highlighted {
                dear_imgui_rs::StyleColor::CheckMark
            } else {
                dear_imgui_rs::StyleColor::Border
            });
            let border_thickness = if is_highlighted { 2.0 } else { 1.0 };

            draw_panel_rect(
                draw_list,
                timeline_min,
                timeline_max,
                fill,
                border,
                border_thickness,
            );
        }

        let tree = hierarchy_prefix(&object.branches, object.is_last);
        let name = world
            .get::<&Name>(object.entity)
            .expect("Timeline object must contain a Name component.");
        let label_height = text_size(ui, name.get())[1];
        let tree_width = text_size(ui, &tree)[0];
        let tree_position = [
            layout.content_left + PANEL_TEXT_PADDING,
            top + (OBJECT_HEIGHT - label_height) * 0.5,
        ];
        let name_position = [tree_position[0] + tree_width, tree_position[1]];
        let panel_min = [layout.content_left + PANEL_TEXT_PADDING, top];
        let panel_max = [layout.divider_x - PANEL_TEXT_PADDING, top + OBJECT_HEIGHT];

        let text_clip = draw_list.push_clip_rect(panel_min, panel_max, true);

        draw_list.add_text(
            tree_position,
            ui.get_color_u32(if is_highlighted {
                dear_imgui_rs::StyleColor::CheckMark
            } else {
                dear_imgui_rs::StyleColor::TextDisabled
            }),
            &tree,
        );
        draw_list.add_text(
            name_position,
            ui.get_color_u32(if is_highlighted {
                dear_imgui_rs::StyleColor::CheckMark
            } else {
                dear_imgui_rs::StyleColor::Text
            }),
            name.get(),
        );

        let separator_start =
            name_position[0] + text_size(ui, name.get())[0] + OBJECT_SEPARATOR_GAP;
        let separator_end = panel_max[0];
        if separator_end > separator_start {
            draw_list.add_line_h(
                separator_start,
                separator_end,
                top + OBJECT_HEIGHT * 0.5,
                ui.get_color_u32(if is_highlighted {
                    dear_imgui_rs::StyleColor::CheckMark
                } else {
                    dear_imgui_rs::StyleColor::Separator
                }),
                1.0,
            );
        }

        drop(text_clip);
        drop(name);

        top += OBJECT_HEIGHT + TRACK_SPACING;

        if tracks_height > 0.0 {
            tracks::draw(
                &world,
                ui,
                draw_list,
                layout,
                time,
                tracks::ObjectTracks {
                    entity: object.entity,
                    lifetime,
                    time_offset: scene_range[0],
                    top,
                    name_x: name_position[0] + TRACK_TEXT_INDENT,
                },
            );
            top += tracks_height;
        }
    }

    drop(clip);
    drop(world);

    if !interactive {
        return;
    }

    let left = dear_imgui_rs::MouseButton::Left;

    let view_hovered = ui.is_window_hovered();

    if view_hovered && ui.is_mouse_clicked(left) {
        let double_clicked = ui.is_mouse_double_clicked(left) && hovered_object.is_some();

        state.press_entity(hovered_object, double_clicked);
    }

    if let Some((entity, toggle)) = ui
        .is_mouse_released(left)
        .then(|| {
            state.release_entity(
                hovered_object,
                hovered_object.is_some(),
                view_hovered,
                ui.mouse_drag_delta(left),
            )
        })
        .flatten()
    {
        match entity {
            Some(entity) if toggle => state.toggle_object(entity),
            Some(entity) => editor.select_entity(entity),
            None => editor.clear_selection(),
        }
    }
}

fn collect_rows(
    world: &hecs::World,
    parent: hecs::Entity,
    branches: &mut Vec<bool>,
    selected: Option<hecs::Entity>,
    ancestor_selected: bool,
    state: &State,
    rows: &mut Vec<ObjectRow>,
) {
    let children = world
        .get::<&TreeNode>(parent)
        .expect("Timeline parent must contain a Node component.")
        .children
        .clone()
        .unwrap_or_default();

    for (index, entity) in children.iter().copied().enumerate() {
        let is_last = index + 1 == children.len();
        let is_highlighted = ancestor_selected || selected == Some(entity);
        let node = world
            .get::<&TreeNode>(entity)
            .expect("Timeline object must contain a Node component.");
        let has_children = node
            .children
            .as_ref()
            .is_some_and(|children| !children.is_empty());

        if node.lifetime[0].is_finite() {
            rows.push(ObjectRow {
                entity,
                lifetime: node.lifetime,
                branches: branches.clone(),
                is_last,
                is_highlighted,
                tracks_height: expanded_tracks_height(world, state, entity),
            });
        }

        if !has_children {
            continue;
        }

        branches.push(!is_last);
        collect_rows(
            world,
            entity,
            branches,
            selected,
            is_highlighted,
            state,
            rows,
        );
        branches.pop();
    }
}

fn expanded_tracks_height(world: &hecs::World, state: &State, entity: hecs::Entity) -> f32 {
    if state.is_object_expanded(entity) {
        tracks::height(world, entity)
    } else {
        0.0
    }
}

fn vertically_visible(top: f32, bottom: f32, viewport_top: f32, viewport_bottom: f32) -> bool {
    bottom >= viewport_top && top <= viewport_bottom
}

#[cfg(test)]
mod tests {
    use super::vertically_visible;

    #[test]
    fn vertical_culling_keeps_only_rows_touching_the_viewport() {
        assert!(!vertically_visible(0.0, 20.0, 21.0, 100.0));
        assert!(vertically_visible(20.0, 40.0, 21.0, 100.0));
        assert!(!vertically_visible(101.0, 120.0, 21.0, 100.0));
    }
}
