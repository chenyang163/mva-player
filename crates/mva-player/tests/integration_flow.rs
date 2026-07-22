//! Phase 1.7 integration pipeline test.
//!
//! Validates the full data flow without egui / window / GPU:
//!
//! ```text
//! Audio/Clock → Engine → Snapshot → Scene → Renderer → DrawList
//! ```
//!
//! No UI, no egui, no GPU.  Pure data‑flow verification.

use mva_core::config::animation::AnimationConfig;
use mva_core::config::app::AppConfig;
use mva_core::engine::Engine;
use mva_core::state::PlaybackState;
use mva_renderer::{DrawCommand, Renderer, RendererConfig, Viewport};
use mva_scene::{BlendMode, EvaluatedLayerKind, LayerId, Rgba};
use mva_timeline::model::*;

// ---------------------------------------------------------------------------
// inline test‑project builder (duplicated from the binary's test_project.rs
// so the integration test stays self‑contained)
// ---------------------------------------------------------------------------

fn test_project() -> Project {
    Project {
        metadata: ProjectMetadata {
            title: "Pipeline Test".into(),
            duration: Some(10.0),
            format_version: "1.0".into(),
            id: "pipeline-test".into(),
            ..Default::default()
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: "(sine)".into(),
            },
            duration: 10.0,
            sample_rate: 44100,
            channels: 2,
            volume_envelope: None,
        },
        lyrics: LyricTimeline {
            tracks: vec![LyricTrack {
                role: LyricRole::Original,
                language: None,
                offset: 0.0,
                lines: vec![
                    LyricLine {
                        start: 0.0,
                        end: Some(5.0),
                        text: "Line One".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 5.0,
                        end: None,
                        text: "Line Two".into(),
                        words: None,
                    },
                ],
            }],
        },
        animation: AnimationTimeline {
            layers: vec![Layer {
                id: LayerId("lyric".into()),
                name: String::new(),
                kind: LayerKind::Text {
                    source: TextSource::LyricLine,
                    style: TextStyle {
                        font_family: None,
                        font_size: 42.0,
                        color: Rgba::WHITE,
                    },
                },
                transform: Transform {
                    opacity: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 0.0,
                                value: 0.0,
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                time: 1.0,
                                value: 1.0,
                                easing: Easing::Hold,
                            },
                        ],
                    },
                    ..Transform::default()
                },
                visible_range: (0.0, 10.0),
                parent: None,
                blend_mode: BlendMode::Normal,
            }],
        },
        effect_timeline: EffectTimeline::default(),
    }
}

// ---------------------------------------------------------------------------
// the test
// ---------------------------------------------------------------------------

#[test]
fn test_player_pipeline_flow() {
    // 1. create engine + load project
    let mut engine = Engine::new(AppConfig::default(), AnimationConfig::default());
    engine
        .handle_command(mva_core::PlayerCommand::LoadProject(Box::new(
            test_project(),
        )))
        .unwrap();

    // 2. set position to t=3.0 s (mid‑way through "Line One")
    engine.update_position(3.0);

    // 3. snapshot
    let snap = engine.snapshot();

    // 4. assert state / position
    assert_eq!(snap.state, PlaybackState::Ready); // LoadProject transitions to Ready
    assert!((snap.position - 3.0).abs() < 0.001);
    assert!((snap.duration - 10.0).abs() < 0.001);

    // 5. scene exists
    let scene = snap.scene.expect("scene must exist when project loaded");
    assert_eq!(scene.layers.len(), 1);

    // 6. evaluated layer content
    let layer = &scene.layers[0];
    assert!(layer.visible);
    assert_eq!(layer.layer_index, 0);

    let EvaluatedLayerKind::Text { text, style } = &layer.kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Line One");
    assert_eq!(style.font_size, 42.0);

    // opacity: t=3.0 → after keyframe at 1.0 → opacity == 1.0
    assert!((layer.transform.opacity - 1.0).abs() < 0.001);

    // 7. lyric index
    assert_eq!(snap.active_lyric_index, Some(0));

    // 8. renderer → DrawList
    let renderer = Renderer::new(RendererConfig::default());
    let vp = Viewport {
        width: 1280.0,
        height: 720.0,
    };
    let dl = renderer.render(&scene, &vp);

    // 9. draw list has a text command
    assert!(!dl.commands.is_empty());
    match &dl.commands[0] {
        DrawCommand::Text {
            text,
            font_size,
            opacity,
            ..
        } => {
            assert_eq!(text, "Line One");
            assert_eq!(*font_size, 42.0);
            assert!((opacity - 1.0).abs() < 0.001);
        }
        _ => panic!("expected Text command"),
    }

    // 10. seek to t=7.0 — second line should be active
    engine.update_position(7.0);
    let snap2 = engine.snapshot();
    let scene2 = snap2.scene.unwrap();
    let EvaluatedLayerKind::Text { text, .. } = &scene2.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Line Two");
    assert_eq!(snap2.active_lyric_index, Some(1));
}

#[test]
fn test_player_pipeline_empty_project_does_not_panic() {
    let engine = Engine::new(AppConfig::default(), AnimationConfig::default());
    let snap = engine.snapshot();
    assert_eq!(snap.state, PlaybackState::Stopped);
    assert!(snap.scene.is_none());
}
