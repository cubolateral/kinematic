//! Shared floating window and dialog templates, independent of editor state.

use dear_imgui_rs::{Condition, ModalPopupToken, StyleColor, StyleVar, Ui, Window, WindowFlags};

pub fn style(
    ui: &Ui,
    border_radius: f32,
    window_spacing: f32,
) -> Vec<dear_imgui_rs::StyleStackToken<'_>> {
    let mut tokens = [
        StyleVar::DockingSeparatorSize(window_spacing),
        StyleVar::WindowPadding(crate::theme::WINDOW_PADDING),
        StyleVar::FramePadding(crate::theme::FRAME_PADDING),
        StyleVar::ItemSpacing(crate::theme::ITEM_SPACING),
        StyleVar::WindowRounding(border_radius),
        StyleVar::PopupRounding(border_radius),
        StyleVar::FrameRounding(border_radius),
        StyleVar::MenuItemRounding(border_radius),
        StyleVar::SelectableRounding(border_radius),
        StyleVar::ChildRounding(border_radius),
        StyleVar::GrabRounding(border_radius),
        StyleVar::TabRounding(border_radius),
        StyleVar::ScrollbarRounding(border_radius),
        StyleVar::ImageRounding(border_radius),
        StyleVar::DragDropTargetRounding(border_radius),
        StyleVar::ChildBorderSize(0.0),
        StyleVar::WindowBorderSize(0.0),
        StyleVar::PopupBorderSize(0.0),
        StyleVar::FrameBorderSize(0.0),
    ]
    .into_iter()
    .map(|value| ui.push_style_var(value))
    .collect::<Vec<_>>();

    tokens.reverse();

    tokens
}

pub fn actions(ui: &Ui, actions: &[(&str, bool)]) -> Option<usize> {
    let style = ui.clone_style();
    let width = actions
        .iter()
        .map(|(label, _)| ui.calc_text_size(label)[0] + style.frame_padding()[0] * 2.0)
        .sum::<f32>()
        + style.item_spacing()[0] * actions.len().saturating_sub(1) as f32;

    crate::widgets::separator(ui);

    ui.spacing();
    ui.set_cursor_pos_x(ui.cursor_pos_x() + (ui.content_region_avail_width() - width).max(0.0));

    let mut clicked = None;

    for (index, (label, enabled)) in actions.iter().enumerate() {
        if index > 0 {
            ui.same_line();
        }

        let _disabled = ui.begin_disabled_with_cond(!enabled);

        if crate::controls::button(ui, label) {
            clicked = Some(index);
        }
    }

    clicked
}

pub fn window<'ui>(ui: &'ui Ui, title: &'ui str) -> Window<'ui> {
    center_next(ui);

    ui.window(title)
        .collapsed(false, Condition::Always)
        .flags(WindowFlags::NO_DOCKING | WindowFlags::NO_COLLAPSE | WindowFlags::NO_SAVED_SETTINGS)
        .size(
            ui.main_viewport().size().map(|value| value * 0.5),
            Condition::Appearing,
        )
}

pub fn modal<'ui>(
    ui: &'ui Ui,
    title: &'ui str,
    open: &'ui mut bool,
) -> Option<ModalPopupToken<'ui>> {
    sized_modal(ui, title, open, [480.0, 0.0])
}

pub fn sized_modal<'ui>(
    ui: &'ui Ui,
    title: &'ui str,
    open: &'ui mut bool,
    size: [f32; 2],
) -> Option<ModalPopupToken<'ui>> {
    modal_popup(ui, title, Some(open), size)
}

pub(crate) fn modal_popup<'ui>(
    ui: &'ui Ui,
    title: &'ui str,
    open: Option<&'ui mut bool>,
    size: [f32; 2],
) -> Option<ModalPopupToken<'ui>> {
    center_next(ui);

    let auto_height = size[1] == 0.0;
    // A known width lets ImGui measure wrapped content before centering the dialog.
    let condition = if auto_height {
        Condition::Always
    } else {
        Condition::FirstUseEver
    };

    set_next_size(
        [
            size[0].min((ui.main_viewport().size()[0] - 32.0).max(1.0)),
            size[1],
        ],
        condition,
    );

    let flags = if auto_height {
        WindowFlags::ALWAYS_AUTO_RESIZE | WindowFlags::NO_SAVED_SETTINGS
    } else {
        WindowFlags::empty()
    };
    let popup = ui
        .begin_modal_popup_config(title)
        .flags(flags | WindowFlags::NO_COLLAPSE | WindowFlags::NO_SAVED_SETTINGS);
    let popup = if let Some(open) = open {
        popup.opened(open)
    } else {
        popup
    };

    popup.begin()
}

fn center_next(ui: &Ui) {
    let center = ui.main_viewport().center();

    // The ImGui builders do not expose a position pivot or modal size.
    unsafe {
        dear_imgui_rs::sys::igSetNextWindowPos(
            dear_imgui_rs::sys::ImVec2 {
                x: center[0],
                y: center[1],
            },
            Condition::Appearing as i32,
            dear_imgui_rs::sys::ImVec2 { x: 0.5, y: 0.5 },
        );
    }
}

fn set_next_size(size: [f32; 2], condition: Condition) {
    unsafe {
        dear_imgui_rs::sys::igSetNextWindowSize(
            dear_imgui_rs::sys::ImVec2 {
                x: size[0],
                y: size[1],
            },
            condition as i32,
        );
    }
}

pub fn panel<T>(ui: &Ui, title: &str, content: impl FnOnce() -> T) -> Option<T> {
    crate::widgets::hide_window_tab_bar(ui);

    ui.window(title)
        .collapsed(false, Condition::Always)
        .flags(WindowFlags::NO_COLLAPSE)
        .build(content)
}

