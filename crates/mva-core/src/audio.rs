//! [`AudioController`] trait — transport abstraction.
//!
//! Defined in `mva-core`; implemented in `mva-audio`.  Separated
//! from [`PlaybackClock`] (sensor vs. actuator).

/// Errors surfaced by [`AudioController::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioError {
    /// The audio output device is not available.
    DeviceUnavailable,
    /// An unrecoverable backend error occurred.
    BackendError(String),
}

/// Audio transport control — separated from [`PlaybackClock`].
///
/// # Thread safety
///
/// Implementations must be `Send + Sync`.  Methods are called from
/// the main (UI) thread; the audio backend drives its own thread
/// internally.  Internal synchronisation is the implementor's
/// responsibility.
///
/// # Separation from PlaybackClock
///
/// `AudioController` **executes** transport commands.
/// [`PlaybackClock`] **reads** the current position.
pub trait AudioController: Send + Sync {
    /// Execute a transport command on the audio device.
    fn apply(&self, command: super::effect::AudioCommand) -> Result<(), AudioError>;
}
