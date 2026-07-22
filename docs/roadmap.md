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
