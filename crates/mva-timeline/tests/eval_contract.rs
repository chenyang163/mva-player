//! Deterministic tests for the timeline evaluation engine.
//!
//! Constraints from the Phase 1.2 brief:
//! 1. same timeline + same timestamp = same output
//! 2. before first keyframe
//! 3. after last keyframe
//! 4. empty track handling
//! 5. missing lyric handling

use mva_timeline::eval::{EvaluatedLayerKind, evaluate, value_at};
use mva_timeline::model::*;

// =========================================================================
// value_at  (all constraints 2–4)
// =========================================================================

fn track_from_pairs_f32(pairs: &[(f64, f32)]) -> Track<f32> {
    Track {
        keyframes: pairs
            .iter()
            .map(|&(t, v)| Keyframe {
                time: t,
                value: v,
                easing: Easing::default(),
            })
            .collect(),
    }
}

#[test]
fn value_at_before_first_keyframe() {
    let tk = track_from_pairs_f32(&[(1.0, 10.0), (2.0, 20.0)]);
    // t before first keyframe → hold at first value
    assert_eq!(value_at(&tk, 0.5, 0.0), 10.0);
    assert_eq!(value_at(&tk, 1.0, 0.0), 10.0); // exact first
}

#[test]
fn value_at_after_last_keyframe() {
    let tk = track_from_pairs_f32(&[(1.0, 10.0), (2.0, 20.0)]);
    // t after last keyframe → hold at last value
    assert_eq!(value_at(&tk, 2.0, 0.0), 20.0);
    assert_eq!(value_at(&tk, 5.0, 0.0), 20.0);
}

#[test]
fn value_at_empty_track_returns_default() {
    let tk: Track<f32> = Track::default();
    assert_eq!(value_at(&tk, 0.5, 99.0), 99.0);
    assert_eq!(value_at(&tk, 999.0, -1.0), -1.0);
}

#[test]
fn value_at_single_keyframe_holds() {
    let tk = track_from_pairs_f32(&[(3.0, 42.0)]);
    assert_eq!(value_at(&tk, -1.0, 0.0), 42.0); // before
    assert_eq!(value_at(&tk, 3.0, 0.0), 42.0); // exact
    assert_eq!(value_at(&tk, 100.0, 0.0), 42.0); // after
}

#[test]
fn value_at_linear_interpolation() {
    let mut tk = track_from_pairs_f32(&[(0.0, 0.0), (2.0, 10.0)]);
    // all Linear (default)
    assert_eq!(value_at(&tk, 0.5, -1.0), 2.5);
    assert_eq!(value_at(&tk, 1.0, -1.0), 5.0);
    assert_eq!(value_at(&tk, 1.5, -1.0), 7.5);

    // explicitly linear
    tk.keyframes[0].easing = Easing::Linear;
    assert_eq!(value_at(&tk, 0.5, -1.0), 2.5);
}

#[test]
fn value_at_hold_creates_steps() {
    let tk = Track {
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: Easing::Hold,
            },
            Keyframe {
                time: 1.0,
                value: 5.0,
                easing: Easing::Linear,
            },
        ],
    };
    // Hold means: keep 0.0 until next keyframe at 1.0
    assert_eq!(value_at(&tk, 0.0, -1.0), 0.0);
    assert_eq!(value_at(&tk, 0.5, -1.0), 0.0); // still held
    assert_eq!(value_at(&tk, 0.999, -1.0), 0.0);
    assert_eq!(value_at(&tk, 1.0, -1.0), 5.0); // jumps
}

#[test]
fn value_at_with_named_ease_out_cubic() {
    let tk = Track {
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: Easing::Named(NamedEase::EaseOutCubic),
            },
            Keyframe {
                time: 1.0,
                value: 1.0,
                easing: Easing::Hold,
            },
        ],
    };
    // At t=0.5, ease_out_cubic(0.5) ≈ 0.875
    let v = value_at(&tk, 0.5, -1.0);
    assert!(v > 0.8, "ease_out_cubic mid → quick start, slow end");
    assert!(v < 1.0);

    // At t=0 (exact keyframe) → 0.0
    assert_eq!(value_at(&tk, 0.0, -1.0), 0.0);

    // At t=1.0 (next keyframe) → 1.0
    assert_eq!(value_at(&tk, 1.0, -1.0), 1.0);
}

