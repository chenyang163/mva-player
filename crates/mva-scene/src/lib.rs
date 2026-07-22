//! # mva-scene — Renderer-Independent Scene IR
//!
//! This crate is the **shared intermediate representation** between
//! [`mva-timeline`] (evaluation) and [`mva-renderer`] (drawing).
//!
//! It is a leaf crate with no dependencies on other `mva-*` crates.
//! Both the timeline evaluator and the renderer depend on it, but
//! the renderer never depends on the timeline.
//!
//! ## Contents
//!
//! - **[`Scene`]** / [`EvaluatedLayer`] / [`EvaluatedLayerKind`] —
//!   the fully-evaluated frame output by the timeline engine (§5).
//! - **[`ComputedTransform`]** — resolved transform at time `t`.
//! - Building‑block types shared across the engine: [`Vec2`],
//!   [`LayerId`], [`BlendMode`], [`TextStyle`].
//!
//! ## What does NOT live here
//!
//! - Keyframes, easing curves, tracks — these are timeline model
//!   concerns (`mva-timeline`).
//! - Audio clock, commands, playback state — application runtime
//!   (`mva-core`).
//! - GPU / paint types (`egui`, `wgpu`) — rendering backend.
//!
//! ## Serialisation
//!
//! All types derive `serde::Serialize` / `Deserialize` for debugging
//! and future format work.  This is not a hard requirement.

#![forbid(unsafe_code)]

mod colour;
mod effect_ir;
mod layer;
mod scene;
mod transform;
mod units;

pub use colour::Rgba;
pub use effect_ir::ActiveEffect;
pub use layer::{BlendMode, EvaluatedLayer, EvaluatedLayerKind, LayerId, TextStyle};
pub use scene::Scene;
pub use transform::ComputedTransform;
pub use units::Vec2;
