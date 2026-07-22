//! Renderer integration tests — Scene → DrawList pipeline.

#![allow(clippy::float_cmp)]

use mva_renderer::{DrawCommand, DrawList, Renderer, RendererConfig, Viewport};
use mva_scene::{
    ComputedTransform, EvaluatedLayer, EvaluatedLayerKind, LayerId, Scene, TextStyle, Vec2,
};
use mva_types::{AssetRef, EffectTarget, ParamValue};

fn vp() -> Viewport {
    Viewport {
        width: 1280.0,
        height: 720.0,
    }
}

fn default_scene() -> Scene {
    Scene {
        layers: vec![EvaluatedLayer {
            id: LayerId("lyric".into()),
            name: "Lyric".into(),
            layer_index: 0,
            kind: EvaluatedLayerKind::Text {
                text: "Hello".into(),
                style: TextStyle {
                    font_family: None,
                    font_size: 42.0,
                    color: [255; 4].into(),
                },
            },
            transform: ComputedTransform::default(),
            visible: true,
            blend_mode: mva_scene::BlendMode::Normal,
        }],
        effects: vec![],
    }
}

fn as_text(cmd: &DrawCommand) -> (&str, f32, [u8; 4], f32, f32, f32) {
    match cmd {
        DrawCommand::Text {
            text,
            font_size,
            color,
            x,
            y,
            opacity,
        } => (text, *font_size, *color, *x, *y, *opacity),
        _ => panic!("expected Text command"),
    }
}

#[test]
fn empty_scene_produces_empty_draw_list() {
    let r = Renderer::new(RendererConfig::default());
    let dl = r.render(
        &Scene {
            layers: vec![],
            effects: vec![],
        },
        &vp(),
    );
    assert!(dl.commands.is_empty());
}

#[test]
fn text_layer_produces_text_command() {
    let r = Renderer::new(RendererConfig::default());
    let dl = r.render(&default_scene(), &vp());
    let (text, font_size, ..) = as_text(&dl.commands[0]);
    assert_eq!(text, "Hello");
    assert_eq!(font_size, 42.0);
}

#[test]
fn invisible_layer_is_culled() {
    let r = Renderer::new(RendererConfig::default());
    let mut scene = default_scene();
    scene.layers[0].visible = false;
    assert!(r.render(&scene, &vp()).commands.is_empty());
}

#[test]
fn zero_opacity_layer_is_culled() {
    let r = Renderer::new(RendererConfig::default());
    let mut scene = default_scene();
    scene.layers[0].transform.opacity = 0.0;
    assert!(r.render(&scene, &vp()).commands.is_empty());
}

#[test]
fn layer_outside_viewport_is_culled() {
    let r = Renderer::new(RendererConfig::default());
    let mut scene = default_scene();
    scene.layers[0].transform.position = Vec2 {
        x: -9999.0,
        y: -9999.0,
    };
    assert!(r.render(&scene, &vp()).commands.is_empty());
}

#[test]
fn text_centered_by_default() {
    let r = Renderer::new(RendererConfig::default());
    let vp = Viewport {
        width: 800.0,
        height: 600.0,
    };
    let (_, _, _, x, y, _) = as_text(&r.render(&default_scene(), &vp).commands[0]);
    assert_eq!(x, 400.0);
    assert_eq!(y, 300.0);
}

#[test]
fn layer_position_offsets_text() {
    let r = Renderer::new(RendererConfig::default());
    let mut scene = default_scene();
    scene.layers[0].transform.position = Vec2 { x: 100.0, y: -50.0 };
    let (_, _, _, x, y, _) = as_text(&r.render(&scene, &vp()).commands[0]);
    assert_eq!(x, 640.0 + 100.0);
    assert_eq!(y, 360.0 - 50.0);
}

#[test]
fn scale_affects_font_size() {
    let r = Renderer::new(RendererConfig::default());
    let mut scene = default_scene();
    scene.layers[0].transform.scale = Vec2 { x: 2.0, y: 2.0 };
    let (_, font_size, ..) = as_text(&r.render(&scene, &vp()).commands[0]);
    assert_eq!(font_size, 84.0);
}

#[test]
fn opacity_passed_through() {
    let r = Renderer::new(RendererConfig::default());
    let mut scene = default_scene();
    scene.layers[0].transform.opacity = 0.75;
    let (.., opacity) = as_text(&r.render(&scene, &vp()).commands[0]);
    assert_eq!(opacity, 0.75);
}

