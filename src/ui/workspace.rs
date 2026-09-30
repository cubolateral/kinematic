use super::{inspector, preview, scene_tree, settings, timeline};

const TIMELINE_HEIGHT_RATIO: f32 = 0.35;
const LEFT_PANEL_WIDTH_RATIO: f32 = 0.2;
const PREVIEW_WIDTH_RATIO: f32 = 0.75;

pub(super) fn apply_default_layout(ui: &dear_imgui_rs::Ui, dock: dear_imgui_rs::Id) {
    dear_imgui_rs::DockBuilder::remove_node(ui, dock);
    dear_imgui_rs::DockBuilder::add_node(ui, dock, dear_imgui_rs::DockNodeFlags::NONE);
    dear_imgui_rs::DockBuilder::set_node_size(ui, dock, ui.main_viewport().work_size());

    let (bottom_node, top_node) = dear_imgui_rs::DockBuilder::split_node(
        ui,
        dock,
        dear_imgui_rs::SplitDirection::Down,
        TIMELINE_HEIGHT_RATIO,
    );
    let (left_node, timeline_node) = dear_imgui_rs::DockBuilder::split_node(
        ui,
        bottom_node,
        dear_imgui_rs::SplitDirection::Left,
        LEFT_PANEL_WIDTH_RATIO,
    );
    let (preview_node, details_node) = dear_imgui_rs::DockBuilder::split_node(
        ui,
        top_node,
        dear_imgui_rs::SplitDirection::Right,
        PREVIEW_WIDTH_RATIO,
    );

    dear_imgui_rs::DockBuilder::dock_window(ui, scene_tree::WINDOW_NAME, left_node);
    dear_imgui_rs::DockBuilder::dock_window(ui, preview::WINDOW_NAME, preview_node);
    dear_imgui_rs::DockBuilder::dock_window(ui, inspector::WINDOW_NAME, details_node);
    dear_imgui_rs::DockBuilder::dock_window(ui, settings::WINDOW_NAME, details_node);
    dear_imgui_rs::DockBuilder::dock_window(ui, timeline::WINDOW_NAME, timeline_node);
    dear_imgui_rs::DockBuilder::finish(ui, dock);
}
