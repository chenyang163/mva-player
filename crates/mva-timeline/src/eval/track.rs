//! Track sampling: `value_at` from a sorted-keyframe track (§4.6).
//!
//! Algorithm: binary search → apply outgoing keyframe's easing →
//! lerp between values via the [`Interpolate`] trait.  Pure, no state.

use crate::model::{Easing, Keyframe, Track};

use super::easing::apply_named_ease;
use super::interpolate::Interpolate;

/// Sample a track at time `t`.
///
/// Returns `default` when the track has no keyframes.  When `t` is
/// before the first keyframe the first value is held; when `t` is at
/// or after the last keyframe the last value is held (§4.6).
///
/// `T` must be [`Interpolate`] (for lerp) and `Clone` (for boundary
/// holds).  `f32` and [`Vec2`](crate::model::Vec2) both satisfy this;
/// callers pass their identity defaults explicitly.
pub fn value_at<T: Interpolate>(track: &Track<T>, t: f64, default: T) -> T {
    sample_track(track.keyframes.as_slice(), t, default)
}

// -------------------------------------------------------------------
// internal binary-search + easing engine
// -------------------------------------------------------------------

fn sample_track<T: Interpolate>(keyframes: &[Keyframe<T>], t: f64, default: T) -> T {
    if keyframes.is_empty() {
        return default;
    }
    let first = &keyframes[0];
    let last = &keyframes[keyframes.len() - 1];

    // After last or before first → hold at boundary.
    // `>= last` comes first: when multiple keyframes share the same
    // time and `t` equals that time, the *last* keyframe wins (AE
    // convention).
    if t >= last.time {
        return last.value.clone();
    }
    if t <= first.time {
        return first.value.clone();
    }

    // Binary search: last keyframe where time <= t
    let idx = keyframes.partition_point(|k| k.time <= t) - 1;
    let a = &keyframes[idx];
    let b = &keyframes[idx + 1];

    apply_keyframe_segment(a, b, t)
}

fn apply_keyframe_segment<T: Interpolate>(a: &Keyframe<T>, b: &Keyframe<T>, t: f64) -> T {
    match a.easing {
        Easing::Hold => a.value.clone(),
        _ => {
            let raw = safe_t(a.time, b.time, t);
            let eased = match a.easing {
                Easing::Named(e) => apply_named_ease(e, raw),
                _ => raw, // Linear (and default)
            };
            a.value.lerp(&b.value, eased)
        }
    }
}

/// Normalize `t` into `[0, 1]` within the segment, guarding against
/// degenerate (zero-duration) segments.
fn safe_t(seg_start: f64, seg_end: f64, t: f64) -> f64 {
    let denom = seg_end - seg_start;
    if denom <= 0.0 {
        return 1.0;
    }
    let raw = (t - seg_start) / denom;
    raw.clamp(0.0, 1.0)
}
