# MVA Player — Development Roadmap

Tracking implementation status against the architecture phase plan (§11).
See `docs/architecture.md` §11 for the full table.

## Status

| Phase | Status | Deliverable |
|---|---|---|
| **1.1** | ✅ Done | Timeline data model (`mva-timeline::model`): `Project`, `AudioTimeline`, `LyricTimeline`, `AnimationTimeline`, `Layer`/`LayerKind::Text`, `Track`/`Keyframe`/`Easing`(Hold/Linear/Named), `Transform`, `Vec2` — plain serde types per §4/§5/§3.4. 12 contract tests passing. |
| **1.2** | ✅ Done | Timeline evaluation (`mva-timeline::eval`): binary search + easing + lerp, lyric lookup, `evaluate(t) -> Scene` — pure, deterministic. 23 eval + 12 model tests. |
| **1.3** | ✅ Done | `mva-core` runtime: `AppConfig`/`AnimationConfig`, `PlayerCommand`, `PlaybackState`, `EngineSnapshot` (with scene eval), `CoreError`, `PlaybackClock` trait, `Engine` state machine. |
| **1.4** | ✅ Done | `mva-audio` engine: rodio 0.22.2 transport, `PlaybackClock` impl (state‑aware), `AudioError`. |
| **1.5** | ✅ Done | `mva-scene` leaf crate (shared IR); `mva-renderer`: Scene→DrawList pipeline (text, z‑sort, cull, layout), `RendererConfig`. |
| **1.6** | ✅ Done | `mva-ui` (MvaUiApp, 4 panels, painter adapter) + `mva-player` binary shell (wiring only). 90 total tests. |
| **1.7** | ✅ Done | Integration demo: synthetic test project + sine‑wave audio at startup. Full pipeline validated end‑to‑end. 92 tests (90 + 2 integration). |
| **1.7‑FU** | ✅ Done | Follow‑up: demo bootstrap annotated, integration pipeline automated test, playback‑end + volume sync documented as Phase 2 items. |

## Phase 2 — Final Design Approved

See `docs/phase2-architecture.md` for complete design.
Key architectural decisions (final):

- **`ProjectLoader` trait** in `mva-core` — shared contract; `mva-format` implements it.
- **`EngineEffect` output model** — `Engine::handle_command` returns `Vec<EngineEffect>`; engine stays pure.
- **`AudioController` trait** in `mva-core` (transport only, `Send + Sync`); `PlaybackClock` unchanged (position only).
- **`mva-types` narrow extraction** — Project, metadata, audio source, lyrics, track primitives, own Vec2 (leaf).
- **`PlaybackError` enum** replaces `Error(String)`.
- **7‑state `PlaybackState`**: `Stopped → Loading → Ready → Playing ↔ Paused → Finished → Error`.
- **`EngineEffect::LoadProject { path: PathBuf }`** — native path; Windows non‑UTF‑8 safe.
- **Forbidden:** `mva-format → mva-player`.

### Implementation order

| Step | Deliverable |
|---|---|
| 1 | `mva-types` narrow extraction (Vec2 defined in‑crate) |
| 2 | `EngineEffect` + `AudioController` trait + `mva-audio` impl |
| 3 | `PlaybackState` extension + `PlaybackError` |
| 4 | `ProjectLoader` trait + `mva-format` crate |
| 5 | Integration validation with real files |

| 3 | — | Effects + plugins (PluginHost v1, wgpu effects) |
| 4 | — | MVA format v1 read |
| 5 | — | Creator/Editor v1 + MVA format write + export |
| 6+ | — | Plugin loaders (WASM/native), graph editor, online lyrics, `mva-types` extraction |

## Phase 4 — Progress Log

| Step | Status | Deliverable |
|---|---|---|
| **4.0-demo** | ✅ Done | First showcase demo: `examples/lyric_demo/` — loose `.mva` JSON manifest (architecture §6.2) referencing a real CC BY 4.0 MP3, `lyrics.lrc`, and `lyric.anim.json`; `mva-format` reads the manifest (forward-tolerant, refuses format majors ≥ 2); the player binary loads the project's audio source on open (`SharedAudioPlayer::load_file`). Verified by `crates/mva-player/tests/demo_showcase.rs` + `crates/mva-format/tests/manifest_tests.rs`. |
| **M1** | ✅ Done | CLI infrastructure: clap 4 derive, `cli.rs`, `StartupMode`, parse tests, `impl Display for ProjectLoadError` |
| **M2** | ✅ Done | Startup modes: `startup.rs` bootstrap, `main.rs` rewrite, Empty/Demo/OpenProject, audio device failure graceful exit |
| **M3** | ✅ Done | Unified loading: `activate_project` (prepare/activate), convergence of two loading entries, `autoplay_on_open` config field, Known Limitation test |
| **M4** | ✅ Done | Config system: `config::loader` (dual-pass parse), `load_app_config`, warning system (UI + stderr), `config_warnings` in MvaUiApp |
| **M5** | ✅ Done | Native file dialog: rfd integration, File → Open File / Open Folder, UI panel layout fixes |
| 4.1 | ⬜ Planned | `.mva` ZIP container read (same manifest schema as `manifest.json` entry) |
| 4.2 | ⬜ Planned | Audio duration probing for loose-folder projects (currently duration comes from the manifest metadata) |
