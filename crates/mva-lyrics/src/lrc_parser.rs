//! LRC format parser → [`LyricTrack`].

use mva_types::{LyricLine, LyricTimeline, LyricTrack};
use thiserror::Error;

/// Lyric parsing errors.
#[derive(Debug, Error)]
pub enum LyricParseError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

/// Parse an LRC string into a [`LyricTimeline`] with one
/// [`LyricTrack`] (role = `Original`).
pub fn parse_lrc(content: &str) -> Result<LyricTimeline, LyricParseError> {
    let parsed =
        lrc::Lyrics::from_str(content).map_err(|e| LyricParseError::Parse(e.to_string()))?;

    let lines: Vec<LyricLine> = parsed
        .get_timed_lines()
        .iter()
        .filter(|(_, text)| !text.is_empty())
        .map(|(tag, text)| {
            // lrc::Timestamp::get_timestamp returns milliseconds as i64
            let start_ms = tag.get_timestamp();
            LyricLine {
                start: start_ms as f64 / 1000.0,
                end: None,
                text: text.to_string(),
                words: None,
            }
        })
        .collect();

    Ok(LyricTimeline {
        tracks: vec![LyricTrack {
            role: mva_types::LyricRole::Original,
            language: None,
            offset: 0.0,
            lines,
        }],
    })
}
