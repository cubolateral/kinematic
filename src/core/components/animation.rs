use crate::core::{Easing, Track, TrackInfo, TrackValue};

/// Associates a track with the component field it animates.
///
/// The component type completes the field metadata stored by [`Track`] to form
/// the key used while compiling tweens.
pub(crate) struct AnimationTrack {
    type_id: std::any::TypeId,
    pub(crate) track: Track,
}

#[derive(Default)]
pub(crate) struct Animation {
    pub tracks: Vec<AnimationTrack>,
}

impl Animation {
    pub fn animate(
        &mut self,
        current_time: f32,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: TrackValue,
        to: TrackValue,
        duration: f32,
        easing: Easing,
    ) {
        let index = match self
            .tracks
            .iter()
            .position(|track| track.type_id == type_id && track.track.info.id == track_info.id)
        {
            Some(index) => index,
            None => {
                self.tracks.push(AnimationTrack {
                    type_id,
                    track: Track::new(track_info),
                });
                self.tracks.len() - 1
            }
        };

        // An existing track must continue from its latest compiled value. This
        // lets repeated tasks start from the preceding repetition's target
        // instead of returning to the value captured when they were declared.
        let from = self.tracks[index]
            .track
            .keyframes
            .last()
            .map_or(from, |keyframe| keyframe.value.clone());

        // Each tween contributes a start keyframe that owns the easing for the
        // following segment, followed by a target keyframe with no outgoing easing.
        self.tracks[index]
            .track
            .set_keyframe(current_time, from, Some(easing));
        self.tracks[index]
            .track
            .set_keyframe(current_time + duration, to, None);
    }
}
