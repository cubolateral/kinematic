pub(super) fn draw_panel_rect(
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    min: [f32; 2],
    max: [f32; 2],
    fill: Option<u32>,
    border: u32,
    border_thickness: f32,
) {
    if let Some(fill) = fill {
        draw_list.add_rect(min, max, fill).filled(true).build();
    }

    draw_list
        .add_rect(min, max, border)
        .thickness(border_thickness)
        .build();
}

pub(super) fn text_size(ui: &dear_imgui_rs::Ui, text: &str) -> [f32; 2] {
    ui.current_font()
        .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, text)
}

pub(super) fn hierarchy_prefix(branches: &[bool], is_last: bool) -> String {
    let mut prefix = String::new();

    for continues in branches {
        prefix.push_str(if *continues { "│  " } else { "   " });
    }

    prefix.push_str(if is_last { "└─ " } else { "├─ " });
    prefix
}

pub(super) fn hide_single_window_tab(ui: &dear_imgui_rs::Ui) {
    ui.set_next_window_class(
        &dear_imgui_rs::WindowClass::default()
            .dock_node_flags_override_set(dear_imgui_rs::DockFlags::AUTO_HIDE_TAB_BAR),
    );
}

pub(super) trait NumericValue {
    fn offset(&mut self, amount: f32) -> bool;
}

impl NumericValue for f32 {
    fn offset(&mut self, amount: f32) -> bool {
        let next = *self + amount;
        if next == *self {
            return false;
        }

        *self = next;
        true
    }
}

impl NumericValue for i32 {
    fn offset(&mut self, amount: f32) -> bool {
        let amount = amount.round() as i64;
        if amount == 0 {
            return false;
        }

        let next = i64::from(*self)
            .saturating_add(amount)
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        if next == *self {
            return false;
        }

        *self = next;
        true
    }
}

impl NumericValue for u32 {
    fn offset(&mut self, amount: f32) -> bool {
        let amount = amount.round() as i64;
        if amount == 0 {
            return false;
        }

        let next = i64::from(*self)
            .saturating_add(amount)
            .clamp(0, i64::from(u32::MAX)) as u32;
        if next == *self {
            return false;
        }

        *self = next;
        true
    }
}

pub(super) fn numeric_input_arrows<T: NumericValue>(ui: &dear_imgui_rs::Ui, value: &mut T) -> bool {
    if !ui.is_item_active() {
        return false;
    }

    let changed = if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::UpArrow, true) {
        value.offset(1.0)
    } else if ui.is_key_pressed_with_repeat(dear_imgui_rs::Key::DownArrow, true) {
        value.offset(-1.0)
    } else {
        false
    };

    if changed {
        // InputScalar keeps its own text buffer while editing. Ask it to reload the value
        // written above on the next frame, otherwise it would restore the old text.
        unsafe {
            let state =
                dear_imgui_rs::sys::igGetInputTextState(dear_imgui_rs::sys::igGetActiveID());
            if !state.is_null() {
                dear_imgui_rs::sys::ImGuiInputTextState_ReloadUserBufAndKeepSelection(state);
            }
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::{NumericValue, hierarchy_prefix};

    #[test]
    fn hierarchy_prefix_preserves_branch_connections() {
        assert_eq!(hierarchy_prefix(&[], false), "├─ ");
        assert_eq!(hierarchy_prefix(&[true, false], true), "│     └─ ");
    }

    #[test]
    fn numeric_offsets_round_and_saturate_integer_values() {
        let mut signed = 10i32;
        assert!(signed.offset(-1.4));
        assert_eq!(signed, 9);

        let mut unsigned = 0u32;
        assert!(!unsigned.offset(-1.0));
        assert_eq!(unsigned, 0);
    }
}
