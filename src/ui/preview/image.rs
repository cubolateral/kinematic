use crate::{
    core::types::{Vector2, vec2},
    editor::Editor,
};

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

pub(super) fn draw(
    ui: &dear_imgui_rs::Ui,
    preview: PreviewImage,
    available: [f32; 2],
    interactive: bool,
) -> Option<Vector2> {
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

    if !interactive {
        return None;
    }

    let image_min = ui.item_rect_min();
    let image_max = ui.item_rect_max();
    let clicked = if ui.is_item_hovered() && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left) {
        let mouse = ui.io().mouse_pos();
        let x = (mouse[0] - image_min[0]) / size[0] * preview.size[0] - preview.size[0] * 0.5;
        let y = (mouse[1] - image_min[1]) / size[1] * preview.size[1] - preview.size[1] * 0.5;

        Some(vec2(x, y))
    } else {
        None
    };
    let half = VIEWPORT_OUTLINE_THICKNESS * 0.5;
    let min = [image_min[0] - half, image_min[1] - half];
    let max = [image_max[0] + half, image_max[1] + half];

    ui.get_window_draw_list()
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .thickness(VIEWPORT_OUTLINE_THICKNESS)
        .build();

    clicked
}
