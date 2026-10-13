use crate::ui::{controls, icons};
use dear_imgui_rs::{StyleColor, Ui, WindowFlags};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
}
impl Severity {
    fn of(message: &str) -> Self {
        if message.starts_with("Warning: ") {
            Self::Warning
        } else {
            Self::Error
        }
    }
    fn color(self) -> [f32; 4] {
        match self {
            Self::Error => [0.95, 0.33, 0.33, 1.0],
            Self::Warning => [0.89, 0.80, 0.33, 1.0],
        }
    }
}

#[derive(Default)]
pub(super) struct Footer {
    expanded: Option<Severity>,
}
impl Footer {
    fn toggle(&mut self, severity: Severity) {
        self.expanded = (self.expanded != Some(severity)).then_some(severity);
    }
    fn visible<'a>(&self, messages: &'a [String]) -> Vec<&'a str> {
        if let Some(severity) = self.expanded {
            messages
                .iter()
                .filter(|message| Severity::of(message) == severity)
                .map(String::as_str)
                .collect()
        } else {
            messages
                .iter()
                .rev()
                .find(|message| Severity::of(message) == Severity::Error)
                .or_else(|| messages.last())
                .map(|message| vec![message.as_str()])
                .unwrap_or_default()
        }
    }
    pub fn height(&self, ui: &Ui, messages: &[String]) -> f32 {
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
        messages: &[String],
        status: &str,
        position: &str,
        height: f32,
    ) -> Option<String> {
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
                    if draw_message(ui, message, [left_width, row_height], true) {
                        clicked = Some((*message).to_owned());
                    }
                } else {
                    ui.text_disabled(status);
                }
            });
        if show_position {
            ui.set_cursor_screen_pos([origin[0] + left_width + 8.0, origin[1]]);
            ui.text_disabled(position);
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
                    for (index, message) in self.visible(messages).iter().enumerate() {
                        let _id = ui.push_id(index as i32);
                        let size = [ui.content_region_avail()[0].max(1.0), ui.text_line_height()];
                        if draw_message(ui, message, size, false) {
                            clicked = Some((*message).to_owned());
                        }
                    }
                });
        }
        clicked
    }
}

fn counter_labels(messages: &[String]) -> Vec<(Severity, String)> {
    [
        (Severity::Error, icons::CIRCLE_X),
        (Severity::Warning, icons::WARNING),
    ]
    .into_iter()
    .filter_map(|(severity, icon)| {
        let count = messages
            .iter()
            .filter(|message| Severity::of(message) == severity)
            .count();
        (count > 0).then(|| (severity, format!("{icon} {count}")))
    })
    .collect()
}
fn message_parts(message: &str, summary: bool) -> (String, &str) {
    let severity = if Severity::of(message) == Severity::Error {
        "Error"
    } else {
        "Warning"
    };
    if let Some((_, line, column)) = super::diagnostic_location(message) {
        if let Some((_, rest)) = message.split_once(".lua:") {
            if let Some((_, rest)) = rest.split_once(':') {
                let text = rest
                    .split_once(':')
                    .filter(|(column, _)| column.parse::<usize>().is_ok())
                    .map_or(rest, |(_, text)| text)
                    .trim();
                let prefix = if summary {
                    format!("{severity} at ({}, {}): ", line + 1, column + 1)
                } else {
                    format!("Line {}: ", line + 1)
                };
                return (prefix, text);
            }
        }
    }
    (
        format!("{severity}: "),
        message.trim_start_matches("Warning: "),
    )
}
fn draw_message(ui: &Ui, message: &str, size: [f32; 2], summary: bool) -> bool {
    let clicked = ui.invisible_button("Diagnostic", size);
    let (min, max) = ui.item_rect();
    let draw = ui.get_window_draw_list();
    let _clip = draw.push_clip_rect(min, max, true);
    if ui.is_item_hovered() {
        draw.add_rect(min, max, ui.get_color_u32(StyleColor::HeaderHovered))
            .filled(true)
            .build();
    }
    let (prefix, text) = message_parts(message, summary);
    let color = {
        let _color = ui.push_style_color(StyleColor::Text, Severity::of(message).color());
        ui.get_color_u32(StyleColor::Text)
    };
    let top = min[1] + (size[1] - ui.text_line_height()) * 0.5;
    let prefix_width = ui.calc_text_size(&prefix)[0];
    draw.add_text([min[0], top], color, &prefix);
    if !summary {
        draw.add_line(
            [min[0], top + ui.text_line_height() - 1.0],
            [min[0] + prefix_width, top + ui.text_line_height() - 1.0],
            color,
        )
        .build();
    }
    draw.add_text(
        [min[0] + prefix_width, top],
        ui.get_color_u32(StyleColor::Text),
        text.lines().next().unwrap_or(text),
    );
    clicked
}

#[cfg(test)]
mod tests {
    use super::{Footer, Severity, counter_labels, message_parts};

    #[test]
    fn both_counters_remain_clickable_in_a_narrow_footer() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
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
            "LuaLS main.lua:1:1: A long diagnostic message that must not push the warning counter out of view.".into(),
            "Warning: LuaLS main.lua:2:1: Unused variable.".into(),
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
            "LuaLS main.lua:11:5: Missing name.".into(),
            "Warning: LuaLS main.lua:12:1: Unused name.".into(),
        ];
        let labels = counter_labels(&messages);
        assert_eq!(labels.len(), 2);
        assert!(labels[0].0 == Severity::Error && labels[0].1.ends_with(" 1"));
        assert!(labels[1].0 == Severity::Warning && labels[1].1.ends_with(" 1"));
        assert!(counter_labels(&messages[1..])[0].0 == Severity::Warning);
        assert_eq!(counter_labels(&messages[1..]).len(), 1);
        assert!(counter_labels(&[]).is_empty());
        assert_eq!(
            message_parts(&messages[0], false),
            ("Line 11: ".into(), "Missing name.")
        );
        assert_eq!(
            message_parts(&messages[0], true),
            ("Error at (11, 5): ".into(), "Missing name.")
        );
        assert_eq!(
            message_parts(&messages[1], true),
            ("Warning at (12, 1): ".into(), "Unused name.")
        );
        assert_eq!(
            message_parts("Runtime error: main.lua:9: Missing name.", false),
            ("Line 9: ".into(), "Missing name.")
        );
    }

    #[test]
    fn filters_toggle_and_collapse_to_the_last_error() {
        let messages = vec![
            "LuaLS main.lua:1:1: First error.".into(),
            "LuaLS main.lua:2:1: Last error.".into(),
            "Warning: LuaLS main.lua:3:1: Unused variable.".into(),
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
