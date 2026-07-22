//! Scene evaluation: `AnimationTimeline::evaluate(t) -> Scene` (§5).
//!
//! The evaluator is **pure**: same `(timeline, lyrics, t)` →
//! same `Scene` every time.  No hidden state, no side effects.
//!
//! The output types ([`Scene`], [`EvaluatedLayer`], …) live in
//! `mva-scene` — the shared IR between timeline and renderer.

use crate::model::animation::{AnimationTimeline, TextSource};
use crate::model::lyric::LyricTimeline;
use mva_scene::{ComputedTransform, EvaluatedLayer, EvaluatedLayerKind, Scene};
use mva_types::{EffectTimeline, Vec2};

use super::effect::evaluate_effects;
use super::lyric::{active_lyric_text, active_lyric_word};
use super::track::value_at;

// -------------------------------------------------------------------
// evaluation entry point
// -------------------------------------------------------------------

/// Pure evaluation of the animation timeline at time `t`.
///
/// This is the function the architecture calls
/// `AnimationTimeline::evaluate(t) -> Scene` (§5, §7.1).  It is a
/// free function (not a method on the model types) to keep the model
/// free of engine logic (§3.4).
pub fn evaluate(
    timeline: &AnimationTimeline,
    lyrics: &LyricTimeline,
    effects: &EffectTimeline,
    t: f64,
) -> Scene {
    let layers: Vec<EvaluatedLayer> = timeline
        .layers
        .iter()
        .enumerate()
        .map(|(i, layer)| {
            let kind = evaluate_layer_kind(&layer.kind, lyrics, t);
            let transform = evaluate_transform(&layer.transform, t);
            let visible = layer.visible_range.0 <= t && t < layer.visible_range.1;
            EvaluatedLayer {
                id: layer.id.clone(),
                name: layer.name.clone(),
                layer_index: i,
                kind,
                transform,
                visible,
                blend_mode: layer.blend_mode,
            }
        })
        .collect();
    Scene {
        layers,
        effects: evaluate_effects(effects, t),
    }
}

// -------------------------------------------------------------------
// sub-evaluators
// -------------------------------------------------------------------

fn evaluate_layer_kind(
    kind: &crate::model::animation::LayerKind,
    lyrics: &LyricTimeline,
    t: f64,
) -> EvaluatedLayerKind {
    match kind {
        crate::model::animation::LayerKind::Text { source, style } => {
            let text = resolve_text(source, lyrics, t);
            EvaluatedLayerKind::Text {
                text,
                style: style.clone(),
            }
        }
        crate::model::animation::LayerKind::Image { asset } => EvaluatedLayerKind::Image {
            asset: asset.clone(),
        },
    }
}

fn resolve_text(source: &TextSource, lyrics: &LyricTimeline, t: f64) -> String {
    match source {
        TextSource::Static { text } => text.clone(),
        TextSource::LyricLine => active_lyric_text(lyrics, t).unwrap_or("").to_owned(),
        TextSource::LyricWord => active_lyric_word(lyrics, t).unwrap_or("").to_owned(),
    }
}

fn evaluate_transform(xf: &crate::model::animation::Transform, t: f64) -> ComputedTransform {
    ComputedTransform {
        position: to_scene_vec2(value_at(&xf.position, t, Vec2 { x: 0.0, y: 0.0 })),
        scale: to_scene_vec2(value_at(&xf.scale, t, Vec2 { x: 1.0, y: 1.0 })),
        rotation: value_at(&xf.rotation, t, 0.0),
        opacity: value_at(&xf.opacity, t, 1.0),
        anchor: to_scene_vec2(value_at(&xf.anchor, t, Vec2 { x: 0.0, y: 0.0 })),
    }
}

/// Convert `mva_types::Vec2` → `mva_scene::Vec2` at the scene boundary.
fn to_scene_vec2(v: Vec2) -> mva_scene::Vec2 {
    mva_scene::Vec2 { x: v.x, y: v.y }
}
