use kinematic::{hecs, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TrackEnum)]
enum PlaybackMode {
    Once,
    Loop,
    PingPong,
}

#[derive(Clone, Trackable)]
struct Playback {
    #[track]
    mode: PlaybackMode,
    #[track]
    queued_mode: PlaybackMode,
}

#[derive(Clone, Trackable)]
struct PlaybackFallback {
    #[track]
    fallback_mode: PlaybackMode,
}

#[derive(Object)]
#[object(spatial = "2d", builder = "playback")]
struct PlaybackObject {
    #[trackable]
    playback: Playback,
    #[trackable]
    fallback: PlaybackFallback,
}

impl Default for PlaybackObject {
    fn default() -> Self {
        Self {
            playback: Playback {
                mode: PlaybackMode::Once,
                queued_mode: PlaybackMode::Once,
            },
            fallback: PlaybackFallback {
                fallback_mode: PlaybackMode::Once,
            },
        }
    }
}

#[derive(Default)]
struct PlaybackScene {
    playback: Option<PlaybackObjectHandler>,
}

impl SceneBuilder for PlaybackScene {
    fn build(&mut self, scene: &mut Scene) {
        let playback = playback().build(scene);
        scene.world_2d().add(&playback);
        playback.set_mode(PlaybackMode::Loop);
        playback.set_queued_mode(PlaybackMode::PingPong);
        playback.set_fallback_mode(PlaybackMode::Loop);
        assert_eq!(playback.get_mode(), PlaybackMode::Loop);
        assert_eq!(playback.get_queued_mode(), PlaybackMode::PingPong);
        assert_eq!(playback.get_fallback_mode(), PlaybackMode::Loop);
        playback
            .mode(PlaybackMode::PingPong)
            .queued_mode(PlaybackMode::Loop)
            .fallback_mode(PlaybackMode::Once)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();
        self.playback = Some(playback);
    }
}

#[test]
fn track_enum_provides_values_and_variant_metadata() {
    assert_eq!(
        <PlaybackMode as kinematic::core::TrackEnum>::VARIANTS,
        &["Once", "Loop", "PingPong"]
    );
    assert_eq!(
        PlaybackMode::Loop.into_track_value(),
        TrackValue::Enum("Loop")
    );
    assert_eq!(
        PlaybackMode::from_track_value(TrackValue::Enum("PingPong")),
        Some(PlaybackMode::PingPong)
    );
    assert_eq!(
        PlaybackMode::from_track_value(TrackValue::Enum("Missing")),
        None
    );
    assert_eq!(
        Playback::track(0).choices,
        TrackChoices::Enum(&["Once", "Loop", "PingPong"])
    );
    assert_eq!(Playback::track(1).choices, Playback::track(0).choices);
    assert_eq!(
        PlaybackFallback::track(0).choices,
        Playback::track(0).choices
    );
}

#[test]
fn enum_tracks_are_discrete_and_keep_typed_handler_api() {
    assert_eq!(
        TrackValue::Enum("Once").lerp(&TrackValue::Enum("Loop"), 0.999),
        TrackValue::Enum("Once")
    );
    assert_eq!(
        TrackValue::Enum("Once").lerp(&TrackValue::Enum("Loop"), 1.0),
        TrackValue::Enum("Loop")
    );

    let mut scene = Scene::new();
    let mut builder = PlaybackScene::default();
    assert_eq!(scene.build(&mut builder), 2.0);
    let playback = builder.playback.unwrap();

    scene.update(1.999);
    assert_eq!(playback.get_mode(), PlaybackMode::Loop);
    assert_eq!(playback.get_queued_mode(), PlaybackMode::PingPong);
    assert_eq!(playback.get_fallback_mode(), PlaybackMode::Loop);

    scene.update(2.0);
    assert_eq!(playback.get_mode(), PlaybackMode::PingPong);
    assert_eq!(playback.get_queued_mode(), PlaybackMode::Loop);
    assert_eq!(playback.get_fallback_mode(), PlaybackMode::Once);
}
