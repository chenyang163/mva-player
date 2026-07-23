//! CLI argument parsing for `mva-player` (Phase 4 M1).
//!
//! Uses clap 4 derive to produce a [`StartupMode`] from command-line
//! arguments.  All path handling uses [`PathBuf`] (OsStr‑based) for
//! Windows non‑UTF‑8 compatibility — no `String` conversions.

use std::path::PathBuf;

use clap::Parser;

/// The resolved startup mode after CLI parsing.
///
/// Three variants map to the three launch paths:
/// - `Empty` — idle player window
/// - `Demo` — built‑in Phase 3 showcase
/// - `OpenProject` — load from disk
#[derive(Debug, PartialEq, Eq)]
pub enum StartupMode {
    /// No arguments: open an idle player window.
    Empty,
    /// `--demo`: play the built-in showcase project.
    Demo,
    /// `<PATH>`: open the specified project / audio file / folder.
    OpenProject(PathBuf),
}

/// CLI surface for `mva-player` (Phase 4, §4.1).
///
/// ```text
/// mva-player                         → StartupMode::Empty
/// mva-player --demo                  → StartupMode::Demo
/// mva-player path/to/project.mva     → StartupMode::OpenProject
/// ```
///
/// `--demo` and a positional path are mutually exclusive; clap
/// produces a parse error when both are supplied.
#[derive(Parser)]
#[command(name = "mva-player", about = "Music Visual Animation Player", version)]
pub struct Cli {
    /// Play the built-in showcase project (synthetic audio).
    #[arg(long, conflicts_with = "path")]
    pub demo: bool,

    /// Project to open: .mva manifest, audio file, or loose project folder.
    #[arg(value_parser)]
    pub path: Option<PathBuf>,
}

impl Cli {
    /// Parse `std::env::args_os()` and convert into a [`StartupMode`].
    ///
    /// This is the primary entry-point called from `main()`.
    /// On parse failure clap prints a usage message and the process
    /// exits with code 2 (standard CLAP behaviour for CLI misuse).
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Consume `self` and produce the resolved [`StartupMode`].
    pub fn into_startup_mode(self) -> StartupMode {
        match (self.demo, self.path) {
            (true, _) => StartupMode::Demo,
            (false, Some(p)) => StartupMode::OpenProject(p),
            (false, None) => StartupMode::Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let cli = Cli::try_parse_from(["mva-player"]).unwrap();
        assert_eq!(cli.into_startup_mode(), StartupMode::Empty);
    }

    #[test]
    fn demo_flag() {
        let cli = Cli::try_parse_from(["mva-player", "--demo"]).unwrap();
        assert_eq!(cli.into_startup_mode(), StartupMode::Demo);
    }

    #[test]
    fn open_project_path() {
        let cli = Cli::try_parse_from(["mva-player", "demo.mva"]).unwrap();
        assert_eq!(
            cli.into_startup_mode(),
            StartupMode::OpenProject(PathBuf::from("demo.mva"))
        );
    }

    #[test]
    fn demo_and_path_conflict() {
        let result = Cli::try_parse_from(["mva-player", "--demo", "demo.mva"]);
        assert!(
            result.is_err(),
            "--demo and a positional path must conflict"
        );
    }

    #[test]
    fn unknown_flag() {
        let result = Cli::try_parse_from(["mva-player", "--invalid"]);
        assert!(result.is_err(), "unknown flag must produce an error");
    }

    #[test]
    fn empty_path_after_double_dash() {
        let cli = Cli::try_parse_from(["mva-player", "--"]).unwrap();
        assert_eq!(cli.into_startup_mode(), StartupMode::Empty);
    }
}
