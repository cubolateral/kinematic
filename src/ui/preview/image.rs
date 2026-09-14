use crate::{
    core::types::{Vector2, vec2},
    editor::Editor,
};

const CANVAS_OUTLINE: [f32; 4] = [0.7, 0.7, 0.7, 1.0];

#[derive(Clone, Copy)]
pub(super) struct PreviewImage {
    size: [f32; 2],
    texture: dear_imgui_rs::TextureId,
}

pub(super) fn preview(editor: &mut Editor) -> PreviewImage {
    image(editor.get_preview())
}

pub(super) fn editor(editor: &mut Editor) -> PreviewImage {
    image(editor.get_editor_2d())
}

pub(super) fn editor_3d(editor: &mut Editor) -> PreviewImage {
    image(editor.get_editor_3d())
}

fn image(canvas: &mut crate::editor::Canvas) -> PreviewImage {
    let (width, height) = canvas.get_size();
    PreviewImage {
        size: [width.max(1) as f32, height.max(1) as f32],
        texture: canvas.get_imgui_texture_id(),
    }
}

pub(super) fn draw(ui: &dear_imgui_rs::Ui, preview: PreviewImage, available: [f32; 2]) {
    let (_, size, min, max) = placement(ui.cursor_screen_pos(), available, preview.size);
    ui.set_cursor_screen_pos(min);
    ui.image_config(preview.texture, size)
        .uv0([0.0, 1.0])
        .uv1([1.0, 0.0])
        .build();
    ui.get_window_draw_list()
        .add_rect(min, max, CANVAS_OUTLINE)
        .thickness(1.0)
        .build();
}

pub(super) fn draw_editor(
    ui: &dear_imgui_rs::Ui,
    preview: PreviewImage,
    available: [f32; 2],
    editor: &mut Editor,
    background: [f32; 4],
) -> Option<Vector2> {
    let origin = ui.cursor_screen_pos();
    let size = [available[0].max(1.0), available[1].max(1.0)];
    let min = origin;
    let max = [min[0] + size[0], min[1] + size[1]];
    let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    editor.set_editor_2d_viewport(size);
    let camera_view = editor.editor_2d_camera_view();
    let display_source = if camera_view {
        let (width, height) = editor.editor_2d_canvas_size();
        [width.max(1) as f32, height.max(1) as f32]
    } else {
        preview.size
    };
    let display_scale = (size[0] / display_source[0]).min(size[1] / display_source[1]);
    ui.invisible_button_options(
        "Editor 2D canvas.",
        size,
        dear_imgui_rs::InvisibleButtonOptions::new()
            .flags(dear_imgui_rs::ButtonFlags::ALLOW_OVERLAP)
            .mouse_buttons(dear_imgui_rs::InvisibleButtonMouseButtons::MIDDLE),
    );

    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let mouse = ui.io().mouse_pos();
    let middle = dear_imgui_rs::MouseButton::Middle;
    if active && !camera_view && ui.is_mouse_dragging(middle) {
        let delta = editor.editor_mouse_delta(ui.io().mouse_delta());
        editor.pan_editor_2d([delta[0] / display_scale, delta[1] / display_scale]);
        wrap_pointer(ui, editor, min, max);
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    }
    if hovered && !camera_view {
        let anchor = [
            (mouse[0] - center[0]) / display_scale,
            (mouse[1] - center[1]) / display_scale,
        ];
        editor.zoom_editor_2d_at(ui.io().mouse_wheel(), anchor);
    }

    let draw_list = ui.get_window_draw_list();
    let _clip = draw_list.push_clip_rect(min, max, true);
    draw_list
        .add_rect(min, max, background)
        .filled(true)
        .build();
    let (pan, zoom) = if camera_view {
        ([0.0; 2], 1.0)
    } else {
        editor.editor_2d_view()
    };
    draw_list.add_image(preview.texture, min, max, [0.0, 1.0], [1.0, 0.0], [1.0; 4]);
    if !camera_view {
        draw_world_outline(
            &draw_list,
            editor.editor_2d_camera_outline(),
            center,
            display_scale,
            pan,
            zoom,
            CANVAS_OUTLINE,
            1.0,
        );
    }
    let selection = editor.editor_2d_selection_outline();
    draw_world_outline(
        &draw_list,
        selection,
        center,
        display_scale,
        pan,
        zoom,
        [0.0, 0.0, 0.0, 1.0],
        3.0,
    );
    draw_world_outline(
        &draw_list,
        selection,
        center,
        display_scale,
        pan,
        zoom,
        [1.0, 1.0, 1.0, 1.0],
        1.0,
    );

    let over_camera = mouse[0] >= min[0] + 8.0
        && mouse[0] <= min[0] + 8.0 + ui.frame_height()
        && mouse[1] >= min[1] + 8.0
        && mouse[1] <= min[1] + 8.0 + ui.frame_height();
    if !hovered || over_camera || !ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left) {
        return None;
    }
    Some(vec2(
        ((mouse[0] - center[0]) / display_scale - pan[0]) / zoom,
        ((mouse[1] - center[1]) / display_scale - pan[1]) / zoom,
    ))
}

