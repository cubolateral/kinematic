use crate::widgets::text_size;

use super::{
    layout::{Layout, TimeRange},
    metrics::{GRID_TARGET_SPACING, LABEL_PADDING, SCRUBBER_HEIGHT},
};

pub fn draw_time_grid_lines(
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
    let color = ui.get_color_u32(dear_imgui_rs::StyleColor::TreeLines);
    let mut tick = first_tick;

    while tick <= time.end + step * 0.001 {
        let x = time.x(layout, tick);

        draw_list.add_line_v(
            x,
            layout.top + SCRUBBER_HEIGHT - 10.0,
            layout.top + SCRUBBER_HEIGHT - 2.0,
            color,
            1.0,
        );

        tick += step;
    }
}

pub fn draw_time_grid_labels(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    fps: u32,
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
            format_timecode(tick, fps),
        );

        tick += step;
    }
}

pub fn draw_playhead(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    playhead_x: f32,
) {
    draw_list.add_line_v(
        playhead_x,
        layout.viewport_top,
        layout.bottom,
        ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark),
        2.0,
    );
}

pub fn draw_time_panel(
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    playhead_x: f32,
    fps: u32,
) {
    let accent = ui.style_color(dear_imgui_rs::StyleColor::CheckMark);
    let text_color = crate::theme::contrast_text(accent);
    let text = format_timecode(time.current, fps);
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

    draw_list.add_line_v(
        playhead_x,
        max[1] - 1.0,
        layout.viewport_top.max(max[1]),
        ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark),
        2.0,
    );
    draw_list
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark),
        )
        .rounding(ui.clone_style().frame_rounding())
        .filled(true)
        .build();
    draw_list.add_text(position, text_color, text);
}

pub fn draw_mouse_indicator(
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

pub fn format_timecode(time: f32, fps: u32) -> String {
    let fps = fps.max(1) as u64;
    let total_frames = (time.max(0.0) as f64 * fps as f64).round() as u64;
    let frames = total_frames % fps;
    let total_seconds = total_frames / fps;

    format!(
        "{:02}:{:02}:{:02}:{:02}",
        total_seconds / 3600,
        (total_seconds / 60) % 60,
        total_seconds % 60,
        frames
    )
}

pub fn format_time(time: f32, decimals: usize) -> String {
    let scale = 10_u64.pow(decimals as u32);
    let ticks = (time.max(0.0) as f64 * scale as f64).round() as u64;
    let seconds = ticks % (60 * scale);
    let whole_seconds = seconds / scale;
    let fraction = seconds % scale;
    let seconds = if decimals == 0 {
        format!("{whole_seconds:02}")
    } else {
        format!("{whole_seconds:02}:{fraction:0decimals$}")
    };
    let minutes = ticks / (60 * scale);

    if minutes < 60 {
        format!("{minutes:02}:{seconds}")
    } else {
        format!("{}:{:02}:{seconds}", minutes / 60, minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::{format_time, format_timecode};

    #[test]
    fn timecode_always_includes_padded_hours_and_frames() {
        assert_eq!(format_timecode(5.0, 60), "00:00:05:00");
        assert_eq!(format_timecode(0.5, 60), "00:00:00:30");
        assert_eq!(format_timecode(65.25, 60), "00:01:05:15");
        assert_eq!(format_timecode(3661.5, 60), "01:01:01:30");
    }

    #[test]
    fn clock_time_rolls_over_minutes_and_hours() {
        assert_eq!(format_time(0.0, 0), "00:00");
        assert_eq!(format_time(59.996, 2), "01:00:00");
        assert_eq!(format_time(65.25, 2), "01:05:25");
        assert_eq!(format_time(3661.5, 2), "1:01:01:50");
    }
}

pub fn progress_line(
    ui: &dear_imgui_rs::Ui,
    draw: &dear_imgui_rs::DrawListMut<'_>,
    min: [f32; 2],
    width: f32,
    ratio: f32,
) {
    let x = min[0] + width * ratio.clamp(0.0, 1.0);

    draw.add_line_h(
        x,
        min[0] + width,
        min[1],
        ui.get_color_u32(dear_imgui_rs::StyleColor::FrameBg),
        2.0,
    );
    draw.add_line_h(
        min[0],
        x,
        min[1],
        ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
        2.0,
    );
}

pub fn duration_bar(ui: &dear_imgui_rs::Ui, time: f32, duration: f32) {
    let min = ui.cursor_screen_pos();
    let width = ui.content_region_avail_width().max(1.0);
    let line_y = min[1] + 1.0;

    ui.dummy([width, 3.0]);

    progress_line(
        ui,
        &ui.get_window_draw_list(),
        [min[0], line_y],
        width,
        if duration > 0.0 { time / duration } else { 0.0 },
    );
}
