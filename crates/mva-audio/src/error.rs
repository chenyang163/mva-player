//! Audio error types (architecture §11).
//!
//! `mva-audio` exposes one `thiserror` enum: [`AudioError`].

use thiserror::Error;

/// Errors originating from the audio engine.
#[derive(Debug, Error)]
pub enum AudioError {
    /// No audio source has been loaded; call `load_file()` or
    /// `load_source()` first.
    #[error("no audio source loaded")]
    NoSource,

    /// The requested transport action is not valid in the current
    /// playback state (e.g. `pause()` while already paused).
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// The audio backend (rodio / cpal / WASAPI) returned an error.
    #[error("audio backend error: {0}")]
    Backend(String),

    /// Symphonia could not decode the file (unsupported format,
    /// corrupt data, …).
    #[error("decode error: {0}")]
    Decode(String),

    /// File I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
