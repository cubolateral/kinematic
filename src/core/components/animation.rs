use crate::core::{
    Easing, Track, TrackInfo, TrackValue,
    types::{Quaternion, Vector3},
};

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
        self.track_mut(type_id, track_info)
            .add_tween(current_time, from, to, duration, easing);
    }

    pub fn animate_rotation(
        &mut self,
        current_time: f32,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: Quaternion,
        axis: Vector3,
        angle: f32,
        duration: f32,
        easing: Easing,
    ) {
        self.track_mut(type_id, track_info).add_rotation_tween(
            current_time,
            from,
            axis,
            angle,
            duration,
            easing,
        );
    }

    fn track_mut(
        &mut self,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
    ) -> &mut Track {
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

        &mut self.tracks[index].track
    }
}
