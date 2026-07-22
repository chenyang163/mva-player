//! Engine state-machine tests.
//!
//! Covers:
//! - command handling (Play / Pause / Stop / Seek / Volume / LoadProject)
//! - invalid state transitions
//! - snapshot contents (scene evaluation, lyric index)
//! - no-project-loaded behaviour

#![allow(clippy::float_cmp)]

use mva_core::config::animation::AnimationConfig;
use mva_core::config::app::AppConfig;
use mva_core::effect::{AudioCommand, EngineEffect};
use mva_core::engine::Engine;
use mva_core::state::PlaybackState;
use mva_core::{EngineSnapshot, PlayerCommand};
use mva_timeline::eval::EvaluatedLayerKind;
use mva_timeline::model::*;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn default_engine() -> Engine {
    Engine::new(AppConfig::default(), AnimationConfig::default())
}

fn sample_project() -> Project {
    Project {
        metadata: ProjectMetadata {
            title: "Test".into(),
            duration: Some(120.0),
            format_version: "1.0".into(),
            id: "test-id".into(),
            ..Default::default()
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: "song.mp3".into(),
            },
            duration: 120.0,
            sample_rate: 44100,
            channels: 2,
            volume_envelope: None,
        },
        lyrics: LyricTimeline {
            tracks: vec![LyricTrack {
                role: LyricRole::Original,
                offset: 0.0,
                language: None,
                lines: vec![
                    LyricLine {
                        start: 10.0,
                        end: Some(14.0),
                        text: "Hello".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 14.0,
                        end: None,
                        text: "World".into(),
                        words: None,
                    },
                ],
            }],
        },
        animation: AnimationTimeline {
            layers: vec![Layer {
                id: LayerId("lyric".into()),
                name: "Lyric".into(),
                kind: LayerKind::Text {
                    source: TextSource::LyricLine,
                    style: TextStyle {
                        font_family: None,
                        font_size: 42.0,
                        color: [255; 4].into(),
                    },
                },
                transform: Transform {
                    opacity: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 9.6,
                                value: 0.0,
                                easing: Easing::Named(NamedEase::EaseOutCubic),
                            },
                            Keyframe {
                                time: 10.0,
                                value: 1.0,
                                easing: Easing::Hold,
                            },
                        ],
                    },
                    scale: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 9.6,
                                value: Vec2 { x: 0.9, y: 0.9 },
                                easing: Easing::Named(NamedEase::EaseOutCubic),
                            },
                            Keyframe {
                                time: 10.0,
                                value: Vec2 { x: 1.0, y: 1.0 },
                                easing: Easing::Hold,
                            },
                        ],
                    },
                    ..Transform::default()
                },
                visible_range: (0.0, 120.0),
                parent: None,
                blend_mode: BlendMode::Normal,
            }],
        },
        effect_timeline: EffectTimeline::default(),
    }
}

// ---------------------------------------------------------------------------
// state transitions
// ---------------------------------------------------------------------------

#[test]
fn engine_starts_stopped() {
    let eng = default_engine();
    let snap = eng.snapshot();
    assert_eq!(snap.state, PlaybackState::Stopped);
    assert_eq!(snap.position, 0.0);
    assert!(snap.scene.is_none());
}

#[test]
fn play_from_stopped_transitions_to_playing() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.handle_command(PlayerCommand::Play).unwrap();
    assert_eq!(eng.snapshot().state, PlaybackState::Playing);
}

#[test]
fn pause_from_playing_then_resume() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.handle_command(PlayerCommand::Play).unwrap();
    eng.update_position(5.0);
    eng.handle_command(PlayerCommand::Pause).unwrap();
    let snap = eng.snapshot();
    assert_eq!(snap.state, PlaybackState::Paused);
    assert_eq!(snap.position, 5.0); // position preserved
    // resume
    eng.handle_command(PlayerCommand::Play).unwrap();
    assert_eq!(eng.snapshot().state, PlaybackState::Playing);
}

#[test]
fn stop_resets_position() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.handle_command(PlayerCommand::Play).unwrap();
    eng.update_position(30.0);
    eng.handle_command(PlayerCommand::Stop).unwrap();
    let snap = eng.snapshot();
    assert_eq!(snap.state, PlaybackState::Stopped);
    assert_eq!(snap.position, 0.0);
}

// ---------------------------------------------------------------------------
// invalid transitions
// ---------------------------------------------------------------------------

#[test]
fn play_while_playing_is_idempotent() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.handle_command(PlayerCommand::Play).unwrap();
    // Second Play is idempotent — no error.
    assert!(eng.handle_command(PlayerCommand::Play).is_ok());
}

#[test]
fn pause_while_not_playing_is_ignored() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    // Pause while Ready — silently ignored, no error.
    assert!(eng.handle_command(PlayerCommand::Pause).is_ok());
}

#[test]
fn play_without_project_returns_no_project_loaded() {
    let mut eng = default_engine();
    let err = eng.handle_command(PlayerCommand::Play).unwrap_err();
    assert!(matches!(err, mva_core::CoreError::NoProjectLoaded));
    assert_eq!(eng.snapshot().state, PlaybackState::Stopped);
}

// ---------------------------------------------------------------------------
// EngineEffect output tests (Phase 2 Step 2)
// ---------------------------------------------------------------------------

fn engine_with_project() -> Engine {
    let mut eng = Engine::new(AppConfig::default(), AnimationConfig::default());
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng
}

#[test]
fn play_produces_audio_play_effect() {
    let mut eng = engine_with_project();
    let effects = eng.handle_command(PlayerCommand::Play).unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0], EngineEffect::Audio(AudioCommand::Play));
    assert_eq!(eng.snapshot().state, PlaybackState::Playing);
}

