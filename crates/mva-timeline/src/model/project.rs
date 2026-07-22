//! Project root model (§4, §4.1).
//!
//! `Project` is the single contract between parser, format, engine,
//! renderer, and editor (architecture decision 3).

use serde::{Deserialize, Serialize};

use mva_types::EffectTimeline;

use super::animation::AnimationTimeline;
pub use mva_types::{AudioTimeline, LyricTimeline, ProjectMetadata};

/// One loaded song / one `.mva` document — the single contract.
///
/// # Forward tolerance (§6.3)
///
/// Readers ignore unknown fields.  Every field except `metadata` and
/// `audio` has a serde default, so artifacts from newer versions stay
/// readable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Song/project metadata.
    pub metadata: ProjectMetadata,
    /// The audio timeline (master clock domain).
    pub audio: AudioTimeline,
    /// All lyric tracks (may be empty).
    #[serde(default)]
    pub lyrics: LyricTimeline,
    /// The animation timeline (may be empty).
    #[serde(default)]
    pub animation: AnimationTimeline,
    /// The effect timeline (may be empty).
    #[serde(default)]
    pub effect_timeline: EffectTimeline,
}
