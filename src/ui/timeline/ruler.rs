use crate::ui::widgets::text_size;

use super::{
    layout::{Layout, TimeRange},
    metrics::{GRID_TARGET_SPACING, LABEL_PADDING, SCRUBBER_HEIGHT},
};

pub(super) fn draw_panel_divider(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    top: f32,
    bottom: f32,
) {
    draw_list.add_line_v(
        layout.divider_x,
        top,
        bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::Separator),
        1.0,
    );
}

pub(super) fn draw_scrubber(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
) {
    let text_y = layout.top + 8.0;
    let line_y = layout.top + SCRUBBER_HEIGHT - 2.0;
    let end_text = format!("{:.2}s", time.end);
    let end_width = text_size(ui, &end_text)[0];

    draw_list.add_text(
        [layout.timeline_right() - end_width, text_y],
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        end_text,
    );
    draw_list.add_line_h(
        playhead_x,
        layout.timeline_right(),
        line_y,
        ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg),
        2.0,
    );
    draw_list.add_line_h(
        layout.timeline_left,
        playhead_x,
        line_y,
        ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
        2.0,
    );
}

pub(super) fn draw_time_grid_lines(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
) {
    let step = grid_step(time, layout.timeline_width);
    if step <= 0.0 {
        return;
    }

    let first_tick = (time.start / step).ceil() * step;
    let color = ui.get_color_u32(dear_imgui_rs::StyleColor::Separator);
    let mut tick = first_tick;

    while tick <= time.end + step * 0.001 {
        let x = time.x(layout, tick);
        draw_list.add_line_v(x, layout.viewport_top, layout.bottom, color, 1.0);
        tick += step;
    }
}

pub(super) fn draw_time_grid_labels(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
) {
    let step = grid_step(time, layout.timeline_width);
    if step <= 0.0 {
        return;
    }

    let first_tick = (time.start / step).ceil() * step;
    let color = ui.get_color_u32(dear_imgui_rs::StyleColor::TextDisabled);
    let mut tick = first_tick;

    while tick <= time.end + step * 0.001 {
        draw_list.add_text(
            [time.x(layout, tick) + 3.0, layout.top + 8.0],
            color,
            format_grid_time(tick, step),
        );
        tick += step;
    }
}

pub(super) fn draw_playhead(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    playhead_x: f32,
) {
    draw_list.add_line_v(
        playhead_x,
        layout.viewport_top,
        layout.bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
        2.0,
    );
}

pub(super) fn draw_time_panel(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
) {
    let text_color = ui.get_color_u32(dear_imgui_rs::StyleColor::Text);
    draw_list.add_line_v(
        playhead_x,
        layout.top + SCRUBBER_HEIGHT - 10.0,
        layout.viewport_top,
        text_color,
        2.0,
    );

    let text = format!("{:.2}s", time.current);
    let size = text_size(ui, &text);
    let position = [playhead_x - size[0] * 0.5, layout.top + 8.0];
    let min = [
        position[0] - LABEL_PADDING[0],
        position[1] - LABEL_PADDING[1],
    ];
    let max = [
        position[0] + size[0] + LABEL_PADDING[0],
        position[1] + size[1] + LABEL_PADDING[1],
    ];

    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::PopupBg),
        )
        .filled(true)
        .build();
    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .build();
    draw_list.add_text(position, text_color, text);
}

pub(super) fn draw_mouse_indicator(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
) {
    let mouse = ui.io().mouse_pos();
    if !layout.contains_timeline_x(mouse[0])
        || mouse[1] < layout.viewport_top
        || mouse[1] > layout.bottom
    {
        return;
    }

    draw_list.add_line_v(
        mouse[0],
        layout.viewport_top,
        layout.bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::TextDisabled),
        1.0,
    );
}

fn grid_step(time: TimeRange, width: f32) -> f32 {
    let span = time.end - time.start;
    if span <= 0.0 || width <= 0.0 {
        return 0.0;
    }

    let raw_step = span / (width / GRID_TARGET_SPACING);
    let magnitude = 10.0_f32.powf(raw_step.log10().floor());
    let normalized = raw_step / magnitude;
    let multiple = if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    };

    multiple * magnitude
}

fn format_grid_time(time: f32, step: f32) -> String {
    let mut decimals = 0;
    let mut value = step.abs();
    while value < 1.0 && decimals < 6 {
        value *= 10.0;
        decimals += 1;
    }

    format!("{:.*}s", decimals, time)
}

#[cfg(test)]
mod tests {
    use super::format_grid_time;

    #[test]
    fn grid_time_precision_follows_the_grid_step() {
        assert_eq!(format_grid_time(5.0, 1.0), "5s");
        assert_eq!(format_grid_time(0.5, 0.1), "0.5s");
        assert_eq!(format_grid_time(0.025, 0.01), "0.025s");
    }
}
