//! Animation configuration (`config/animation.toml`, §10).
//!
//! These parameters control the auto-generated animated lyric text
//! layer in Phase 1 — the evaluator pipeline reads them to produce
//! opacity / scale keyframes on the default lyric layer (§11).

use serde::{Deserialize, Serialize};

/// Animation subsystem configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationConfig {
    /// Defaults for the lyric text layer generated in Phase 1.
    pub lyric_layer: LyricLayerConfig,
}

/// Parameters that drive the opacity fade + scale animation on the
/// default lyric text layer (§11 acceptance criterion 3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LyricLayerConfig {
    /// Fade-in duration in seconds.
    pub fade_in_duration: f64,
    /// Fade-out duration in seconds.
    pub fade_out_duration: f64,
    /// Start scale (uniform; applied to both x and y).
    pub scale_from: f32,
    /// End scale (uniform).
    pub scale_to: f32,
    /// Named easing function applied to both opacity and scale
    /// animation (e.g. `"ease_out_cubic"`).
    pub default_easing: String,
}

impl AnimationConfig {
    /// Parse an `AnimationConfig` from a TOML string.
    ///
    /// # Validation
    ///
    /// The `lyric_layer.default_easing` string is validated against
    /// the known [`NamedEase`](mva_timeline::model::NamedEase)
    /// variants.  Invalid names produce a configuration error (never
    /// a silent fallback).
    pub fn from_toml(toml_str: &str) -> Result<Self, crate::error::CoreError> {
        let config: Self =
            toml::from_str(toml_str).map_err(|e| crate::error::CoreError::Config(e.to_string()))?;
        validate_easing(&config.lyric_layer.default_easing)?;
        Ok(config)
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            lyric_layer: LyricLayerConfig {
                fade_in_duration: 0.4,
                fade_out_duration: 0.3,
                scale_from: 0.9,
                scale_to: 1.0,
                default_easing: "ease_out_cubic".into(),
            },
        }
    }
}

// -------------------------------------------------------------------
// validation helpers
// -------------------------------------------------------------------

/// Validate that `name` is a recognised [`NamedEase`] variant.
///
/// This list must stay synchronised with
/// `mva_timeline::model::NamedEase`.
fn validate_easing(name: &str) -> Result<(), crate::error::CoreError> {
    match name {
        "ease_in_sine" | "ease_out_sine" | "ease_in_out_sine" | "ease_in_quad"
        | "ease_out_quad" | "ease_in_out_quad" | "ease_in_cubic" | "ease_out_cubic"
        | "ease_in_out_cubic" | "ease_in_quart" | "ease_out_quart" | "ease_in_out_quart"
        | "ease_in_expo" | "ease_out_expo" | "ease_in_out_expo" | "ease_in_circ"
        | "ease_out_circ" | "ease_in_out_circ" => Ok(()),
        other => Err(crate::error::CoreError::Config(format!(
            "unknown easing name: '{other}' \
             (expected one of: ease_in_sine, ease_out_sine, …, ease_in_out_circ)"
        ))),
    }
}
