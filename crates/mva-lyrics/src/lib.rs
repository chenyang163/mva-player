//! # mva-lyrics — Lyric Parser
//!
//! Parses `.lrc` files into [`LyricTimeline`] structures.

#![forbid(unsafe_code)]

mod lrc_parser;

pub use lrc_parser::{LyricParseError, parse_lrc};
