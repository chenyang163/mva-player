//! Phase 4 M3 — startup bootstrap & runtime service.
//!
//! Two logical zones live in this file:
//!
//! ## Bootstrap zone (one-shot)
//!
//! Called once from `main()`: dispatches [`StartupMode`] to load
//! content (or nothing) into the engine and audio subsystems before
//! the UI loop starts.
//!
//! ## Runtime service zone (session-lifetime)
//!
//! [`activate_project`] is called repeatedly during a session (e.g.
//! from the "Open File" button).  Its internal two-phase boundary
//! (prepare / activate) is defined for future async migration.
//! When this file grows large enough, `activate_project` is the first
//! candidate to extract into its own module.
//!
//! # Lock contract for `activate_project`
//!
//! The caller MUST NOT hold the [`Engine`](mva_core::engine::Engine)
//! lock when calling this function.  The **activate** phase
//! acquires the lock internally.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;
use mva_audio::{AudioPlayer, SharedAudioPlayer};
use mva_core::AudioController;
use mva_core::PlayerCommand;
use mva_core::effect::EngineEffect;
use mva_core::engine::Engine;
use mva_core::loader::ProjectLoader;
use mva_core::state::PlaybackError;
use mva_timeline::model::AudioSource;

use crate::cli::StartupMode;
use crate::demo;

// ============================================================================
// Bootstrap zone
// ============================================================================

/// Bootstrap the engine and audio subsystems based on the resolved
/// [`StartupMode`].
///
/// Returns `(SharedAudioPlayer, on_effect_callback)`.  The `SharedAudioPlayer`
/// is used both as the [`PlaybackClock`](mva_core::PlaybackClock) source
/// and the audio controller in the effect dispatch callback.
///
/// `autoplay` controls whether project open triggers automatic playback.
/// `--demo` is exempt from this gating — it always plays.
pub fn boot(
    mode: StartupMode,
    audio_player: AudioPlayer,
    engine: &Arc<Mutex<Engine>>,
    loader: Arc<dyn ProjectLoader>,
    autoplay: bool,
) -> (SharedAudioPlayer, Box<dyn Fn(EngineEffect)>) {
    let shared_audio = match mode {
        StartupMode::Empty => SharedAudioPlayer::new(audio_player),

        StartupMode::Demo => {
            audio_player
                .load_source(demo::make_demo_sine())
                .unwrap_or_else(|e| {
                    show_error_window_and_exit(&format!("failed to load demo audio source: {e}"))
                });
            audio_player.play().unwrap_or_else(|e| {
                show_error_window_and_exit(&format!("failed to start demo playback: {e}"))
            });

            let shared = SharedAudioPlayer::new(audio_player);

            let mut eng = engine.lock().unwrap();
            let project = demo::make_demo_project();
            eng.handle_command(PlayerCommand::LoadProject(Box::new(project)))
                .unwrap_or_else(|e| {
                    show_error_window_and_exit(&format!("failed to load demo project: {e}"))
                });
            let effects = eng.handle_command(PlayerCommand::Play).unwrap_or_else(|e| {
                show_error_window_and_exit(&format!("failed to start demo engine: {e}"))
            });
            drop(eng);

            for eff in effects {
                if let EngineEffect::Audio(cmd) = eff {
                    shared.apply(cmd).unwrap_or_else(|e| {
                        show_error_window_and_exit(&format!(
                            "failed to apply demo audio effect: {e:?}"
                        ))
                    });
                }
            }

            shared
        }

        StartupMode::OpenProject(path) => {
            let shared = SharedAudioPlayer::new(audio_player);
            activate_project(path, engine, &shared, &*loader, autoplay);
            shared
        }
    };

    let on_effect = build_on_effect(engine.clone(), shared_audio.clone(), loader, autoplay);
    (shared_audio, on_effect)
}

// ============================================================================
// Runtime service zone
// ============================================================================

