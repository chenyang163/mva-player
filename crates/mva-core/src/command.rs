//! [`PlayerCommand`] — the command enum sent from the UI to the
//! engine (architecture §7.2).

use std::path::PathBuf;

use mva_timeline::model::Project;

/// Commands the UI sends to the [`Engine`](crate::engine::Engine).
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    /// Toggle or start playback.
    Play,
    /// Pause playback (position is retained).
    Pause,
    /// Stop playback and reset position to 0.
    Stop,
    /// Seek to an absolute position in seconds.
    Seek(f64),
    /// Set playback volume `0.0–1.0`.
    SetVolume(f32),
    /// Replace the currently loaded project.
    LoadProject(Box<Project>),
    /// User selected a file or directory to open.  Engine transitions
    /// to [`PlaybackState::Loading`] and emits
    /// [`EngineEffect::LoadProject`](crate::effect::EngineEffect::LoadProject).
    OpenFile(PathBuf),
}
