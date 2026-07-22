//! The fully-evaluated scene — a flat, z-ordered list of evaluated
//! layers at a single time point (§5).

use super::EvaluatedLayer;
use super::effect_ir::ActiveEffect;

/// A fully-evaluated frame at time `t`.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// Layers in z-order, bottom first.
    pub layers: Vec<EvaluatedLayer>,
    /// Resolved effects to apply (empty when no effects are active).
    pub effects: Vec<ActiveEffect>,
}

impl Scene {
    /// An empty scene — useful as a sentinel when no project is
    /// loaded.
    pub fn empty() -> Self {
        Self {
            layers: Vec::new(),
            effects: Vec::new(),
        }
    }
}
