# MVA Player

[![CI](https://github.com/chenyang163/mva-player/actions/workflows/rust.yml/badge.svg)](https://github.com/chenyang163/mva-player/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

The **reference runtime implementation** of the MVA open media format — a
timeline-driven animation engine with lyrics synchronization, image layering,
effect pipeline, and a composable crate architecture.

## Project Vision

MVA is designed to be more than a file format — it aims to provide an
**extensible media composition runtime** where:

- Content is organized on **timelines**, not just encoded frames
- Animation, text, images, and effects are **first-class primitives**
- The **same pipeline** drives both playback and authoring
- A **plugin ecosystem** extends capabilities without forking

> This is an early-stage project. The format specification and APIs are
> evolving and may change.

## Why MVA?

Existing media formats focus primarily on efficient encoding of audio/video
streams. MVA explores a different space:

| Conventional Formats | MVA |
|----------------------|-----|
| Video / audio encoding | Timeline-based composition |
| Frame-level decoding | Keyframe-driven animation |
| Fixed visual output | Real-time effect pipeline |
| Passive playback | Extensible runtime |

MVA does not compete with MP4, MKV, WebM, or Lottie — it addresses a different
problem: how to represent, render, and interact with composited media
presentations programmatically.

## Current Status

| Phase | Status | Deliverable |
|-------|--------|-------------|
| Phase 1 — Workspace & Core Runtime |  Done | `mva-core`, `mva-audio`, `mva-renderer`, `mva-ui`, `mva-player` binary |
| Phase 2 — Timeline & Lyrics |  Done | `mva-timeline` eval engine, `mva-lyrics` LRC parser, keyframe evaluation |
| Phase 3 — Images & Effects |  Done | Image asset pipeline, effect timeline, effect draw pipeline, end-to-end demo |
| Phase 4 — Project Loading & Format |   | Work in progress |

Phase 3 delivers:
- **Text rendering** — font-backed lyric display with fade-in/out animation
- **Image pipeline** — asset loading, layout, and compositing
- **Effect pipeline** — keyframe-driven effect evaluation and GPU-ready draw commands
- **Demo playback** — full end-to-end demo with synthetic project, audio, and visual output

## Architecture

```
mva-types         <- Pure data types (leaf crate)
    |
mva-timeline      <- Timeline engine (model + evaluation)
    |
mva-scene         <- Shared intermediate representation (IR)
    |
mva-renderer      <- Scene -> DrawList pipeline
    |
mva-ui            <- egui/eframe UI layer + painter adapter

mva-player        <- Binary shell (wiring only)
  +-- mva-core    <- Runtime engine, config, commands, events
  +-- mva-audio   <- Audio transport (rodio)
  +-- mva-format  <- Project loader (loose files -> .mva)
```

### Crate Responsibilities

| Crate | Role |
|-------|------|
| `mva-types` | Pure serde data types — `Project`, `Track`, `Keyframe`, `Layer`, `LyricLine`, audio/effect types. Strict leaf crate. |
| `mva-timeline` | Timeline data model + pure evaluation engine. Binary search, easing, interpolation, keyframe evaluation, lyric lookup. |
| `mva-scene` | Shared IR between renderer and UI. `Scene`, `Layer`, `Transform`, `DrawCommand`, `EffectIR`. |
| `mva-renderer` | Scene -> DrawList pipeline: layout, z-sort, culling, viewport mapping. Config-driven. |
| `mva-ui` | egui/eframe application shell. Four-panel layout (controls, viewport, settings, info). Painter adapter for DrawList rendering. |
| `mva-core` | Runtime engine: state machine (7-state), config loading, `PlayerCommand` channel, `EngineSnapshot`, `PlaybackClock` trait. |
| `mva-audio` | Rodio-based audio transport implementing `PlaybackClock`. Gapless playback, device selection. |
| `mva-lyrics` | LRC lyric file parser. Converts timestamped lyric lines into `LyricTimeline` objects. |
| `mva-format` | Project loader implementing the `ProjectLoader` trait. Reads loose file projects (mp3 + lrc + toml). Future: `.mva` container read/write. |
| `mva-player` | Binary shell. Wires all crates together with zero business logic. |

## Roadmap

### Completed

- **Phase 1** — Workspace architecture, core runtime engine, audio transport,
  renderer pipeline, UI layer, binary shell
- **Phase 2** — Timeline engine with pure evaluation, LRC lyric parsing,
  keyframe interpolation
- **Phase 3** — Image asset pipeline, effect timeline system, effect draw
  pipeline, end-to-end demo playback

### Planned

- Format specification stabilization (`.mva` container)
- Additional rendering backends (WebGPU, software fallback)
- Plugin ecosystem (WASM / native effect plugins)
- Cross-platform runtime verification (macOS, Linux, Windows)

## Demo Screenshot

> Screenshot will be added at `docs/images/demo.png`.

## Getting Started

### Prerequisites

- Rust toolchain **1.85+** (edition 2024)
- Audio output device

### Build & Run

```bash
cargo run -p mva-player
```

### Run Tests

```bash
cargo test --workspace
```

### Lint

```bash
cargo clippy --workspace
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines and
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community standards.

## License

This project is licensed under the **MIT License** ([LICENSE](LICENSE)).

**Why MIT:** MVA is designed as an open format ecosystem. A permissive license
encourages third-party implementations of players and tools without legal
friction. If the ecosystem grows, **Apache-2.0** may be considered as a future
option for its explicit patent grant.

## Development Notes

### AI-Assisted Development

This project uses AI tools as development assistants. The workflow is:

| Role | Responsibility |
|------|----------------|
| **ChatGPT** | Project planning, architecture discussions, documentation assistance |
| **DeepSeek V4 Pro** | Architecture review, code review, implementation assistance |
| **Human Developer** | Architecture decisions, integration, testing, final verification |

All AI output is reviewed, tested, and integrated by a human developer before
inclusion. AI is not the code author.

### Code Similarity and Licensing Notice

AI-generated suggestions may reflect patterns from publicly available code.
The maintainer commits to:

- Reviewing all AI-generated code before inclusion
- Avoiding intentional reproduction of copyrighted implementations
- Respecting third-party licenses
- Replacing code whose provenance cannot be confirmed

If you believe this project contains code that infringes on your copyright,
please contact the maintainer or open a [License Concern issue](CONTRIBUTING.md#issue-types).

---

*MVA Player — the reference runtime for the MVA open media format.*
