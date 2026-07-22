//! Audio player: rodio-backed transport with position reporting
//! (architecture §7.4).
//!
//! Implements [`PlaybackClock`](mva_core::PlaybackClock) and
//! [`AudioController`](mva_core::AudioController).  The two traits
//! are deliberately separate (§3 of `docs/phase2-architecture.md`).
//!
//! # Threading
//!
//! rodio drives audio on its own internal thread.  Mutable state
//! is protected by a [`std::sync::Mutex`].

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mva_core::PlaybackClock;
use mva_core::audio::{AudioController, AudioError as CoreAudioError};
use mva_core::effect::AudioCommand;
use rodio::Player;
use rodio::decoder::Decoder;
use rodio::source::Source;
use rodio::stream::{DeviceSinkBuilder, MixerDeviceSink};

use crate::error::AudioError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Stopped,
    Playing,
    Paused,
}

struct Inner {
    state: State,
    duration: f64,
    source_path: Option<PathBuf>,
}

/// The audio playback engine.
pub struct AudioPlayer {
    _sink: MixerDeviceSink,
    player: Player,
    inner: Mutex<Inner>,
    sample_rate: u32,
    channels: u16,
}

impl AudioPlayer {
    /// Create a new audio player using the system default output device.
    pub fn new() -> Result<Self, AudioError> {
        let sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| AudioError::Backend(e.to_string()))?;
        let player = Player::connect_new(sink.mixer());
        Ok(Self {
            _sink: sink,
            player,
            inner: Mutex::new(Inner {
                state: State::Stopped,
                duration: 0.0,
                source_path: None,
            }),
            sample_rate: 0,
            channels: 0,
        })
    }

    /// Load an audio file from disk.
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), AudioError> {
        let path = path.as_ref().to_owned();
        let file = File::open(&path)?;
        let source = Decoder::try_from(BufReader::new(file))
            .map_err(|e| AudioError::Decode(e.to_string()))?;

        let dur = source
            .total_duration()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        self.sample_rate = source.sample_rate().get();
        self.channels = source.channels().get();

        self.player.clear();
        self.player.append(source);
        self.player.pause();

        let inner = self.inner.get_mut().unwrap();
        *inner = Inner {
            state: State::Stopped,
            duration: dur,
            source_path: Some(path),
        };
        Ok(())
    }

    /// Load a rodio [`Source`] directly (in‑memory, no file).
    pub fn load_source<S>(&mut self, source: S) -> Result<(), AudioError>
    where
        S: Source + Send + 'static,
        <S as Iterator>::Item: Send + 'static,
    {
        let dur = source
            .total_duration()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        self.sample_rate = source.sample_rate().get();
        self.channels = source.channels().get();

        self.player.clear();
        self.player.append(source);
        self.player.pause();

        let inner = self.inner.get_mut().unwrap();
        *inner = Inner {
            state: State::Stopped,
            duration: dur,
            source_path: None,
        };
        Ok(())
    }

    /// Start or resume playback.
    pub fn play(&mut self) -> Result<(), AudioError> {
        let inner = self.inner.get_mut().unwrap();
        apply_play(&self.player, inner)
    }

    /// Pause playback.
    pub fn pause(&mut self) -> Result<(), AudioError> {
        let inner = self.inner.get_mut().unwrap();
        if inner.state != State::Playing {
            return Err(AudioError::InvalidState(format!(
                "cannot pause while {:?}",
                inner.state
            )));
        }
        self.player.pause();
        inner.state = State::Paused;
        Ok(())
    }

    /// Stop playback and clear the queue.
    pub fn stop(&mut self) -> Result<(), AudioError> {
        self.player.stop();
        self.inner.get_mut().unwrap().state = State::Stopped;
        Ok(())
    }

    /// Set playback volume `0.0–1.0`.
    pub fn set_volume(&self, vol: f32) {
        self.player.set_volume(vol);
    }

    /// Stream duration in seconds.
    pub fn duration_seconds(&self) -> f64 {
        self.inner.lock().unwrap().duration
    }

    /// Sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Number of audio channels.
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

