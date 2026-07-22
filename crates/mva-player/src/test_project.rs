//! Phase 1.7 integration demo bootstrap — a synthetic `.mva`‑shaped
//! project used to validate the full pipeline without requiring real
//! media files.
//!
//! **This is Phase 1.7 demo bootstrap code.**
//! It will be replaced by the real project‑loading pipeline when
//! `mva-format` arrives (Phase 4).  Do **not** treat these helpers
//! as production APIs or stable interfaces.

use rodio::source::Source;
use std::time::Duration;

use mva_scene::Rgba;
use mva_timeline::model::*;

/// This is Phase 1.7 demo bootstrap code.
/// It will be replaced by real project loading (Phase 4, `mva-format`).
///
/// Build a test [`Project`] with:
/// - 4‑line lyric track (lines at 1, 5, 9, 13 s)
/// - One [`LayerKind::Text`] layer bound to [`TextSource::LyricLine`]
/// - Animated opacity (fade‑in) + scale on the first line
pub fn make_test_project() -> Project {
    Project {
        metadata: ProjectMetadata {
            title: "Integration Test".into(),
            artist: Some("MVA Player".into()),
            duration: Some(30.0),
            languages: vec!["en".into()],
            format_version: "1.0".into(),
            id: "integration-test".into(),
            ..Default::default()
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: "(sine wave)".into(),
            },
            duration: 30.0,
            sample_rate: 44100,
            channels: 2,
            volume_envelope: None,
        },
        lyrics: LyricTimeline {
            tracks: vec![LyricTrack {
                role: LyricRole::Original,
                language: Some("en".into()),
                offset: 0.0,
                lines: vec![
                    LyricLine {
                        start: 1.0,
                        end: Some(5.0),
                        text: "Hello, MVA!".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 5.0,
                        end: Some(9.0),
                        text: "This is a test".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 9.0,
                        end: Some(13.0),
                        text: "of the lyric engine".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 13.0,
                        end: None,
                        text: "Phase 1.7 integration".into(),
                        words: None,
                    },
                ],
            }],
        },
        animation: AnimationTimeline {
            layers: vec![Layer {
                id: LayerId("lyric".into()),
                name: "Lyric Layer".into(),
                kind: LayerKind::Text {
                    source: TextSource::LyricLine,
                    style: TextStyle {
                        font_family: None,
                        font_size: 48.0,
                        color: Rgba::WHITE,
                    },
                },
                transform: Transform {
                    opacity: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 0.5,
                                value: 0.0,
                                easing: Easing::Named(NamedEase::EaseOutCubic),
                            },
                            Keyframe {
                                time: 1.0,
                                value: 1.0,
                                easing: Easing::Hold,
                            },
                        ],
                    },
                    scale: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 0.5,
                                value: Vec2 { x: 0.9, y: 0.9 },
                                easing: Easing::Named(NamedEase::EaseOutCubic),
                            },
                            Keyframe {
                                time: 1.0,
                                value: Vec2 { x: 1.0, y: 1.0 },
                                easing: Easing::Hold,
                            },
                        ],
                    },
                    ..Transform::default()
                },
                visible_range: (0.0, 30.0),
                parent: None,
                blend_mode: BlendMode::Normal,
            }],
        },
        effect_timeline: EffectTimeline::default(),
    }
}

/// This is Phase 1.7 demo bootstrap code.
/// It will be replaced by real file / source loading (Phase 4+).
///
/// Build a 30‑second 440 Hz sine wave suitable for
/// [`AudioPlayer::load_source`].
pub fn make_test_sine() -> impl rodio::source::Source<Item = f32> + Send + 'static {
    rodio::source::SineWave::new(440.0).take_duration(Duration::from_secs_f32(30.0))
}
