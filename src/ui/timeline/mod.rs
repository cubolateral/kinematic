mod audio;
mod interaction;
mod scene;
mod state;

use crate::editor::Editor;
use kinematic_ui::{
    timeline::{layout, metrics, ruler, waveform},
    windows,
};

use crate::ui::preview::shortcuts;
use layout::{Layout, TimeRange};
use metrics::SCRUBBER_HEIGHT;
use state::Interaction;
pub(super) use state::State;

#[cfg(test)]
impl State {
    pub(in crate::ui) fn selected_audio_rect(&self) -> Option<[[f32; 2]; 2]> {
        self.audio_clip_rect
    }
}

pub(super) const WINDOW_NAME: &str = "Timeline";

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    keyboard: bool,
    dragged_audio: Option<&std::path::Path>,
) {
    let is_exporting = editor.is_exporting();

    if is_exporting {
        state.cancel_interactions();
    }

    shortcuts(editor.timeline_mut(), ui, !is_exporting && keyboard);

    windows::panel(ui, WINDOW_NAME, || {
        audio_shortcuts(editor, ui, keyboard);
        let _disabled = ui.begin_disabled_with_cond(is_exporting);

        let focused_scene = editor.focused_scene_index();
        let time = {
            let timeline = editor.timeline_mut();

            state.sync_focus(focused_scene, timeline.duration());

            let (start, end) = state.view_range();

            TimeRange {
                current: timeline.time(),
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

                scene::draw(editor, ui, &draw_list, layout, time, state, !is_exporting);

                audio::draw(editor, ui, &draw_list, layout, time, state, dragged_audio);

                ruler::draw_mouse_indicator(ui, &draw_list, layout);

                ruler::draw_playhead(ui, &draw_list, layout, playhead_x);

                view = Some((layout, mouse));
            });

        let Some((layout, mouse)) = view else { return };

        let timeline_hovered = (header_hovered || objects_hovered)
            && layout.contains_timeline_x(mouse[0])
            && (layout.top..=layout.bottom).contains(&mouse[1]);
        if timeline_hovered
            && !state.audio_hovered
            && (ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left)
                || ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Right))
        {
            editor.clear_audio_selection();
        }
        let draw_list = ui.get_window_draw_list();
        let playhead_x = time.x(layout, time.current);

        ruler::draw_scrubber(ui, &draw_list, layout, time);

        ruler::draw_time_grid_lines(ui, &draw_list, layout, time);

        ruler::draw_time_grid_labels(ui, &draw_list, layout, time);

        ruler::draw_time_panel(ui, &draw_list, layout, time, playhead_x);

        if is_exporting {
            state.interaction = Interaction::None;
            editor.timeline_mut().is_controlling = false;
        } else {
            interaction::update(editor.timeline_mut(), ui, layout, state, timeline_hovered);
        }
    });
}

pub(in crate::ui) fn audio_shortcuts(editor: &mut Editor, ui: &dear_imgui_rs::Ui, keyboard: bool) {
    if keyboard
        && ui.is_window_focused_with_flags(dear_imgui_rs::FocusedFlags::ROOT_AND_CHILD_WINDOWS)
        && editor.audio_editable()
        && editor.selected_audio().is_some()
        && !ui.io().want_text_input()
        && !ui.is_any_item_active()
    {
        if crate::ui::inputs::active(ui, crate::ui::inputs::Action::SplitAudio) {
            editor.split_audio();
        }
        if crate::ui::inputs::active(ui, crate::ui::inputs::Action::DeleteAudio) {
            editor.delete_audio();
        }
    }
}

pub(in crate::ui) fn is_audio_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ["wav", "mp3", "ogg", "flac"]
                .iter()
                .any(|format| ext.eq_ignore_ascii_case(format))
        })
}