#[test]
fn value_at_with_named_ease_in_quad() {
    let tk = Track {
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: Easing::Named(NamedEase::EaseInQuad),
            },
            Keyframe {
                time: 1.0,
                value: 1.0,
                easing: Easing::Linear,
            },
        ],
    };
    // ease_in_quad(0.5) = 0.25 → slow start
    let v = value_at(&tk, 0.5, -1.0);
    assert!(v < 0.3, "ease_in_quad at mid → slow start, quick end");
}

#[test]
fn value_at_zero_duration_segment_returns_last_keyframe() {
    // When multiple keyframes share the same time and `t` equals that
    // time, the last keyframe wins (AE convention).
    let tk = Track {
        keyframes: vec![
            Keyframe {
                time: 1.0,
                value: 100.0,
                easing: Easing::Linear,
            },
            Keyframe {
                time: 1.0,
                value: 200.0,
                easing: Easing::Linear,
            },
        ],
    };
    assert_eq!(value_at(&tk, 1.0, -1.0), 200.0);
}

// =========================================================================
// value_at
// =========================================================================

#[test]
fn value_at_linear_lerp() {
    let tk = Track {
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: Vec2 { x: 0.0, y: 0.0 },
                easing: Easing::Linear,
            },
            Keyframe {
                time: 2.0,
                value: Vec2 { x: 10.0, y: 20.0 },
                easing: Easing::Linear,
            },
        ],
    };
    let v = value_at(&tk, 0.5, Vec2::default());
    assert_eq!(v.x, 2.5);
    assert_eq!(v.y, 5.0);
}

#[test]
fn value_at_empty_track_default() {
    let tk: Track<Vec2> = Track::default();
    let def = Vec2 { x: 5.0, y: 6.0 };
    assert_eq!(value_at(&tk, 1.0, def), def);
}

// =========================================================================
// lyric lookup (bridges to evaluate)
// =========================================================================

fn simple_lyric() -> LyricTimeline {
    LyricTimeline {
        tracks: vec![LyricTrack {
            role: LyricRole::Original,
            language: Some("en".into()),
            offset: 0.0,
            lines: vec![
                LyricLine {
                    start: 5.0,
                    end: Some(9.0),
                    text: "Hello".into(),
                    words: None,
                },
                LyricLine {
                    start: 9.0,
                    end: None,
                    text: "World".into(),
                    words: None,
                },
            ],
        }],
    }
}

#[test]
fn active_lyric_line_in_range() {
    let lyrics = simple_lyric();
    let layer = Layer {
        id: LayerId("l".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::LyricLine,
            style: TextStyle {
                font_family: None,
                font_size: 20.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };

    // At t=5.0 (start of "Hello")
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 5.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Hello");

    // At t=8.0 (still "Hello")
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 8.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Hello");

    // At t=9.0 (start of "World")
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 9.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "World");

    // At t=12.0 (still "World" — open-ended)
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 12.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "World");
}

