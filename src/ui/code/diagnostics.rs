//! Converts runtime and LuaLS diagnostics into presentation data.

use kinematic_ui::diagnostics::{Message, Severity};

#[derive(Default)]
pub(super) struct Footer(kinematic_ui::diagnostics::Footer);

impl Footer {
    pub fn height(&self, ui: &dear_imgui_rs::Ui, messages: &[String]) -> f32 {
        self.0.height(ui, &presentation(messages))
    }
    pub fn draw(
        &mut self,
        ui: &dear_imgui_rs::Ui,
        messages: &[String],
        status: &str,
        position: &str,
        height: f32,
    ) -> Option<String> {
        self.0
            .draw(ui, &presentation(messages), status, position, height)
            .and_then(|index| messages.get(index).cloned())
    }
}

fn severity(message: &str) -> Severity {
    if message.starts_with("Warning: ") {
        Severity::Warning
    } else {
        Severity::Error
    }
}

fn presentation(messages: &[String]) -> Vec<Message<'_>> {
    messages
        .iter()
        .enumerate()
        .map(|(id, raw)| {
            let (_, text) = message_parts(raw, false);
            Message {
                id,
                severity: severity(raw),
                text,
                location: super::diagnostic_location(raw).map(|(_, line, column)| [line, column]),
            }
        })
        .collect()
}

fn message_parts(message: &str, summary: bool) -> (String, &str) {
    let severity = if severity(message) == Severity::Error {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lua_locations_become_display_data_without_losing_the_source_message() {
        let raw = vec![
            "LuaLS main.lua:11:5: Missing name.".into(),
            "Warning: LuaLS main.lua:12:1: Unused name.".into(),
        ];
        let view = presentation(&raw);
        assert_eq!(view[0].id, 0);
        assert_eq!(view[0].text, "Missing name.");
        assert_eq!(view[0].location, Some([10, 4]));
        assert_eq!(view[1].severity, Severity::Warning);
        assert_eq!(
            message_parts(&raw[0], true),
            ("Error at (11, 5): ".into(), "Missing name.")
        );
        assert_eq!(
            message_parts("Runtime error: main.lua:9: Missing name.", false),
            ("Line 9: ".into(), "Missing name.")
        );
    }
}
