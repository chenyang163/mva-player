//! # mva-ui — UI Layer
//!
//! egui/eframe application shell, panel widgets, and the
//! [`DrawList`] → egui shapes painter adapter.
//!
//! ## Dependency rules (§3.2 rule 3, `docs/ui-architecture.md`)
//!
//! This is the **only** crate that depends on egui/eframe.
//! It never sees `mva-audio` types — the audio engine is behind
//! `Box<dyn PlaybackClock>`.

#![forbid(unsafe_code)]

mod app;
mod painter;
mod panels;

pub use app::MvaUiApp;
