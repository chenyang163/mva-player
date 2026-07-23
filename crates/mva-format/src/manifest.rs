//! Loose `.mva` manifest reading (architecture §6.2, §6.3).
//!
//! The first readable form of the MVA format is the **loose project**:
//! a JSON manifest (`*.mva`) that references sibling media entries by
//! relative path — the same layout the ZIP container will carry later:
//!
//! ```json
//! {
//!   "format_version": "1.0",
//!   "generator": "mva-player 0.1.0",
//!   "id": "lyric-demo",
//!   "metadata": { "title": "…", "artist": "…", "duration": 125.0 },
//!   "entries": {
//!     "audio": "assets/song.mp3",
//!     "lyrics": ["lyrics.lrc"],
//!     "animation": "assets/lyric.anim.json"
//!   }
//! }
//! ```
//!
//! Entry paths are resolved relative to the manifest's directory, so a
//! loose project folder is fully relocatable.  Unknown manifest fields
//! are ignored (forward tolerance, §6.3); `format_version` majors ≥ 2
//! are refused.  The future ZIP container (Phase 4) will reuse this
//! manifest schema for its `manifest.json` entry.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use mva_core::loader::ProjectLoadError;
use mva_timeline::model::{
    AnimationTimeline, AudioSource, AudioTimeline, Project, ProjectMetadata,
};
use mva_types::{EffectTimeline, LyricTimeline};

fn default_format_version() -> String {
    "1.0".into()
}

/// The loose `.mva` manifest (architecture §6.2).
///
/// Pure serde data; unknown fields are ignored by serde's default
/// behaviour (forward tolerance, §6.3).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MvaManifest {
    /// Semver format version; any `1.x` is accepted, `≥2.0` refused.
    #[serde(default = "default_format_version")]
    pub format_version: String,
    /// Tool that wrote the manifest (informational).
    #[serde(default)]
    pub generator: Option<String>,
    /// Unique project id.
    #[serde(default)]
    pub id: Option<String>,
    /// Song metadata subset.
    #[serde(default)]
    pub metadata: ManifestMetadata,
    /// Media entry paths, relative to the manifest directory.
    pub entries: ManifestEntries,
}

/// Metadata subset carried by the manifest (architecture §6.2).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ManifestMetadata {
    /// Song title.
    #[serde(default)]
    pub title: Option<String>,
    /// Artist name.
    #[serde(default)]
    pub artist: Option<String>,
    /// Song duration in seconds (drives the engine clock domain).
    #[serde(default)]
    pub duration: Option<f64>,
}

/// Media entries referenced by the manifest.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ManifestEntries {
    /// The audio file (required; exactly one).
    pub audio: String,
    /// Lyric files (LRC).  Each file becomes one [`LyricTrack`].
    #[serde(default)]
    pub lyrics: Vec<String>,
    /// Optional `*.anim.json` animation timeline.
    #[serde(default)]
    pub animation: Option<String>,
}

/// Load a loose `.mva` JSON manifest into a [`Project`].
pub fn load_manifest(manifest_path: &Path) -> Result<Project, ProjectLoadError> {
    let text = fs::read_to_string(manifest_path)
        .map_err(|e| ProjectLoadError::Io(format!("read {}: {e}", manifest_path.display())))?;
    let manifest: MvaManifest = serde_json::from_str(&text).map_err(|e| {
        ProjectLoadError::InvalidManifest(format!("{}: {e}", manifest_path.display()))
    })?;

    check_format_version(&manifest.format_version)?;

    let base_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

    // ---- audio (required) ----
    let audio_path = base_dir.join(&manifest.entries.audio);
    if !audio_path.is_file() {
        return Err(ProjectLoadError::Io(format!(
            "audio entry not found: {}",
            audio_path.display()
        )));
    }

    // ---- lyrics (optional, LRC only for now) ----
    let mut lyrics = LyricTimeline { tracks: vec![] };
    for entry in &manifest.entries.lyrics {
        let lrc_path = base_dir.join(entry);
        let content = fs::read_to_string(&lrc_path)
            .map_err(|e| ProjectLoadError::Io(format!("read {}: {e}", lrc_path.display())))?;
        let mut timeline = mva_lyrics::parse_lrc(&content)
            .map_err(|e| ProjectLoadError::InvalidLyrics(format!("{}: {e}", lrc_path.display())))?;
        lyrics.tracks.append(&mut timeline.tracks);
    }

    // ---- animation (optional *.anim.json) ----
    let animation = match &manifest.entries.animation {
        Some(entry) => {
            let anim_path = base_dir.join(entry);
            let text = fs::read_to_string(&anim_path)
                .map_err(|e| ProjectLoadError::Io(format!("read {}: {e}", anim_path.display())))?;
            serde_json::from_str::<AnimationTimeline>(&text).map_err(|e| {
                ProjectLoadError::InvalidManifest(format!("{}: {e}", anim_path.display()))
            })?
        }
        None => AnimationTimeline { layers: vec![] },
    };

    let title = manifest.metadata.title.clone().unwrap_or_else(|| {
        manifest_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_owned()
    });
    let duration = manifest.metadata.duration;

    Ok(Project {
        metadata: ProjectMetadata {
            title,
            artist: manifest.metadata.artist,
            duration,
            format_version: manifest.format_version,
            id: manifest.id.unwrap_or_default(),
            created_with: manifest.generator,
            ..Default::default()
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: audio_path.to_string_lossy().into_owned(),
            },
            duration: duration.unwrap_or(0.0),
            sample_rate: 0,
            channels: 0,
            volume_envelope: None,
        },
        lyrics,
        animation,
        effect_timeline: EffectTimeline::default(),
    })
}

/// Reader rule (§6.3): accept any `1.x`; refuse `≥2.0` with a clear error.
fn check_format_version(version: &str) -> Result<(), ProjectLoadError> {
    let major = version
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok());
    match major {
        Some(m) if m >= 2 => Err(ProjectLoadError::UnsupportedFormat(format!(
            "mva format_version {version} — this player reads 1.x manifests"
        ))),
        _ => Ok(()),
    }
}
