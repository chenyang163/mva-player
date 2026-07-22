//! Resolved transform at a single time point.

use super::Vec2;

/// Computed transform values after all tracks have been sampled.
///
/// Identity defaults are applied when a track has no keyframes (§5):
/// - position / anchor → `(0, 0)`
/// - scale → `(1, 1)`
/// - rotation → `0` (degrees)
/// - opacity → `1` (fully opaque)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComputedTransform {
    /// Position in local space.
    pub position: Vec2,
    /// Scale multiplier.
    pub scale: Vec2,
    /// Rotation in degrees (clockwise, screen‑space y‑down).
    pub rotation: f32,
    /// Opacity `0.0–1.0`.
    pub opacity: f32,
    /// Anchor point (pivot for scale / rotation).
    pub anchor: Vec2,
}

impl Default for ComputedTransform {
    fn default() -> Self {
        Self {
            position: Vec2 { x: 0.0, y: 0.0 },
            scale: Vec2 { x: 1.0, y: 1.0 },
            rotation: 0.0,
            opacity: 1.0,
            anchor: Vec2 { x: 0.0, y: 0.0 },
        }
    }
}
