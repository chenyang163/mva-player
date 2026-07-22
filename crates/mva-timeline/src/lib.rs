//! # mva-timeline — MVA Timeline Engine
//!
//! Owns the timeline **data model** and its **evaluation** at time `t`
//! (architecture §3.1, §4–§5). Pure computation: no audio, no GPU,
//! no UI. Dependency rule (§3.2): this crate must never depend on
//! egui/eframe/wgpu types, and the [`model`] module must not reference
//! `mva-core` types (§3.4).
//!
//! ## Phase 1.1 scope (this milestone)
//!
//! Only the serialized **data model** ([`model`]) — plain serde types
//! forming the contract that `*.anim.json`, `manifest.json` and the
//! future `.mva` container serialize (§3.4, architecture decision 9).
//!
//! Not here yet (later milestones, architecture §11):
//!
//! - track sampling / easing evaluation (`value_at`), lyric lookup,
//!   `AnimationTimeline::evaluate(t) -> Scene` — Phase 1.2,
//! - `Easing::CubicBezier`, `LayerKind::Image`/`Shape` — Phase 2,
//! - `EffectTimeline`, `LayerKind::ParticleEmitter` — Phase 3.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod eval;
pub mod model;
