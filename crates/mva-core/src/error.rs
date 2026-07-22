//! Core error types (architecture §11).
//!
//! Each crate exposes one `thiserror` enum; `mva-core` errors cover
//! configuration, state-machine violations, and IO.

use crate::state::PlaybackState;
use thiserror::Error;

/// Errors originating from `mva-core` (config, engine, clock).
#[derive(Debug, Error)]
pub enum CoreError {
    /// The requested operation requires a loaded project.
    #[error("no project is loaded")]
    NoProjectLoaded,

    /// An invalid state transition was attempted.
    #[error("invalid state: cannot {attempted} while {current}")]
    InvalidState {
        /// The current playback state.
        current: PlaybackState,
        /// The action that was attempted (e.g. `"play"`, `"pause"`).
        attempted: &'static str,
    },

    /// Configuration parse error.
    #[error("config error: {0}")]
    Config(String),

    /// IO error (file read, …).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl CoreError {
    /// Convenience constructor for [`CoreError::InvalidState`].
    pub fn invalid_state(current: PlaybackState, attempted: &'static str) -> Self {
        Self::InvalidState { current, attempted }
    }
}
