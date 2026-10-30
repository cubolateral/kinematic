//! Consistent action labels for menus and tooltips.

use dear_imgui_rs::Ui;

pub fn label(action: &str, shortcut: &str) -> String {
    let action = action.trim().trim_end_matches('.');
    if shortcut.is_empty() {
        action.to_owned()
    } else {
        format!("{action} ({shortcut})")
    }
}

pub fn tooltip(ui: &Ui, action: &str, shortcut: &str) {
    if ui.is_item_hovered() {
        crate::widgets::tooltip(ui, label(action, shortcut));
    }
}

pub fn menu_item(ui: &Ui, action: &str, shortcut: &str) -> bool {
    crate::controls::menu_item(ui, label(action, shortcut))
}

#[cfg(test)]
mod tests {
    #[test]
    fn action_labels_use_parentheses_without_a_trailing_period() {
        assert_eq!(
            super::label("Focus selection.", "Ctrl + Space"),
            "Focus selection (Ctrl + Space)"
        );
        assert_eq!(super::label(" Reset camera... ", "R"), "Reset camera (R)");
        assert_eq!(super::label("Front view.", ""), "Front view");
    }
}
