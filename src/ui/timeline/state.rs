use super::metrics::DRAG_DIRECTION_THRESHOLD;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Interaction {
    #[default]
    None,
    Scrubber,
    Tracks,
    Zoom,
    Pan,
    Event,
}

#[derive(Clone, Copy)]
pub(super) struct EventDrag {
    pub scene: usize,
    pub event: usize,
    pub file_event: usize,
    pub duration: f32,
    grab_offset: f32,
}

#[derive(Default)]
pub(in crate::ui) struct State {
    duration: f32,
    view_start: f32,
    view_end: f32,
    pub(super) interaction: Interaction,
    pressed_entity: Option<Option<hecs::Entity>>,
    pressed_toggle: bool,
    expanded_objects: std::collections::HashSet<hecs::Entity>,
    event_drag: Option<EventDrag>,
}

impl State {
    pub fn sync_duration(&mut self, duration: f32) {
        if self.duration != duration {
            self.duration = duration;
            self.view_start = 0.0;
            self.view_end = duration;
        }
    }

    pub fn view_range(&self) -> (f32, f32) {
        (self.view_start, self.view_end)
    }

    pub fn press(&mut self, timeline_hovered: bool, right_pressed: bool) {
        self.interaction = if timeline_hovered {
            if right_pressed {
                Interaction::Scrubber
            } else {
                Interaction::Tracks
            }
        } else {
            Interaction::None
        };
    }

    pub fn resolve_tracks(&mut self, delta: [f32; 2]) {
        if self.interaction != Interaction::Tracks {
            return;
        }

        let horizontal = delta[0].abs();
        let vertical = delta[1].abs();
        if horizontal.max(vertical) >= DRAG_DIRECTION_THRESHOLD {
            self.interaction = if vertical > horizontal {
                Interaction::Zoom
            } else {
                Interaction::Pan
            };
        }
    }

    pub fn zoom(&mut self, delta_y: f32, anchor: f32) {
        if self.duration <= 0.0 || delta_y == 0.0 {
            return;
        }

        let old_span = (self.view_end - self.view_start).max(f32::EPSILON);
        let min_span = (self.duration * 0.01).max(0.05).min(self.duration);
        let new_span = (old_span * (delta_y * 0.01).exp()).clamp(min_span, self.duration);

        let anchor = anchor.clamp(0.0, 1.0);
        let anchor_time = self.view_start + old_span * anchor;
        let new_start = anchor_time - new_span * anchor;

        self.set_view(new_start, new_start + new_span);
    }

    pub fn pan(&mut self, delta_x: f32, width: f32) {
        if self.duration <= 0.0 || width <= 0.0 {
            return;
        }

        let span = self.view_end - self.view_start;
        self.set_view(
            self.view_start - delta_x / width * span,
            self.view_end - delta_x / width * span,
        );
    }

    pub fn press_entity(&mut self, entity: Option<hecs::Entity>, toggle: bool) {
        self.pressed_entity = Some(entity);
        self.pressed_toggle = toggle;
    }

    pub fn release_entity(
        &mut self,
        entity: Option<hecs::Entity>,
        toggle: bool,
        over_view: bool,
        drag_delta: [f32; 2],
    ) -> Option<(Option<hecs::Entity>, bool)> {
        let pressed = self.pressed_entity.take()?;
        let pressed_toggle = std::mem::take(&mut self.pressed_toggle);
        let moved = drag_delta[0].abs().max(drag_delta[1].abs()) >= DRAG_DIRECTION_THRESHOLD;

        if over_view
            && matches!(self.interaction, Interaction::None | Interaction::Tracks)
            && !moved
            && pressed == entity
            && (!pressed_toggle || toggle)
        {
            Some((entity, pressed_toggle))
        } else {
            None
        }
    }

    pub fn toggle_object(&mut self, entity: hecs::Entity) {
        if !self.expanded_objects.remove(&entity) {
            self.expanded_objects.insert(entity);
        }
    }

    pub fn is_object_expanded(&self, entity: hecs::Entity) -> bool {
        self.expanded_objects.contains(&entity)
    }

    pub(super) fn begin_event_drag(
        &mut self,
        scene: usize,
        event: usize,
        file_event: usize,
        duration: f32,
        mouse_duration: f32,
    ) {
        self.event_drag = Some(EventDrag {
            scene,
            event,
            file_event,
            duration,
            grab_offset: mouse_duration - duration,
        });
        self.interaction = Interaction::Event;
    }

    pub(super) fn drag_event(&mut self, mouse_duration: f32) {
        if let Some(drag) = &mut self.event_drag {
            drag.duration = (mouse_duration - drag.grab_offset).max(0.0);
        }
    }

    pub(super) fn event_drag(&self) -> Option<EventDrag> {
        self.event_drag
    }

    pub(super) fn finish_event_drag(&mut self) -> Option<EventDrag> {
        self.interaction = Interaction::None;
        self.event_drag.take()
    }

    fn set_view(&mut self, start: f32, end: f32) {
        let span = (end - start).clamp(0.0, self.duration);
        self.view_start = start.clamp(0.0, self.duration - span);
        self.view_end = self.view_start + span;
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn duration_change_resets_the_visible_range() {
        let mut state = State::default();

        state.sync_duration(12.0);

        assert_eq!(state.view_range(), (0.0, 12.0));
    }

    #[test]
    fn zoom_and_pan_keep_the_view_inside_the_duration() {
        let mut state = State::default();
        state.sync_duration(10.0);

        state.zoom(-100.0, 0.5);
        let (zoomed_start, zoomed_end) = state.view_range();
        assert!(zoomed_start > 0.0);
        assert!(zoomed_end < 10.0);

        state.pan(-1_000.0, 100.0);
        let (panned_start, panned_end) = state.view_range();
        assert!(panned_start >= 0.0);
        assert!(panned_end <= 10.0);
    }

    #[test]
    fn event_drag_changes_only_duration_and_clamps_it_to_zero() {
        let mut state = State::default();
        state.begin_event_drag(2, 3, 4, 5.0, 7.0);

        state.drag_event(8.0);
        assert_eq!(state.event_drag().unwrap().duration, 6.0);

        state.drag_event(-2.0);

        let drag = state.finish_event_drag().unwrap();
        assert_eq!(drag.scene, 2);
        assert_eq!(drag.event, 3);
        assert_eq!(drag.file_event, 4);
        assert_eq!(drag.duration, 0.0);
    }
}
