# Changelog

All notable changes to MVA Player are documented in this file.

---

## [0.2.0]

### Added

- Application startup workflow with three modes: Empty, Demo, OpenProject
- CLI project loading (`mva-player <path>`) and `--demo` flag
- GUI project loading via File → Open File / Open Folder
- Native file dialog support (rfd)
- Configuration system: `config/app.toml` loading with discovery (exe → cwd → default)
- `autoplay_on_open` configuration option
- Configuration warning reporting (UI panel + stderr)
- `PlaybackError::Unknown` now carries human-readable error text from `ProjectLoadError::Display`
- Known Limitation engine test: open-failure preserves old project and prevents Play

### Improved

- Startup architecture: `main.rs` reduced to thin composition root; startup logic in `startup.rs`
- Unified project loading pipeline (`activate_project` with prepare/activate two-phase boundary)
- UI panel organisation: dedicated menu bar, status bar with config warnings

### Fixed

- Fixed invisible File menu caused by egui panel ordering
- Fixed invisible playback controls caused by panel layout conflict
- Fixed invisible configuration warning area (status bar now uses `Panel::bottom`)

### Known Limitations

- Loose folder loading does not provide accurate duration metadata
- Lyric discovery for loose folders depends on filename matching
- File dialog opens synchronously while engine lock is held (accepted design)
- Configuration is load-once at startup; no runtime reload

---

## [0.1.0] — Initial Prototype

**Released:** 2026-07-22

### Workspace & Core
- Rust workspace architecture with 10 crates
- `mva-core` runtime engine: 7-state state machine, config loading, command/event bus
- `mva-audio`: rodio-based audio transport with `PlaybackClock` implementation

### Timeline System
- `mva-timeline`: data model (`Project`, `Track`, `Keyframe`, `Layer`, `Transform`)
- Pure evaluation engine: binary search, easing (hold / linear / named), interpolation
- `mva-lyrics`: LRC lyric file parser → `LyricTimeline`

### Rendering Pipeline
- `mva-scene`: shared intermediate representation (Scene, Layer, DrawList, EffectIR)
- `mva-renderer`: Scene → DrawList pipeline with layout, z-sort, culling, viewport mapping

### UI
- `mva-ui`: egui/eframe application with 4-panel layout (controls, viewport, settings, info)
- Painter adapter for DrawList rendering

### Effects & Images (Phase 3)
- Image asset pipeline: loading, layout, compositing
- Effect timeline: keyframe-driven visual effects
- Effect draw pipeline: GPU-ready draw commands

### Demo
- `mva-player` binary: full end-to-end demo with synthetic project
- Text rendering, image rendering, effect debug rendering, audio playback
