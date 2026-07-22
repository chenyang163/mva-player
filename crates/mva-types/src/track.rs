//! Animated property primitives: [`Track`], [`Keyframe`], [`Easing`]
//! (architecture §4.6).

use serde::{Deserialize, Serialize};

/// One animated property lane: a time-ordered list of keyframes.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Track<T> {
    #[serde(default)]
    pub keyframes: Vec<Keyframe<T>>,
}

/// A single keyed value plus the easing used when *leaving* this
/// keyframe toward the next one.
///
/// Only `Clone` (not `Copy`) is derived — future value types
/// (e.g. `String`, bezier paths) must not be forced to `Copy`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: f64,
    pub value: T,
    #[serde(default)]
    pub easing: Easing,
}

/// Easing curve applied between two keyframes (§4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    Hold,
    #[default]
    Linear,
    Named(NamedEase),
}

/// Named easing curves (§4.6), evaluated via `simple-easing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedEase {
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
}
