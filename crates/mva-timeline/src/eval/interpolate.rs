//! Interpolation trait for sampled values.
//!
//! Lives in the evaluation layer (NOT the model) so the model stays
//! pure serde data types (§3.4).  Implementors must also be `Clone`
//! because sampling copies values out of keyframe storage.

use mva_types::ParamValue;
use mva_types::Vec2;

/// A type that can be linearly interpolated between two values and
/// cloned out of a [`Keyframe`](crate::model::Keyframe).
///
/// # Why `Clone` supertrait?
///
/// The track sampler must be able to return a copy of a keyframe's
/// value (the "hold at boundary" case).  Requiring only `Clone`
/// (rather than `Copy`) keeps the door open for non-trivial value
/// types (e.g. `String`, future bezier-path handles) while still
/// allowing `f32` / `Vec2` to work trivially.
pub trait Interpolate: Clone {
    /// Linearly interpolate from `self` (at `t = 0`) toward `other`
    /// (at `t = 1`), driven by a (possibly eased) normalised `t`.
    ///
    /// Callsites have already applied the easing function to `t`,
    /// so the implementor sees a value between 0 and 1 regardless of
    /// whether the segment was `Hold` (never called — the sampler
    /// shortcuts), `Linear`, or `Named` easing.
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

// -------------------------------------------------------------------
// built-in impls (Phase 1.2 — enough for the full Transform set)
// -------------------------------------------------------------------

impl Interpolate for f32 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        let t = t as f32;
        self + (other - self) * t
    }
}

impl Interpolate for Vec2 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        let t = t as f32;
        Vec2 {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl Interpolate for ParamValue {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        match (self, other) {
            (ParamValue::Float { value: a }, ParamValue::Float { value: b }) => {
                let t = t as f32;
                ParamValue::Float {
                    value: a + (b - a) * t,
                }
            }
            _ => *self, // Bool / Int / mixed types: step (hold)
        }
    }
}