/// Shared project-loading pipeline — the single entry point for all
/// real-file loading (CLI startup + UI Open).
///
/// ## Two-phase boundary
///
/// | Phase    | Content                                       | Engine lock |
/// |----------|-----------------------------------------------|-------------|
/// | prepare  | `loader.load(&path)` — I/O + parse            | no          |
/// | activate | audio switch → `LoadProject` → optional `Play` | yes (internal) |
///
/// ## Lock contract
///
/// The caller MUST NOT hold the Engine lock when calling this
/// function.  The activate phase acquires it internally.
///
/// ## Embedded audio support
///
/// [`AudioSource::Embedded`] skips `load_file` and proceeds directly
/// to `LoadProject`.  Full ZIP‑container support is deferred.
pub fn activate_project(
    path: PathBuf,
    engine: &Arc<Mutex<Engine>>,
    shared_audio: &SharedAudioPlayer,
    loader: &dyn ProjectLoader,
    autoplay: bool,
) {
    // ── prepare ──────────────────────────────────────────────────
    let project = match loader.load(&path) {
        Ok(p) => p,
        Err(e) => {
            // ProjectLoadError::Display → PlaybackError::Unknown(String)
            // (depends on M1 impl Display for ProjectLoadError)
            let mut eng = engine.lock().unwrap();
            eng.set_error(PlaybackError::Unknown(e.to_string()));
            return;
        }
    };

    // ── activate: audio switch ───────────────────────────────────
    // extract ExternalFile path BEFORE acquiring engine lock
    let audio_path = match &project.audio.source {
        AudioSource::ExternalFile { path } => Some(path.clone()),
        // Embedded data (future ZIP container) — no file-based loading
        AudioSource::Embedded { .. } => {
            // TODO: support for embedded audio in ZIP container
            None
        }
    };

    if let Some(ref audio_path) = audio_path {
        if let Err(e) = shared_audio.load_file(audio_path) {
            eprintln!("mva-player: audio load failed: {e:?}");
            let mut eng = engine.lock().unwrap();
            eng.set_error(PlaybackError::DecodeFailed);
            return;
        }
    }

    // ── activate: engine ─────────────────────────────────────────
    let play_effects = {
        let mut eng = engine.lock().unwrap();
        let _ = eng.handle_command(PlayerCommand::LoadProject(Box::new(project)));

        if autoplay {
            eng.handle_command(PlayerCommand::Play)
                .expect("Play after LoadProject should never fail")
        } else {
            Vec::new()
        }
    }; // engine lock dropped

    // ── dispatch audio effects outside lock ──────────────────────
    for eff in play_effects {
        if let EngineEffect::Audio(cmd) = eff {
            if let Err(e) = shared_audio.apply(cmd) {
                eprintln!("mva-player: autoplay audio failed: {e:?}");
            }
        }
    }
}

// ── Effect dispatch callback ──────────────────────────────────────

fn build_on_effect(
    engine: Arc<Mutex<Engine>>,
    shared_audio: SharedAudioPlayer,
    loader: Arc<dyn ProjectLoader>,
    autoplay: bool,
) -> Box<dyn Fn(EngineEffect)> {
    Box::new(move |effect| match effect {
        EngineEffect::Audio(cmd) => {
            let _ = shared_audio.apply(cmd);
        }
        EngineEffect::LoadProject { path } => {
            activate_project(path, &engine, &shared_audio, &*loader, autoplay);
        }
    })
}

// ============================================================================
// Fatal error window (audio device unavailable)
// ============================================================================

struct ErrorWindow {
    msg: String,
}

impl eframe::App for ErrorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("MVA Player — Fatal Error");
            ui.separator();
            ui.label(&self.msg);
            ui.separator();
            if ui.button("Exit").clicked() {
                std::process::exit(1);
            }
        });
    }
}

/// Show a minimal error window using the existing eframe dependency,
/// then exit with code 1.
///
/// The error message is also written to stderr (visible in console
/// / terminal launch scenarios).  If the window system itself is
/// unavailable (`eframe::run_native` fails), the process still exits
/// with code 1 after the stderr output.
pub fn show_error_window_and_exit(msg: &str) -> ! {
    eprintln!("mva-player fatal error: {msg}");
    let msg_owned = msg.to_string();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 200.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "MVA Player — Fatal Error",
        options,
        Box::new(move |_cc| Ok(Box::new(ErrorWindow { msg: msg_owned }))),
    );
    std::process::exit(1);
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mva_format::MvaLoader;

    #[test]
    fn prepare_nonexistent_path() {
        let path = PathBuf::from("__nonexistent__/not_a_project.mva");
        let loader = MvaLoader::new(mva_format::LoaderConfig::default());
        let result = loader.load(&path);
        assert!(result.is_err(), "nonexistent path should fail");
        let err_text = result.unwrap_err().to_string();
        assert!(
            !err_text.is_empty(),
            "error text should be non-empty, got: {err_text:?}"
        );
    }

    #[test]
    fn prepare_invalid_manifest_text() {
        // Point at a directory that exists but has no valid project content.
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let loader = MvaLoader::new(mva_format::LoaderConfig::default());
        let result = loader.load(&path);
        assert!(result.is_err(), "crate root should fail to load as project");
        let err_text = result.unwrap_err().to_string();
        assert!(
            !err_text.is_empty(),
            "error text should be non-empty, got: {err_text:?}"
        );
    }
}
