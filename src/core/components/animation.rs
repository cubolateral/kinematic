use crate::core::{Easing, Track, TrackId, TrackSetter, TrackValue};

/// Associates a track with the component field it animates.
///
/// The key lives here because it is only needed while compiling tweens into
/// tracks; a compiled [`Track`] only needs its setter and keyframes.
pub(crate) struct AnimationTrack {
    type_id: std::any::TypeId,
    track_id: TrackId,
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
        track_id: TrackId,
        track_setter: TrackSetter,
        from: TrackValue,
        to: TrackValue,
        duration: f32,
        easing: Easing,
    ) {
        let index = match self
            .tracks
            .iter()
            .position(|track| track.type_id == type_id && track.track_id == track_id)
        {
            Some(index) => index,
            None => {
                self.tracks.push(AnimationTrack {
                    type_id,
                    track_id,
                    track: Track::new(track_setter),
                });
                self.tracks.len() - 1
            }
        };

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
