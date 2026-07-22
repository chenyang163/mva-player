//! Project metadata (§4.1).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Metadata of a project (§4.1).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub cover_image: Option<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub created_with: Option<String>,
    #[serde(default)]
    pub format_version: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub custom: BTreeMap<String, String>,
}
