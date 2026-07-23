//! Audio engine integration tests.
//!
//! Covers:
//! - position increases during playback
//! - pause freezes the clock
//! - stop resets state
//! - missing file returns error

#![allow(clippy::float_cmp)]

use std::time::Duration;

use mva_audio::AudioPlayer;
use mva_core::PlaybackClock;
use rodio::source::SineWave;
use rodio::source::Source;

fn make_player_with_sine() -> AudioPlayer {
    let player = AudioPlayer::new().expect("open default audio device");
    // 440 Hz sine wave, 5 seconds long, then truncate to avoid
    // sleeping in tests.
    let source = SineWave::new(440.0).take_duration(Duration::from_secs_f64(5.0));
    player.load_source(source).expect("load sine wave");
    player
}

#[test]
fn position_increases_during_playback() {
    let player = make_player_with_sine();
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
    let player = make_player_with_sine();
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
    let player = make_player_with_sine();
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
    let player = make_player_with_sine();
    player.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    player.stop().unwrap();

    // The PlaybackClock impl returns 0.0 when the engine is Stopped.
    let pos = player.position_seconds();
    assert!(pos < 0.01, "stop should reset position to 0, got {pos}");
}

#[test]
fn missing_file_returns_decode_error() {
    let player = AudioPlayer::new().expect("open default audio device");
    let result = player.load_file("non_existent_audio_file.mp3");
    assert!(result.is_err(), "missing file should return an error");
}

#[test]
fn play_without_source_returns_error() {
    let player = AudioPlayer::new().expect("open default audio device");
    let result = player.play();
    assert!(result.is_err(), "play without source should fail");
}

#[test]
fn pause_while_stopped_returns_error() {
    let player = make_player_with_sine();
    let err = player.pause().unwrap_err();
    assert!(
        format!("{err}").contains("invalid state"),
        "pause while stopped should return InvalidState"
    );
}

#[test]
fn pause_while_already_paused_returns_error() {
    let player = make_player_with_sine();
    player.play().unwrap();
    player.pause().unwrap();
    let err = player.pause().unwrap_err();
    assert!(
        format!("{err}").contains("invalid state"),
        "double pause should return InvalidState"
    );
}