pub(super) fn draw_editor_3d(
    ui: &dear_imgui_rs::Ui,
    preview: PreviewImage,
    available: [f32; 2],
    editor: &mut Editor,
) -> (bool, Option<Vector2>) {
    let size = [available[0].max(1.0), available[1].max(1.0)];
    editor.request_editor_3d_size(size);
    ui.invisible_button_options(
        "Editor 3D canvas.",
        size,
        dear_imgui_rs::InvisibleButtonOptions::new()
            .flags(dear_imgui_rs::ButtonFlags::ALLOW_OVERLAP)
            .mouse_buttons(dear_imgui_rs::InvisibleButtonMouseButtons::RIGHT),
    );
    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let min = ui.item_rect_min();
    let max = ui.item_rect_max();
    ui.get_window_draw_list().add_image(
        preview.texture,
        min,
        max,
        [0.0, 1.0],
        [1.0, 0.0],
        [1.0; 4],
    );
    let mouse = ui.io().mouse_pos();
    let over_camera = mouse[0] >= min[0] + 8.0
        && mouse[0] <= min[0] + 8.0 + ui.frame_height()
        && mouse[1] >= min[1] + 8.0
        && mouse[1] <= min[1] + 8.0 + ui.frame_height();
    let over_controls = mouse[0] >= max[0] - 112.0
        && mouse[0] <= max[0]
        && mouse[1] >= min[1] + 8.0
        && mouse[1] <= min[1] + 8.0 + ui.frame_height();
    let clicked = (hovered
        && !over_camera
        && !over_controls
        && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left))
    .then(|| {
        vec2(
            mouse[0] - (min[0] + max[0]) * 0.5,
            mouse[1] - (min[1] + max[1]) * 0.5,
        )
    });
    (active, clicked)
}

pub(super) fn wrap_pointer(
    ui: &dear_imgui_rs::Ui,
    editor: &mut Editor,
    min: [f32; 2],
    max: [f32; 2],
) {
    const EDGE: f32 = 2.0;
    const INSET: f32 = 6.0;

    let mouse = ui.io().mouse_pos();
    let mut target = mouse;
    if mouse[0] <= min[0] + EDGE {
        target[0] = max[0] - INSET;
    } else if mouse[0] >= max[0] - EDGE {
        target[0] = min[0] + INSET;
    }
    if mouse[1] <= min[1] + EDGE {
        target[1] = max[1] - INSET;
    } else if mouse[1] >= max[1] - EDGE {
        target[1] = min[1] + INSET;
    }
    if target == mouse {
        return;
    }

    let viewport = ui.main_viewport().pos();
    editor.request_editor_mouse_warp([target[0] - viewport[0], target[1] - viewport[1]]);
}

fn placement(
    origin: [f32; 2],
    available: [f32; 2],
    source: [f32; 2],
) -> (f32, [f32; 2], [f32; 2], [f32; 2]) {
    let available = [available[0].max(1.0), available[1].max(1.0)];
    let scale = (available[0] / source[0]).min(available[1] / source[1]);
    let size = [source[0] * scale, source[1] * scale];
    let min = [
        origin[0] + (available[0] - size[0]) * 0.5,
        origin[1] + (available[1] - size[1]) * 0.5,
    ];
    (scale, size, min, [min[0] + size[0], min[1] + size[1]])
}

#[allow(clippy::too_many_arguments)]
fn draw_world_outline(
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    points: Option<[skia_safe::Point; 4]>,
    center: [f32; 2],
    display_scale: f32,
    pan: [f32; 2],
    zoom: f32,
    color: [f32; 4],
    thickness: f32,
) {
    let Some(points) = points else {
        return;
    };
    let project = |point: skia_safe::Point| {
        [
            center[0] + (pan[0] + point.x * zoom) * display_scale,
            center[1] + (pan[1] + point.y * zoom) * display_scale,
        ]
    };
    for index in 0..4 {
        draw_list
            .add_line(
                project(points[index]),
                project(points[(index + 1) % 4]),
                color,
            )
            .thickness(thickness)
            .build();
    }
}