pub fn panel_with_footer<R>(
    ui: &Ui,
    title: &str,
    expanded: bool,
    content: impl FnOnce(),
    footer: impl FnOnce() -> R,
) -> Option<R> {
    crate::widgets::hide_window_tab_bar(ui);

    ui.window(title)
        .collapsed(false, Condition::Always)
        .flags(
            WindowFlags::NO_COLLAPSE
                | WindowFlags::NO_SCROLLBAR
                | WindowFlags::NO_SCROLL_WITH_MOUSE,
        )
        .build(|| {
            let origin = ui.cursor_screen_pos();
            let available = ui.content_region_avail();
            let bottom = ui.window_pos()[1] + ui.window_size()[1];
            let footer_y = bottom - ui.clone_style().window_padding()[1] - ui.frame_height();

            if expanded {
                let height = (footer_y - origin[1] - ui.clone_style().item_spacing()[1]).max(1.0);
                ui.child_window("Panel contents")
                    .size([available[0], height])
                    .build(ui, content);
            }

            ui.set_cursor_screen_pos([origin[0], footer_y]);

            footer()
        })
}

pub fn tabs<'ui>(ui: &'ui Ui, id: &str) -> Option<dear_imgui_rs::TabBarToken<'ui>> {
    ui.tab_bar(id)
}

pub fn tab<'ui>(ui: &'ui Ui, label: &str) -> Option<dear_imgui_rs::TabItemToken<'ui>> {
    ui.tab_item(label)
}

pub fn anchored<R>(
    ui: &Ui,
    name: &str,
    position: [f32; 2],
    size: Option<[f32; 2]>,
    flags: WindowFlags,
    content: impl FnOnce() -> R,
) -> Option<R> {
    let _padding = ui.push_style_var(StyleVar::WindowPadding([12.0, 12.0]));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
    let background = ui.clone_style().color(StyleColor::WindowBg);
    let _background = ui.push_style_color(StyleColor::WindowBg, background);
    let _child = ui.push_style_color(StyleColor::ChildBg, background);
    let mut window = ui
        .window(name)
        .collapsed(false, Condition::Always)
        .position(position, Condition::Always)
        .flags(
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                | WindowFlags::NO_FOCUS_ON_APPEARING
                | WindowFlags::NO_DOCKING
                | WindowFlags::NO_SAVED_SETTINGS
                | flags,
        );

    if let Some(size) = size {
        window = window.size(size, Condition::Always);
    }

    window.build(content)
}

pub fn fullscreen<R>(
    ui: &Ui,
    title: &str,
    position: [f32; 2],
    size: [f32; 2],
    interactive: bool,
    content: impl FnOnce() -> R,
) -> Option<R> {
    let flags = WindowFlags::NO_DECORATION
        | WindowFlags::NO_MOVE
        | WindowFlags::NO_DOCKING
        | WindowFlags::NO_SAVED_SETTINGS
        | WindowFlags::NO_NAV_FOCUS;
    let _padding = (!interactive).then(|| ui.push_style_var(StyleVar::WindowPadding([0.0; 2])));
    let _background =
        (!interactive).then(|| ui.push_style_color(StyleColor::WindowBg, [0.0, 0.0, 0.0, 1.0]));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
    let _image_border = ui.push_style_var(StyleVar::ImageBorderSize(0.0));

    ui.window(title)
        .collapsed(false, Condition::Always)
        .position(position, Condition::Always)
        .size(size, Condition::Always)
        .flags(
            flags
                | if interactive {
                    WindowFlags::empty()
                } else {
                    WindowFlags::NO_INPUTS
                },
        )
        .build(content)
}

pub fn viewport_panel<R>(ui: &Ui, title: &str, content: impl FnOnce() -> R) -> Option<R> {
    crate::widgets::hide_window_tab_bar(ui);

    ui.window(title)
        .collapsed(false, Condition::Always)
        .flags(
            WindowFlags::NO_SCROLLBAR
                | WindowFlags::NO_SCROLL_WITH_MOUSE
                | WindowFlags::NO_COLLAPSE,
        )
        .build(content)
}

pub fn viewport_body<R>(ui: &Ui, content: impl FnOnce() -> R) -> Option<R> {
    let available = ui.content_region_avail();
    let spacing = ui.clone_style().item_spacing()[1];
    let height = (available[1] - ui.frame_height().max(25.0) - spacing * 3.0 - 3.0).max(1.0);

    ui.child_window("Preview viewport")
        .size([available[0], height])
        .flags(
            WindowFlags::NO_SCROLLBAR
                | WindowFlags::NO_SCROLL_WITH_MOUSE
                | WindowFlags::NO_COLLAPSE,
        )
        .build(ui, content)
}

#[cfg(test)]
mod tests {
    #[test]
    fn floating_windows_reset_to_half_the_viewport_on_reopen() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        for _ in 0..2 {
            let ui = context.frame();

            super::window(ui, "Window size test")
                .build(|| assert_eq!(ui.window_size(), [640.0, 360.0]));

            context.end_frame();
        }

        let ui = context.frame();

        super::window(ui, "Window size test")
            .size([300.0, 200.0], dear_imgui_rs::Condition::Always)
            .build(|| assert_eq!(ui.window_size(), [300.0, 200.0]));

        context.end_frame();
        context.frame();
        context.end_frame();

        let ui = context.frame();

        super::window(ui, "Window size test")
            .build(|| assert_eq!(ui.window_size(), [640.0, 360.0]));

        context.end_frame();

        let mut saved = String::new();

        context.save_ini_settings(&mut saved);

        assert!(!saved.contains("[Window][Window size test]"));
    }
}
