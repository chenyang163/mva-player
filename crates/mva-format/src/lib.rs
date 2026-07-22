//! # mva-format — MVA Format Engine
//!
//! Layer 2 of the architecture (§1).  Knows about file formats.
//! Implements [`ProjectLoader`](mva_core::ProjectLoader).

#![forbid(unsafe_code)]

mod loader;

pub use loader::{LoaderConfig, MvaLoader};
