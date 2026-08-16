mod inspector;
mod preview;
mod timeline;

use crate::editor::Editor;

pub(crate) struct Ui;

impl Ui {
    pub fn draw(editor: &mut Editor, ui: &mut dear_imgui_rs::Ui) {
        dock_layout(ui);
        inspector::draw(editor, ui);
        preview::draw(editor, ui);
        timeline::draw(editor, ui);
    }
}

fn dock_layout(ui: &dear_imgui_rs::Ui) {
    let dock = ui.dockspace_over_main_viewport();

    static INITIALIZE: std::sync::Once = std::sync::Once::new();
    INITIALIZE.call_once(|| {
        dear_imgui_rs::DockBuilder::remove_node(ui, dock);
        dear_imgui_rs::DockBuilder::add_node(ui, dock, dear_imgui_rs::DockNodeFlags::NONE);
        dear_imgui_rs::DockBuilder::set_node_size(ui, dock, ui.main_viewport().size());

        let (inspector, right) = dear_imgui_rs::DockBuilder::split_node(
            ui,
            dock,
            dear_imgui_rs::SplitDirection::Left,
            0.2,
        );
        let (timeline, preview) = dear_imgui_rs::DockBuilder::split_node(
            ui,
            right,
            dear_imgui_rs::SplitDirection::Down,
            0.35,
        );

        dear_imgui_rs::DockBuilder::dock_window(ui, "Inspector", inspector);
        dear_imgui_rs::DockBuilder::dock_window(ui, "Preview", preview);
        dear_imgui_rs::DockBuilder::dock_window(ui, "Timeline", timeline);
        dear_imgui_rs::DockBuilder::finish(ui, dock);
    });
}
