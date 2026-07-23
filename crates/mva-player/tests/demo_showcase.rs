//! Showcase demo verification (`examples/lyric_demo`).
//!
//! Proves the core MVA concept end-to-end with real files:
//!
//! 1. `demo.mva` (loose JSON manifest) loads through `MvaLoader`.
//! 2. The referenced MP3 decodes via the rodio/symphonia chain and its
//!    duration matches the manifest metadata.
//! 3. The engine evaluates the loaded project: lyric lines become the
//!    active text layer content at the right times (lyrics sync).

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use mva_core::PlayerCommand;
use mva_core::config::animation::AnimationConfig;
use mva_core::config::app::AppConfig;
use mva_core::effect::{AudioCommand, EngineEffect};
use mva_core::engine::Engine;
use mva_core::loader::ProjectLoader;
use mva_core::state::PlaybackState;
use mva_format::{LoaderConfig, MvaLoader};
use mva_scene::EvaluatedLayerKind;
use rodio::decoder::Decoder;
use rodio::source::Source;

#[test]
fn demo_autoplay_pipeline_produces_play_effect() {
    let project = load_demo_project();

    let mut engine = Engine::new(AppConfig::default(), AnimationConfig::default());

    // Step 1 — LoadProject
    engine
        .handle_command(PlayerCommand::LoadProject(Box::new(project)))
        .unwrap();
    assert_eq!(engine.snapshot().state, PlaybackState::Ready);

    // Step 2 — Play (autoplay simulation)
    let effects = engine.handle_command(PlayerCommand::Play).unwrap();

    // Step 3 — Verify the chain: real project → Engine → Audio(Play) effect
    assert_eq!(engine.snapshot().state, PlaybackState::Playing);
    assert!(
        effects.contains(&EngineEffect::Audio(AudioCommand::Play)),
        "effects from Play must contain Audio(Play): {effects:?}"
    );
}

fn demo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/lyric_demo")
}

fn load_demo_project() -> mva_timeline::model::Project {
    let loader = MvaLoader::new(LoaderConfig::default());
    loader
        .load(&demo_dir().join("demo.mva"))
        .expect("demo.mva must load")
}

#[test]
fn demo_manifest_loads() {
    let project = load_demo_project();

    // Metadata from the manifest.
    assert_eq!(project.metadata.title, "Monkeys Spinning Monkeys");
    assert_eq!(project.metadata.artist.as_deref(), Some("Kevin MacLeod"));
    assert_eq!(project.metadata.format_version, "1.0");

    // Audio entry resolved relative to the manifest directory.
    let mva_timeline::model::AudioSource::ExternalFile { path } = &project.audio.source else {
        panic!("demo project must reference an external audio file");
    };
    assert!(
        path.replace('\\', "/")
            .ends_with("examples/lyric_demo/assets/monkeys-spinning-monkeys.mp3"),
        "audio path must resolve into the demo assets dir, got {path}"
    );

    // Lyrics parsed from lyrics.lrc.
    assert_eq!(project.lyrics.tracks.len(), 1);
    let lines = &project.lyrics.tracks[0].lines;
    assert_eq!(lines.len(), 8, "demo lyric timeline has 8 lines");
    assert!(lines.windows(2).all(|w| w[0].start < w[1].start));

    // One lyric text layer from assets/lyric.anim.json.
    assert_eq!(project.animation.layers.len(), 1);
}

#[test]
fn demo_audio_decodes_and_matches_manifest() {
    let project = load_demo_project();
    let mva_timeline::model::AudioSource::ExternalFile { path } = &project.audio.source else {
        panic!("demo project must reference an external audio file");
    };

    let file = File::open(path).expect("open demo mp3");
    let source = Decoder::try_from(BufReader::new(file)).expect("decode demo mp3");

    // MP3 streams often report no total_duration; decode fully and
    // derive the duration from the sample count (also proves the
    // whole file decodes without errors).
    let sample_rate = f64::from(source.sample_rate().get());
    let channels = f64::from(source.channels().get());
    let decoded = source.count() as f64 / (sample_rate * channels);

    let declared = project
        .metadata
        .duration
        .expect("manifest declares a duration");
    assert!(
        (decoded - declared).abs() < 0.5,
        "manifest duration {declared} must match decoded duration {decoded}"
    );
}

#[test]
fn demo_lyrics_sync_with_engine_clock() {
    let project = load_demo_project();
    let expected: Vec<(f64, String)> = project.lyrics.tracks[0]
        .lines
        .iter()
        .map(|l| (l.start, l.text.clone()))
        .collect();

    let mut engine = Engine::new(AppConfig::default(), AnimationConfig::default());
    engine
        .handle_command(PlayerCommand::LoadProject(Box::new(project)))
        .unwrap();

    // Sample the scene in the middle of every lyric line: the text
    // layer must show that line (AudioClock → evaluate → Scene).
    for (start, text) in &expected {
        let t = start + 1.0;
        engine.update_position(t);
        let snap = engine.snapshot();
        let scene = snap.scene.expect("scene exists with project loaded");
        let EvaluatedLayerKind::Text { text: shown, .. } = &scene.layers[0].kind else {
            panic!("expected Text layer at t={t}");
        };
        assert_eq!(shown, text, "wrong lyric at t={t}");
    }

    // Before the first line: no lyric is active.
    engine.update_position(0.0);
    let snap = engine.snapshot();
    assert_eq!(snap.active_lyric_index, None);

    // The loaded scene renders to a draw command (load → render path).
    engine.update_position(18.0);
    let snap = engine.snapshot();
    let scene = snap.scene.unwrap();
    let renderer = mva_renderer::Renderer::new(mva_renderer::RendererConfig::default());
    let vp = mva_renderer::Viewport {
        width: 1280.0,
        height: 720.0,
    };
    let draw_list = renderer.render(&scene, &vp);
    assert!(
        matches!(
            draw_list.commands.first(),
            Some(mva_renderer::DrawCommand::Text { text, .. }) if text == "One audio file, one shared timeline"
        ),
        "renderer must emit the active lyric as a text command"
    );
}
