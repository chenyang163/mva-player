//! MVA Player — binary shell (Phase 3 demo).
//!
//! Composition root: loads the Phase 3 demo project at startup
//! (Text + Image + Effect layers), starts audio playback, wires
//! the effect dispatch callback, and launches the UI.

mod demo;

use std::sync::{Arc, Mutex};

use mva_audio::{AudioPlayer, SharedAudioPlayer};
use mva_core::AudioController;
use mva_core::PlayerCommand;
use mva_core::effect::EngineEffect;
use mva_core::engine::Engine;
use mva_core::loader::ProjectLoader;
use mva_format::MvaLoader;
use mva_renderer::{Renderer, RendererConfig};
use mva_ui::MvaUiApp;

use mva_core::config::animation::AnimationConfig;
use mva_core::config::app::AppConfig;

fn main() {
    let app_cfg = AppConfig::default();
    let anim_cfg = AnimationConfig::default();
    let rend_cfg = RendererConfig::default();

    // --- audio (sine wave demo source) ---
    let audio_player = AudioPlayer::new().expect("open default audio device");
    audio_player.set_volume(app_cfg.general.volume);
    audio_player
        .load_source(demo::make_demo_sine())
        .expect("load sine");
    audio_player.play().expect("start audio");
    let shared_audio = SharedAudioPlayer::new(audio_player);
    let clock: Box<dyn mva_core::PlaybackClock> = Box::new(shared_audio.clone());

    // --- engine + demo project ---
    let engine = Arc::new(Mutex::new(Engine::new(app_cfg.clone(), anim_cfg)));
    {
        let mut eng = engine.lock().unwrap();
        let project = demo::make_demo_project();
        eng.handle_command(PlayerCommand::LoadProject(Box::new(project)))
            .expect("load demo project");
        let effects = eng
            .handle_command(PlayerCommand::Play)
            .expect("start engine");
        // Dispatch the Play effect immediately (before eframe).
        for eff in effects {
            match eff {
                EngineEffect::Audio(cmd) => {
                    shared_audio.apply(cmd).expect("audio play");
                }
                EngineEffect::LoadProject { .. } => {}
            }
        }
    }

    // --- format loader (for user file opening) ---
    let loader: Arc<dyn ProjectLoader> =
        Arc::new(MvaLoader::new(mva_format::LoaderConfig::default()));

    // --- renderer ---
    let renderer = Renderer::new(rend_cfg);

    // --- effect dispatch callback ---
    let ac = shared_audio.clone();
    let engine_clone = engine.clone();
    let loader_clone = loader.clone();
    let on_effect: Box<dyn Fn(EngineEffect)> = Box::new(move |effect| match effect {
        EngineEffect::Audio(cmd) => {
            let _ = ac.apply(cmd);
        }
        EngineEffect::LoadProject { path } => match loader_clone.load(&path) {
            Ok(project) => {
                use mva_core::state::PlaybackError;
                // Real-file playback: load the project's audio source
                // into the player before the project goes live.
                if let mva_timeline::model::AudioSource::ExternalFile { path } =
                    &project.audio.source
                {
                    if let Err(_e) = ac.load_file(path) {
                        let mut eng = engine_clone.lock().unwrap();
                        eng.set_error(PlaybackError::DecodeFailed);
                        return;
                    }
                }
                let mut eng = engine_clone.lock().unwrap();
                let _ = eng.handle_command(PlayerCommand::LoadProject(Box::new(project)));
            }
            Err(_e) => {
                let mut eng = engine_clone.lock().unwrap();
                use mva_core::state::PlaybackError;
                eng.set_error(PlaybackError::ProjectLoadFailed);
            }
        },
    });

    // --- UI ---
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([app_cfg.window.width as f32, app_cfg.window.height as f32]),
        ..Default::default()
    };

    eframe::run_native(
        "MVA Player — Phase 3 Demo",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(MvaUiApp::new(
                cc, engine, clock, renderer, on_effect,
            )))
        }),
    )
    .expect("eframe failed");
}
