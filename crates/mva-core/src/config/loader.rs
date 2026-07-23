//! Configuration loader for `app.toml` (Phase 4 M4).
//!
//! Two-pass parsing:
//! - **Pass A**: parse raw `toml::Value` and check for unknown keys
//!   against a known-key whitelist.
//! - **Pass B**: deserialize with serde (`AppConfig::from_toml`).
//!   Serde ignores unknown fields by default, so Pass A handles
//!   detection while Pass B retains valid fields.
//!
//! Configuration is **best-effort**: missing / unreadable / unparseable
//! files never prevent startup.  Fallback is always `AppConfig::default()`
//! with warnings collected in a `Vec<String>`.
//!
//! # Discovery order (`config_dir: None`)
//!
//! 1. `<exe_dir>/config/app.toml`
//! 2. `<cwd>/config/app.toml`
//! 3. built-in defaults

use std::path::{Path, PathBuf};

use crate::config::app::AppConfig;

// ---------------------------------------------------------------------------
// Known-key whitelists (keep in sync with AppConfig / WindowConfig / GeneralConfig)
// ---------------------------------------------------------------------------

/// Known **top-level** sections in `app.toml`.
const KNOWN_APP_SECTIONS: &[&str] = &["window", "general"];

/// Known keys inside `[window]`.
const KNOWN_WINDOW_KEYS: &[&str] = &["width", "height"];

/// Known keys inside `[general]`.
const KNOWN_GENERAL_KEYS: &[&str] = &["language", "last_directory", "volume", "autoplay_on_open"];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load an `AppConfig` from disk, returning warnings for anything
/// that could not be honoured.
///
/// - `Some(path)`: read `path/app.toml` directly (used by tests).
/// - `None`: auto-discovery via `find_config_dir()`.
///
/// ## Behaviour summary
///
/// | Scenario                          | AppConfig          | Warnings |
/// |-----------------------------------|--------------------|----------|
/// | file found, fully valid           | file values        | empty    |
/// | file found, unknown keys present  | file values        | 1 per unknown key |
/// | file found, broken TOML / missing field | default      | 1        |
/// | file not found / dir missing      | default            | empty    |
pub fn load_app_config(config_dir: Option<&Path>) -> (AppConfig, Vec<String>) {
    let dir = match config_dir {
        Some(d) => Some(PathBuf::from(d)),
        None => find_config_dir(),
    };

    let mut warnings: Vec<String> = Vec::new();

    match dir {
        Some(ref d) => {
            let toml_path = d.join("app.toml");
            if toml_path.is_file() {
                match std::fs::read_to_string(&toml_path) {
                    Ok(content) => parse_with_warnings(&content, &mut warnings),
                    Err(e) => {
                        push_warning(
                            &mut warnings,
                            &format!("config/app.toml could not be read: {e}. Using defaults."),
                        );
                        (AppConfig::default(), warnings)
                    }
                }
            } else {
                // config dir exists but no app.toml — first run, silent
                (AppConfig::default(), warnings)
            }
        }
        None => (AppConfig::default(), warnings),
    }
}

// ---------------------------------------------------------------------------
// Internal: discovery
// ---------------------------------------------------------------------------

fn find_config_dir() -> Option<PathBuf> {
    // 1. exe-relative config/
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let cfg = exe_dir.join("config");
            if cfg.join("app.toml").is_file() {
                return Some(cfg);
            }
        }
    }

    // 2. cwd-relative config/
    if let Ok(cwd) = std::env::current_dir() {
        let cfg = cwd.join("config");
        if cfg.join("app.toml").is_file() {
            return Some(cfg);
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Internal: dual-pass parse
// ---------------------------------------------------------------------------

fn parse_with_warnings(content: &str, warnings: &mut Vec<String>) -> (AppConfig, Vec<String>) {
    // ── Pass A: unknown-key scan ─────────────────────────────────
    check_unknown_keys(content, warnings);

    // ── Pass B: serde deserialisation ────────────────────────────
    match AppConfig::from_toml(content) {
        Ok(cfg) => (cfg, std::mem::take(warnings)),
        Err(e) => {
            push_warning(
                warnings,
                &format!("config/app.toml could not be parsed: {e}. Using defaults."),
            );
            (AppConfig::default(), std::mem::take(warnings))
        }
    }
}

fn check_unknown_keys(content: &str, warnings: &mut Vec<String>) {
    let root: toml::Value = match toml::from_str(content) {
        Ok(v) => v,
        Err(_) => return, // Pass B will handle the parse error
    };

    let Some(table) = root.as_table() else { return };

    // Check top-level sections
    for (section_key, section_value) in table {
        if !KNOWN_APP_SECTIONS.contains(&section_key.as_str()) {
            push_warning(
                warnings,
                &format!("unknown config section: [{section_key}]"),
            );
            continue;
        }

        // Check keys inside known sections
        let known_keys = match section_key.as_str() {
            "window" => KNOWN_WINDOW_KEYS,
            "general" => KNOWN_GENERAL_KEYS,
            _ => continue,
        };

        if let Some(section_table) = section_value.as_table() {
            for key in section_table.keys() {
                if !known_keys.contains(&key.as_str()) {
                    push_warning(
                        warnings,
                        &format!("unknown config key in [{section_key}]: {key}"),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn push_warning(warnings: &mut Vec<String>, msg: &str) {
    eprintln!("[mva-player] config: {msg}");
    warnings.push(msg.to_string());
}
