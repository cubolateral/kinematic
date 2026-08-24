use crate::{core::types::vec2, editor::Editor};

const VIEWPORT_OUTLINE_THICKNESS: f32 = 1.0;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    let is_exporting = editor.is_exporting();
    let mut clicked = None;
    let (name, resolution, fps) = {
        let project = editor.get_project();
        (project.name, project.resolution, project.fps)
    };
    let (source_size, texture) = {
        let preview = editor.get_preview();
        let (width, height) = preview.get_size();
        (
            [width.max(1) as f32, height.max(1) as f32],
            preview.get_imgui_texture_id(),
        )
    };

    ui.set_next_window_class(
        &dear_imgui_rs::WindowClass::default()
            .dock_node_flags_override_set(dear_imgui_rs::DockFlags::AUTO_HIDE_TAB_BAR),
    );

    ui.window("Preview").build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        ui.text(format!(
            "[PROJECT INFO] Name: {name} / Resolution: {}x{} / FPS: {fps}",
            resolution.0, resolution.1,
        ));
        ui.separator();

        let available = ui.content_region_avail();
        let scale = (available[0] / source_size[0]).min(available[1] / source_size[1]);

        let size = [
            (source_size[0] * scale).max(1.0),
            (source_size[1] * scale).max(1.0),
        ];

        ui.set_cursor_pos_x(ui.cursor_pos_x() + (available[0] - size[0]) * 0.5);
        ui.set_cursor_pos_y(ui.cursor_pos_y() + (available[1] - size[1]) * 0.5);
        ui.image_config(texture, size)
            .uv0([0.0, 1.0])
            .uv1([1.0, 0.0])
            .build();

        let image_min = ui.item_rect_min();
        let image_max = ui.item_rect_max();

        if ui.is_item_hovered() && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left) {
            let mouse = ui.io().mouse_pos();
            let x = (mouse[0] - image_min[0]) / size[0] * source_size[0] - source_size[0] * 0.5;
            let y = (mouse[1] - image_min[1]) / size[1] * source_size[1] - source_size[1] * 0.5;

            clicked = Some(vec2(x, y));
        }

        let mut min = image_min;
        let mut max = image_max;

        let half = VIEWPORT_OUTLINE_THICKNESS * 0.5;

        min[0] -= half;
        min[1] -= half;
        max[0] += half;
        max[1] += half;

        ui.get_window_draw_list()
            .add_rect(
                min,
                max,
                ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
            )
            .thickness(VIEWPORT_OUTLINE_THICKNESS)
            .build();
    });

    if let Some(point) = clicked {
        editor.select_at(point);
    }
}
