//! Evaluation engine: samples tracks/easing, looks up lyrics, and
//! produces a renderer-independent [`Scene`] (§5, §7.1).
//!
//! Every function in this module is **pure** — same inputs → same
//! outputs, always.  No mutable state, no side effects.
//!
//! The evaluator lives in a separate module from [`model`] to honour
//! the hard separation of plain data types from engine logic (§3.4).

pub mod easing;
pub mod effect;
pub mod interpolate;
pub mod lyric;
pub mod scene;
pub mod track;

pub use effect::evaluate_effects;
pub use interpolate::Interpolate;
pub use lyric::{active_lyric_index, active_lyric_text, active_lyric_word};
pub use scene::evaluate;
pub use track::value_at;

pub use mva_scene::{ComputedTransform, EvaluatedLayer, EvaluatedLayerKind, Scene};