#[test]
fn active_lyric_line_in_gap() {
    let lyrics = LyricTimeline {
        tracks: vec![LyricTrack {
            role: LyricRole::Original,
            language: None,
            offset: 0.0,
            lines: vec![
                LyricLine {
                    start: 5.0,
                    end: Some(8.0),
                    text: "A".into(),
                    words: None,
                },
                LyricLine {
                    start: 12.0,
                    end: None,
                    text: "B".into(),
                    words: None,
                },
            ],
        }],
    };
    let layer = Layer {
        id: LayerId("l".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::LyricLine,
            style: TextStyle {
                font_family: None,
                font_size: 20.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };
    // Gap between 8 and 12: no line active
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 10.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert!(text.is_empty(), "gap → empty text");
}

#[test]
fn missing_lyric_timeline_yields_empty_text() {
    let lyrics = LyricTimeline::default(); // no tracks at all
    let layer = Layer {
        id: LayerId("l".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::LyricLine,
            style: TextStyle {
                font_family: None,
                font_size: 20.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 5.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert!(text.is_empty(), "no lyrics → empty text, never crashes");
}

#[test]
fn static_text_source_always_returns_text() {
    let layer = Layer {
        id: LayerId("l".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::Static {
                text: "FIXED".into(),
            },
            style: TextStyle {
                font_family: None,
                font_size: 20.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };
    let lyrics = LyricTimeline::default();
    for t in [0.0, 5.0, 999.0_f64] {
        let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), t);
        let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
            panic!("expected Text");
        };
        assert_eq!(text, "FIXED", "static text at t={t}");
    }
}

// =========================================================================
// Scene evaluation (opacity + scale animation, visibility)
// =========================================================================

fn animated_opacity_layer() -> Layer {
    Layer {
        id: LayerId("l1".into()),
        name: "anim".into(),
        kind: LayerKind::Text {
            source: TextSource::Static {
                text: "Animated".into(),
            },
            style: TextStyle {
                font_family: None,
                font_size: 24.0,
                color: [255; 4].into(),
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
            scale: Track {
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        value: Vec2 { x: 0.8, y: 0.8 },
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
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    }
}

#[test]
fn scene_evaluates_opacity_and_scale_at_midpoint() {
    let tl = AnimationTimeline {
        layers: vec![animated_opacity_layer()],
    };
    let lyrics = LyricTimeline::default();

    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 0.5);
    let l = &scene.layers[0];
    assert!(l.visible);

    // opacity: linear from 0→1, at 0.5 → 0.5
    assert!((l.transform.opacity - 0.5).abs() < 1e-6);

    // scale: ease_out_cubic at 0.5 → ≈ 0.875 from 0.8 toward 1.0 (≈0.975)
    assert!(l.transform.scale.x > 0.95);
    assert!(l.transform.scale.y > 0.95);
}

#[test]
fn scene_evaluates_opacity_at_start_and_end() {
    let tl = AnimationTimeline {
        layers: vec![animated_opacity_layer()],
    };
    let lyrics = LyricTimeline::default();

    let t0 = evaluate(&tl, &lyrics, &EffectTimeline::default(), 0.0);
    assert!((t0.layers[0].transform.opacity - 0.0).abs() < 1e-6);
    assert_eq!(t0.layers[0].transform.scale.x, 0.8);

    let t1 = evaluate(&tl, &lyrics, &EffectTimeline::default(), 1.0);
    assert!((t1.layers[0].transform.opacity - 1.0).abs() < 1e-6);
    assert_eq!(t1.layers[0].transform.scale.x, 1.0);
}

// =========================================================================
// determinism (constraint 1)
// =========================================================================

#[test]
fn same_input_produces_same_output() {
    let tl = AnimationTimeline {
        layers: vec![animated_opacity_layer()],
    };
    let lyrics = simple_lyric();

    let a = evaluate(&tl, &lyrics, &EffectTimeline::default(), 2.5);
    let b = evaluate(&tl, &lyrics, &EffectTimeline::default(), 2.5);
    assert_eq!(a, b);

    // Different t within animation range produces different result
    let c = evaluate(&tl, &lyrics, &EffectTimeline::default(), 0.7);
    assert_ne!(a, c);
}

// =========================================================================
// visibility / visible_range
// =========================================================================

#[test]
fn layer_visible_only_within_range() {
    let layer = Layer {
        id: LayerId("v".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::Static { text: "x".into() },
            style: TextStyle {
                font_family: None,
                font_size: 10.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (5.0, 10.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };
    let lyrics = LyricTimeline::default();

    assert!(!evaluate(&tl, &lyrics, &EffectTimeline::default(), 0.0).layers[0].visible);
    assert!(!evaluate(&tl, &lyrics, &EffectTimeline::default(), 4.999).layers[0].visible);
    assert!(evaluate(&tl, &lyrics, &EffectTimeline::default(), 5.0).layers[0].visible); // exact start
    assert!(evaluate(&tl, &lyrics, &EffectTimeline::default(), 7.0).layers[0].visible);
    assert!(!evaluate(&tl, &lyrics, &EffectTimeline::default(), 10.0).layers[0].visible); // exact end (exclusive)
    assert!(!evaluate(&tl, &lyrics, &EffectTimeline::default(), 20.0).layers[0].visible);
}

// =========================================================================
// identity-transform defaults
// =========================================================================

#[test]
fn empty_transform_uses_identity_defaults() {
    let layer = Layer {
        id: LayerId("id".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::Static { text: "".into() },
            style: TextStyle {
                font_family: None,
                font_size: 10.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };
    let lyrics = LyricTimeline::default();

    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 42.0);
    let xf = &scene.layers[0].transform;
    assert_eq!(xf.position.x, 0.0);
    assert_eq!(xf.position.y, 0.0);
    assert_eq!(xf.scale.x, 1.0);
    assert_eq!(xf.scale.y, 1.0);
    assert_eq!(xf.rotation, 0.0);
    assert_eq!(xf.opacity, 1.0);
    assert_eq!(xf.anchor.x, 0.0);
    assert_eq!(xf.anchor.y, 0.0);
}

// =========================================================================
// lyric offset
// =========================================================================

#[test]
fn lyric_offset_shifts_effective_time() {
    let lyrics = LyricTimeline {
        tracks: vec![LyricTrack {
            role: LyricRole::Original,
            language: None,
            offset: 0.5, // positive shift: lyrics appear 0.5s later
            lines: vec![LyricLine {
                start: 1.0,
                end: None,
                text: "Delayed".into(),
                words: None,
            }],
        }],
    };
    let layer = Layer {
        id: LayerId("l".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::LyricLine,
            style: TextStyle {
                font_family: None,
                font_size: 20.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };

    // Without offset, at t=1.0 the line would be active.
    // With offset +0.5, effective_t = 0.5 → before the line.
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 1.0);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert!(
        text.is_empty(),
        "offset +0.5: at t=1.0, effective=0.5 → before"
    );

    // At t=1.5, effective_t = 1.0 → line starts
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 1.5);
    let EvaluatedLayerKind::Text { text, .. } = &scene.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Delayed");
}

// =========================================================================
// multi-layer scene
// =========================================================================

#[test]
fn multi_layer_scene_preserves_z_order() {
    let make = |id: &str| Layer {
        id: LayerId(id.into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::Static { text: id.into() },
            style: TextStyle {
                font_family: None,
                font_size: 10.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };

    let tl = AnimationTimeline {
        layers: vec![make("bg"), make("fg"), make("overlay")],
    };
    let lyrics = LyricTimeline::default();
    let scene = evaluate(&tl, &lyrics, &EffectTimeline::default(), 0.0);

    assert_eq!(scene.layers.len(), 3);
    assert_eq!(scene.layers[0].id, LayerId("bg".into()));
    assert_eq!(scene.layers[1].id, LayerId("fg".into()));
    assert_eq!(scene.layers[2].id, LayerId("overlay".into()));

    // layer_index tracks z-order
    assert_eq!(scene.layers[0].layer_index, 0);
    assert_eq!(scene.layers[2].layer_index, 2);
}

// =========================================================================
// LyricWord binding (Phase 2 model, evaluated now for completeness)
// =========================================================================

#[test]
fn lyric_word_binding_returns_active_word() {
    let lyrics = LyricTimeline {
        tracks: vec![LyricTrack {
            role: LyricRole::Original,
            language: None,
            offset: 0.0,
            lines: vec![LyricLine {
                start: 1.0,
                end: Some(4.0),
                text: "full line".into(),
                words: Some(vec![
                    LyricWord {
                        text: "Hello".into(),
                        start: 1.0,
                        end: None,
                    },
                    LyricWord {
                        text: "World".into(),
                        start: 2.5,
                        end: None,
                    },
                ]),
            }],
        }],
    };
    let layer = Layer {
        id: LayerId("w".into()),
        name: String::new(),
        kind: LayerKind::Text {
            source: TextSource::LyricWord,
            style: TextStyle {
                font_family: None,
                font_size: 20.0,
                color: [255; 4].into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };
    let tl = AnimationTimeline {
        layers: vec![layer],
    };

    let s1 = evaluate(&tl, &lyrics, &EffectTimeline::default(), 1.5);
    let EvaluatedLayerKind::Text { text, .. } = &s1.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Hello");

    let s2 = evaluate(&tl, &lyrics, &EffectTimeline::default(), 3.0);
    let EvaluatedLayerKind::Text { text, .. } = &s2.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "World");
}
