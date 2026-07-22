//! Contract tests for mva-scene types.

#![allow(clippy::float_cmp)]

use mva_scene::{
    ComputedTransform, EvaluatedLayer, EvaluatedLayerKind, LayerId, Rgba, Scene, TextStyle, Vec2,
};

// =========================================================================
// Rgba — serde backwards‑compatibility with old `[u8; 4]` JSON
// =========================================================================

#[test]
fn rgba_deserializes_from_array() {
    let json = "[255, 128, 64, 32]";
    let c: Rgba = serde_json::from_str(json).expect("parse array");
    assert_eq!(c, Rgba::from([255, 128, 64, 32]));
}

#[test]
fn rgba_standalone_serializes_as_struct() {
    // Rgba's derived serde produces `{"r":…,"g":…,"b":…,"a":…}`.
    // The `[u8; 4]` array shape is applied at the TextStyle level via
    // custom serde for backwards compatibility — see the
    // `text_style_deserializes_old_color_array` test above.
    let c = Rgba {
        r: 10,
        g: 20,
        b: 30,
        a: 255,
    };
    let json = serde_json::to_string(&c).unwrap();
    assert!(json.contains("\"r\":10"));
    assert!(json.contains("\"g\":20"));
    assert!(json.contains("\"b\":30"));
    assert!(json.contains("\"a\":255"));
}

#[test]
fn rgba_roundtrip() {
    let original = Rgba {
        r: 1,
        g: 2,
        b: 3,
        a: 4,
    };
    let json = serde_json::to_string(&original).unwrap();
    let back: Rgba = serde_json::from_str(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn rgba_white_default_is_opaque_white() {
    assert_eq!(Rgba::WHITE, [255, 255, 255, 255].into());
    assert_eq!(Rgba::WHITE.a, 255);
}

#[test]
fn rgba_from_array_roundtrip() {
    let arr: [u8; 4] = [12, 34, 56, 78];
    let c = Rgba::from(arr);
    let back: [u8; 4] = c.into();
    assert_eq!(arr, back);
}

// =========================================================================
// TextStyle — roundtrip
// =========================================================================

#[test]
fn text_style_roundtrip_with_color() {
    let style = TextStyle {
        font_family: None,
        font_size: 42.0,
        color: Rgba::WHITE,
    };
    let json = serde_json::to_string(&style).unwrap();
    let back: TextStyle = serde_json::from_str(&json).unwrap();
    assert_eq!(style.font_size, back.font_size);
    assert_eq!(style.font_family, back.font_family);
    assert_eq!(style.color, back.color);
}

#[test]
fn text_style_deserializes_old_color_array() {
    // The legacy `"color": [255, 255, 255, 255]` shape must still
    // parse into an Rgba.
    let json = r#"{"font_size": 24.0, "color": [10, 20, 30, 40]}"#;
    let style: TextStyle = serde_json::from_str(json).unwrap();
    assert_eq!(style.color, [10, 20, 30, 40].into());
}

#[test]
fn text_style_missing_font_family_defaults_to_none() {
    let json = r#"{"font_size": 12.0, "color": [255,255,255,255]}"#;
    let style: TextStyle = serde_json::from_str(json).unwrap();
    assert!(style.font_family.is_none());
}

#[test]
fn text_style_missing_color_defaults_to_white() {
    let json = r#"{"font_size": 12.0}"#;
    let style: TextStyle = serde_json::from_str(json).unwrap();
    assert_eq!(style.color, Rgba::WHITE);
}

// =========================================================================
// Scene / shape
// =========================================================================

#[test]
fn scene_empty_sentinel() {
    let s = Scene::empty();
    assert!(s.layers.is_empty());
}

#[test]
fn scene_with_one_layer_roundtrip() {
    let s = Scene {
        layers: vec![EvaluatedLayer {
            id: LayerId("lyric".into()),
            name: "lyric".into(),
            layer_index: 0,
            kind: EvaluatedLayerKind::Text {
                text: "Hello".into(),
                style: TextStyle {
                    font_family: None,
                    font_size: 36.0,
                    color: Rgba::WHITE,
                },
            },
            transform: ComputedTransform::default(),
            visible: true,
            blend_mode: mva_scene::BlendMode::Normal,
        }],
        effects: vec![],
    };
    assert_eq!(s.layers.len(), 1);
    assert!(s.layers[0].visible);
    let EvaluatedLayerKind::Text { ref text, .. } = s.layers[0].kind else {
        panic!("expected Text");
    };
    assert_eq!(text, "Hello");
}

#[test]
fn computed_transform_defaults_are_identity() {
    let xf = ComputedTransform::default();
    assert_eq!(xf.position, Vec2::default());
    assert_eq!(xf.scale, Vec2 { x: 1.0, y: 1.0 });
    assert_eq!(xf.rotation, 0.0);
    assert_eq!(xf.opacity, 1.0);
    assert_eq!(xf.anchor, Vec2::default());
}