#[test]
fn pause_produces_audio_pause_effect() {
    let mut eng = engine_with_project();
    eng.handle_command(PlayerCommand::Play).unwrap();
    let effects = eng.handle_command(PlayerCommand::Pause).unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0], EngineEffect::Audio(AudioCommand::Pause));
}

#[test]
fn set_volume_produces_audio_set_volume_effect() {
    let mut eng = engine_with_project();
    let effects = eng.handle_command(PlayerCommand::SetVolume(0.5)).unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(
        effects[0],
        EngineEffect::Audio(AudioCommand::SetVolume(0.5))
    );
}

#[test]
fn stop_produces_audio_stop_effect() {
    let mut eng = engine_with_project();
    eng.handle_command(PlayerCommand::Play).unwrap();
    let effects = eng.handle_command(PlayerCommand::Stop).unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0], EngineEffect::Audio(AudioCommand::Stop));
}

#[test]
fn seek_produces_audio_seek_effect() {
    let mut eng = engine_with_project();
    let effects = eng.handle_command(PlayerCommand::Seek(30.0)).unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0], EngineEffect::Audio(AudioCommand::Seek(30.0)));
}

#[test]
fn open_file_produces_load_project_effect() {
    let mut eng = default_engine();
    use std::path::PathBuf;
    let path = PathBuf::from("/tmp/test_song");
    let effects = eng
        .handle_command(PlayerCommand::OpenFile(path.clone()))
        .unwrap();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0], EngineEffect::LoadProject { path });
    assert_eq!(eng.snapshot().state, PlaybackState::Loading);
}

#[test]
fn load_project_produces_no_audio_effect() {
    let mut eng = default_engine();
    let effects = eng
        .handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    assert!(
        effects.is_empty(),
        "LoadProject should not produce an audio effect"
    );
}

// ---------------------------------------------------------------------------
// seek & volume
// ---------------------------------------------------------------------------

#[test]
fn seek_clamps_to_bounds() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.handle_command(PlayerCommand::Seek(-5.0)).unwrap();
    assert_eq!(eng.snapshot().position, 0.0); // clamped

    eng.handle_command(PlayerCommand::Seek(500.0)).unwrap();
    assert_eq!(eng.snapshot().position, 120.0); // clamped to duration
}

#[test]
fn volume_clamps_to_0_1() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::SetVolume(-0.5)).unwrap();
    assert_eq!(eng.snapshot().volume, 0.0);
    eng.handle_command(PlayerCommand::SetVolume(2.0)).unwrap();
    assert_eq!(eng.snapshot().volume, 1.0);
    eng.handle_command(PlayerCommand::SetVolume(0.42)).unwrap();
    assert_eq!(eng.snapshot().volume, 0.42);
}

// ---------------------------------------------------------------------------
// snapshot: scene evaluation
// ---------------------------------------------------------------------------

#[test]
fn snapshot_with_project_evaluates_scene() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    // At position 10.0: lyric "Hello" is active, opacity just reached 1.0
    eng.update_position(10.0);
    let snap = eng.snapshot();

    assert_eq!(snap.duration, 120.0);
    assert_eq!(snap.position, 10.0);
    assert!(snap.scene.is_some());

    let scene = snap.scene.unwrap();
    assert_eq!(scene.layers.len(), 1);
    let layer = &scene.layers[0];
    assert!(layer.visible);

    let EvaluatedLayerKind::Text { text, .. } = &layer.kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Hello");

    // At t=10.0, opacity kf reached → opacity == 1.0
    assert_eq!(layer.transform.opacity, 1.0);
    assert_eq!(layer.transform.scale.x, 1.0);
}

#[test]
fn snapshot_lyric_index_tracks_active_line() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();

    // Before first line
    eng.update_position(0.0);
    assert_eq!(eng.snapshot().active_lyric_index, None);

    // First line (index 0)
    eng.update_position(10.0);
    assert_eq!(eng.snapshot().active_lyric_index, Some(0));

    // Second line (index 1)
    eng.update_position(15.0);
    assert_eq!(eng.snapshot().active_lyric_index, Some(1));
}

#[test]
fn snapshot_without_project_is_empty() {
    let eng = default_engine();
    let snap = eng.snapshot();
    assert_eq!(snap.state, PlaybackState::Stopped);
    assert_eq!(snap.duration, 0.0);
    assert!(snap.scene.is_none());
    assert_eq!(snap.active_lyric_index, None);
}

#[test]
fn engine_snapshot_empty_is_sentinel() {
    let sentinel = EngineSnapshot::empty();
    assert_eq!(sentinel.state, PlaybackState::Stopped);
    assert_eq!(sentinel.duration, 0.0);
    assert!(sentinel.scene.is_none());
}

// ---------------------------------------------------------------------------
// position update (from external clock)
// ---------------------------------------------------------------------------

#[test]
fn update_position_changes_snapshot_position() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.update_position(25.5);
    assert_eq!(eng.snapshot().position, 25.5);
}

#[test]
fn update_position_clamps_to_duration() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.update_position(999.0);
    assert_eq!(eng.snapshot().position, 120.0); // clamped
}

// ---------------------------------------------------------------------------
// determinism
// ---------------------------------------------------------------------------

#[test]
fn snapshot_is_deterministic() {
    let mut eng = default_engine();
    eng.handle_command(PlayerCommand::LoadProject(Box::new(sample_project())))
        .unwrap();
    eng.update_position(12.0);
    let a = eng.snapshot();
    let b = eng.snapshot();
    assert_eq!(a, b);

    // Different position → different scene (determinism proof)
    eng.update_position(13.0);
    let c = eng.snapshot();
    assert_ne!(a, c);
}
