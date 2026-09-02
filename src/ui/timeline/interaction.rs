use crate::editor::Timeline;

use super::{
    layout::Layout,
    metrics::DRAG_DIRECTION_THRESHOLD,
    state::{Interaction, State},
};

pub(super) fn update(
    timeline: &mut Timeline,
    ui: &dear_imgui_rs::Ui,
    layout: Layout,
    state: &mut State,
    timeline_hovered: bool,
) {
    let mouse = ui.io().mouse_pos();
    let mouse_delta = ui.io().mouse_delta();
    let left = dear_imgui_rs::MouseButton::Left;
    let right = dear_imgui_rs::MouseButton::Right;
    let left_down = ui.is_mouse_down(left);
    let right_down = ui.is_mouse_down(right);
    let left_pressed = ui.is_mouse_clicked(left);
    let right_pressed = ui.is_mouse_clicked(right);

    if !left_down && !right_down {
        state.interaction = Interaction::None;
    } else if left_pressed || right_pressed {
        state.press(timeline_hovered, right_pressed || right_down);
    } else {
        state.resolve_tracks(ui.mouse_drag_delta_with_threshold(left, DRAG_DIRECTION_THRESHOLD));
    }

    if state.interaction == Interaction::Zoom && left_down {
        let anchor = ((mouse[0] - layout.timeline_left) / layout.timeline_width).clamp(0.0, 1.0);
        state.zoom(mouse_delta[1], anchor);
    }

    if state.interaction == Interaction::Pan && left_down {
        state.pan(mouse_delta[0], layout.timeline_width);
    }

    timeline.is_controlling = state.interaction == Interaction::Scrubber && right_down;
    if timeline.is_controlling {
        let (start, end) = state.view_range();
        let visible_time = start
            + ((mouse[0] - layout.timeline_left) / layout.timeline_width).clamp(0.0, 1.0)
                * (end - start);
        timeline.go_to(visible_time);
    }
}
