//! Window controls drawn inside an application menu bar.

use dear_imgui_rs::{StyleColor, Ui};

use crate::{controls, theme};

pub struct Bar<'ui> {
    _content: dear_imgui_rs::StyleStackToken<'ui>,
    _bar: dear_imgui_rs::MainMenuBarToken<'ui>,
    _padding: dear_imgui_rs::StyleStackToken<'ui>,
}

pub fn begin(ui: &Ui) -> Option<Bar<'_>> {
    let padding = ui.clone_style().frame_padding();
    let outer_padding = ui.push_style_var(dear_imgui_rs::StyleVar::FramePadding([
        padding[0],
        padding[1] * 2.0
            + ui.clone_style()
                .docking_separator_size()
                .min(theme::FRAME_PADDING[1])
                * 0.5,
    ]));
    let bar = ui.begin_main_menu_bar()?;
    let content = ui.push_style_var(dear_imgui_rs::StyleVar::FramePadding(padding));
    let position = ui.window_pos();

    let x = ui.cursor_screen_pos()[0];

    ui.new_line();
    ui.set_cursor_screen_pos([x, position[1] + padding[1]]);
    ui.align_text_to_frame_padding();

    Some(Bar {
        _content: content,
        _bar: bar,
        _padding: outer_padding,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Minimize,
    ToggleMaximize,
    Close,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    pub visible: bool,
    pub minimize: bool,
    pub maximize: bool,
    pub maximized: bool,
    pub macos: bool,
}

impl Controls {
    pub fn width(self, ui: &Ui) -> f32 {
        if !self.visible {
            return 0.0;
        }

        let count = 1 + usize::from(self.minimize) + usize::from(self.maximize);

        ui.frame_height() * count as f32 + ui.clone_style().item_spacing()[0] * (count - 1) as f32
    }

    pub fn draw(self, ui: &Ui) -> Option<Action> {
        if !self.visible {
            return None;
        }

        let buttons = if self.macos {
            [Action::Close, Action::Minimize, Action::ToggleMaximize]
        } else {
            [Action::Minimize, Action::ToggleMaximize, Action::Close]
        };
        let mut action = None;
        let mut first = true;

        for button in buttons {
            if (button == Action::Minimize && !self.minimize)
                || (button == Action::ToggleMaximize && !self.maximize)
            {
                continue;
            }

            if !first {
                ui.same_line();
            }

            first = false;

            if window_button(ui, button, self.maximized, self.macos) {
                action = Some(button);
            }
        }

        action
    }
}

fn window_button(ui: &Ui, action: Action, maximized: bool, macos: bool) -> bool {
    let label = match action {
        Action::Minimize => "Minimize",
        Action::ToggleMaximize if maximized => "Restore",
        Action::ToggleMaximize => "Maximize",
        Action::Close => "Close",
    };
    let _id = ui.push_id(label);
    let size = ui.frame_height();
    let transparent = ui.push_style_color(StyleColor::Button, [0.0; 4]);
    let close_hover = (action == Action::Close && !macos)
        .then(|| ui.push_style_color(StyleColor::ButtonHovered, theme::ERROR));
    let clicked = controls::button_sized(ui, "##WindowControl", [size; 2]);

    drop(close_hover);
    drop(transparent);

    let (min, max) = ui.item_rect();
    let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let radius = ui.current_font_size() * 0.27;
    let draw = ui.get_window_draw_list();
    let hovered = ui.is_item_hovered();
    let color = if macos {
        let background = match action {
            Action::Close => [1.0, 0.37, 0.34, 1.0],
            Action::Minimize => [1.0, 0.74, 0.18, 1.0],
            Action::ToggleMaximize => [0.16, 0.79, 0.25, 1.0],
        };

        draw.add_circle(center, radius * 1.5, background)
            .filled(true)
            .build();

        [0.15, 0.15, 0.15, f32::from(hovered)]
    } else if action == Action::Close && hovered {
        [1.0; 4]
    } else {
        ui.style_color(StyleColor::Text)
    };
    let left = center[0] - radius;
    let right = center[0] + radius;
    let top = center[1] - radius;
    let bottom = center[1] + radius;

    match action {
        Action::Minimize => {
            draw.add_line([left, center[1]], [right, center[1]], color)
                .build();
        }
        Action::ToggleMaximize => {
            let offset = if maximized { radius * 0.55 } else { 0.0 };

            if maximized {
                draw.add_rect([left + offset, top], [right, bottom - offset], color)
                    .build();
            }

            draw.add_rect([left, top + offset], [right - offset, bottom], color)
                .build();
        }
        Action::Close => {
            draw.add_line([left, top], [right, bottom], color).build();
            draw.add_line([left, bottom], [right, top], color).build();
        }
    }

    clicked
}

#[cfg(test)]
mod tests {
    use super::{Action, Controls};

    #[test]
    fn menu_text_and_icon_button_text_share_the_same_vertical_position() {
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

        let ui = context.frame();
        let mut ranges = Vec::new();

        {
            let _bar = super::begin(ui).unwrap();
            {
                let _id = ui.push_id("Icon button");
                crate::controls::text_button(
                    ui,
                    "Align",
                    [ui.calc_text_size("Align")[0] + 12.0, ui.frame_height()],
                );
                ranges.push(ui.item_rect());
            }
            {
                let _id = ui.push_id("Menu");
                let _menu = crate::controls::menu(ui, "Align");
                ranges.push(ui.item_rect());
            }
        }

        let frame = context.render_legacy();
        let heights = ranges
            .iter()
            .map(|(min, max)| {
                frame
                    .draw_data()
                    .draw_lists()
                    .flat_map(|list| list.vtx_buffer())
                    .filter(|vertex| vertex.pos[0] > min[0] && vertex.pos[0] < max[0])
                    .map(|vertex| vertex.pos[1])
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), y| {
                        (min.min(y), max.max(y))
                    })
            })
            .collect::<Vec<_>>();

        assert!(heights[0].0.is_finite());
        assert_eq!(heights[0], heights[1]);
    }

    #[test]
    fn visible_controls_fit_the_bar_and_close_emits_an_action() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let controls = Controls {
            visible: true,
            minimize: false,
            maximize: true,
            ..Controls::default()
        };
        let mut close_center = [0.0; 2];

        for _ in 0..2 {
            let ui = context.frame();

            if let Some(_bar) = ui.begin_main_menu_bar() {
                let start = ui.cursor_screen_pos();

                assert_eq!(controls.draw(ui), None);

                let (min, max) = ui.item_rect();

                assert!((max[0] - start[0] - controls.width(ui)).abs() < 1.0);
                assert!(max[1] <= ui.window_pos()[1] + ui.window_size()[1]);

                close_center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
            }

            context.end_frame();
        }

        context.io_mut().add_mouse_pos_event(close_center);

        for pressed in [true, false] {
            context
                .io_mut()
                .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, pressed);

            let ui = context.frame();

            if let Some(_bar) = ui.begin_main_menu_bar() {
                assert_eq!(controls.draw(ui), (!pressed).then_some(Action::Close));
            }

            context.end_frame();
        }
    }
}
