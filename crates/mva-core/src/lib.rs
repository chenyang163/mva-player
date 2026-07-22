//! # mva-core — Application Runtime
//!
//! Owns the application state machine, configuration structs,
//! command/event types, and the playback clock trait (architecture
//! §3.1, §7.2).
//!
//! ## Dependency rules (§3.2)
//!
//! - `mva-core` depends on `mva-timeline` (for the data model and
//!   evaluation output types) — permitted by the Phase 1.3 brief.
//! - `mva-timeline` must **never** depend on `mva-core` (§3.4).
//! - No egui/eframe/wgpu types; no audio/plugin types in this crate.
//!
//! ## What lives here (Phase 1.3)
//!
//! - [`config`] — `AppConfig`, `AnimationConfig` (§10 config-first rule).
//! - [`Engine`] — state machine: accept [`PlayerCommand`]s,
//!   maintain playback state, and produce per-frame
//!   [`EngineSnapshot`]s (with scene evaluation).
//! - [`PlayerCommand`] — UI → engine commands.
//! - [`PlaybackState`] — the playback lifecycle.
//! - [`EngineSnapshot`] — polled once per frame by the UI (§7.2).
//! - [`PlaybackClock`] — trait abstracting the audio position source
//!   (to be implemented by `mva-audio`).
//! - [`config`] variants: [`AppConfig`](config::app::AppConfig),
//!   [`AnimationConfig`](config::animation::AnimationConfig),
//!   [`AudioConfig`](config::audio::AudioConfig),
//!   [`LyricsConfig`](config::lyrics::LyricsConfig).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audio;
pub mod clock;
pub mod command;
pub mod config;
pub mod effect;
pub mod engine;
pub mod error;
pub mod loader;
pub mod state;

pub use audio::{AudioController, AudioError};
pub use clock::PlaybackClock;
pub use command::PlayerCommand;
pub use effect::{AudioCommand, EngineEffect};
pub use engine::Engine;
pub use error::CoreError;
pub use loader::{ProjectLoadError, ProjectLoader};
pub use state::{EngineSnapshot, PlaybackError, PlaybackState};
