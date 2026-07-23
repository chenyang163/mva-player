//! MVA Player — composition root (Phase 4 M2).
//!
//! Responsibilities:
//! - CLI parsing → [`StartupMode`](crate::cli::StartupMode)
//! - Configuration loading
//! - Subsystem initialisation (audio, engine, loader, renderer)
//! - Startup dispatch → [`startup::boot`]
//! - UI launch via `eframe::run_native`
//!
//! No project-loading logic or UI business rules live here.

mod cli;
mod demo;
mod startup;

use std::sync::{Arc, Mutex};

use mva_audio::AudioPlayer;
use mva_core::config::animation::AnimationConfig;
use mva_core::engine::Engine;
use mva_core::loader::ProjectLoader;
use mva_format::MvaLoader;
use mva_renderer::{Renderer, RendererConfig};
use mva_ui::MvaUiApp;

use crate::cli::Cli;

fn main() {
    let mode = Cli::parse_args().into_startup_mode();

    let (app_cfg, config_warnings) = mva_core::config::loader::load_app_config(None);
    let anim_cfg = AnimationConfig::default();
    let rend_cfg = RendererConfig::default();

    let audio_player =
        AudioPlayer::new().unwrap_or_else(|e| startup::show_error_window_and_exit(&e.to_string()));
    audio_player.set_volume(app_cfg.general.volume);

    let engine = Arc::new(Mutex::new(Engine::new(app_cfg.clone(), anim_cfg)));

    let loader: Arc<dyn ProjectLoader> =
        Arc::new(MvaLoader::new(mva_format::LoaderConfig::default()));

    let autoplay = app_cfg.general.autoplay_on_open;

    let (shared_audio, on_effect) =
        startup::boot(mode, audio_player, &engine, loader.clone(), autoplay);

    let clock: Box<dyn mva_core::PlaybackClock> = Box::new(shared_audio.clone());

    let renderer = Renderer::new(rend_cfg);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([app_cfg.window.width as f32, app_cfg.window.height as f32]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "MVA Player",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(MvaUiApp::new(
                cc,
                engine,
                clock,
                renderer,
                on_effect,
                config_warnings,
            )))
        }),
    ) {
        eprintln!("mva-player fatal error: eframe init failed: {e}");
        std::process::exit(1);
    }
}
