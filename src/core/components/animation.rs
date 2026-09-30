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
                .same_field(&TrackTarget::property(type_id, track_info))
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
        base: TrackValue,
    ) -> Option<TrackValue> {
        let target = TrackTarget::property(type_id, track_info);
        let mut result = None;
        let mut value = base;
        for track in &self.tracks {
            if track.track.target.same_field(&target)
                && let Some(sample) = track.track.sample(time)
            {
                value = match track.track.target.channel() {
                    Some(channel) => value.with_channel(&sample, channel),
                    None => sample,
                };
                result = Some(value.clone());
            }
        }
        result
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
                .same_field(&TrackTarget::property(type_id, track_info))
            {
                let channel = track.track.target.channel();
                for keyframe in &mut track.track.keyframes {
                    let replace = |current: &mut TrackValue| match channel {
                        Some(channel) if current.channel(channel) == before.channel(channel) => {
                            *current = current.with_channel(value, channel);
                        }
                        None if *current == *before => *current = value.clone(),
                        _ => {}
                    };
                    replace(&mut keyframe.value);
                    if let Some((_, original)) = &mut keyframe.original_end {
                        replace(original);
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
                    track: Track::for_target(target),
                });
                self.tracks.len() - 1
            }
        };

        &mut self.tracks[index].track
    }
}
