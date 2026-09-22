use crate::core::{Track, TrackInfo, TrackTarget, TrackValue};

/// Associates an object with one component-property or shader-uniform track.
pub(crate) struct AnimationTrack {
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
            track
                .track
                .target
                .same(&TrackTarget::property(type_id, track_info))
                && !track.track.keyframes.is_empty()
        })
    }

    pub(crate) fn animates_uniform(&self, name: &str) -> bool {
        self.tracks.iter().any(|track| {
            track.track.target.uniform_name() == Some(name) && !track.track.keyframes.is_empty()
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
            .find(|track| {
                track
                    .track
                    .target
                    .same(&TrackTarget::property(type_id, track_info))
            })
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
            if track
                .track
                .target
                .same(&TrackTarget::property(type_id, track_info))
            {
                for keyframe in &mut track.track.keyframes {
                    if keyframe.value == *before {
                        keyframe.value = value.clone();
                    }
                }
            }
        }
    }

    pub(crate) fn target_mut(&mut self, target: TrackTarget) -> &mut Track {
        let index = match self
            .tracks
            .iter()
            .position(|track| track.track.target.same(&target))
        {
            Some(index) => index,
            None => {
                self.tracks.push(AnimationTrack {
                    track: match target {
                        TrackTarget::Property { type_id, info } => Track::property(type_id, info),
                        TrackTarget::Uniform(name) => Track::uniform(name),
                    },
                });
                self.tracks.len() - 1
            }
        };

        &mut self.tracks[index].track
    }
}
