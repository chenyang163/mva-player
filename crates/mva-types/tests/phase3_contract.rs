//! Phase 3 contract tests for new mva-types (AssetRef, EffectTimeline,
//! ParamValue).

#![allow(clippy::float_cmp)]

use mva_types::*;

// =========================================================================
// AssetRef
// =========================================================================

#[test]
fn asset_ref_file_roundtrip() {
    let original = AssetRef::File {
        path: "test.png".into(),
    };
    let json = serde_json::to_string(&original).unwrap();
    assert!(json.contains("\"kind\":\"file\""));
    assert!(json.contains("\"path\":\"test.png\""));
    let back: AssetRef = serde_json::from_str(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn asset_ref_pkg_roundtrip() {
    let original = AssetRef::Pkg {
        path: "assets/images/cover.jpg".into(),
    };
    let json = serde_json::to_string(&original).unwrap();
    assert!(json.contains("\"kind\":\"pkg\""));
    let back: AssetRef = serde_json::from_str(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn asset_ref_deserializes_file_json() {
    let json = r#"{"kind":"file","path":"/music/cover.jpg"}"#;
    let r: AssetRef = serde_json::from_str(json).unwrap();
    assert_eq!(
        r,
        AssetRef::File {
            path: "/music/cover.jpg".into()
        }
    );
}

// =========================================================================
// ParamValue
// =========================================================================

#[test]
fn param_value_float_roundtrip() {
    let v = ParamValue::Float { value: 2.72 };
    let json = serde_json::to_string(&v).unwrap();
    assert!(json.contains("\"type\":\"float\""));
    let back: ParamValue = serde_json::from_str(&json).unwrap();
    assert_eq!(v, back);
}

#[test]
fn param_value_bool_roundtrip() {
    let v = ParamValue::Bool { value: true };
    let json = serde_json::to_string(&v).unwrap();
    let back: ParamValue = serde_json::from_str(&json).unwrap();
    assert_eq!(v, back);
}

#[test]
fn param_value_int_roundtrip() {
    let v = ParamValue::Int { value: -42 };
    let json = serde_json::to_string(&v).unwrap();
    let back: ParamValue = serde_json::from_str(&json).unwrap();
    assert_eq!(v, back);
}

// =========================================================================
// EffectTimeline
// =========================================================================

#[test]
fn effect_timeline_empty_roundtrip() {
    let tl = EffectTimeline { effects: vec![] };
    let json = serde_json::to_string(&tl).unwrap();
    let back: EffectTimeline = serde_json::from_str(&json).unwrap();
    assert!(back.effects.is_empty());
}

#[test]
fn effect_instance_full_roundtrip() {
    let instance = EffectInstance {
        time_range: (1.0, 5.0),
        effect_id: "bloom".into(),
        target: EffectTarget::WholeScene,
        parameters: vec![EffectParam {
            name: "intensity".into(),
            track: Track {
                keyframes: vec![
                    Keyframe {
                        time: 1.0,
                        value: ParamValue::Float { value: 0.0 },
                        easing: Easing::Linear,
                    },
                    Keyframe {
                        time: 2.0,
                        value: ParamValue::Float { value: 1.0 },
                        easing: Easing::Linear,
                    },
                ],
            },
        }],
    };

    let json = serde_json::to_string_pretty(&instance).unwrap();
    assert!(json.contains("\"bloom\""));
    assert!(json.contains("\"intensity\""));

    let back: EffectInstance = serde_json::from_str(&json).unwrap();
    assert_eq!(instance.effect_id, back.effect_id);
    assert_eq!(instance.time_range, back.time_range);
    assert_eq!(instance.parameters.len(), back.parameters.len());
    assert_eq!(
        instance.parameters[0].track.keyframes[1].value,
        ParamValue::Float { value: 1.0 }
    );
}

#[test]
fn effect_target_layer_roundtrip() {
    let target = EffectTarget::Layer {
        layer_id: "lyric-main".into(),
    };
    let json = serde_json::to_string(&target).unwrap();
    assert!(json.contains("\"type\":\"layer\""));
    let back: EffectTarget = serde_json::from_str(&json).unwrap();
    assert_eq!(target, back);
}
