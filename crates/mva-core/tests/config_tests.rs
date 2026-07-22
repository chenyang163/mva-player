//! Configuration round-trip tests — TOML parsing and defaults.

#![allow(clippy::float_cmp)]

use mva_core::config::animation::AnimationConfig;
use mva_core::config::app::AppConfig;

// ---------------------------------------------------------------------------
// AppConfig
// ---------------------------------------------------------------------------

#[test]
fn app_config_defaults_match_toml_file() {
    let toml = include_str!("../../../config/app.toml");
    let from_file = AppConfig::from_toml(toml).expect("parse app.toml");
    let default = AppConfig::default();
    assert_eq!(from_file, default);
}

#[test]
fn app_config_parses_custom_toml() {
    let toml = r#"
[window]
width = 1920
height = 1080

[general]
language = "zh-CN"
last_directory = "/music"
volume = 0.5
"#;
    let cfg = AppConfig::from_toml(toml).expect("parse");
    assert_eq!(cfg.window.width, 1920);
    assert_eq!(cfg.window.height, 1080);
    assert_eq!(cfg.general.language, "zh-CN");
    assert_eq!(cfg.general.last_directory, "/music");
    assert_eq!(cfg.general.volume, 0.5);
}

#[test]
fn app_config_missing_optional_fields_default() {
    let toml = r#"
[window]
width = 800
height = 600

[general]
language = "en"
volume = 0.9
"#;
    let cfg = AppConfig::from_toml(toml).expect("parse");
    assert_eq!(cfg.general.last_directory, ""); // serde default
}

// ---------------------------------------------------------------------------
// AnimationConfig
// ---------------------------------------------------------------------------

#[test]
fn animation_config_defaults_match_toml_file() {
    let toml = include_str!("../../../config/animation.toml");
    let from_file = AnimationConfig::from_toml(toml).expect("parse animation.toml");
    let default = AnimationConfig::default();
    assert_eq!(from_file, default);
}

#[test]
fn animation_config_parses_custom_toml() {
    let toml = r#"
[lyric_layer]
fade_in_duration = 0.6
fade_out_duration = 0.5
scale_from = 0.8
scale_to = 1.1
default_easing = "ease_in_quad"
"#;
    let cfg = AnimationConfig::from_toml(toml).expect("parse");
    assert!((cfg.lyric_layer.fade_in_duration - 0.6).abs() < 1e-9);
    assert!((cfg.lyric_layer.fade_out_duration - 0.5).abs() < 1e-9);
    assert!((cfg.lyric_layer.scale_from - 0.8).abs() < 1e-6);
    assert!((cfg.lyric_layer.scale_to - 1.1).abs() < 1e-6);
    assert_eq!(cfg.lyric_layer.default_easing, "ease_in_quad");
}

#[test]
fn animation_config_rejects_invalid_easing() {
    let toml = r#"
[lyric_layer]
fade_in_duration = 0.4
fade_out_duration = 0.3
scale_from = 0.9
scale_to = 1.0
default_easing = "not_a_real_easing"
"#;
    let err = AnimationConfig::from_toml(toml).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("unknown easing name"),
        "expected easing validation error, got: {msg}"
    );
}
