# Changelog

All notable changes to MVA Player are documented in this file.

---

## Unreleased

### MVA Format (Phase 4 start)
- `mva-format`: loose `.mva` JSON manifest reader (architecture §6.2/§6.3) —
  resolves audio/lyrics/animation entries relative to the manifest,
  forward-tolerant parsing, refuses `format_version` majors ≥ 2
- `mva-core`: new `ProjectLoadError::InvalidManifest` variant
- `serde_json` promoted to a runtime dependency of `mva-format`
  (as planned in `docs/dependencies.md`)

### Real-file playback
- `mva-audio`: `SharedAudioPlayer::load_file()` — load an audio file
  through the shared handle (interior mutability for stream metadata)
- `mva-player`: opening a project now loads its audio source into the
  player — real music plays instead of only the startup sine wave

### Demo
- `examples/lyric_demo/`: first MVA showcase — real CC BY 4.0 music
  (Kevin MacLeod), LRC lyric timeline, JSON animation timeline,
  loose `demo.mva` manifest; covered by automated tests
  (`crates/mva-player/tests/demo_showcase.rs`)
- `docs/demo-assets.md`: asset provenance and license records

---

## v0.1.0 — Initial Prototype

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
