//! Runtime state types: [`PlaybackState`] and [`EngineSnapshot`]
//! (architecture §7.2, `docs/phase2-architecture.md` §7).
//!
//! The UI polls [`EngineSnapshot`] once per frame (immediate-mode
//! pattern — no push events for position).

use std::fmt;

use mva_timeline::eval::Scene;

/// Playback lifecycle — 7‑state machine (Phase 2).
///
/// Transitions are driven by [`PlayerCommand`]s
/// ([`Engine::handle_command`](crate::engine::Engine::handle_command))
/// and by the audio clock ([`update_position`](crate::engine::Engine::update_position)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// No project loaded, or user explicitly stopped.
    Stopped,
    /// Project is being loaded (I/O in progress, decoder probing).
    /// Phase 2 Step 4 will transition here from the composition root.
    Loading,
    /// Project loaded successfully — transport controls are enabled.
    Ready,
    /// Audio is actively playing.
    Playing,
    /// Playback paused by user; position retained.
    Paused,
    /// Audio reached end of track; position == duration.
    Finished,
    /// A recoverable or unrecoverable error occurred.
    Error,
}

impl fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Stopped => "Stopped",
            Self::Loading => "Loading",
            Self::Ready => "Ready",
            Self::Playing => "Playing",
            Self::Paused => "Paused",
            Self::Finished => "Finished",
            Self::Error => "Error",
        };
        write!(f, "{s}")
    }
}

/// Structured playback error — backend‑agnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackError {
    /// The audio output device is not available.
    AudioDeviceUnavailable,
    /// Audio decoding failed (corrupt file, unsupported format).
    DecodeFailed,
    /// Project loading failed (file not found, invalid structure).
    ProjectLoadFailed,
    /// An otherwise‑unclassified error.
    Unknown(String),
}

/// Immutable per-frame snapshot polled by the UI (§7.2).
#[derive(Debug, Clone, PartialEq)]
pub struct EngineSnapshot {
    /// Current playback state.
    pub state: PlaybackState,
    /// Audio-clock position in seconds.
    pub position: f64,
    /// Total duration in seconds (0 if no project loaded).
    pub duration: f64,
    /// Current volume `0.0–1.0`.
    pub volume: f32,
    /// Index of the active lyric line within the chosen track, if any.
    pub active_lyric_index: Option<usize>,
    /// Fully evaluated scene at the current position (None if no
    /// project is loaded).
    pub scene: Option<Scene>,
    /// Active error, if [`PlaybackState::Error`].
    pub error: Option<PlaybackError>,
}

impl EngineSnapshot {
    /// Sentinel snapshot — safe for the UI to render a placeholder.
    pub fn empty() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: 0.0,
            duration: 0.0,
            volume: 0.8,
            active_lyric_index: None,
            scene: None,
            error: None,
        }
    }
}
