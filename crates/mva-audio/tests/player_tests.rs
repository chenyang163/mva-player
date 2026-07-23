//! Audio engine integration tests.
//!
//! Covers:
//! - position increases during playback
//! - pause freezes the clock
//! - stop resets state
//! - missing file returns error
//!
//! Tests gracefully skip when the audio backend is unavailable (e.g.
//! headless CI or machines without a sound card).  CI environments
//! should configure a virtual ALSA device (null PCM) so the real
//! assertions still execute.

#![allow(clippy::float_cmp)]

use std::time::Duration;

use mva_audio::AudioPlayer;
use mva_core::PlaybackClock;
use rodio::source::SineWave;
use rodio::source::Source;

fn try_open_player() -> Option<AudioPlayer> {
    match AudioPlayer::new() {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("Skipping test: cannot open audio device ({e})");
            None
        }
    }
}

fn try_make_player_with_sine() -> Option<AudioPlayer> {
    let player = try_open_player()?;
    let source = SineWave::new(440.0).take_duration(Duration::from_secs_f64(5.0));
    match player.load_source(source) {
        Ok(()) => Some(player),
        Err(e) => {
            eprintln!("Skipping test: cannot load sine source ({e})");
            None
        }
    }
}

#[test]
fn position_increases_during_playback() {
    let Some(player) = try_make_player_with_sine() else {
        return;
    };
    player.play().unwrap();

    // Let the audio thread advance a few frames.
    std::thread::sleep(Duration::from_millis(150));

    let pos = player.position_seconds();
    assert!(
        pos > 0.01,
        "position should advance during playback, got {pos}"
    );
}

#[test]
fn pause_freezes_clock() {
    let Some(player) = try_make_player_with_sine() else {
        return;
    };
    player.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    player.pause().unwrap();

    let pos1 = player.position_seconds();
    std::thread::sleep(Duration::from_millis(100));
    let pos2 = player.position_seconds();

    // Allow a tiny drift, but position must not advance significantly.
    assert!(
        (pos2 - pos1).abs() < 0.02,
        "pause should freeze clock: {pos1} → {pos2}"
    );
}

#[test]
fn pause_then_resume_continues_playback() {
    let Some(player) = try_make_player_with_sine() else {
        return;
    };
    player.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    player.pause().unwrap();
    let paused_pos = player.position_seconds();

    player.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let resumed_pos = player.position_seconds();

    assert!(
        resumed_pos > paused_pos,
        "playback should continue after resume"
    );
}

#[test]
fn stop_resets_position_to_zero() {
    let Some(player) = try_make_player_with_sine() else {
        return;
    };
    player.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    player.stop().unwrap();

    // The PlaybackClock impl returns 0.0 when the engine is Stopped.
    let pos = player.position_seconds();
    assert!(pos < 0.01, "stop should reset position to 0, got {pos}");
}

#[test]
fn missing_file_returns_decode_error() {
    let Some(player) = try_open_player() else {
        return;
    };
    let result = player.load_file("non_existent_audio_file.mp3");
    assert!(result.is_err(), "missing file should return an error");
}

#[test]
fn shared_handle_loads_real_file_and_plays() {
    // Exercises the Phase 4 real-file path used by the player binary:
    // SharedAudioPlayer::load_file() + transport through the handle.
    let mp3 = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/lyric_demo/assets/monkeys-spinning-monkeys.mp3");
    let Some(player) = try_open_player() else {
        return;
    };
    let shared = mva_audio::SharedAudioPlayer::new(player);

    shared
        .load_file(&mp3)
        .expect("load demo mp3 through handle");
    mva_core::audio::AudioController::apply(&shared, mva_core::effect::AudioCommand::Play)
        .expect("play through handle");

    std::thread::sleep(Duration::from_millis(200));
    let pos = mva_core::PlaybackClock::position_seconds(&shared);
    assert!(
        pos > 0.01,
        "position should advance while playing a real file, got {pos}"
    );
}

#[test]
fn play_without_source_returns_error() {
    let Some(player) = try_open_player() else {
        return;
    };
    let result = player.play();
    assert!(result.is_err(), "play without source should fail");
}

#[test]
fn pause_while_stopped_returns_error() {
    let Some(player) = try_make_player_with_sine() else {
        return;
    };
    let err = player.pause().unwrap_err();
    assert!(
        format!("{err}").contains("invalid state"),
        "pause while stopped should return InvalidState"
    );
}

#[test]
fn pause_while_already_paused_returns_error() {
    let Some(player) = try_make_player_with_sine() else {
        return;
    };
    player.play().unwrap();
    player.pause().unwrap();
    let err = player.pause().unwrap_err();
    assert!(
        format!("{err}").contains("invalid state"),
        "double pause should return InvalidState"
    );
}
