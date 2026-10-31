//! Consistent action labels for menus and tooltips.

use dear_imgui_rs::{TableFlags, TableOptions, TableSizingPolicy, Ui};

pub fn compact(shortcut: &str) -> String {
    shortcut.replace(" + ", "+")
}

fn action_label(action: &str) -> &str {
    action.trim().trim_end_matches('.')
}

fn hint(ui: &Ui, action: &str, shortcut: &str) {
    ui.text(action_label(action));
    if !shortcut.is_empty() {
        ui.same_line();
        ui.text_disabled(compact(shortcut));
    }
}

pub fn tooltip(ui: &Ui, action: &str, shortcut: &str) {
    if ui.is_item_hovered() {
        ui.tooltip(|| hint(ui, action, shortcut));
    }
}

pub fn menu_item(ui: &Ui, action: &str, shortcut: &str) -> bool {
    menu_item_enabled_selected(ui, action, shortcut, false, true)
}

pub fn menu_item_enabled_selected(
    ui: &Ui,
    action: &str,
    shortcut: &str,
    selected: bool,
    enabled: bool,
) -> bool {
    ui.menu_item_enabled_selected(
        action_label(action),
        Some(compact(shortcut)),
        selected,
        enabled,
    )
}

pub fn hints(ui: &Ui, entries: &[(&str, &str)]) {
    let Some(_table) = ui.begin_table_with_sizing(
        "Shortcut hints",
        2,
        TableOptions::new()
            .flags(TableFlags::NO_SAVED_SETTINGS)
            .sizing_policy(TableSizingPolicy::FixedFit),
        [0.0, 0.0],
        0.0,
    ) else {
        return;
    };
    for (action, shortcut) in entries {
        ui.table_next_row();
        ui.table_next_column();
        ui.text(action_label(action));
        ui.table_next_column();
        ui.text_disabled(compact(shortcut));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn action_labels_remove_surrounding_spaces_and_trailing_periods() {
        assert_eq!(super::action_label("Focus selection."), "Focus selection");
        assert_eq!(super::action_label(" Reset camera... "), "Reset camera");
        assert_eq!(super::action_label("Front view."), "Front view");
    }
}
