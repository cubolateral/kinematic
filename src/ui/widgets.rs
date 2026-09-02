pub(super) fn selection_color(ui: &dear_imgui_rs::Ui) -> u32 {
    ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg)
}

pub(super) fn draw_panel_rect(
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    min: [f32; 2],
    max: [f32; 2],
    fill: Option<u32>,
    border: u32,
) {
    if let Some(fill) = fill {
        draw_list.add_rect(min, max, fill).filled(true).build();
    }

    draw_list.add_rect(min, max, border).build();
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

#[cfg(test)]
mod tests {
    use super::hierarchy_prefix;

    #[test]
    fn hierarchy_prefix_preserves_branch_connections() {
        assert_eq!(hierarchy_prefix(&[], false), "├─ ");
        assert_eq!(hierarchy_prefix(&[true, false], true), "│     └─ ");
    }
}
