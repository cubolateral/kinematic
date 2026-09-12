use crate::core::{Track, TrackInfo, TrackValue};

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
    pub(crate) fn animates(
        &self,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
    ) -> bool {
        self.tracks.iter().any(|track| {
            track.type_id == type_id
                && track.track.info.id == track_info.id
                && !track.track.keyframes.is_empty()
        })
    }

    pub(crate) fn sample(
        &self,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        time: f32,
    ) -> Option<TrackValue> {
        self.tracks
            .iter()
            .find(|track| track.type_id == type_id && track.track.info.id == track_info.id)
            .and_then(|track| track.track.sample(time))
    }

    pub(crate) fn replace_values(
        &mut self,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        before: &TrackValue,
        value: &TrackValue,
    ) {
        for track in &mut self.tracks {
            if track.type_id == type_id && track.track.info.id == track_info.id {
                for keyframe in &mut track.track.keyframes {
                    if keyframe.value == *before {
                        keyframe.value = value.clone();
                    }
                }
            }
        }
    }

    pub(crate) fn track_mut(
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
