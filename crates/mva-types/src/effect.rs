//! Effect timeline model (architecture §4.5).
//!
//! Pure data: no runtime evaluation, no GPU, no plugin host.
//! Parameters reuse the existing [`Track`] mechanism.

use serde::{Deserialize, Serialize};

use super::track::Track;

/// The effect timeline — a list of effect instances applied to the
/// scene at specific time ranges.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EffectTimeline {
    /// Effect instances, sorted by `time_range` start (convention;
    /// not enforced by the model).
    #[serde(default)]
    pub effects: Vec<EffectInstance>,
}

/// One effect instance with a time window and target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectInstance {
    /// Time window `(start, end)` in seconds.
    pub time_range: (f64, f64),
    /// Effect identifier (e.g. `"bloom"`, `"spectrum"`, `"particles.snow"`).
    pub effect_id: String,
    /// What the effect applies to.
    #[serde(default)]
    pub target: EffectTarget,
    /// Animated parameters — each is a [`Track<ParamValue>`].
    #[serde(default)]
    pub parameters: Vec<EffectParam>,
}

/// What the effect targets in the scene.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectTarget {
    /// Apply to the entire scene.
    #[default]
    WholeScene,
    /// Apply to a specific layer.
    Layer {
        /// Layer id.
        layer_id: String,
    },
    /// Apply to the background only.
    Background,
}

/// One animated effect parameter — a named [`Track<ParamValue>`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectParam {
    /// Parameter name (e.g. `"intensity"`, `"threshold"`).
    pub name: String,
    /// Animated value track.
    #[serde(default)]
    pub track: Track<ParamValue>,
}

/// A value that an effect parameter can take.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParamValue {
    /// Floating‑point value.
    Float { value: f32 },
    /// Boolean value.
    Bool { value: bool },
    /// Integer value.
    Int { value: i32 },
}

impl Default for ParamValue {
    fn default() -> Self {
        Self::Float { value: 0.0 }
    }
}
