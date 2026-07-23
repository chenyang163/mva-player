//! Loose‑file loader: detects directory layout or single audio file
//! and constructs a [`Project`].

use std::fs;
use std::path::Path;

use mva_core::loader::{ProjectLoadError, ProjectLoader};
use mva_timeline::model::{
    AnimationTimeline, AudioSource, AudioTimeline, Project, ProjectMetadata,
};
use mva_types::{EffectTimeline, LyricTimeline};

use crate::manifest::load_manifest;

/// Configuration for [`MvaLoader`] (Phase 2 — defaults only).
#[derive(Debug, Clone, Default)]
pub struct LoaderConfig {}

/// The format‑engine implementation.
pub struct MvaLoader {
    _config: LoaderConfig,
}

impl MvaLoader {
    /// Create a new loader with default configuration.
    pub fn new(config: LoaderConfig) -> Self {
        Self { _config: config }
    }
}

impl ProjectLoader for MvaLoader {
    fn load(&self, path: &Path) -> Result<Project, ProjectLoadError> {
        if path.is_dir() {
            self.load_from_dir(path)
        } else if path.is_file() {
            self.load_from_file(path)
        } else {
            Err(ProjectLoadError::Io(format!(
                "not found: {}",
                path.display()
            )))
        }
    }

    fn supported_extensions(&self) -> &[&str] {
        &["mva", "mp3", "flac", "wav", "lrc"]
    }
}

impl MvaLoader {
    fn load_from_dir(&self, dir: &Path) -> Result<Project, ProjectLoadError> {
        // Find the first audio file.
        let audio_path = find_audio_file(dir)?;
        // Derive LRC path (same stem, .lrc extension).
        let lrc_path = audio_path.with_extension("lrc");

        let lyrics = if lrc_path.exists() {
            let content = fs::read_to_string(&lrc_path)
                .map_err(|e| ProjectLoadError::Io(format!("read {}: {e}", lrc_path.display())))?;
            mva_lyrics::parse_lrc(&content).map_err(|e| {
                ProjectLoadError::InvalidLyrics(format!("{}: {e}", lrc_path.display()))
            })?
        } else {
            LyricTimeline { tracks: vec![] }
        };

        build_project(&audio_path, lyrics, dir)
    }

    fn load_from_file(&self, file: &Path) -> Result<Project, ProjectLoadError> {
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // A `.mva` file is a loose JSON manifest (architecture §6.2).
        if ext == "mva" {
            return load_manifest(file);
        }

        if !matches!(ext.as_str(), "mp3" | "flac" | "wav") {
            return Err(ProjectLoadError::UnsupportedFormat(format!(
                ".{ext} — expected mva, mp3, flac, or wav"
            )));
        }

        build_project(file, LyricTimeline { tracks: vec![] }, file)
    }
}

fn find_audio_file(dir: &Path) -> Result<std::path::PathBuf, ProjectLoadError> {
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if matches!(ext.to_lowercase().as_str(), "mp3" | "flac" | "wav") {
                return Ok(path.to_owned());
            }
        }
    }
    Err(ProjectLoadError::NoAudioFile)
}

fn build_project(
    audio_path: &Path,
    lyrics: LyricTimeline,
    _source_path: &Path,
) -> Result<Project, ProjectLoadError> {
    let stem = audio_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    Ok(Project {
        metadata: ProjectMetadata {
            title: stem.to_owned(),
            format_version: "1.0".into(),
            id: String::new(),
            ..Default::default()
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: audio_path.to_string_lossy().into_owned(),
            },
            duration: 0.0,
            sample_rate: 0,
            channels: 0,
            volume_envelope: None,
        },
        lyrics,
        animation: AnimationTimeline { layers: vec![] },
        effect_timeline: EffectTimeline::default(),
    })
}