#[test]
fn z_order_preserved() {
    let r = Renderer::new(RendererConfig::default());
    let scene = Scene {
        layers: vec![
            EvaluatedLayer {
                id: LayerId("bg".into()),
                name: "BG".into(),
                layer_index: 0,
                kind: EvaluatedLayerKind::Text {
                    text: "bg".into(),
                    style: TextStyle {
                        font_family: None,
                        font_size: 20.0,
                        color: [200; 4].into(),
                    },
                },
                transform: ComputedTransform::default(),
                visible: true,
                blend_mode: mva_scene::BlendMode::Normal,
            },
            EvaluatedLayer {
                id: LayerId("fg".into()),
                name: "FG".into(),
                layer_index: 1,
                kind: EvaluatedLayerKind::Text {
                    text: "fg".into(),
                    style: TextStyle {
                        font_family: None,
                        font_size: 20.0,
                        color: [255; 4].into(),
                    },
                },
                transform: ComputedTransform::default(),
                visible: true,
                blend_mode: mva_scene::BlendMode::Normal,
            },
        ],
        effects: vec![],
    };
    let dl = r.render(&scene, &vp());
    assert_eq!(dl.commands.len(), 2);
    assert_eq!(as_text(&dl.commands[0]).0, "bg");
    assert_eq!(as_text(&dl.commands[1]).0, "fg");
}

#[test]
fn draw_list_empty_is_sentinel() {
    assert!(DrawList::empty().commands.is_empty());
}

#[test]
fn renderer_config_from_toml_accepts_empty() {
    let cfg = RendererConfig::from_toml("").expect("empty toml");
    assert_eq!(cfg, RendererConfig::default());
}

#[test]
fn renderer_config_from_toml_parses_section() {
    let toml = "[renderer]\n# future knobs\n";
    let cfg = RendererConfig::from_toml(toml).expect("section toml");
    assert_eq!(cfg, RendererConfig::default());
}

// =========================================================================
// Phase 3 — Image pipeline
// =========================================================================

#[test]
fn image_layer_produces_image_draw_command() {
    let r = Renderer::new(RendererConfig::default());
    let scene = Scene {
        layers: vec![EvaluatedLayer {
            id: LayerId("img".into()),
            name: "".into(),
            layer_index: 0,
            kind: EvaluatedLayerKind::Image {
                asset: AssetRef::File {
                    path: "cover.jpg".into(),
                },
            },
            transform: ComputedTransform {
                position: Vec2 { x: 50.0, y: 60.0 },
                opacity: 0.8,
                ..Default::default()
            },
            visible: true,
            blend_mode: mva_scene::BlendMode::Normal,
        }],
        effects: vec![],
    };

    let dl = r.render(&scene, &vp());
    assert_eq!(dl.commands.len(), 1);
    match &dl.commands[0] {
        DrawCommand::Image(img) => {
            assert_eq!(
                img.asset,
                AssetRef::File {
                    path: "cover.jpg".into()
                }
            );
            assert_eq!(img.x, 50.0);
            assert_eq!(img.y, 60.0);
            assert_eq!(img.opacity, 0.8);
            assert_eq!(img.width, 100.0);
            assert_eq!(img.height, 100.0);
        }
        _ => panic!("expected Image draw command"),
    }
}

// =========================================================================
// Phase 3 — Effect pipeline
// =========================================================================

#[test]
fn effect_scene_produces_effect_draw_command() {
    let r = Renderer::new(RendererConfig::default());
    let scene = Scene {
        layers: vec![],
        effects: vec![mva_scene::ActiveEffect {
            effect_id: "bloom".into(),
            params: vec![("intensity".into(), ParamValue::Float { value: 0.5 })],
            target: EffectTarget::WholeScene,
        }],
    };

    let dl = r.render(&scene, &vp());
    assert_eq!(dl.commands.len(), 1);
    match &dl.commands[0] {
        DrawCommand::Effect(eff) => {
            assert_eq!(eff.effect_id, "bloom");
            assert_eq!(eff.params.len(), 1);
            assert_eq!(eff.params[0].0, "intensity");
            assert_eq!(eff.params[0].1, ParamValue::Float { value: 0.5 });
            assert_eq!(eff.target_rect, (0.0, 0.0, 1280.0, 720.0));
        }
        _ => panic!("expected Effect draw command"),
    }
}
