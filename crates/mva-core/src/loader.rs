//! [`ProjectLoader`] trait — shared project‑loading contract
//! (architecture §3.1, `docs/phase2-architecture.md` §4).
//!
//! Defined in `mva-core` so that every MVA front‑end (player binary,
//! CLI, editor, server) can load projects through the same trait.

use std::fmt;
use std::path::Path;

use mva_timeline::model::Project;

/// Structured error from [`ProjectLoader::load`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectLoadError {
    /// I/O error.
    Io(String),
    /// Directory or path contains no recognised audio file.
    NoAudioFile,
    /// Lyrics file found but could not be parsed.
    InvalidLyrics(String),
    /// A `.mva` manifest was found but could not be parsed or
    /// references entries that cannot be loaded.
    InvalidManifest(String),
    /// The path format is not supported (e.g. a `.docx` was opened).
    UnsupportedFormat(String),
    /// Catch‑all for otherwise‑unclassified errors.
    Unknown(String),
}

impl fmt::Display for ProjectLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::NoAudioFile => {
                write!(f, "no recognised audio file found")
            }
            Self::InvalidLyrics(msg) => write!(f, "invalid lyrics: {msg}"),
            Self::InvalidManifest(msg) => {
                write!(f, "invalid manifest: {msg}")
            }
            Self::UnsupportedFormat(msg) => {
                write!(f, "unsupported format: {msg}")
            }
            Self::Unknown(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProjectLoadError {}

/// Shared project‑loading contract.
///
/// Implementations live in `mva-format` (Phase 2) and may also be
/// provided by plugins (Phase 6+).
pub trait ProjectLoader: Send + Sync {
    /// Load a [`Project`] from the given path (a directory, a single
    /// audio file, or a future `.mva` container).
    fn load(&self, path: &Path) -> Result<Project, ProjectLoadError>;

    /// File extensions this loader can open (for UI filtering).
    fn supported_extensions(&self) -> &[&str];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_no_audio_file() {
        let err = ProjectLoadError::NoAudioFile;
        assert_eq!(err.to_string(), "no recognised audio file found");
    }

    #[test]
    fn display_io() {
        let err = ProjectLoadError::Io("file not found".into());
        assert_eq!(err.to_string(), "I/O error: file not found");
    }
}
