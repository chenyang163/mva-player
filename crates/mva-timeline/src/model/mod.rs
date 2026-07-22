//! # The MVA serialized data model (architecture §4, §3.4)
//!
//! HARD RULE (§3.4): every type that appears in a serialized artifact
//! (`*.anim.json`, `manifest.json`, future `.mva`) lives in this module
//! tree.  This module contains **only plain serde data types**:
//!
//! - no engine logic (sampling/evaluation lives in `crate::eval`,
//!   Phase 1.2),
//! - no types from `mva-core`, egui/eframe, audio, or plugin crates.
//!
//! The module is extractable *verbatim* into the future public
//! `mva-types` crate when the first external consumer appears (§3.4).
//!
//! ## Timebase
//!
//! All times are `f64` **seconds**, continuous, defined by the audio
//! clock — the model is frame-rate independent (§4).
//!
//! ## Phase 1 scope
//!
//! Per architecture §11 (Phase 1 timeline scope):
//! - [`Easing`] variants: `Hold`, `Linear`, `Named` only (no
//!   `CubicBezier` yet),
//! - [`LayerKind`] variants: `Text` only (`Image`/`Shape` Phase 2,
//!   `ParticleEmitter` Phase 3),
//! - [`Project`] carries `LyricTimeline` + `AnimationTimeline` but
//!   **not** `EffectTimeline` (Phase 3).

pub mod animation;
pub mod audio;
pub mod lyric;
pub mod project;
pub mod track;
pub mod units;

// Flat re-exports: the model is one logical namespace.
pub use animation::{
    AnimationTimeline, BlendMode, Layer, LayerId, LayerKind, TextSource, TextStyle, Transform,
};
pub use audio::{AudioSource, AudioTimeline};
pub use lyric::{LyricLine, LyricRole, LyricTimeline, LyricTrack, LyricWord};
pub use project::{Project, ProjectMetadata};
pub use track::{Easing, Keyframe, NamedEase, Track};
pub use units::Vec2;

// Phase 3 model types (from mva-types).
pub use mva_types::EffectTimeline;
