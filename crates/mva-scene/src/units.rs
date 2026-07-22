//! Shared 2D vector.

use serde::{Deserialize, Serialize};

/// 2D vector for spatial values (position, scale, anchor).
///
/// Semantics depend on the consuming context:
/// - position / anchor — pixels (or normalised coords)
/// - scale — multiplier (1.0 = 100%)
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}
