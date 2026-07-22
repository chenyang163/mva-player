//! Phase 3 Step 4 — EffectTimeline evaluation tests.

use mva_timeline::eval::evaluate_effects;
use mva_types::*;

// =========================================================================
// Test 1 — time filtering
// =========================================================================

#[test]
fn effect_timeline_time_filtering() {
    let timeline = EffectTimeline {
        effects: vec![EffectInstance {
            time_range: (10.0, 20.0),
            effect_id: "test".into(),
            target: EffectTarget::WholeScene,
            parameters: vec![],
        }],
    };

    // Before the effect window — no effect.
    let at_5 = evaluate_effects(&timeline, 5.0);
    assert!(at_5.is_empty());

    // Inside the window — one effect.
    let at_15 = evaluate_effects(&timeline, 15.0);
    assert_eq!(at_15.len(), 1);
    assert_eq!(at_15[0].effect_id, "test");

    // After the window — no effect.
    let at_25 = evaluate_effects(&timeline, 25.0);
    assert!(at_25.is_empty());
}

// =========================================================================
// Test 2 — parameter sampling
// =========================================================================

#[test]
fn effect_param_sampling() {
    let timeline = EffectTimeline {
        effects: vec![EffectInstance {
            time_range: (10.0, 20.0),
            effect_id: "fade".into(),
            target: EffectTarget::WholeScene,
            parameters: vec![EffectParam {
                name: "opacity".into(),
                track: Track {
                    keyframes: vec![
                        Keyframe {
                            time: 10.0,
                            value: ParamValue::Float { value: 0.0 },
                            easing: Easing::Linear,
                        },
                        Keyframe {
                            time: 20.0,
                            value: ParamValue::Float { value: 1.0 },
                            easing: Easing::Linear,
                        },
                    ],
                },
            }],
        }],
    };

    // At t=15 (halfway): opacity ≈ 0.5.
    let effects = evaluate_effects(&timeline, 15.0);
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].params.len(), 1);
    assert_eq!(effects[0].params[0].0, "opacity");
    assert_eq!(effects[0].params[0].1, ParamValue::Float { value: 0.5 });
}

// =========================================================================
// Test 3 — Scene integration
// =========================================================================

#[test]
fn scene_integration_with_effects() {
    use mva_timeline::eval::evaluate;
    use mva_timeline::model::*;

    let timeline = AnimationTimeline { layers: vec![] };
    let lyrics = LyricTimeline { tracks: vec![] };
    let effects = EffectTimeline {
        effects: vec![EffectInstance {
            time_range: (0.0, 10.0),
            effect_id: "bloom".into(),
            target: EffectTarget::WholeScene,
            parameters: vec![],
        }],
    };

    let scene = evaluate(&timeline, &lyrics, &effects, 5.0);
    assert_eq!(scene.effects.len(), 1);
    assert_eq!(scene.effects[0].effect_id, "bloom");
    assert_eq!(scene.effects[0].params.len(), 0);
}
