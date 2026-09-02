use super::metrics::{
    OBJECT_PANEL_MAX_WIDTH, OBJECT_PANEL_MIN_WIDTH, OBJECT_PANEL_RATIO, TRACK_SPACING,
};

#[derive(Clone, Copy)]
pub(super) struct TimeRange {
    pub current: f32,
    pub start: f32,
    pub end: f32,
}

impl TimeRange {
    pub fn ratio(self, time: f32) -> f32 {
        if self.end > self.start {
            ((time - self.start) / (self.end - self.start)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn x(self, layout: Layout, time: f32) -> f32 {
        layout.timeline_left + layout.timeline_width * self.ratio(time)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Layout {
    pub content_left: f32,
    pub top: f32,
    pub viewport_top: f32,
    pub bottom: f32,
    pub divider_x: f32,
    pub timeline_left: f32,
    pub timeline_width: f32,
}

impl Layout {
    pub fn new(ui: &dear_imgui_rs::Ui, top: f32) -> Self {
        let [content_left, viewport_top] = ui.cursor_screen_pos();
        let available = ui.content_region_avail_width().max(1.0);
        let [_, window_top] = ui.window_pos();
        let [_, window_height] = ui.window_size();
        let panel_width = (available * OBJECT_PANEL_RATIO)
            .clamp(OBJECT_PANEL_MIN_WIDTH, OBJECT_PANEL_MAX_WIDTH)
            .min((available - 1.0).max(0.0));
        let divider_x = content_left + panel_width;
        let timeline_left = (divider_x + TRACK_SPACING).min(content_left + available);

        Self {
            content_left,
            top,
            viewport_top: viewport_top + ui.scroll_y(),
            bottom: window_top + window_height,
            divider_x,
            timeline_left,
            timeline_width: (content_left + available - timeline_left).max(1.0),
        }
    }

    pub fn timeline_right(self) -> f32 {
        self.timeline_left + self.timeline_width
    }

    pub fn contains_timeline_x(self, x: f32) -> bool {
        (self.timeline_left..=self.timeline_right()).contains(&x)
    }
}
