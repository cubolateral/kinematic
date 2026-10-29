//! Flat timeline lanes, blocks, handles, and waveform strokes.

use super::metrics::{CLIP_HANDLE, EVENT_HEIGHT, PANEL_TEXT_PADDING};
use dear_imgui_rs::{DrawListMut, StyleColor, Ui};

pub fn separator(ui: &Ui, draw: &DrawListMut<'_>, left: f32, right: f32, y: f32) {
    draw.add_line_h(left, right, y, ui.get_color_u32(StyleColor::Separator), 1.0);
}

pub fn scene(
    ui: &Ui,
    draw: &DrawListMut<'_>,
    bounds: [[f32; 2]; 2],
    label: &str,
    active: bool,
    hovered: bool,
    focused: bool,
) -> Option<[f32; 2]> {
    let [min, max] = bounds;
    let fill = ui.get_color_u32(if hovered {
        StyleColor::ButtonHovered
    } else if active {
        StyleColor::HeaderActive
    } else {
        StyleColor::Header
    });
    draw.add_rect(min, [(max[0] - 1.0).max(min[0]), max[1]], fill)
        .rounding(0.0)
        .filled(true)
        .build();
    let y = min[1] + (EVENT_HEIGHT - ui.calc_text_size(label)[1]) * 0.5;
    let arrow_spacing = ui.calc_text_size(crate::icons::ARROW_LEFT)[0] + ui.calc_text_size(" ")[0];
    let _clip = draw.push_clip_rect(min, max, true);
    draw.add_text(
        [
            min[0] + PANEL_TEXT_PADDING + if focused { arrow_spacing } else { 0.0 },
            y,
        ],
        ui.get_color_u32(StyleColor::Text),
        label,
    );
    focused.then_some([min[0] + PANEL_TEXT_PADDING, y])
}

pub fn event_span(ui: &Ui, draw: &DrawListMut<'_>, min: [f32; 2], max: [f32; 2]) {
    if max[0] > min[0] {
        draw.add_rect(
            min,
            max,
            ui.get_color_u32_with_alpha(StyleColor::CheckMark, 0.5),
        )
        .rounding(0.0)
        .filled(true)
        .build();
    }
}

pub fn event_handle(ui: &Ui, draw: &DrawListMut<'_>, bounds: [[f32; 2]; 2], label: &str) {
    let [min, max] = bounds;
    let accent = ui.get_color_u32(StyleColor::CheckMark);
    let text = crate::theme::contrast_text(ui.style_color(StyleColor::CheckMark));
    draw.add_rect(min, max, accent)
        .rounding(0.0)
        .filled(true)
        .build();
    let _clip = draw.push_clip_rect(min, max, true);
    draw.add_text(
        [
            min[0] + PANEL_TEXT_PADDING,
            min[1] + (EVENT_HEIGHT - ui.calc_text_size(label)[1]) * 0.5,
        ],
        text,
        label,
    );
}

pub fn back(ui: &Ui, draw: &DrawListMut<'_>, position: [f32; 2]) -> (bool, bool) {
    ui.set_cursor_screen_pos(position);
    let clicked = ui.invisible_button(
        "##back_to_all_scenes",
        ui.calc_text_size(crate::icons::ARROW_LEFT),
    );
    let hovered = ui.is_item_hovered();
    draw.add_text(
        position,
        ui.get_color_u32(if hovered {
            StyleColor::TextDisabled
        } else {
            StyleColor::Text
        }),
        crate::icons::ARROW_LEFT,
    );
    (clicked, hovered)
}

pub fn audio_header(
    ui: &Ui,
    draw: &DrawListMut<'_>,
    min: [f32; 2],
    width: f32,
    height: f32,
    collapsed: bool,
) -> bool {
    separator(ui, draw, min[0], min[0] + width, min[1] + height);
    let clicked = ui.invisible_button("##audio_header", [width, 24.0]);
    draw.add_text(
        [min[0] + 10.0, min[1] + 4.0],
        ui.get_color_u32(StyleColor::Text),
        if collapsed {
            crate::icons::CHEVRON_RIGHT
        } else {
            crate::icons::CHEVRON_DOWN
        },
    );
    draw.add_text(
        [min[0] + 30.0, min[1] + 4.0],
        ui.get_color_u32(StyleColor::Text),
        "Audio",
    );
    clicked
}

pub fn audio(
    ui: &Ui,
    draw: &DrawListMut<'_>,
    bounds: [[f32; 2]; 2],
    selected: bool,
    hovered: bool,
) {
    let [min, max] = bounds;
    draw.add_rect(
        min,
        max,
        ui.get_color_u32(if hovered {
            StyleColor::ButtonHovered
        } else if selected {
            StyleColor::HeaderActive
        } else {
            StyleColor::Header
        }),
    )
    .rounding(0.0)
    .filled(true)
    .build();
}

pub fn audio_label(ui: &Ui, draw: &DrawListMut<'_>, bounds: [[f32; 2]; 2], label: &str) {
    let [min, max] = bounds;
    let _clip = draw.push_clip_rect(min, max, true);
    draw.add_text(
        [min[0] + CLIP_HANDLE, min[1] + 3.0],
        ui.get_color_u32(StyleColor::Text),
        label,
    );
}

pub fn waveform(
    ui: &Ui,
    draw: &DrawListMut<'_>,
    bounds: [[f32; 2]; 2],
    gain: f32,
    mut peaks: impl FnMut(f32, f32) -> [f32; 2],
) {
    let [min, max] = bounds;
    let center = min[1] + 41.0;
    let scale = 17.0 * gain;
    let accent = ui.get_color_u32(StyleColor::CheckMark);
    for pixel in 0..(max[0] - min[0]).ceil() as usize {
        let x = min[0] + pixel as f32;
        let [low, high] = peaks(x, (x + 1.0).min(max[0]));
        draw.add_line(
            [x, center - high * scale],
            [x, center - low * scale],
            accent,
        )
        .build();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timeline_blocks_remain_square_with_rounded_controls() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .style_mut()
            .set_frame_rounding(crate::theme::CORNER_RADIUS);
        let ui = context.frame();
        super::event_span(
            ui,
            &ui.get_foreground_draw_list(),
            [10.0, 20.0],
            [50.0, 70.0],
        );
        let frame = context.render_legacy();
        let positions = frame
            .draw_data()
            .draw_lists()
            .flat_map(|list| list.vtx_buffer().iter().map(|vertex| vertex.pos))
            .collect::<Vec<_>>();
        assert_eq!(
            positions,
            [[10.0, 20.0], [50.0, 20.0], [50.0, 70.0], [10.0, 70.0]]
        );
    }
}

pub fn audio_lanes(
    ui: &Ui,
    draw: &DrawListMut<'_>,
    top: [f32; 2],
    width: f32,
    height: f32,
    count: usize,
) {
    let first = (((draw.clip_rect_min()[1] - top[1]) / height)
        .ceil()
        .max(1.0) as usize)
        .min(count);
    let last = (((draw.clip_rect_max()[1] - top[1]) / height)
        .floor()
        .max(0.0) as usize)
        .min(count.saturating_sub(1));
    for lane in first..=last {
        separator(
            ui,
            draw,
            top[0],
            top[0] + width,
            top[1] + lane as f32 * height - 1.0,
        );
    }
}
