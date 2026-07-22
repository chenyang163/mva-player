//! Application-level configuration (`config/app.toml`).

use serde::{Deserialize, Serialize};

/// Top-level application configuration, deserialized from
/// `config/app.toml` (§10).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// Window geometry.
    pub window: WindowConfig,
    /// General preferences.
    pub general: GeneralConfig,
}

/// Window size defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowConfig {
    /// Default window width in pixels.
    pub width: u32,
    /// Default window height in pixels.
    pub height: u32,
}

/// General user preferences.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    /// UI language code (e.g. `"en"`, `"zh-CN"`).
    pub language: String,
    /// Last directory the user opened a file from.
    #[serde(default)]
    pub last_directory: String,
    /// Default playback volume `0.0–1.0`.
    pub volume: f32,
}

impl AppConfig {
    /// Parse an `AppConfig` from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, crate::error::CoreError> {
        toml::from_str(toml_str).map_err(|e| crate::error::CoreError::Config(e.to_string()))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 1280,
                height: 720,
            },
            general: GeneralConfig {
                language: "en".into(),
                last_directory: String::new(),
                volume: 0.8,
            },
        }
    }
}
