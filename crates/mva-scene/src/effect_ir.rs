//! Active effect IR — resolved effect data for a single time point.

use mva_types::{EffectTarget, ParamValue};

/// A resolved effect at time `t`, ready for the renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveEffect {
    /// Effect identifier (e.g. `"bloom"`, `"spectrum"`).
    pub effect_id: String,
    /// Evaluated parameters (sampled from tracks at time `t`).
    pub params: Vec<(String, ParamValue)>,
    /// What the effect targets (delegates to `mva_types::EffectTarget`).
    pub target: EffectTarget,
}
