use super::{
    layout::{Layout, TimeRange},
    metrics::{
        KEYFRAME_HITBOX_SIZE, KEYFRAME_HOVER_SCALE, KEYFRAME_RADIUS, PANEL_TEXT_PADDING,
        SEGMENT_THICKNESS, TRACK_HEIGHT, TRACK_SPACING,
    },
};
use crate::{
    core::{Track, components::Animation},
    ui::widgets::text_size,
};

#[derive(Clone, Copy)]
struct TrackView {
    layout: Layout,
    time: TimeRange,
    keyframe: u32,
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
            keyframe: ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            text: ui.get_color_u32(dear_imgui_rs::StyleColor::Text),
            active: ui.get_color_u32(dear_imgui_rs::StyleColor::CheckMark),
            inactive: ui.get_color_u32(dear_imgui_rs::StyleColor::Separator),
            hovered: ui.is_window_hovered(),
        }
    }

    fn object(
        self,
        ui: &dear_imgui_rs::Ui,
        draw_list: &dear_imgui_rs::DrawListMut<'_>,
        animation: &Animation,
        lifetime: [f32; 2],
        time_offset: f32,
        top: f32,
        name_x: f32,
    ) {
        for (row, animation_track) in animation.tracks.iter().enumerate() {
            self.track(
                ui,
                draw_list,
                &animation_track.track,
                lifetime,
                time_offset,
                top + row as f32 * (TRACK_HEIGHT + TRACK_SPACING),
                name_x,
            );
        }
    }

    fn track(
        self,
        ui: &dear_imgui_rs::Ui,
        draw_list: &dear_imgui_rs::DrawListMut<'_>,
        track: &Track,
        lifetime: [f32; 2],
        time_offset: f32,
        top: f32,
        name_x: f32,
    ) {
        let label = track.info.name;
        let clip = draw_list.push_clip_rect(
            [name_x, top],
            [
                self.layout.divider_x - PANEL_TEXT_PADDING,
                top + TRACK_HEIGHT,
            ],
            true,
        );
        draw_list.add_text(
            [name_x, top + (TRACK_HEIGHT - text_size(ui, label)[1]) * 0.5],
            self.text,
            label,
        );
        drop(clip);

        let Some([start, end]) = visible_lifetime(lifetime, self.time) else {
            return;
        };

        let center = top + TRACK_HEIGHT * 0.5;
        let start_x = self.time.x(self.layout, start);
        let end_x = self.time.x(self.layout, end);

        draw_list.add_line_h(
            start_x,
            end_x,
            center,
            self.inactive,
            SEGMENT_THICKNESS * 0.5,
        );

        if self.time.end > self.time.start {
            for pair in track.keyframes.windows(2) {
                let [left, right] = pair else { continue };
                if left.easing.is_some() && right.time > left.time {
                    let segment_start = (time_offset + left.time).max(start);
                    let segment_end = (time_offset + right.time).min(end);
                    if segment_end <= segment_start {
                        continue;
                    }

                    draw_list.add_line_h(
                        self.time.x(self.layout, segment_start),
                        self.time.x(self.layout, segment_end),
                        center,
                        self.active,
                        SEGMENT_THICKNESS,
                    );
                }
            }
        }

        let mut hovered_time = None;

        for keyframe in &track.keyframes {
            let keyframe_time = time_offset + keyframe.time;
            if keyframe_time < start || keyframe_time > end {
                continue;
            }

            let x = self.time.x(self.layout, keyframe_time);
            let hit_half_size = KEYFRAME_HITBOX_SIZE * 0.5;
            let hit_min = [x - hit_half_size, center - hit_half_size];
            let hit_max = [x + hit_half_size, center + hit_half_size];
            let hovered = self.hovered && ui.is_mouse_hovering_rect(hit_min, hit_max);

            draw_list
                .add_circle(
                    [x, center],
                    KEYFRAME_RADIUS * if hovered { KEYFRAME_HOVER_SCALE } else { 1.0 },
                    self.keyframe,
                )
                .filled(true)
                .build();

            if hovered {
                hovered_time = Some((keyframe.time, keyframe_time));
            }
        }

        if let Some((local_time, project_time)) = hovered_time {
            ui.tooltip(|| {
                let keyframes = track.keyframes_at(local_time);

                for (index, keyframe) in keyframes.iter().enumerate() {
                    if index > 0 {
                        ui.separator();
                    }

                    ui.text(format!("Time: {project_time:.2}s"));
                    ui.text(format!("Value: {}", keyframe.value));
                    match keyframe.easing {
                        Some(easing) => ui.text(format!("Easing: {easing:?}")),
                        None => ui.text("Easing: None"),
                    }
                }
            });
        }
    }
}

pub(super) fn height(world: &hecs::World, entity: hecs::Entity) -> f32 {
    world.get::<&Animation>(entity).map_or(0.0, |animation| {
        animation.tracks.len() as f32 * (TRACK_HEIGHT + TRACK_SPACING)
    })
}

pub(super) struct ObjectTracks {
    pub entity: hecs::Entity,
    pub lifetime: [f32; 2],
    pub time_offset: f32,
    pub top: f32,
    pub name_x: f32,
}

pub(super) fn draw(
    world: &hecs::World,
    ui: &dear_imgui_rs::Ui,
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    layout: Layout,
    time: TimeRange,
    object: ObjectTracks,
) {
    let Ok(animation) = world.get::<&Animation>(object.entity) else {
        return;
    };

    let view = TrackView::new(ui, layout, time);
    view.object(
        ui,
        draw_list,
        &animation,
        object.lifetime,
        object.time_offset,
        object.top,
        object.name_x,
    );
}

fn visible_lifetime(lifetime: [f32; 2], time: TimeRange) -> Option<[f32; 2]> {
    let start = lifetime[0].max(time.start);
    let end = lifetime[1].min(time.end);

    (end > start).then_some([start, end])
}
