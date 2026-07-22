//! [`EngineEffect`] — side‑effects the Engine produces but does not
//! execute (architecture §3.2, `docs/phase2-architecture.md` §3).
//!
//! The Engine stays a pure state machine.  Effects are returned from
//! [`handle_command`](crate::engine::Engine::handle_command) as a
//! `Vec<EngineEffect>` and applied by the composition root (binary).

/// Commands forwarded to the audio device via [`AudioController`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioCommand {
    /// Start or resume playback.
    Play,
    /// Pause playback (position retained).
    Pause,
    /// Stop playback and reset position.
    Stop,
    /// Seek to an absolute position.
    Seek(f64),
    /// Set output volume `0.0–1.0`.
    SetVolume(f32),
}

/// Effects the Engine cannot apply itself.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEffect {
    /// Forward a transport command to the audio device.
    Audio(AudioCommand),
    /// Load a project from the given path.
    ///
    /// The composition root calls [`ProjectLoader::load`] and then
    /// feeds the resulting [`Project`] back to the engine via
    /// [`PlayerCommand::LoadProject`].
    ///
    /// Uses [`PathBuf`] (not `String`) because Windows paths may
    /// contain non‑UTF‑8 bytes.
    LoadProject {
        /// Filesystem path to the project (directory or file).
        path: std::path::PathBuf,
    },
}
