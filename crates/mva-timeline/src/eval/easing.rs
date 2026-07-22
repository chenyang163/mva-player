//! Easing evaluation: maps [`NamedEase`] variants to `simple-easing`
//! Penner curves (research §6).
//!
//! Phase 1.2: `Hold`, `Linear`, `Named` only (§11).
//! `CubicBezier` arrives in a later milestone.

use crate::model::NamedEase;

/// Apply a named easing to a normalized `t ∈ [0, 1]`, returning the
/// eased `t' ∈ [0, 1]`.
///
/// `simple-easing` expects `f32`; we convert, evaluate, and cast back
/// to `f64`.  The loss is negligible for visual animation.
pub(crate) fn apply_named_ease(ease: NamedEase, t: f64) -> f64 {
    let t32 = t.clamp(0.0, 1.0) as f32;
    let v: f32 = match ease {
        NamedEase::EaseInSine => simple_easing::sine_in(t32),
        NamedEase::EaseOutSine => simple_easing::sine_out(t32),
        NamedEase::EaseInOutSine => simple_easing::sine_in_out(t32),
        NamedEase::EaseInQuad => simple_easing::quad_in(t32),
        NamedEase::EaseOutQuad => simple_easing::quad_out(t32),
        NamedEase::EaseInOutQuad => simple_easing::quad_in_out(t32),
        NamedEase::EaseInCubic => simple_easing::cubic_in(t32),
        NamedEase::EaseOutCubic => simple_easing::cubic_out(t32),
        NamedEase::EaseInOutCubic => simple_easing::cubic_in_out(t32),
        NamedEase::EaseInQuart => simple_easing::quart_in(t32),
        NamedEase::EaseOutQuart => simple_easing::quart_out(t32),
        NamedEase::EaseInOutQuart => simple_easing::quart_in_out(t32),
        NamedEase::EaseInExpo => simple_easing::expo_in(t32),
        NamedEase::EaseOutExpo => simple_easing::expo_out(t32),
        NamedEase::EaseInOutExpo => simple_easing::expo_in_out(t32),
        NamedEase::EaseInCirc => simple_easing::circ_in(t32),
        NamedEase::EaseOutCirc => simple_easing::circ_out(t32),
        NamedEase::EaseInOutCirc => simple_easing::circ_in_out(t32),
    };
    v as f64
}
