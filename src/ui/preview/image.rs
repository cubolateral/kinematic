use crate::{
    core::types::{Vector2, vec2},
    editor::Editor,
};

use super::State;

const VIEWPORT_OUTLINE_THICKNESS: f32 = 1.0;

#[derive(Clone, Copy)]
pub(super) struct PreviewImage {
    size: [f32; 2],
    texture: dear_imgui_rs::TextureId,
}

pub(super) fn preview(editor: &mut Editor) -> PreviewImage {
    let preview = editor.get_preview();
    let (width, height) = preview.get_size();

    PreviewImage {
        size: [width.max(1) as f32, height.max(1) as f32],
        texture: preview.get_imgui_texture_id(),
    }
}

pub(super) fn draw(ui: &dear_imgui_rs::Ui, preview: PreviewImage, available: [f32; 2]) {
    let scale =
        (available[0].max(1.0) / preview.size[0]).min(available[1].max(1.0) / preview.size[1]);
    let size = [preview.size[0] * scale, preview.size[1] * scale];
    let origin = ui.cursor_screen_pos();

    ui.set_cursor_screen_pos([
        origin[0] + (available[0] - size[0]) * 0.5,
        origin[1] + (available[1] - size[1]) * 0.5,
    ]);
    ui.image_config(preview.texture, size)
        .uv0([0.0, 1.0])
        .uv1([1.0, 0.0])
        .build();

    let image_min = ui.item_rect_min();
    let image_max = ui.item_rect_max();
    let draw_list = ui.get_window_draw_list();
    draw_outline(ui, &draw_list, image_min, image_max);
}

pub(super) fn draw_interactive(
    ui: &dear_imgui_rs::Ui,
    preview: PreviewImage,
    available: [f32; 2],
    state: &mut State,
) -> Option<Vector2> {
    let available = [available[0].max(1.0), available[1].max(1.0)];
    let viewport_min = ui.cursor_screen_pos();
    let viewport_max = [
        viewport_min[0] + available[0],
        viewport_min[1] + available[1],
    ];
    let viewport_center = [
        viewport_min[0] + available[0] * 0.5,
        viewport_min[1] + available[1] * 0.5,
    ];

    ui.invisible_button("Preview canvas.", available);

    let hovered = ui.is_item_hovered();
    let mouse = ui.io().mouse_pos();
    let left = dear_imgui_rs::MouseButton::Left;

    if hovered && ui.is_mouse_clicked(left) {
        state.press();
    }
    if ui.is_mouse_down(left) {
        state.drag(ui.mouse_drag_delta(left), ui.io().mouse_delta());
    }

    if hovered {
        state.zoom_at(
            ui.io().mouse_wheel(),
            [mouse[0] - viewport_center[0], mouse[1] - viewport_center[1]],
        );
    }

    if state.is_panning() {
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    }

    let fit_scale = (available[0] / preview.size[0]).min(available[1] / preview.size[1]);
    let scale = fit_scale * state.zoom();
    let size = [preview.size[0] * scale, preview.size[1] * scale];
    let pan = state.pan();
    let image_min = [
        viewport_center[0] + pan[0] - size[0] * 0.5,
        viewport_center[1] + pan[1] - size[1] * 0.5,
    ];
    let image_max = [image_min[0] + size[0], image_min[1] + size[1]];

    let draw_list = ui.get_window_draw_list();
    let _clip = draw_list.push_clip_rect(viewport_min, viewport_max, true);
    draw_list.add_image(
        preview.texture,
        image_min,
        image_max,
        [0.0, 1.0],
        [1.0, 0.0],
        [1.0, 1.0, 1.0, 1.0],
    );
    draw_outline(ui, &draw_list, image_min, image_max);

    let over_image = hovered
        && (image_min[0]..=image_max[0]).contains(&mouse[0])
        && (image_min[1]..=image_max[1]).contains(&mouse[1]);
    if !ui.is_mouse_released(left) || !state.release(over_image) {
        return None;
    }

    let x = (mouse[0] - image_min[0]) / scale - preview.size[0] * 0.5;
    let y = (mouse[1] - image_min[1]) / scale - preview.size[1] * 0.5;

    Some(vec2(x, y))
}

fn draw_outline(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    image_min: [f32; 2],
    image_max: [f32; 2],
) {
    let half = VIEWPORT_OUTLINE_THICKNESS * 0.5;
    let min = [image_min[0] - half, image_min[1] - half];
    let max = [image_max[0] + half, image_max[1] + half];

    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .thickness(VIEWPORT_OUTLINE_THICKNESS)
        .build();
}
