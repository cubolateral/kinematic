//! Shared floating window and dialog templates, independent of editor state.

use dear_imgui_rs::{Condition, ModalPopupToken, PopupToken, Ui, Window, WindowFlags};

pub(super) fn window<'ui>(ui: &'ui Ui, title: &'ui str, size: [f32; 2]) -> Window<'ui> {
    center_next(ui);
    ui.window(title)
        .flags(WindowFlags::NO_DOCKING)
        .size(size, Condition::FirstUseEver)
}

pub(super) fn modal<'ui>(
    ui: &'ui Ui,
    title: &'ui str,
    open: &'ui mut bool,
) -> Option<ModalPopupToken<'ui>> {
    sized_modal(ui, title, open, [480.0, 0.0])
}

pub(super) fn sized_modal<'ui>(
    ui: &'ui Ui,
    title: &'ui str,
    open: &'ui mut bool,
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
    ui.begin_modal_popup_config(title)
        .opened(open)
        .flags(flags)
        .begin()
}

pub(super) fn popup<'ui>(ui: &'ui Ui, title: &str, width: f32) -> Option<PopupToken<'ui>> {
    center_next(ui);
    set_next_size([width, 0.0], Condition::Always);
    ui.begin_popup(title)
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
