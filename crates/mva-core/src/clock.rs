//! [`PlaybackClock`] trait — abstracts the source of the current
//! audio position (architecture §3.1, §7.4).
//!
//! The canonical implementation is [`AudioPlayer`] in `mva-audio`
//! (rodio `Player` wrapper, Phase 1.4).  Mock / test implementations
//! are also valid for CI and unit testing.

/// A clock that reports the current playback position in seconds.
///
/// # Threading (§7.4)
///
/// Implementations may read from an `AtomicU64` sample counter
/// published by the audio thread; callers poll this trait on the main
/// (UI) thread once per frame.  It must be cheap and non-blocking.
pub trait PlaybackClock {
    /// Current playback position in continuous seconds.
    ///
    /// Returns `0.0` when the clock is stopped or no audio is loaded.
    fn position_seconds(&self) -> f64;
}
