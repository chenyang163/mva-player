//! Loose `.mva` manifest loader tests (architecture §6.2, §6.3).
//!
//! Uses synthetic loose projects in a per-test temp directory — no
//! real media decoding happens in the loader (pure data + I/O).

use std::fs;
use std::path::PathBuf;

use mva_core::loader::{ProjectLoadError, ProjectLoader};
use mva_format::{LoaderConfig, MvaLoader};

/// Create a unique temp dir for a test (cleaned up by the caller).
fn temp_project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mva-manifest-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_project(dir: &std::path::Path, manifest: &str) {
    fs::write(dir.join("test.mva"), manifest).unwrap();
    fs::write(dir.join("song.mp3"), b"not real audio").unwrap();
    fs::write(
        dir.join("lyrics.lrc"),
        "[00:01.00]first line\n[00:05.00]second line\n",
    )
    .unwrap();
}

fn loader() -> MvaLoader {
    MvaLoader::new(LoaderConfig::default())
}

#[test]
fn loads_minimal_manifest() {
    let dir = temp_project("minimal");
    write_project(
        &dir,
        r#"{
            "format_version": "1.0",
            "id": "unit-test",
            "metadata": { "title": "Unit", "artist": "Test", "duration": 42.0 },
            "entries": { "audio": "song.mp3", "lyrics": ["lyrics.lrc"] }
        }"#,
    );

    let project = loader().load(&dir.join("test.mva")).unwrap();
    assert_eq!(project.metadata.title, "Unit");
    assert_eq!(project.metadata.artist.as_deref(), Some("Test"));
    assert_eq!(project.metadata.duration, Some(42.0));
    assert_eq!(project.metadata.id, "unit-test");
    assert!((project.audio.duration - 42.0).abs() < 1e-9);
    assert_eq!(project.lyrics.tracks.len(), 1);
    assert_eq!(project.lyrics.tracks[0].lines.len(), 2);
    assert!((project.lyrics.tracks[0].lines[0].start - 1.0).abs() < 1e-9);
    assert!(project.animation.layers.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn resolves_entries_relative_to_manifest_dir() {
    let dir = temp_project("relative");
    fs::create_dir_all(dir.join("assets")).unwrap();
    write_project(
        &dir,
        r#"{
            "entries": { "audio": "assets/song.mp3" }
        }"#,
    );
    // Move the audio into assets/ for this layout.
    fs::rename(dir.join("song.mp3"), dir.join("assets/song.mp3")).unwrap();

    let project = loader().load(&dir.join("test.mva")).unwrap();
    let mva_timeline::model::AudioSource::ExternalFile { path } = &project.audio.source else {
        panic!("manifest audio entry must become an external file source");
    };
    let norm = path.replace('\\', "/");
    assert!(
        norm.ends_with("assets/song.mp3"),
        "audio entry must resolve relative to the manifest, got {norm}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn loads_animation_entry() {
    let dir = temp_project("animation");
    write_project(
        &dir,
        r#"{
            "entries": {
                "audio": "song.mp3",
                "animation": "scene.anim.json"
            }
        }"#,
    );
    fs::write(
        dir.join("scene.anim.json"),
        r#"{
            "layers": [{
                "id": "lyric",
                "kind": {
                    "type": "text",
                    "source": { "type": "lyric_line" },
                    "style": { "font_size": 36.0, "color": [255, 255, 255, 255] }
                },
                "visible_range": [0.0, 10.0]
            }]
        }"#,
    )
    .unwrap();

    let project = loader().load(&dir.join("test.mva")).unwrap();
    assert_eq!(project.animation.layers.len(), 1);
    assert_eq!(project.animation.layers[0].id.0, "lyric");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rejects_future_major_version() {
    let dir = temp_project("version");
    write_project(
        &dir,
        r#"{ "format_version": "2.0", "entries": { "audio": "song.mp3" } }"#,
    );

    let err = loader().load(&dir.join("test.mva")).unwrap_err();
    assert!(
        matches!(err, ProjectLoadError::UnsupportedFormat(_)),
        "format_version 2.x must be refused, got {err:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rejects_invalid_json() {
    let dir = temp_project("bad-json");
    write_project(&dir, "{ not json");

    let err = loader().load(&dir.join("test.mva")).unwrap_err();
    assert!(
        matches!(err, ProjectLoadError::InvalidManifest(_)),
        "invalid JSON must be an InvalidManifest error, got {err:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rejects_missing_audio_entry() {
    let dir = temp_project("missing-audio");
    write_project(&dir, r#"{ "entries": { "audio": "gone.mp3" } }"#);

    let err = loader().load(&dir.join("test.mva")).unwrap_err();
    assert!(
        matches!(err, ProjectLoadError::Io(_)),
        "missing audio entry must be an Io error, got {err:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn supported_extensions_include_mva() {
    assert!(loader().supported_extensions().contains(&"mva"));
}
