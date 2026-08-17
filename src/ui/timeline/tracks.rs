use super::{
    KEYFRAME_HALF_SIZE, KEYFRAME_HITBOX_SIZE, KEYFRAME_HOVER_SCALE, Layout, SCRUBBER_HEIGHT,
    SEGMENT_THICKNESS, TRACK_HEIGHT, TRACK_SPACING, TRACK_TIMELINE_PADDING, TimeRange, text_size,
};
use crate::{
    core::{Track, components::Animation},
    editor::Editor,
};

#[derive(Clone, Copy)]
struct TrackView {
    layout: Layout,
    time: TimeRange,
    text: u32,
    active: u32,
    inactive: u32,
    hovered: bool,
}

impl TrackView {
    fn new(ui: &dear_imgui_rs::Ui, layout: Layout, time: TimeRange) -> Self {
        Self {
            layout,
            time,
            text: ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            active: ui.get_color_u32(dear_imgui_rs::StyleColor::SliderGrabActive),
            inactive: ui.get_color_u32(dear_imgui_rs::StyleColor::Separator),
            hovered: ui.is_window_hovered(),
        }
    }

    fn entity(
        self,
        ui: &dear_imgui_rs::Ui,
        draw_list: &dear_imgui_rs::DrawListMut<'_>,
        entity: hecs::Entity,
        animation: &Animation,
    ) {
        let id = entity.id();
        let clip = ui.push_clip_rect(
            [self.layout.content_left, self.layout.window_top],
            [
                self.layout.divider_x - TRACK_TIMELINE_PADDING,
                self.layout.bottom,
            ],
            true,
        );
        ui.separator_with_text(format!("Entity {id}"));
        drop(clip);

        let origin = ui.cursor_screen_pos();
        let height = animation.tracks.len().checked_sub(1).map_or(0.0, |last| {
            last as f32 * (TRACK_HEIGHT + TRACK_SPACING) + TRACK_HEIGHT
        });
        ui.dummy([ui.content_region_avail_width().max(1.0), height.max(1.0)]);

        for (row, animation_track) in animation.tracks.iter().enumerate() {
            self.track(ui, draw_list, &animation_track.track, origin, row);
        }
        ui.spacing();
    }

    fn track(
        self,
        ui: &dear_imgui_rs::Ui,
        draw_list: &dear_imgui_rs::DrawListMut<'_>,
        track: &Track,
        origin: [f32; 2],
        row: usize,
    ) {
        let top = origin[1] + row as f32 * (TRACK_HEIGHT + TRACK_SPACING);
        let center = top + TRACK_HEIGHT * 0.5;

        draw_list.add_line_h(
            self.layout.timeline_left,
            self.layout.timeline_right(),
            center,
            self.inactive,
            SEGMENT_THICKNESS * 0.5,
        );

        if self.time.end > self.time.start {
            for pair in track.keyframes.windows(2) {
                let [left, right] = pair else { continue };
                if left.easing.is_some() && right.time > left.time {
                    draw_list.add_line_h(
                        self.time.x(self.layout, left.time),
                        self.time.x(self.layout, right.time),
                        center,
                        self.active,
                        SEGMENT_THICKNESS,
                    );
                }
            }
        }

        let clip = draw_list.push_clip_rect(
            [origin[0], top],
            [
                (self.layout.divider_x - TRACK_SPACING).max(origin[0] + 1.0),
                top + TRACK_HEIGHT,
            ],
            true,
        );
        draw_list.add_text(
            [
                origin[0],
                top + (TRACK_HEIGHT - text_size(ui, track.info.name)[1]) * 0.5,
            ],
            self.text,
            track.info.name,
        );
        drop(clip);

        let mut hovered_keyframe = None;

        for keyframe in &track.keyframes {
            let x = self.time.x(self.layout, keyframe.time);
            let hit_half_size = KEYFRAME_HITBOX_SIZE * 0.5;
            let hit_min = [x - hit_half_size, center - hit_half_size];
            let hit_max = [x + hit_half_size, center + hit_half_size];
            let hovered = self.hovered && ui.is_mouse_hovering_rect(hit_min, hit_max);
            let half_size = KEYFRAME_HALF_SIZE * if hovered { KEYFRAME_HOVER_SCALE } else { 1.0 };
            let min = [x - half_size, center - half_size];
            let max = [x + half_size, center + half_size];

            draw_list.add_rect(min, max, [1.0; 4]).filled(true).build();

            if hovered {
                hovered_keyframe = Some(keyframe);
            }
        }

        if let Some(keyframe) = hovered_keyframe {
            ui.tooltip(|| {
                ui.text(format!("Time: {:.2}s", keyframe.time));
                ui.text(format!("Value: {}", keyframe.value));
                match keyframe.easing {
                    Some(easing) => ui.text(format!("Easing: {easing:?}")),
                    None => ui.text("Easing: None"),
                }
            });
        }
    }
}

pub(super) fn draw(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
) {
    ui.set_cursor_screen_pos([layout.content_left, layout.top + SCRUBBER_HEIGHT]);
    ui.separator();
    ui.dummy([0.0, TRACK_SPACING]);

    let world = editor.get_scene().get_world();
    let mut query = world.query::<(hecs::Entity, &Animation)>();
    let mut entities: Vec<_> = query.iter().collect();
    entities.sort_by_key(|(entity, _)| entity.id());

    let view = TrackView::new(ui, layout, time);
    for (entity, animation) in entities {
        view.entity(ui, draw_list, entity, animation);
    }
}
