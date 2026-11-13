use dear_imgui_rs::{PopupQueryFlags, Ui};
use kinematic_ui::{controls, icons, title_bar, widgets};

#[derive(Default)]
pub(super) struct State {
    pub controls: title_bar::Controls,
    pub action: Option<title_bar::Action>,
    pub drag_area: Option<[[f32; 2]; 2]>,
}

impl State {
    pub fn begin(&mut self, ui: &Ui) {
        self.drag_area = None;

        if self.controls.visible && self.controls.macos {
            self.action = self.controls.draw(ui).or(self.action);

            ui.same_line();
        }
    }

    pub fn logo(&self, ui: &Ui) {
        let [x, y] = ui.cursor_screen_pos();
        let frame = ui.frame_height();
        let size = ui.calc_text_size(icons::HOME)[0] * 1.5;
        let padding = (frame - size) * 0.5;
        let point = |px: f32, py: f32| {
            [
                x + padding + (px - 0.125) * size / 0.75,
                y + padding + (py - 0.125) * size / 0.75,
            ]
        };
        let color = ui.style_color(dear_imgui_rs::StyleColor::Text);
        let draw = ui.get_window_draw_list();

        draw.add_rect(point(0.125, 0.125), point(0.3125, 0.875), color)
            .filled(true)
            .build();
        draw.add_triangle(
            point(0.875, 0.875),
            point(0.875, 0.125),
            point(0.3125, 0.5),
            color,
        )
        .filled(true)
        .build();
        ui.dummy([frame; 2]);
        ui.same_line();
    }

    pub fn finish(&mut self, ui: &Ui, settings_open: Option<&mut bool>) {
        let divider = settings_open.is_none();
        let spacing = ui.clone_style().item_spacing()[0];
        let settings_width = if settings_open.is_some() {
            ui.frame_height() + spacing
        } else {
            0.0
        };
        let controls_width = if self.controls.macos {
            0.0
        } else {
            self.controls.width(ui)
        };
        let start = ui.cursor_pos()[0];
        let right = (ui.window_size()[0]
            - ui.clone_style().window_padding()[0]
            - controls_width
            - settings_width)
            .max(start);
        let position = ui.window_pos();
        let viewport = ui.main_viewport().pos();

        self.drag_area = (self.controls.visible && right > start).then_some([
            [position[0] - viewport[0] + start, position[1] - viewport[1]],
            [
                position[0] - viewport[0] + right,
                position[1] - viewport[1] + ui.window_size()[1],
            ],
        ]);

        if settings_open.is_some() || controls_width > 0.0 {
            ui.set_cursor_pos_x(right);
        }

        if let Some(open) = settings_open {
            if controls::text_button(ui, icons::SETTINGS, [ui.frame_height(); 2]) {
                *open = true;
            }

            if ui.is_item_hovered() {
                widgets::tooltip(ui, "Editor settings");
            }

            ui.same_line();
        }

        if !self.controls.macos {
            self.action = self.controls.draw(ui).or(self.action);
        }

        if divider {
            widgets::horizontal_divider(
                ui,
                position[1] + ui.window_size()[1]
                    - ui.clone_style()
                        .docking_separator_size()
                        .min(kinematic_ui::theme::FRAME_PADDING[1]),
            );
        }
    }

    pub fn drag_area(&self, ui: &Ui) -> Option<[[f32; 2]; 2]> {
        if ui.is_any_item_active() || ui.is_popup_open_with_flags("", PopupQueryFlags::ANY_POPUP) {
            None
        } else {
            self.drag_area
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use kinematic_ui::{controls, title_bar::Controls};

    #[test]
    fn dragging_excludes_menus_controls_and_open_popups() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let mut bar = State {
            controls: Controls {
                visible: true,
                minimize: true,
                maximize: true,
                ..Controls::default()
            },
            ..State::default()
        };

        for macos in [false, true] {
            bar.controls.macos = macos;

            for home in [true, false] {
                let ui = context.frame();
                let mut settings_open = false;

                if let Some(_menu) = kinematic_ui::title_bar::begin(ui) {
                    bar.begin(ui);

                    if !home {
                        for label in ["Project", "Editor", "Export"] {
                            let _menu = controls::menu(ui, label);
                        }
                    }

                    let menu_end = ui.cursor_screen_pos()[0];

                    bar.finish(ui, home.then_some(&mut settings_open));

                    if !macos {
                        let (button_min, button_max) = ui.item_rect();
                        let top = button_min[1] - ui.window_pos()[1];
                        let bottom = ui.window_pos()[1] + ui.window_size()[1]
                            - button_max[1]
                            - ui.clone_style()
                                .docking_separator_size()
                                .min(kinematic_ui::theme::FRAME_PADDING[1]);

                        assert!(
                            (top - bottom).abs() < 1.0,
                            "Title bar controls need equal vertical margins."
                        );
                    }

                    let [min, max] = bar.drag_area(ui).unwrap();

                    assert!(min[0] >= menu_end);
                    assert!(max[0] < ui.main_viewport().size()[0]);
                    assert!(max[0] > min[0]);

                    if macos {
                        assert!(min[0] >= bar.controls.width(ui));
                    } else {
                        assert!(max[0] <= ui.item_rect_min()[0]);
                    }
                }

                ui.open_popup("Blocking popup");

                assert!(bar.drag_area(ui).is_none());

                if let Some(_popup) = ui.begin_popup("Blocking popup") {
                    ui.close_current_popup();
                }

                context.end_frame();
            }
        }
    }
}