// -----------------------------------------------------------------------
// PlaybackClock
// -----------------------------------------------------------------------

impl PlaybackClock for AudioPlayer {
    fn position_seconds(&self) -> f64 {
        if self.inner.lock().unwrap().state == State::Stopped {
            return 0.0;
        }
        self.player.get_pos().as_secs_f64()
    }
}

// -----------------------------------------------------------------------
// AudioController — transport via &self (Mutex‑protected state)
// -----------------------------------------------------------------------

impl AudioController for AudioPlayer {
    fn apply(&self, command: AudioCommand) -> Result<(), CoreAudioError> {
        match command {
            AudioCommand::Play => {
                let mut inner = self.inner.lock().unwrap();
                apply_play(&self.player, &mut inner).map_err(to_core_err)
            }
            AudioCommand::Pause => {
                let mut inner = self.inner.lock().unwrap();
                if inner.state != State::Playing {
                    return Ok(());
                }
                self.player.pause();
                inner.state = State::Paused;
                Ok(())
            }
            AudioCommand::Stop => {
                self.player.stop();
                self.inner.lock().unwrap().state = State::Stopped;
                Ok(())
            }
            AudioCommand::Seek(seconds) => {
                let dur = std::time::Duration::from_secs_f64(seconds);
                self.player
                    .try_seek(dur)
                    .map_err(|e| CoreAudioError::BackendError(e.to_string()))?;
                Ok(())
            }
            AudioCommand::SetVolume(vol) => {
                self.player.set_volume(vol);
                Ok(())
            }
        }
    }
}

// -----------------------------------------------------------------------
// SharedAudioPlayer — Arc newtype implementing both traits
// -----------------------------------------------------------------------

/// A shareable handle to an [`AudioPlayer`], implementing both
/// [`PlaybackClock`] and [`AudioController`] via an inner `Arc`.
#[derive(Clone)]
pub struct SharedAudioPlayer(Arc<AudioPlayer>);

impl SharedAudioPlayer {
    /// Wrap an existing [`AudioPlayer`] in a shareable handle.
    pub fn new(inner: AudioPlayer) -> Self {
        Self(Arc::new(inner))
    }
}

impl PlaybackClock for SharedAudioPlayer {
    fn position_seconds(&self) -> f64 {
        self.0.position_seconds()
    }
}

impl AudioController for SharedAudioPlayer {
    fn apply(&self, command: AudioCommand) -> Result<(), CoreAudioError> {
        self.0.apply(command)
    }
}

// -----------------------------------------------------------------------
// helpers
// -----------------------------------------------------------------------

fn apply_play(player: &Player, inner: &mut Inner) -> Result<(), AudioError> {
    match inner.state {
        State::Playing => Ok(()),
        State::Paused => {
            player.play();
            inner.state = State::Playing;
            Ok(())
        }
        State::Stopped => {
            if player.empty() {
                let path = inner.source_path.as_ref().ok_or(AudioError::NoSource)?;
                let file = File::open(path)?;
                let source = Decoder::try_from(BufReader::new(file))
                    .map_err(|e| AudioError::Decode(e.to_string()))?;
                inner.duration = source
                    .total_duration()
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(inner.duration);
                player.append(source);
            }
            player.play();
            inner.state = State::Playing;
            Ok(())
        }
    }
}

fn to_core_err(e: AudioError) -> CoreAudioError {
    match e {
        AudioError::NoSource => CoreAudioError::BackendError("no source loaded".into()),
        AudioError::Backend(m) | AudioError::Decode(m) => CoreAudioError::BackendError(m),
        AudioError::InvalidState(m) => CoreAudioError::BackendError(m),
        AudioError::Io(e) => CoreAudioError::BackendError(e.to_string()),
    }
}
