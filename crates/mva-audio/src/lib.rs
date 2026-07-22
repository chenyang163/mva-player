//! # mva-audio — Audio Engine
//!
//! Rodio‑based audio playback and [`PlaybackClock`] implementation
//! (architecture §3.1, §7.4).
//!
//! ## What this crate does
//!
//! - Open and decode audio files (MP3 / FLAC / WAV via Symphonia).
//! - Transport controls: `play` / `pause` / `stop` / volume.
//! - Report the current position in seconds for the timeline engine.
//!
//! ## What this crate does NOT do
//!
//! - Waveform visualisation, DSP effects, sync correction.
//! - Control the timeline, renderer, or UI — it is a pure time
//!   source consumed by [`mva-core`].
//!
//! ## Dependency
//!
//! Only `mva-core` (for [`PlaybackClock`]).  No `mva-timeline`,
//! no `mva-renderer`, no `mva-ui`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod error;
pub mod player;

pub use error::AudioError;
pub use player::{AudioPlayer, SharedAudioPlayer};
