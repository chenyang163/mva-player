//! Renderer configuration (counterpart of `config/renderer.toml`).

use serde::{Deserialize, Serialize};

/// Static renderer settings loaded at startup.
///
/// # Ownership
///
/// This crate **never** reads config files from disk.  The owning
/// binary deserialises `config/renderer.toml` at startup and injects
/// the resulting struct into [`Renderer::new`](crate::Renderer::new).
///
/// # Phase 1.6
///
/// The struct is intentionally empty — all runtime state lives in
/// [`Viewport`](crate::Viewport).  Future phases will add quality
/// presets, AA samples, effect knobs, etc.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct RendererConfig {}

impl RendererConfig {
    /// Parse from the contents of `config/renderer.toml`.
    ///
    /// I/O is handled by the caller (the binary); this method only
    /// deserialises.
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str).map_err(|e| e.to_string())
    }
}
