use super::{
    layout::{Layout, TimeRange},
    metrics::{DRAG_DIRECTION_THRESHOLD, PANEL_TEXT_PADDING, TRACK_SPACING},
    tracks,
};
use crate::{
    core::components::{Name, TreeNode},
    editor::Editor,
    ui::{
        icons::{CHEVRON_DOWN, CHEVRON_RIGHT},
        widgets::{draw_panel_rect, text_size},
    },
};

const OBJECT_HEIGHT: f32 = 24.0;
const ARROW_WIDTH: f32 = 20.0;

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    interactive: bool,
) {
    let top = ui.cursor_screen_pos()[1];
    let selected = editor.selected_entity();
    let scene_range = editor.scene_range();
    let expanded = editor.timeline_tracks_expanded();
    let world = editor.scene_mut().world();
    let object = selected.and_then(|entity| {
        world
            .get::<&TreeNode>(entity)
            .ok()
            .and_then(|node| node.is_activated.then_some((entity, node.lifetime)))
    });
    let tracks_height = object.map_or(0.0, |(entity, _)| {
        if expanded {
            tracks::height(&world, entity)
        } else {
            0.0
        }
    });
    let _clip = draw_list.push_clip_rect(
        [layout.content_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );
    let mut toggle = false;

    if let Some((entity, local_lifetime)) = object {
        let lifetime = [
            scene_range[0] + local_lifetime[0],
            (scene_range[0] + local_lifetime[1]).min(scene_range[1]),
        ];
        let start = lifetime[0].max(time.start);
        let end = lifetime[1].min(time.end);
        if end > start {
            let min = [time.x(layout, start), top];
            let max = [time.x(layout, end), top + OBJECT_HEIGHT + tracks_height];
            draw_panel_rect(
                draw_list,
                min,
                max,
                Some(ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg)),
                ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark),
                2.0,
            );

            if interactive {
                ui.set_cursor_screen_pos(min);
                let clicked =
                    ui.invisible_button("##timeline_tracks", [max[0] - min[0], max[1] - min[1]]);
                let drag = ui.mouse_drag_delta(dear_imgui_rs::MouseButton::Left);
                toggle = clicked && drag[0].abs().max(drag[1].abs()) < DRAG_DIRECTION_THRESHOLD;
            }
            let arrow = if expanded {
                CHEVRON_DOWN
            } else {
                CHEVRON_RIGHT
            };
            let name = world
                .get::<&Name>(entity)
                .expect("Timeline object must contain a Name component.");
            let _name_clip = draw_list.push_clip_rect(min, max, true);
            draw_list.add_text(
                [
                    min[0] + PANEL_TEXT_PADDING,
                    top + (OBJECT_HEIGHT - text_size(ui, arrow)[1]) * 0.5,
                ],
                ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
                arrow,
            );
            draw_list.add_text(
                [
                    min[0] + PANEL_TEXT_PADDING + ARROW_WIDTH,
                    top + (OBJECT_HEIGHT - text_size(ui, name.get())[1]) * 0.5,
                ],
                ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
                name.get(),
            );

            if tracks_height > 0.0 {
                let _tracks_clip =
                    draw_list.push_clip_rect([min[0], top + OBJECT_HEIGHT], [max[0], max[1]], true);
                tracks::draw(
                    &world,
                    ui,
                    draw_list,
                    layout,
                    time,
                    tracks::ObjectTracks {
                        entity,
                        lifetime,
                        time_offset: scene_range[0],
                        top: top + OBJECT_HEIGHT + TRACK_SPACING,
                    },
                );
            }
        }
    } else {
        let min = [layout.content_left, top];
        let max = [layout.timeline_right(), top + OBJECT_HEIGHT];
        draw_panel_rect(
            draw_list,
            min,
            max,
            Some(ui.get_color_u32(dear_imgui_rs::StyleColor::WindowBg)),
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
            1.0,
        );
        draw_list.add_text(
            [min[0] + PANEL_TEXT_PADDING, top + 4.0],
            ui.get_color_u32(dear_imgui_rs::StyleColor::TextDisabled),
            "No object selected.",
        );
    }

    ui.set_cursor_screen_pos([layout.content_left, top]);
    ui.dummy([
        layout.timeline_width,
        OBJECT_HEIGHT + TRACK_SPACING + tracks_height,
    ]);

    drop(world);
    if toggle {
        editor.toggle_timeline_tracks();
    }
}
