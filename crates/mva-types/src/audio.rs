//! Audio timeline model (§4.2) — the master clock domain.

use serde::{Deserialize, Serialize};

use super::track::Track;

/// The audio timeline: the master clock all other timelines sync to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioTimeline {
    pub source: AudioSource,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub sample_rate: u32,
    #[serde(default)]
    pub channels: u8,
    #[serde(default)]
    pub volume_envelope: Option<Track<f32>>,
}

/// Location of the audio stream (§4.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AudioSource {
    Embedded { entry_path: String },
    ExternalFile { path: String },
}
