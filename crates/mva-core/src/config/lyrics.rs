//! Lyrics subsystem configuration (`config/lyrics.toml`, §10).

use serde::{Deserialize, Serialize};

/// Lyrics parsing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LyricsConfig {
    /// Parsing preferences.
    pub parse: LyricsParseConfig,
}

/// Parser tuning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LyricsParseConfig {
    /// Fallback text encoding if the file is not valid UTF-8.
    pub encoding: String,
    /// Global offset shift in seconds applied to every lyric line.
    pub offset_adjust: f64,
}

impl LyricsConfig {
    /// Parse a `LyricsConfig` from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, crate::error::CoreError> {
        toml::from_str(toml_str).map_err(|e| crate::error::CoreError::Config(e.to_string()))
    }
}

impl Default for LyricsConfig {
    fn default() -> Self {
        Self {
            parse: LyricsParseConfig {
                encoding: "utf-8".into(),
                offset_adjust: 0.0,
            },
        }
    }
}
