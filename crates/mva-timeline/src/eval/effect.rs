//! Effect timeline evaluator: `EffectTimeline` → `Vec<ActiveEffect>`
//! (architecture §4.5, `docs/phase3-architecture.md`).
//!
//! Pure evaluation: same `(timeline, t)` → same output.

use mva_scene::ActiveEffect;
use mva_types::{EffectTimeline, ParamValue};

use super::track::value_at;

/// Evaluate an [`EffectTimeline`] at time `t`, returning the list of
/// currently active effects with their sampled parameters.
pub fn evaluate_effects(timeline: &EffectTimeline, t: f64) -> Vec<ActiveEffect> {
    timeline
        .effects
        .iter()
        .filter(|inst| t >= inst.time_range.0 && t < inst.time_range.1)
        .map(|inst| {
            let params: Vec<(String, ParamValue)> = inst
                .parameters
                .iter()
                .map(|p| {
                    let sampled = value_at(&p.track, t, ParamValue::default());
                    (p.name.clone(), sampled)
                })
                .collect();
            ActiveEffect {
                effect_id: inst.effect_id.clone(),
                params,
                target: inst.target.clone(),
            }
        })
        .collect()
}
