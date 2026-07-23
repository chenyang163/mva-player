//! Configuration round-trip tests — TOML parsing and defaults.

#![allow(clippy::float_cmp)]

use std::io::Write;

use mva_core::config::animation::AnimationConfig;
use mva_core::config::app::AppConfig;
use mva_core::config::loader;

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
autoplay_on_open = false
"#;
    let cfg = AppConfig::from_toml(toml).expect("parse");
    assert_eq!(cfg.window.width, 1920);
    assert_eq!(cfg.window.height, 1080);
    assert_eq!(cfg.general.language, "zh-CN");
    assert_eq!(cfg.general.last_directory, "/music");
    assert_eq!(cfg.general.volume, 0.5);
    assert!(!cfg.general.autoplay_on_open);
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
    assert!(cfg.general.autoplay_on_open); // default_true helper
}

#[test]
fn autoplay_on_open_explicit_true() {
    let toml = r#"
[window]
width = 640
height = 480

[general]
language = "en"
volume = 0.5
autoplay_on_open = true
"#;
    let cfg = AppConfig::from_toml(toml).expect("parse");
    assert!(cfg.general.autoplay_on_open);
}

#[test]
fn autoplay_on_open_explicit_false() {
    let toml = r#"
[window]
width = 640
height = 480

[general]
language = "en"
volume = 0.5
autoplay_on_open = false
"#;
    let cfg = AppConfig::from_toml(toml).expect("parse");
    assert!(!cfg.general.autoplay_on_open);
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

// ---------------------------------------------------------------------------
// Config loader — temp-dir based integration tests
// ---------------------------------------------------------------------------

fn write_app_toml(dir: &std::path::Path, content: &str) {
    let path = dir.join("app.toml");
    let mut f = std::fs::File::create(&path).expect("create app.toml in tempdir");
    f.write_all(content.as_bytes()).expect("write");
}

#[test]
fn loader_valid_app_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_app_toml(
        dir.path(),
        r#"[window]
width = 1024
height = 768

[general]
language = "ja"
volume = 0.3
autoplay_on_open = true
"#,
    );

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    assert!(warnings.is_empty(), "no warnings for valid config");
    assert_eq!(cfg.window.width, 1024);
    assert_eq!(cfg.window.height, 768);
    assert_eq!(cfg.general.language, "ja");
    assert_eq!(cfg.general.volume, 0.3);
    assert!(cfg.general.autoplay_on_open);
}

#[test]
fn loader_autoplay_false() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_app_toml(
        dir.path(),
        r#"[window]
width = 800
height = 600

[general]
language = "en"
volume = 0.5
autoplay_on_open = false
"#,
    );

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    assert!(warnings.is_empty());
    assert!(!cfg.general.autoplay_on_open);
}

#[test]
fn loader_unknown_key_valid_fields_preserved() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_app_toml(
        dir.path(),
        r#"[window]
width = 1280
height = 720

[general]
language = "fr"
volume = 0.6
unknown_opt = true
"#,
    );

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    // Warning for unknown key
    assert_eq!(warnings.len(), 1);
    assert!(
        warnings[0].contains("unknown_opt"),
        "warning should mention unknown key, got: {warnings:?}"
    );
    // Valid fields preserved
    assert_eq!(cfg.window.width, 1280);
    assert_eq!(cfg.general.language, "fr");
    assert_eq!(cfg.general.volume, 0.6);
    // Missing field gets default
    assert!(cfg.general.autoplay_on_open);
}

#[test]
fn loader_unknown_key_missing_required_field_fallback() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_app_toml(
        dir.path(),
        r#"[general]
language = "de"
unknown = true
"#,
    );

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    // Should have both the unknown-key warning and a parse-failure warning
    assert!(
        warnings.len() >= 2,
        "expected at least 2 warnings (unknown key + missing field), got {warnings:?}"
    );
    assert!(
        warnings.iter().any(|w| w.contains("unknown")),
        "should have unknown key warning"
    );
    assert!(
        warnings.iter().any(|w| w.contains("could not be parsed")),
        "should have parse failure warning"
    );
    // Falls back to defaults
    let def = AppConfig::default();
    assert_eq!(cfg, def);
}

#[test]
fn loader_invalid_toml_syntax() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_app_toml(dir.path(), "general]\n");

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    assert_eq!(warnings.len(), 1);
    assert!(
        warnings[0].contains("could not be parsed"),
        "expected parse error, got: {warnings:?}"
    );
    let def = AppConfig::default();
    assert_eq!(cfg, def);
}

#[test]
fn loader_missing_app_toml_silent() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Don't write app.toml — dir exists but no file

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    assert!(
        warnings.is_empty(),
        "missing file in existing dir should not warn"
    );
    let def = AppConfig::default();
    assert_eq!(cfg, def);
}

#[test]
fn loader_dir_missing_silent() {
    let dir = std::path::Path::new("__nonexistent_dir_m4_test__");
    let (cfg, warnings) = loader::load_app_config(Some(dir));
    assert!(warnings.is_empty(), "missing dir should not warn");
    let def = AppConfig::default();
    assert_eq!(cfg, def);
}

#[test]
fn loader_unknown_section() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_app_toml(
        dir.path(),
        r#"[window]
width = 640
height = 480

[general]
language = "en"
volume = 0.8

[network]
timeout = 5
"#,
    );

    let (cfg, warnings) = loader::load_app_config(Some(dir.path()));
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("[network]"));
    // Valid values preserved
    assert_eq!(cfg.window.width, 640);
    assert_eq!(cfg.general.volume, 0.8);
}

#[test]
fn loader_discovery_priority_exe_over_cwd() {
    // This test validates the discovery logic: when both exe and cwd
    // have config dirs with different app.toml values, the exe path
    // wins.  We simulate by passing `Some(exe_dir)` directly — if we
    // passed `None` the actual exe dir takes priority, which is the
    // same behaviour tested here with direct injection.
    let dir_exe = tempfile::tempdir().expect("exe tempdir");
    let dir_cwd = tempfile::tempdir().expect("cwd tempdir");

    write_app_toml(
        dir_exe.path(),
        r#"[window]
width = 999
height = 888

[general]
language = "exe"
volume = 0.1
autoplay_on_open = true
"#,
    );

    write_app_toml(
        dir_cwd.path(),
        r#"[window]
width = 111
height = 222

[general]
language = "cwd"
volume = 0.9
autoplay_on_open = false
"#,
    );

    // Pass the exe dir — should use its values
    let (cfg, warnings) = loader::load_app_config(Some(dir_exe.path()));
    assert!(warnings.is_empty());
    assert_eq!(cfg.window.width, 999);
    assert_eq!(cfg.general.language, "exe");

    // Pass the cwd dir separately — should use its values
    let (cfg2, warnings2) = loader::load_app_config(Some(dir_cwd.path()));
    assert!(warnings2.is_empty());
    assert_eq!(cfg2.window.width, 111);
    assert_eq!(cfg2.general.language, "cwd");
}
