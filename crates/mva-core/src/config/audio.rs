//! Audio subsystem configuration (`config/audio.toml`, §10).

use serde::{Deserialize, Serialize};

/// Audio playback configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioConfig {
    /// Playback preferences.
    pub playback: AudioPlaybackConfig,
    /// Output device selection.
    #[serde(default)]
    pub output: AudioOutputConfig,
}

/// Playback tuning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioPlaybackConfig {
    /// Audio sink buffer size in milliseconds.
    pub buffer_size_ms: u32,
    /// Reserved: gapless transition toggle.
    #[serde(default)]
    pub gapless_enabled: bool,
}

/// Audio output device selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AudioOutputConfig {
    /// Device name as reported by cpal; empty = system default.
    #[serde(default)]
    pub device: String,
}

impl AudioConfig {
    /// Parse an `AudioConfig` from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, crate::error::CoreError> {
        toml::from_str(toml_str).map_err(|e| crate::error::CoreError::Config(e.to_string()))
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            playback: AudioPlaybackConfig {
                buffer_size_ms: 50,
                gapless_enabled: false,
            },
            output: AudioOutputConfig::default(),
        }
    }
}
