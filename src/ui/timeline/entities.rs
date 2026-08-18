use super::{Layout, SCRUBBER_HEIGHT, TRACK_SPACING, TimeRange, text_size};
use crate::{
    core::components::{Inspection, Node},
    editor::Editor,
};

const ENTITY_HEIGHT: f32 = 24.0;

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
) {
    ui.set_cursor_screen_pos([layout.timeline_left, layout.top + SCRUBBER_HEIGHT]);
    ui.separator();
    ui.dummy([0.0, TRACK_SPACING]);

    let origin = ui.cursor_screen_pos();
    let world = editor.get_scene().get_world();
    let mut query = world.query::<(&Node, &Inspection)>();
    let entities: Vec<_> = query
        .iter()
        .filter(|(node, _)| node.lifetime[0].is_finite())
        .collect();

    let height = entities.len() as f32 * (ENTITY_HEIGHT + TRACK_SPACING);
    ui.dummy([layout.timeline_width, height.max(1.0)]);

    let clip = draw_list.push_clip_rect(
        [layout.timeline_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );

    for (row, (node, inspection)) in entities.into_iter().enumerate() {
        let start = node.lifetime[0].max(time.start);
        let end = node.lifetime[1].min(time.end);
        if end <= start {
            continue;
        }

        let top = origin[1] + row as f32 * (ENTITY_HEIGHT + TRACK_SPACING);
        let min = [time.x(layout, start), top];
        let max = [time.x(layout, end), top + ENTITY_HEIGHT];
        let hovered = ui.is_window_hovered() && ui.is_mouse_hovering_rect(min, max);
        let color = ui.get_color_u32(if hovered {
            dear_imgui_rs::StyleColor::ButtonHovered
        } else {
            dear_imgui_rs::StyleColor::Button
        });

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

        let name = inspection.object_name;
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
}
