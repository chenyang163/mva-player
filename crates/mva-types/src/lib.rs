//! # mva-types — MVA Format Data Types
//!
//! Pure serde data types extracted from `mva-timeline::model` per
//! `docs/phase2-architecture.md` §6.
//!
//! This is a **leaf crate** — it depends on no other `mva-*` crate.
//! Types that reference evaluation‑coupled structures
//! (`AnimationTimeline`, `Layer`, `Transform`) remain in
//! `mva-timeline` until Phase 4.

#![forbid(unsafe_code)]

mod asset;
mod audio;
mod effect;
mod lyric;
mod metadata;
mod track;
mod units;

pub use asset::AssetRef;
pub use audio::{AudioSource, AudioTimeline};
pub use effect::{EffectInstance, EffectParam, EffectTarget, EffectTimeline, ParamValue};
pub use lyric::{LyricLine, LyricRole, LyricTimeline, LyricTrack, LyricWord};
pub use metadata::ProjectMetadata;
pub use track::{Easing, Keyframe, NamedEase, Track};
pub use units::Vec2;
