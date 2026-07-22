//! Phase 3 LayerKind::Image tests (requires mva‑timeline types).

use mva_timeline::model::*;
use mva_types::AssetRef;

#[test]
fn image_layer_kind_roundtrip() {
    let kind = LayerKind::Image {
        asset: AssetRef::File {
            path: "bg.png".into(),
        },
    };
    let json = serde_json::to_string(&kind).unwrap();
    assert!(json.contains("\"type\":\"image\""));
    assert!(json.contains("\"bg.png\""));
    let back: LayerKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}

#[test]
fn image_layer_full_json_roundtrip() {
    let layer = Layer {
        id: LayerId("bg-img".into()),
        name: "Background".into(),
        kind: LayerKind::Image {
            asset: AssetRef::Pkg {
                path: "assets/bg.jpg".into(),
            },
        },
        transform: Transform::default(),
        visible_range: (0.0, 100.0),
        parent: None,
        blend_mode: BlendMode::Normal,
    };

    let json = serde_json::to_string_pretty(&layer).unwrap();
    let back: Layer = serde_json::from_str(&json).unwrap();
    assert_eq!(layer.id, back.id);

    let LayerKind::Image { asset } = &back.kind else {
        panic!("expected Image layer");
    };
    assert_eq!(
        *asset,
        AssetRef::Pkg {
            path: "assets/bg.jpg".into()
        }
    );
}
