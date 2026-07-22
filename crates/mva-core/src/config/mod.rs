//! Configuration types (§10).
//!
//! Configuration-First rule (Agent.md): every adjustable parameter
//! lives in a config file + a corresponding Rust struct.  These
//! structs live here; the default TOML files live in `config/*.toml`
//! at the repository root.

pub mod animation;
pub mod app;
pub mod audio;
pub mod lyrics;
