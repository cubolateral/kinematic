use super::{code, inspector, preview, scene_tree, timeline};
use dear_imgui_rs::{DockLayout, DockLayoutApply, DockSplit};

pub(super) fn draw(ui: &dear_imgui_rs::Ui) {
    let layout = DockLayout::split(
        DockSplit::Down,
        0.35,
        DockLayout::split(
            DockSplit::Left,
            0.25,
            tabs([scene_tree::WINDOW_NAME, code::FILES_WINDOW]),
            tabs([
                timeline::WINDOW_NAME,
                code::DIAGNOSTICS_WINDOW,
                code::CONSOLE_WINDOW,
            ]),
        ),
        DockLayout::split(
            DockSplit::Right,
            0.75,
            DockLayout::split(
                DockSplit::Left,
                0.5,
                tabs([code::WINDOW_NAME]),
                tabs([preview::WINDOW_NAME]),
            ),
            tabs([inspector::WINDOW_NAME]),
        ),
    );
    ui.dockspace()
        .layout(&layout, DockLayoutApply::IfMissing)
        .build()
        .expect("Workspace layout must be valid.");
}

fn tabs<const N: usize>(names: [&str; N]) -> DockLayout {
    DockLayout::tabs(names.map(|name| dear_imgui_rs::WindowKey::new(name, name).unwrap()))
}
