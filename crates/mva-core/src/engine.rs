//! Application engine: pure state machine that processes
//! [`PlayerCommand`]s and produces per-frame [`EngineSnapshot`]s
//! (architecture §7.2, `docs/phase2-architecture.md` §7).

use mva_timeline::eval::{active_lyric_index, evaluate};
use mva_timeline::model::Project;

use crate::PlayerCommand;
use crate::config::animation::AnimationConfig;
use crate::config::app::AppConfig;
use crate::effect::{AudioCommand, EngineEffect};
use crate::error::CoreError;
use crate::state::{EngineSnapshot, PlaybackError, PlaybackState};

/// The application runtime engine — pure state machine.
pub struct Engine {
    app_config: AppConfig,
    animation_config: AnimationConfig,
    state: PlaybackState,
    error: Option<PlaybackError>,
    position: f64,
    duration: f64,
    volume: f32,
    project: Option<Project>,
}

impl Engine {
    /// Create a new engine with the given configuration.
    pub fn new(app_config: AppConfig, animation_config: AnimationConfig) -> Self {
        let volume = app_config.general.volume;
        Self {
            app_config,
            animation_config,
            state: PlaybackState::Stopped,
            error: None,
            position: 0.0,
            duration: 0.0,
            volume,
            project: None,
        }
    }

    /// Returns a reference to the application config.
    pub fn app_config(&self) -> &AppConfig {
        &self.app_config
    }

    /// Returns a reference to the animation config.
    pub fn animation_config(&self) -> &AnimationConfig {
        &self.animation_config
    }

    /// Set engine state directly (useful for tests simulating
    /// external transitions like `Finished` or `Error`).
    pub fn set_state(&mut self, state: PlaybackState) {
        self.state = state;
    }

    /// Set an error on the engine (transitions to `Error`).
    pub fn set_error(&mut self, error: PlaybackError) {
        self.error = Some(error);
        self.state = PlaybackState::Error;
    }

    // ------------------------------------------------------------------
    // command processing (pure)
    // ------------------------------------------------------------------

    /// Process a command from the UI.
    ///
    /// Updates internal state and returns [`EngineEffect`]s for the
    /// composition root.  The Engine itself performs **no** I/O.
    pub fn handle_command(&mut self, cmd: PlayerCommand) -> Result<Vec<EngineEffect>, CoreError> {
        let mut effects = Vec::new();
        match cmd {
            PlayerCommand::Play => {
                if self.project.is_none() {
                    return Err(CoreError::NoProjectLoaded);
                }
                match self.state {
                    PlaybackState::Ready | PlaybackState::Paused | PlaybackState::Finished => {
                        self.state = PlaybackState::Playing;
                        // Reset position if replaying from Finished.
                        if self.duration > 0.0 && self.position >= self.duration {
                            self.position = 0.0;
                        }
                        effects.push(EngineEffect::Audio(AudioCommand::Play));
                    }
                    PlaybackState::Stopped => {
                        // Stopped with a project: treat as Ready→Playing.
                        self.state = PlaybackState::Playing;
                        effects.push(EngineEffect::Audio(AudioCommand::Play));
                    }
                    PlaybackState::Playing => {
                        // Idempotent — still emit effect for robustness.
                        effects.push(EngineEffect::Audio(AudioCommand::Play));
                    }
                    PlaybackState::Loading | PlaybackState::Error => {
                        return Err(CoreError::invalid_state(self.state, "play"));
                    }
                }
            }
            PlayerCommand::Pause => {
                if self.state == PlaybackState::Playing {
                    self.state = PlaybackState::Paused;
                    effects.push(EngineEffect::Audio(AudioCommand::Pause));
                }
                // Other states: ignore silently (idempotent).
            }
            PlayerCommand::Stop => {
                if self.state == PlaybackState::Error {
                    self.error = None;
                }
                self.state = PlaybackState::Stopped;
                self.position = 0.0;
                effects.push(EngineEffect::Audio(AudioCommand::Stop));
            }
            PlayerCommand::Seek(seconds) => {
                let pos = seconds.max(0.0).min(self.duration);
                self.position = pos;
                effects.push(EngineEffect::Audio(AudioCommand::Seek(pos)));
            }
            PlayerCommand::SetVolume(vol) => {
                let v = vol.clamp(0.0, 1.0);
                self.volume = v;
                effects.push(EngineEffect::Audio(AudioCommand::SetVolume(v)));
            }
            PlayerCommand::LoadProject(project) => {
                let project = *project;
                self.duration = project.metadata.duration.unwrap_or(project.audio.duration);
                self.project = Some(project);
                self.state = PlaybackState::Ready;
                self.position = 0.0;
                self.error = None;
            }
            PlayerCommand::OpenFile(path) => {
                self.state = PlaybackState::Loading;
                self.error = None;
                effects.push(EngineEffect::LoadProject { path });
            }
        }
        Ok(effects)
    }

    // ------------------------------------------------------------------
    // clock integration
    // ------------------------------------------------------------------

    /// Called by the binary each frame with the current audio-clock
    /// position.  If the position reaches or exceeds duration while
    /// `Playing`, transitions to [`PlaybackState::Finished`].
    pub fn update_position(&mut self, position: f64) {
        self.position = position.max(0.0).min(self.duration);
        if self.state == PlaybackState::Playing
            && self.duration > 0.0
            && self.position >= self.duration
        {
            self.state = PlaybackState::Finished;
            self.position = self.duration;
        }
    }

    // ------------------------------------------------------------------
    // snapshot
    // ------------------------------------------------------------------

    /// Produce an immutable snapshot of the current engine state.
    pub fn snapshot(&self) -> EngineSnapshot {
        let (lyric_idx, scene) = if let Some(ref project) = self.project {
            let lyrics = &project.lyrics;
            let timeline = &project.animation;
            let idx = active_lyric_index(lyrics, self.position);
            let scene = evaluate(timeline, lyrics, &project.effect_timeline, self.position);
            (idx, Some(scene))
        } else {
            (None, None)
        };

        EngineSnapshot {
            state: self.state,
            position: self.position,
            duration: self.duration,
            volume: self.volume,
            active_lyric_index: lyric_idx,
            scene,
            error: self.error.clone(),
        }
    }
}
