use crate::{
    core::{
        objects::CanvasFilter,
        types::{Vector2, vec2},
    },
    editor::Editor,
};

pub(super) use kinematic_ui::canvas::{Image as PreviewImage, draw, draw_navigable};

pub(super) fn preview(editor: &mut Editor) -> PreviewImage {
    image(editor.preview_mut())
}

pub(super) fn editor(editor: &mut Editor) -> PreviewImage {
    image(editor.editor_2d_mut())
}

pub(super) fn editor_3d(editor: &mut Editor) -> PreviewImage {
    image(editor.editor_3d_mut())
}

fn image(canvas: &mut crate::editor::Canvas) -> PreviewImage {
    let (width, height) = canvas.size();

    PreviewImage {
        size: [width.max(1) as f32, height.max(1) as f32],
        texture: canvas.imgui_texture_id(),
        nearest: canvas.target.filter() == CanvasFilter::Nearest,
    }
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

    let (pan, zoom) = if camera_view {
        ([0.0; 2], 1.0)
    } else {
        editor.editor_2d_view()
    };
    kinematic_ui::canvas::draw_editor_2d(
        ui,
        &kinematic_ui::canvas::Editor2D {
            image: preview,
            bounds: [min, max],
            display_scale,
            view: kinematic_ui::canvas::View { pan, zoom },
            background,
            axes: editor.editor_2d_axes(),
            project: points(editor.editor_2d_project_outline()),
            camera: (!camera_view)
                .then(|| editor.editor_2d_camera_outline())
                .flatten()
                .map(points),
            selection: editor.editor_2d_selection_outline().map(points),
            margin: editor.margin_visible().then(|| editor.margin_size()),
        },
    );

    let over_camera = mouse[0] >= min[0] + 8.0
        && mouse[0] <= min[0] + 24.0 + ui.frame_height() * 3.0
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
    editor.request_editor_3d_size([available[0].max(1.0), available[1].max(1.0)]);
    let (active, clicked) = kinematic_ui::canvas::draw_editor_3d(ui, preview, available);
    (active, clicked.map(|[x, y]| vec2(x, y)))
}

pub(super) fn wrap_pointer(
    ui: &dear_imgui_rs::Ui,
    editor: &mut Editor,
    min: [f32; 2],
    max: [f32; 2],
) {
    let Some(target) = kinematic_ui::canvas::wrap_target(ui.io().mouse_pos(), min, max) else {
        return;
    };
    let viewport = ui.main_viewport().pos();

    editor.request_editor_mouse_warp([target[0] - viewport[0], target[1] - viewport[1]]);
}

fn points(points: [skia_safe::Point; 4]) -> kinematic_ui::canvas::Quad {
    points.map(|point| [point.x, point.y])
}
