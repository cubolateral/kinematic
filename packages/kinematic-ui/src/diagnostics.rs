use crate::{controls, icons, widgets};
use dear_imgui_rs::{
    InputTextCallback, InputTextCallbackHandler, StyleColor, StyleVar, TextCallbackData, Ui,
    WindowFlags,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}
impl Severity {
    fn color(self) -> [f32; 4] {
        match self {
            Self::Error => crate::theme::ERROR,
            Self::Warning => crate::theme::WARNING,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Message<'a> {
    pub id: usize,
    pub severity: Severity,
    pub text: &'a str,
    pub location: Option<[usize; 2]>,
}

#[derive(Default)]
pub struct Footer {
    expanded: Option<Severity>,
    selection: TextSelection,
    copy: Option<String>,
}
impl Footer {
    pub fn take_copy(&mut self) -> Option<String> {
        self.copy.take()
    }

    fn toggle(&mut self, severity: Severity) {
        self.expanded = (self.expanded != Some(severity)).then_some(severity);
    }

    fn visible<'a, 'text>(&self, messages: &'a [Message<'text>]) -> Vec<&'a Message<'text>> {
        if let Some(severity) = self.expanded {
            messages
                .iter()
                .filter(|message| message.severity == severity)
                .collect()
        } else {
            messages
                .iter()
                .rev()
                .find(|message| message.severity == Severity::Error)
                .or_else(|| messages.last())
                .map(|message| vec![message])
                .unwrap_or_default()
        }
    }

    pub fn height(&self, ui: &Ui, messages: &[Message<'_>]) -> f32 {
        let rows = if self.expanded.is_some() {
            self.visible(messages).len().min(6) as f32
        } else {
            0.0
        };

        ui.frame_height_with_spacing()
            + (rows * ui.text_line_height_with_spacing() + if rows > 0.0 { 8.0 } else { 0.0 })
                .min(ui.content_region_avail()[1] * 0.3)
    }

    pub fn draw(
        &mut self,
        ui: &Ui,
        messages: &[Message<'_>],
        status: &str,
        position: &str,
        height: f32,
    ) -> Option<usize> {
        let mut clicked = None;

        if messages.is_empty() || self.visible(messages).is_empty() {
            self.expanded = None;
        }

        let labels = counter_labels(messages);
        let origin = ui.cursor_screen_pos();
        let width = ui.content_region_avail()[0];
        let row_height = ui.frame_height();
        let counter_width: f32 = labels
            .iter()
            .map(|(_, label)| ui.calc_text_size(label)[0] + 16.0)
            .sum();
        let position_width = ui.calc_text_size(position)[0] + 16.0;
        let show_position = width >= counter_width + position_width + 80.0;
        let left_width =
            (width - counter_width - if show_position { position_width } else { 0.0 }).max(1.0);

        ui.child_window("Diagnostic summary")
            .size([left_width, row_height])
            .flags(WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE)
            .build(ui, || {
                if let Some(message) = Self::default().visible(messages).last() {
                    if draw_message(
                        ui,
                        message,
                        [left_width, row_height],
                        true,
                        &mut self.selection,
                        messages,
                        &mut self.copy,
                    ) {
                        clicked = Some(message.id);
                    }
                } else {
                    widgets::muted(ui, status);
                }
            });

        if show_position {
            ui.set_cursor_screen_pos([origin[0] + left_width + 8.0, origin[1]]);
            widgets::muted(ui, position);
        }

        let mut x = origin[0] + width - counter_width;

        for (severity, label) in &labels {
            ui.set_cursor_screen_pos([x, origin[1]]);

            let _color = ui.push_style_color(StyleColor::Text, severity.color());
            let button_width = ui.calc_text_size(label)[0] + 8.0;

            if controls::text_button(ui, label, [button_width, row_height]) {
                self.toggle(*severity);
            }

            if self.expanded == Some(*severity) {
                let (min, max) = ui.item_rect();

                ui.get_window_draw_list()
                    .add_line([min[0], max[1]], max, ui.get_color_u32(StyleColor::Text))
                    .build();
            }

            x += button_width + 8.0;
        }

        ui.set_cursor_screen_pos(origin);
        ui.dummy([width, row_height]);

        if self.expanded.is_some() && height > ui.frame_height_with_spacing() {
            ui.child_window("Diagnostic list")
                .size([0.0, (height - ui.frame_height_with_spacing()).max(1.0)])
                .build(ui, || {
                    let visible = self.visible(messages);
                    for (index, message) in visible.iter().enumerate() {
                        let _id = ui.push_id(index as i32);
                        let size = [ui.content_region_avail()[0].max(1.0), ui.text_line_height()];

                        if draw_message(
                            ui,
                            message,
                            size,
                            false,
                            &mut self.selection,
                            messages,
                            &mut self.copy,
                        ) {
                            clicked = Some(message.id);
                        }
                    }
                });
        }

        clicked
    }
}

fn counter_labels(messages: &[Message<'_>]) -> Vec<(Severity, String)> {
    [
        (Severity::Error, icons::CIRCLE_X),
        (Severity::Warning, icons::WARNING),
    ]
    .into_iter()
    .filter_map(|(severity, icon)| {
        let count = messages
            .iter()
            .filter(|message| message.severity == severity)
            .count();

        (count > 0).then(|| (severity, format!("{icon} {count}")))
    })
    .collect()
}
fn message_parts<'a>(message: &Message<'a>, summary: bool) -> (String, &'a str) {
    let severity = if message.severity == Severity::Error {
        "Error"
    } else {
        "Warning"
    };
    let prefix = if let Some([line, column]) = message.location {
        if summary {
            format!("{severity} at ({}, {}): ", line + 1, column + 1)
        } else {
            format!("Line {}: ", line + 1)
        }
    } else {
        format!("{severity}: ")
    };
    (prefix, message.text)
}
#[derive(Default)]
struct TextSelection {
    source: String,
    text: String,
}

impl InputTextCallbackHandler for &mut TextSelection {
    fn on_always(&mut self, data: TextCallbackData<'_>) {
        let start = data.selection_start().min(data.selection_end());
        let end = data.selection_start().max(data.selection_end());
        self.source = data.str().to_owned();
        self.text = data.str()[start..end].to_owned();
    }
}

fn draw_message(
    ui: &Ui,
    message: &Message<'_>,
    size: [f32; 2],
    summary: bool,
    selection: &mut TextSelection,
    messages: &[Message<'_>],
    copy: &mut Option<String>,
) -> bool {
    let (prefix, text) = message_parts(message, summary);
    let origin = ui.cursor_screen_pos();
    let prefix_width = ui.calc_text_size(&prefix)[0].min((size[0] - 1.0).max(0.0));
    let clicked = {
        let _color = ui.push_style_color(StyleColor::Text, message.severity.color());
        let _hover_color = ui.push_style_color(StyleColor::TextLink, message.severity.color());
        let clicked = controls::text_button(ui, &prefix, [prefix_width, size[1]]);
        if ui.is_item_hovered() {
            let y = origin[1] + (size[1] + ui.text_line_height()) * 0.5;
            ui.get_window_draw_list()
                .add_line(
                    [origin[0], y],
                    [origin[0] + prefix_width, y],
                    ui.get_color_u32(StyleColor::Text),
                )
                .build();
        }
        clicked
    };
    ui.set_cursor_screen_pos([origin[0] + prefix_width, origin[1]]);
    let _padding = ui.push_style_var(StyleVar::FramePadding([
        0.0,
        ((size[1] - ui.text_line_height()) * 0.5).max(0.0),
    ]));
    let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));
    let _background = ui.push_style_color(StyleColor::FrameBg, [0.0; 4]);
    let _width = ui.push_item_width((size[0] - prefix_width).max(1.0));
    let mut value = text.to_owned();
    let _id = ui.push_id(text);
    ui.input_text("##Diagnostic", &mut value)
        .read_only(true)
        .callback_flags(InputTextCallback::ALWAYS)
        .callback(&mut *selection)
        .build();

    if ui.is_window_hovered()
        && ui.is_mouse_hovering_rect(origin, [origin[0] + size[0], origin[1] + size[1]])
        && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Right)
    {
        ui.open_popup("Diagnostic actions");
    }
    if let Some(_popup) = ui.begin_popup("Diagnostic actions") {
        if controls::menu_item(ui, "Copy") {
            *copy = Some(if selection.source == text && !selection.text.is_empty() {
                selection.text.clone()
            } else {
                format!("{prefix}{text}")
            });
        }
        if controls::menu_item(ui, "Copy All") {
            let all = messages
                .iter()
                .map(|message| {
                    let (prefix, text) = message_parts(message, true);
                    format!("{prefix}{text}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            *copy = Some(all);
        }
    }

    clicked
}

#[cfg(test)]
mod tests {
    use super::{Footer, Message, Severity, counter_labels};

    fn message(id: usize, text: &str) -> Message<'_> {
        Message {
            id,
            severity: if text.starts_with("Warning: ") {
                Severity::Warning
            } else {
                Severity::Error
            },
            text,
            location: None,
        }
    }

    #[test]
    fn error_and_warning_text_can_be_selected() {
        use dear_imgui_rs::{Condition, Key, MouseButton, WindowFlags};
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
        let mut selection = super::TextSelection::default();
        let mut copy = None;
        let frame = |context: &mut dear_imgui_rs::Context,
                     selection: &mut super::TextSelection,
                     copy: &mut Option<String>,
                     severity| {
            let ui = context.frame();
            let message = Message {
                severity,
                ..message(0, "Mensagem com acentuação.")
            };
            let mut position = [0.0; 2];
            ui.window("Selection test")
                .position([0.0, 0.0], Condition::Always)
                .size([600.0, 100.0], Condition::Always)
                .flags(WindowFlags::NO_DECORATION)
                .build(|| {
                    super::draw_message(
                        ui,
                        &message,
                        [560.0, ui.frame_height()],
                        true,
                        selection,
                        &[message],
                        copy,
                    );
                    let (min, max) = ui.item_rect();
                    position = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
                });
            context.end_frame();
            position
        };
        for severity in [Severity::Error, Severity::Warning] {
            let position = frame(&mut context, &mut selection, &mut copy, severity);
            context.io_mut().add_mouse_pos_event(position);
            frame(&mut context, &mut selection, &mut copy, severity);
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, true);
            frame(&mut context, &mut selection, &mut copy, severity);
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, false);
            frame(&mut context, &mut selection, &mut copy, severity);
            context.io_mut().add_key_event(Key::ModCtrl, true);
            context.io_mut().add_key_event(Key::A, true);
            frame(&mut context, &mut selection, &mut copy, severity);
            assert_eq!(selection.text, "Mensagem com acentuação.");
            context.io_mut().add_key_event(Key::A, false);
            context.io_mut().add_key_event(Key::ModCtrl, false);
            frame(&mut context, &mut selection, &mut copy, severity);
        }
    }

    #[test]
    fn both_counters_remain_clickable_in_a_narrow_footer() {
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

        let messages = vec![
            message(
                0,
                "LuaLS main.lua:1:1: A long diagnostic message that must not push the warning counter out of view.",
            ),
            message(0, "Warning: LuaLS main.lua:2:1: Unused variable."),
        ];
        let mut footer = Footer::default();
        let frame = |context: &mut dear_imgui_rs::Context, footer: &mut Footer| {
            let ui = context.frame();
            let mut positions = Vec::new();

            ui.window("Footer test")
                .position([0.0, 0.0], dear_imgui_rs::Condition::Always)
                .size([220.0, 160.0], dear_imgui_rs::Condition::Always)
                .flags(dear_imgui_rs::WindowFlags::NO_DECORATION)
                .build(|| {
                    let origin = ui.cursor_screen_pos();
                    let width = ui.content_region_avail()[0];
                    let labels = counter_labels(&messages);
                    let total: f32 = labels
                        .iter()
                        .map(|(_, label)| ui.calc_text_size(label)[0] + 16.0)
                        .sum();
                    let mut x = origin[0] + width - total;

                    for (_, label) in labels {
                        let size = ui.calc_text_size(&label)[0] + 8.0;

                        positions.push([x + size * 0.5, origin[1] + ui.frame_height() * 0.5]);
                        assert!(x >= origin[0] && x + size <= origin[0] + width);
                        x += size + 8.0;
                    }

                    let height = footer.height(ui, &messages);

                    footer.draw(
                        ui,
                        &messages,
                        "Ready.",
                        "Ln 100, Col 100    Spaces: 4",
                        height,
                    );
                });

            context.end_frame();

            positions
        };

        frame(&mut context, &mut footer);

        let positions = frame(&mut context, &mut footer);

        for (index, severity) in [Severity::Error, Severity::Warning, Severity::Warning]
            .into_iter()
            .enumerate()
        {
            let position = positions[usize::from(severity == Severity::Warning)];

            context.io_mut().add_mouse_pos_event(position);

            frame(&mut context, &mut footer);

            context
                .io_mut()
                .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, true);

            frame(&mut context, &mut footer);

            context
                .io_mut()
                .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, false);

            frame(&mut context, &mut footer);
            assert!(footer.expanded == if index == 2 { None } else { Some(severity) });
        }
    }

    #[test]
    fn counters_preserve_both_severities_and_hide_zero_counts() {
        let messages = vec![
            message(0, "LuaLS main.lua:11:5: Missing name."),
            message(0, "Warning: LuaLS main.lua:12:1: Unused name."),
        ];
        let labels = counter_labels(&messages);

        assert_eq!(labels.len(), 2);
        assert!(labels[0].0 == Severity::Error && labels[0].1.ends_with(" 1"));
        assert!(labels[1].0 == Severity::Warning && labels[1].1.ends_with(" 1"));
        assert!(counter_labels(&messages[1..])[0].0 == Severity::Warning);
        assert_eq!(counter_labels(&messages[1..]).len(), 1);
        assert!(counter_labels(&[]).is_empty());
    }

    #[test]
    fn filters_toggle_and_collapse_to_the_last_error() {
        let messages = vec![
            message(0, "LuaLS main.lua:1:1: First error."),
            message(0, "LuaLS main.lua:2:1: Last error."),
            message(0, "Warning: LuaLS main.lua:3:1: Unused variable."),
        ];
        let mut footer = Footer::default();

        assert_eq!(footer.visible(&messages), [&messages[1]]);
        footer.toggle(Severity::Error);
        assert_eq!(footer.visible(&messages), [&messages[0], &messages[1]]);
        footer.toggle(Severity::Warning);
        assert_eq!(footer.visible(&messages), [&messages[2]]);
        footer.toggle(Severity::Warning);
        assert_eq!(footer.visible(&messages), [&messages[1]]);
        assert_eq!(footer.visible(&messages[2..]), [&messages[2]]);
        assert!(footer.visible(&[]).is_empty());
    }
}
