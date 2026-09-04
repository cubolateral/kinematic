use super::{
    layout::{Layout, TimeRange},
    metrics::{PANEL_TEXT_PADDING, TRACK_SPACING},
};
use crate::{
    editor::Editor,
    ui::widgets::{draw_panel_rect, text_size},
};

const SCENE_HEIGHT: f32 = 32.0;

pub(super) fn draw(
    editor: &Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
) {
    ui.dummy([0.0, TRACK_SPACING]);

    let top = ui.cursor_screen_pos()[1];
    ui.dummy([layout.timeline_right() - layout.content_left, SCENE_HEIGHT]);

    let clip = draw_list.push_clip_rect(
        [layout.timeline_left, layout.viewport_top],
        [layout.timeline_right(), layout.bottom],
        true,
    );
    let active_scene = editor.get_active_scene_index();

    for (index, (name, range)) in editor.get_scenes().enumerate() {
        let start = range[0].max(time.start);
        let end = range[1].min(time.end);
        if end <= start {
            continue;
        }

        let min = [time.x(layout, start), top];
        let max = [time.x(layout, end), top + SCENE_HEIGHT];
        let fill = if index == active_scene {
            active_scene_color(ui)
        } else {
            ui.get_color_u32(dear_imgui_rs::StyleColor::WindowBg)
        };

        draw_panel_rect(
            draw_list,
            min,
            max,
            Some(fill),
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        );

        let text_height = text_size(ui, name)[1];
        let text_clip = draw_list.push_clip_rect(min, max, true);
        draw_list.add_text(
            [
                min[0] + PANEL_TEXT_PADDING,
                top + (SCENE_HEIGHT - text_height) * 0.5,
            ],
            ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            name,
        );
        drop(text_clip);
    }

    drop(clip);
}

fn active_scene_color(ui: &dear_imgui_rs::Ui) -> u32 {
    ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg)
}
