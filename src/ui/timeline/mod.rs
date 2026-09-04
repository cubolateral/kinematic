mod controls;
mod interaction;
mod layout;
mod metrics;
mod objects;
mod ruler;
mod scene;
mod state;
mod tracks;

use crate::{editor::Editor, ui::widgets::hide_single_window_tab};

pub(super) use controls::{fullscreen_controls, shortcuts};
use layout::{Layout, TimeRange};
use metrics::SCRUBBER_HEIGHT;
use state::Interaction;
pub(super) use state::State;

pub(super) const WINDOW_NAME: &str = "Timeline";

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) -> bool {
    let is_exporting = editor.is_exporting();
    let mut toggle_fullscreen = false;

    shortcuts(editor.get_timeline(), ui, !is_exporting);
    hide_single_window_tab(ui);

    ui.window(WINDOW_NAME).build(|| {
        let fps = editor.get_preview_fps();
        let _disabled = ui.begin_disabled_with_cond(is_exporting);

        toggle_fullscreen = controls::draw(editor.get_timeline(), ui, fps, !is_exporting, false);
        ui.spacing();
        ui.separator();

        let time = {
            let timeline = editor.get_timeline();
            state.sync_duration(timeline.get_duration());
            let (start, end) = state.view_range();
            TimeRange {
                current: timeline.get_time(),
                start,
                end,
            }
        };
        let [content_left, top] = ui.cursor_screen_pos();
        let available_height = ui.content_region_avail_height();
        let child_height = (available_height - SCRUBBER_HEIGHT).max(1.0);
        let header_hovered = ui.is_window_hovered_with_flags(
            dear_imgui_rs::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
        );
        let mut view = None;
        let mut objects_hovered = false;

        ui.set_cursor_screen_pos([content_left, top + SCRUBBER_HEIGHT]);
        ui.child_window("Timeline Objects")
            .size([0.0, child_height])
            .flags(dear_imgui_rs::WindowFlags::ALWAYS_VERTICAL_SCROLLBAR)
            .build(ui, || {
                let layout = Layout::new(ui, top);
                let mouse = ui.io().mouse_pos();
                objects_hovered = ui.is_window_hovered_with_flags(
                    dear_imgui_rs::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
                );
                let draw_list = ui.get_window_draw_list();
                let playhead_x = time.x(layout, time.current);

                ruler::draw_time_grid_lines(ui, &draw_list, layout, time);
                scene::draw(editor, ui, &draw_list, layout, time);
                objects::draw(editor, ui, &draw_list, layout, time, state);
                ruler::draw_panel_divider(
                    ui,
                    &draw_list,
                    layout,
                    layout.viewport_top,
                    layout.bottom,
                );
                ruler::draw_mouse_indicator(ui, &draw_list, layout);
                ruler::draw_playhead(ui, &draw_list, layout, playhead_x);
                view = Some((layout, mouse));
            });

        let Some((layout, mouse)) = view else { return };
        let timeline_hovered = (header_hovered || objects_hovered)
            && layout.contains_timeline_x(mouse[0])
            && (layout.top..=layout.bottom).contains(&mouse[1]);
        let draw_list = ui.get_window_draw_list();
        let playhead_x = time.x(layout, time.current);

        ruler::draw_scrubber(ui, &draw_list, layout, time, playhead_x);
        ruler::draw_panel_divider(ui, &draw_list, layout, layout.top, layout.viewport_top);
        ruler::draw_time_grid_labels(ui, &draw_list, layout, time);
        ruler::draw_time_panel(ui, &draw_list, layout, time, playhead_x);

        if is_exporting {
            state.interaction = Interaction::None;
            editor.get_timeline().is_controlling = false;
        } else {
            interaction::update(editor.get_timeline(), ui, layout, state, timeline_hovered);
        }
    });

    toggle_fullscreen
}
