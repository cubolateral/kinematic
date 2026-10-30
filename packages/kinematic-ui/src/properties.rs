use crate::{controls, icons, widgets::text_size};

#[derive(Default)]
pub struct Edit {
    pub changed: bool,
    pub reset: bool,
    pub committed: bool,
}

pub fn row(
    ui: &dear_imgui_rs::Ui,
    name: &str,
    show_reset: bool,
    input: impl FnOnce() -> bool,
) -> Edit {
    let _id = ui.push_id(name);
    let width = ui.content_region_avail_width().max(1.0);

    ui.group(|| {
        let Some(_table) = ui.begin_table_with_sizing(
            "##property",
            2,
            dear_imgui_rs::TableOptions::new()
                .flags(dear_imgui_rs::TableFlags::NO_SAVED_SETTINGS)
                .sizing_policy(dear_imgui_rs::TableSizingPolicy::StretchProp),
            [width, 0.0],
            0.0,
        ) else {
            return Edit::default();
        };

        ui.table_setup_column_stretch_weight("Name", dear_imgui_rs::TableColumnFlags::empty(), 0.4);

        ui.table_setup_column_stretch_weight(
            "Value",
            dear_imgui_rs::TableColumnFlags::empty(),
            0.6,
        );

        ui.table_next_column();

        let position = ui.cursor_screen_pos();
        let name_width = ui.content_region_avail_width();
        let reset_width = if show_reset {
            ui.frame_height() + 4.0
        } else {
            0.0
        };
        let text_width = (name_width - reset_width).max(0.0);
        let draw = ui.get_window_draw_list();
        let clip = draw.push_clip_rect(
            position,
            [position[0] + text_width, position[1] + ui.frame_height()],
            true,
        );

        draw.add_text(
            [
                position[0],
                position[1] + (ui.frame_height() - text_size(ui, name)[1]) * 0.5,
            ],
            ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            name,
        );

        drop(clip);

        drop(draw);
        ui.dummy([text_width, ui.frame_height()]);

        let mut reset = false;

        if show_reset {
            ui.same_line_with_spacing(0.0, 0.0);

            ui.set_cursor_screen_pos([
                position[0] + (name_width - ui.frame_height()).max(0.0),
                position[1],
            ]);

            reset = controls::text_button(ui, icons::RESET, [ui.frame_height(); 2]);

            if ui.is_item_hovered() {
                crate::widgets::tooltip(ui, "Reset to the default value");
            }
        }

        ui.table_next_column();

        let _width = ui.push_item_width(-1.0);
        let (changed, committed) = ui.group(|| {
            let changed = input();
            (changed, ui.is_item_deactivated_after_edit())
        });

        Edit {
            changed,
            reset,
            committed: committed || reset,
        }
    })
}

pub fn field<T: PartialEq>(
    ui: &dear_imgui_rs::Ui,
    name: &str,
    value: &mut T,
    default: T,
    input: impl FnOnce(&mut T) -> bool,
) -> bool {
    let edit = row(ui, name, *value != default, || input(value));

    if edit.reset {
        *value = default;
    }

    edit.changed || edit.reset
}

pub fn components<T, const N: usize>(
    ui: &dear_imgui_rs::Ui,
    values: &mut [T; N],
    names: [&str; N],
    mut input: impl FnMut(&str, &mut T) -> bool,
) -> bool {
    let mut changed = false;

    for (value, name) in values.iter_mut().zip(names) {
        let _id = ui.push_id(name);

        ui.align_text_to_frame_padding();

        let color = match name {
            "x" | "a" => [0.9, 0.45, 0.3, 1.0],
            "y" | "b" => [0.45, 0.8, 0.4, 1.0],
            "z" | "c" => [0.4, 0.65, 0.95, 1.0],
            "w" => [0.8, 0.65, 0.9, 1.0],
            _ => ui.style_color(dear_imgui_rs::StyleColor::Text),
        };

        ui.text_colored(color, name);
        ui.same_line_with_spacing(0.0, 6.0);
        ui.set_next_item_width(ui.content_region_avail_width().max(1.0));

        changed |= input(name, value);
    }

    changed
}

pub fn read_only(ui: &dear_imgui_rs::Ui, name: &str, value: &str) {
    row(ui, name, false, || {
        ui.text_disabled(value);
        false
    });
}

pub fn section(ui: &dear_imgui_rs::Ui, title: &str) {
    ui.spacing();
    ui.text_disabled(title);
    ui.separator();
}

pub fn slider(
    ui: &dear_imgui_rs::Ui,
    name: &str,
    value: &mut f32,
    default: f32,
    range: std::ops::RangeInclusive<f32>,
) -> Edit {
    let mut edit = row(ui, name, *value != default, || {
        controls::slider(ui, "##value", value, range, "%.3f")
    });
    if edit.reset {
        *value = default;
        edit.changed = true;
    }
    edit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_sliders_commit_on_release() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let mut value = 0.2;
        let mut rect = [[0.0; 2]; 2];
        let mut changed_while_dragging = false;
        let mut committed = false;
        for frame in 0..7 {
            if frame >= 2 {
                let [min, max] = rect;
                let ratio = if frame >= 4 { 0.7 } else { 0.4 };
                context.io_mut().add_mouse_pos_event([
                    min[0] + (max[0] - min[0]) * ratio,
                    (min[1] + max[1]) * 0.5,
                ]);
            }
            if frame == 3 || frame == 5 {
                context
                    .io_mut()
                    .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, frame == 3);
            }
            let ui = context.frame();
            ui.window("Slider commit test")
                .position([0.0; 2], dear_imgui_rs::Condition::Always)
                .size([520.0, 300.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    let edit = row(ui, "Volume", false, || {
                        let changed = ui.slider("##value", 0.0, 1.0, &mut value);
                        rect = [ui.item_rect_min(), ui.item_rect_max()];
                        changed
                    });
                    if frame == 3 || frame == 4 {
                        changed_while_dragging |= edit.changed;
                        assert!(!edit.committed);
                    }
                    if frame == 5 {
                        committed = edit.committed;
                    }
                });
            context.end_frame();
        }
        assert!(changed_while_dragging);
        assert!(committed);
    }

    #[test]
    fn reset_restores_the_default_and_disappears() {
        let _context_lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        let default = [1280, 720];
        let mut value = [1920, 1080];
        let mut reset_position = [0.0; 2];

        for frame in 0..6 {
            if frame >= 2 {
                context.io_mut().add_mouse_pos_event(reset_position);
            }

            if frame == 3 || frame == 4 {
                context
                    .io_mut()
                    .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, frame == 3);
            }

            let ui = context.frame();

            ui.window("Property reset test")
                .position([0.0, 0.0], dear_imgui_rs::Condition::Always)
                .size([520.0, 300.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    let changed = field(ui, "Resolution", &mut value, default, |value| {
                        let has_reset = ui.item_id() == ui.get_id(icons::RESET);

                        assert_eq!(has_reset, frame < 5);

                        if has_reset {
                            let (min, max) = ui.item_rect();

                            reset_position = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
                        }

                        components(ui, value, ["Width", "Height"], |_, value| {
                            ui.input_scalar("##value", value).build()
                        })
                    });

                    assert_eq!(changed, frame == 4);

                    let mut enabled = true;

                    assert!(!field(ui, "Enabled", &mut enabled, true, |value| {
                        assert_ne!(ui.item_id(), ui.get_id(icons::RESET));

                        ui.checkbox("##value", value)
                    }));
                });

            context.end_frame();
        }

        assert_eq!(value, default);
    }
}
