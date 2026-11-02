use crate::test_support::*;

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

impl PlaybackScene {
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
        <PlaybackMode as crate::core::TrackEnum>::VARIANTS,
        &["once", "loop", "ping_pong"]
    );
    assert_eq!(
        PlaybackMode::Loop.into_track_value(),
        TrackValue::Enum("loop")
    );
    assert_eq!(
        PlaybackMode::from_track_value(TrackValue::Enum("ping_pong")),
        Some(PlaybackMode::PingPong)
    );
    assert_eq!(
        PlaybackMode::from_track_value(TrackValue::Enum("Missing")),
        None
    );

    for value in ["PingPong", "pingPong", "ping-pong", "PING_PONG"] {
        assert_eq!(
            PlaybackMode::from_track_value(TrackValue::Enum(value)),
            None
        );
    }

    assert_eq!(
        Playback::track(0).choices,
        TrackChoices::Enum(&["once", "loop", "ping_pong"])
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
        TrackValue::Enum("once").lerp(&TrackValue::Enum("loop"), 0.999),
        TrackValue::Enum("once")
    );
    assert_eq!(
        TrackValue::Enum("once").lerp(&TrackValue::Enum("loop"), 1.0),
        TrackValue::Enum("loop")
    );

    let mut scene = Scene::new();
    let mut builder = PlaybackScene::default();

    assert_eq!(
        {
            builder.build(&mut scene);

            scene.finish_build()
        },
        2.0
    );

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
