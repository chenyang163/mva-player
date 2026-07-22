//! Asset reference — a stable identifier for binary resources
//! (architecture §9, `docs/phase2-architecture.md` §6).
//!
//! Pure data: no path resolution, no file I/O, no GPU handles.

use serde::{Deserialize, Serialize};

/// A stable reference to a binary asset.
///
/// Timelines store `AssetRef`s (not filesystem paths) so a `.mva`
/// package moved to another machine still finds its images / fonts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetRef {
    /// A file on the local filesystem (absolute or relative).
    File {
        /// Filesystem path.
        path: String,
    },
    /// An entry inside a `.mva` package.
    Pkg {
        /// Package‑internal path, e.g. `assets/images/cover.jpg`.
        path: String,
    },
}
