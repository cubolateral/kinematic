use crate::editor::Editor;

pub(crate) struct Ui;

impl Ui {
    pub fn draw(editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        let project_name = editor.get_project().name;
        let (project_width, project_height) = editor.get_project().resolution;

        let (preview_width, preview_height) = editor.get_preview().get_size();
        let (preview_width, preview_height) = (preview_width as f32, preview_height as f32);

        ui.window("Preview").build(|| {
            ui.text(format!(
                "[INFO] Project name: {} / Project resolution: {}x{}",
                project_name, project_width, project_height,
            ));
            ui.separator();

            let available = ui.content_region_avail();
            let aspect = (available[0] / preview_width).min(available[1] / preview_height);
            let (image_width, image_height) = (preview_width * aspect, preview_height * aspect);

            // Centralize preview image.
            ui.set_cursor_pos_x(ui.cursor_pos_x() + (available[0] - image_width) * 0.5);
            ui.set_cursor_pos_y(ui.cursor_pos_y() + (available[1] - image_height) * 0.5);

            // Draw preview image.
            ui.image_config(
                editor.get_preview().get_imgui_texture_id(),
                [image_width, image_height],
            )
            .uv0([0.0, 1.0])
            .uv1([1.0, 0.0])
            .build();
        });
    }
}
