//! Lyric error types.

use thiserror::Error;

/// Errors that can occur during lyric parsing.
#[derive(Debug, Error)]
pub enum LyricError {
    /// LRC parsing failed at a specific line.
    #[error("lrc parse error at line {line}: {message}")]
    Parse {
        /// 1-based line number where the error occurred.
        line: usize,
        /// Human-readable description.
        message: String,
    },

    /// I/O error while reading a lyric file.
    #[error("i/o error reading lyric file: {0}")]
    Io(#[from] std::io::Error),
}
