//! Lyric timeline model (§4.3).

use serde::{Deserialize, Serialize};

/// The set of lyric tracks of a project.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LyricTimeline {
    #[serde(default)]
    pub tracks: Vec<LyricTrack>,
}

/// Role of a lyric track (§4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LyricRole {
    #[default]
    Original,
    Translation,
    Romanization,
}

/// One synchronized lyric track (one language / role).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LyricTrack {
    #[serde(default)]
    pub role: LyricRole,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub offset: f64,
    #[serde(default)]
    pub lines: Vec<LyricLine>,
}

/// One lyric line.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LyricLine {
    pub start: f64,
    #[serde(default)]
    pub end: Option<f64>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub words: Option<Vec<LyricWord>>,
}

/// One word with karaoke timing (word-level lyrics, Phase 2).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LyricWord {
    #[serde(default)]
    pub text: String,
    pub start: f64,
    #[serde(default)]
    pub end: Option<f64>,
}
