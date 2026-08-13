use crate::editor::Editor;

pub(crate) struct Ui;

impl Ui {
    const BUTTON_SIZE: f32 = 25.0;
}

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

        ui.window("Timeline").build(|| {
            let current_time = editor.get_timeline().get_current_time();
            let max_time = editor.get_timeline().get_max_time();

            let spacing = unsafe { ui.style().item_spacing() }[0];
            let buttons_width = Self::BUTTON_SIZE * 3.0 + spacing * 2.0;
            let available_width = ui.content_region_avail_width();

            // Centralize buttons.
            ui.set_cursor_pos_x(
                ui.cursor_pos_x() + 0.0_f32.max((available_width - buttons_width) * 0.5),
            );

            if ui
                .button_config("<<")
                .size([Self::BUTTON_SIZE, Self::BUTTON_SIZE])
                .build()
                || ui.is_key_pressed(dear_imgui_rs::Key::LeftArrow)
            {
                editor.get_timeline().go_to_start();
            }

            ui.same_line();

            if ui
                .button_config(if editor.get_timeline().is_playing() {
                    "||"
                } else {
                    "|>"
                })
                .size([Self::BUTTON_SIZE, Self::BUTTON_SIZE])
                .build()
                || ui.is_key_pressed(dear_imgui_rs::Key::Space)
            {
                editor.get_timeline().toggle();
            }

            ui.same_line();

            if ui
                .button_config(">>")
                .size([Self::BUTTON_SIZE, Self::BUTTON_SIZE])
                .build()
                || ui.is_key_pressed(dear_imgui_rs::Key::RightArrow)
            {
                editor.get_timeline().go_to_end();
            }

            ui.dummy([0.0, 8.0]);
            ui.separator();
            ui.dummy([0.0, 8.0]);

            // Scrubber.
            let screen_position = ui.cursor_screen_pos();
            let mut available = ui.content_region_avail();
            available[1] = available[1].max(50.0);

            ui.invisible_button("scrubber_area", available);

            let is_controlling = ui.is_item_active();
            editor.get_timeline().is_controlling = is_controlling;

            if is_controlling {
                editor.get_timeline().go_to(
                    ((ui.io().mouse_pos()[0] - screen_position[0]) / available[0]).clamp(0.0, 1.0)
                        * max_time,
                );
            }

            let draw_list = ui.get_window_draw_list();
            let text_color = ui.get_color_u32(dear_imgui_rs::StyleColor::Text);
            let text_y = screen_position[1];
            let line_y = text_y + 32.0;

            // End time label.
            let text = format!("{:.2}s", max_time);
            let text_width =
                ui.current_font()
                    .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, &text)[0];

            draw_list.add_text(
                [screen_position[0] + available[0] - text_width, text_y],
                text_color,
                text,
            );

            // Scrubber line and knob.
            let knob_position = [
                screen_position[0] + available[0] * (current_time / max_time),
                line_y,
            ];

            draw_list
                .add_line(
                    knob_position,
                    [screen_position[0] + available[0], line_y],
                    ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg),
                )
                .thickness(2.0)
                .build();
            draw_list
                .add_line(
                    knob_position,
                    [screen_position[0], line_y],
                    ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
                )
                .thickness(2.0)
                .build();
            draw_list
                .add_circle(knob_position, 5.0, text_color)
                .filled(true)
                .build();

            // Current time label.
            let padding = [8.0, 4.0];
            let text = format!("{:.2}s", current_time);
            let text_size =
                ui.current_font()
                    .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, &text);
            let text_position = [knob_position[0] - text_size[0] * 0.5, text_y];
            let rect_min = [text_position[0] - padding[0], text_position[1] - padding[1]];
            let rect_max = [
                text_position[0] + text_size[0] + padding[0],
                text_position[1] + text_size[1] + padding[1],
            ];

            draw_list
                .add_rect(
                    rect_min,
                    rect_max,
                    ui.get_color_u32(dear_imgui_rs::StyleColor::PopupBg),
                )
                .filled(true)
                .build();
            draw_list
                .add_rect(
                    rect_min,
                    rect_max,
                    ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
                )
                .build();
            draw_list.add_text(text_position, text_color, text);
        });
    }
}
