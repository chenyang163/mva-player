# MVA Phase 2 — Architecture Design (Final)

Status: **FINAL — approved for implementation**
Date: 2026-07-22
Based on: `docs/architecture.md` (Rev2), `docs/phase2-architecture.md` (prior revisions), Phase 1.1–1.7 achieved implementation.

This revision applies the **Phase 2 final architecture review**:
no blockers; 4 recommended items applied.

---

## Architecture Review Resolution (all rounds)

| Review item | Final decision | Rationale |
|---|---|---|
| ProjectLoader trait location | **`mva-core`** — shared architecture interface | `mva-format`, `mva-cli`, `mva-editor`, `mva-server` all reuse it. §4. |
| Engine holds `Box<dyn AudioController>` | **Rejected.** Engine produces `EngineEffect` values; composition root applies them. | Engine stays a pure state machine. §3. |
| mva‑types scope | **Narrowed.** Phase 2: Project, metadata, AudioSource, Lyric, Track primitives, `Vec2`. | Avoids "everything‑types". §6. |
| `PlaybackState::Error(String)` | **Replaced** with `PlaybackError` enum. | Structured errors. §7. |
| AudioController vs PlaybackClock | **Separated.** `AudioController` = transport. `PlaybackClock` = position. | Preserves Phase 1 design. §3. |
| `EngineEffect` path type | **`PathBuf`** — not `String`. | Windows paths may be non‑UTF‑8. §3.3. |
| `Vec2` in mva‑types | **Own definition** — leaf crate, no dep on `mva-scene`. | §6.1. |
| `AudioController` thread safety | **`Send + Sync`** — documented. | §3.4. |

---

## 1. Phase 2 Goals

| Goal | Deliverable | Scope |
|---|---|---|
| Audio Controller | Engine emits `EngineEffect::Audio(…)`; binary dispatches to audio device via `AudioController` trait | Full |
| Project Loading Pipeline | `mva-core` defines `ProjectLoader` trait; `mva-format` implements it; `EngineEffect::LoadProject` | Full |
| mva‑format crate | Format‑engine crate skeleton + loose‑file loader | Addition |
| EngineEffect output model | Engine returns effects, stays pure state machine | Full |
| Playback State | 7‑state machine: `Stopped → Loading → Ready → Playing ↔ Paused → Finished → Error` | Full |
| mva‑types extraction | Narrow extraction: Project, metadata, audio source, lyrics, track primitives, Vec2 | Addition |

---

## 2. Current Architecture Review (post Phase 1.7)

### 2.1 What works

```
Audio Clock (mva-audio) → Engine (mva-core) → Snapshot (mva-core)
→ Scene (mva-scene) → Renderer (mva-renderer) → DrawList → UI (mva-ui)
```

Phase 1 core pipeline validated end‑to‑end.  92 tests passing.

### 2.2 Pain points addressed in Phase 2

- **Volume gap:** `SetVolume` updates `Engine.volume` but never reaches `AudioPlayer` → `EngineEffect::Audio(SetVolume(…))` dispatched by binary.
- **No real file loading:** `make_test_project()` is demo bootstrap only → `ProjectLoader` trait (in `mva-core`) + `mva-format` implementation.

---

## 3. Audio Controller Design — EngineEffect Model

### 3.1 Problem

`Engine` (in `mva-core`) must relay audio commands to `mva-audio`, but `mva-core` does **not** and must **not** depend on `mva-audio`.  The Engine must stay a pure state machine.

### 3.2 Design: Engine produces effects; composition root applies them

```
UI
 │  PlayerCommand::Play  /  SetVolume  /  Seek
 ▼
┌─────────────────────────────────────────────────┐
│  mva-core — Engine::handle_command(&mut, cmd):   │
│                                                  │
│    fn handle_command(&mut self, cmd)              │
│        -> Result<Vec<EngineEffect>, CoreError>    │
│    {                                              │
│        let mut effects = vec![];                  │
│        match cmd {                                │
│            Play => {                              │
│                self.state = Playing;              │
│                effects.push(EngineEffect::Audio(  │
│                    AudioCommand::Play));          │
│            }                                      │
│            SetVolume(v) => {                      │
│                self.volume = v;                   │
│                effects.push(EngineEffect::Audio(  │
│                    AudioCommand::SetVolume(v)));  │
│            }                                      │
│            …                                      │
│        }                                          │
│        Ok(effects)                                │
│    }                                              │
└──────────────────┬───────────────────────────────┘
                   │ Vec<EngineEffect>
                   ▼
┌─────────────────────────────────────────────────┐
│  mva-player binary (composition root)             │
│                                                   │
│  for effect in effects {                          │
│      match effect {                               │
│          EngineEffect::Audio(cmd) => {            │
│              audio_controller.apply(cmd);         │
│          }                                        │
│          EngineEffect::LoadProject { path } => { │
│              let proj = project_loader.load(&path)?;│
│              engine.load_project(proj);           │
│          }                                        │
│      }                                            │
│  }                                                │
└──────────────────┬───────────────────────────────┘
                   │ Box<dyn AudioController>
                   ▼
┌─────────────────────────────────────────────────┐
│  mva-audio — impl AudioController for AudioPlayer │
│    fn apply(&self, cmd: AudioCommand) { … }       │
└──────────────────────────────────────────────────┘
```

### 3.3 Trait definitions (in `mva-core`)

```rust
// mva-core/src/effect.rs

/// Commands forwarded to the audio device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioCommand {
    Play,
    Pause,
    Stop,
    Seek(f64),         // seconds
    SetVolume(f32),    // 0.0–1.0
}

/// Effects the Engine cannot apply itself — the composition root
/// (binary) processes them after `handle_command`.
#[derive(Debug, Clone)]
pub enum EngineEffect {
    /// Forward an audio command to the audio device.
    Audio(AudioCommand),
    /// Trigger project loading via `ProjectLoader::load(&path)`.
    ///
    /// Uses `PathBuf` (not `String`) because Windows paths may
    /// contain non‑UTF‑8 bytes.  The binary calls the loader with
    /// the native path, then calls `engine.load_project(proj)`.
    LoadProject { path: std::path::PathBuf },
}

// mva-core/src/audio.rs  (new)

/// Audio transport control — separated from [`PlaybackClock`].
///
/// # Thread safety
///
/// Implementations must be `Send + Sync`.  Methods are called from
/// the main (UI) thread; the audio backend drives its own thread
/// internally (rodio does this natively).  Any synchronisation
/// required between the UI thread and the audio thread is the
/// implementor's responsibility.
///
/// # Separation from PlaybackClock
///
/// `AudioController` **executes** transport commands.
/// [`PlaybackClock`] **reads** the current position.
/// These two concerns must not be merged:
/// - The clock is polled every frame (cheap, non‑blocking).
/// - The controller receives discrete commands (infrequent,
///   may involve device interaction).
/// - `mva-ui` only sees `Box<dyn PlaybackClock>` — never
///   `AudioController`.  The composition root (binary) holds both.
pub trait AudioController: Send + Sync {
    /// Apply a transport command to the audio device.
    fn apply(&self, cmd: AudioCommand);
}
```

**Why `EngineEffect` instead of trait objects in Engine:**

| Aspect | `Box<dyn AudioController>` in Engine | `EngineEffect` output |
|---|---|---|
| Engine purity | Impure — side effects via trait call | **Pure** — returns effects, no I/O |
| Testability | Needs mock trait impl | Assert `handle_command` returns expected `EngineEffect`s |
| Composability | New effects require new trait fields | One enum grows; binary handles dispatch |
| Binary role | Thin (just injection) | Mediates effects (acceptable wiring) |

### 3.4 Separation of `AudioController` from `PlaybackClock`

| Trait | Location | Responsibility | Thread‑safe |
|---|---|---|---|
| `PlaybackClock` | `mva-core::clock` (existing, unchanged) | Read current position in seconds | Must be callable from UI thread |
| `AudioController` | `mva-core::audio` (new) | Apply transport commands | `Send + Sync`; internal synchronisation required |

`AudioPlayer` implements **both** traits.  `mva-ui` only sees `Box<dyn PlaybackClock>` — unchanged from Phase 1.  The binary holds `Box<dyn AudioController>` and applies `EngineEffect::Audio(…)` to it.

---

## 4. Project Loading Pipeline

### 4.1 Problem

`mva-core` must provide a shared loading contract so that `mva-format`, future `mva-cli`, `mva-editor`, and `mva-server` all load projects through the same trait.  The Engine delegates loading via `EngineEffect::LoadProject`.

### 4.2 Design: ProjectLoader trait in `mva-core`, implemented in `mva-format`

```
UI → PlayerCommand::OpenFile(path)
        │
        ▼
Engine.handle_command(OpenFile(path))
        │
        │  engine.state = Loading
        │  returns Ok(vec![EngineEffect::LoadProject { path }])
        ▼
mva-player binary (composition root)
        │
        │  project_loader.load(&path)
        ▼
mva-format::MvaLoader  (impl ProjectLoader from mva-core)
        ├── detect(path) → loose dir? single file?
        ├── scan for audio (mp3/flac/wav)
        ├── find matching .lrc
        └── construct Project
        │
        ▼
mva-player binary
        │
        │  engine.load_project(proj)
        │  engine.state = Ready
        ▼
UI reads snapshot with PlaybackState::Ready
```

### 4.3 Trait definition (lives in `mva-core`)

```rust
// mva-core/src/loader.rs  (new)

use std::path::{Path, PathBuf};
use mva_types::Project;

pub enum ProjectLoadError {
    Io(std::io::Error),
    NoAudioFile(PathBuf),
    InvalidLyrics(PathBuf, String),
    UnsupportedFormat(String),
}

/// Shared project‑loading contract.
///
/// Defined in `mva-core` so that **every** consumer of MVA projects
/// — the player binary, a CLI tool, the future editor, a server —
/// can load projects through the same trait.
pub trait ProjectLoader: Send + Sync {
    fn load(&self, path: &Path) -> Result<Project, ProjectLoadError>;
}
```

**Why `ProjectLoader` lives in `mva-core` (not the binary):**

| Reason | Detail |
|---|---|
| Shared interface | `mva-core` is the architecture interface layer (§3.1). Traits that cross crate boundaries belong here. |
| Multi‑consumer reuse | `mva-player`, `mva-cli`, `mva-editor`, `mva-server` all use the same loader contract — defining it once avoids duplication. |
| Dependency direction | `mva-format → mva-core` (clean). No `mva-format → mva-player` dependency. |
| Core purity | `mva-core` defines the **contract**; it does NOT perform file I/O itself. The trait is pure abstraction. |

**Forbidden:** `mva-format → mva-player`.  The format crate only depends on `mva-core` and `mva-timeline`/`mva-types`.

### 4.4 `mva-format` public API (Phase 2)

```rust
// mva-format/lib.rs

pub struct MvaLoader {
    config: LoaderConfig,
}

impl MvaLoader {
    pub fn new(config: LoaderConfig) -> Self;
}

impl mva_core::loader::ProjectLoader for MvaLoader {
    fn load(&self, path: &Path) -> Result<Project, ProjectLoadError>;
}

// Internal
impl MvaLoader {
    fn detect(path: &Path) -> Format;
    fn load_from_dir(&self, dir: &Path) -> Result<Project, ProjectLoadError>;
    fn load_from_file(&self, file: &Path) -> Result<Project, ProjectLoadError>;
}
```

### 4.5 `mva-format` dependencies

```
mva-format
├── mva-core            (ProjectLoader trait, ProjectLoadError)
├── mva-types           (Project, AudioTimeline, LyricTimeline)
├── mva-timeline        (re‑exports mva-types; AnimationTimeline for future)
├── mva-lyrics          (LRC parser: lrc crate → LyricLine)
├── walkdir             (directory scanning)
├── thiserror           (error derive)
└── serde + serde_json  (future: animation.json)
```

**Forbidden:** `mva-format → mva-player` — the format crate must never depend on any binary.

---

## 5. `mva-format` Crate Design

### 5.1 Responsibility

Layer 2 of the architecture (§1).  Knows about file formats.  Implements `ProjectLoader` (defined in `mva-core`).

### 5.2 Phase 2 loading scope

| Feature | Phase 2 | Phase 4+ |
|---|---|---|
| Detect loose directory (mp3 + lrc) | **Yes** | Yes |
| Loose directory with animation.json | Optional | Yes |
| Single file (mp3 only, no lyrics) | **Yes** (degraded: lyrics = empty) | Yes |
| .mva container (ZIP + manifest) | No | Phase 4 |
| Format version validation | No | Phase 4 |

### 5.3 Configuration

```toml
# config/format.toml  (future)
[loader]
loose_auto_lyrics = true
strict_validation = false
encoding_fallback = "UTF-8"
```

Phase 2: config struct exists but uses defaults.

---

## 6. `mva-types` Extraction — Narrow Scope

### 6.1 What moves in Phase 2

Architecture §3.4 always planned this extraction.  Phase 2 triggers it because `mva-format` needs model types without the evaluator.

**Phase 2 scope — only the format‑contract types:**

```
mva-types  (NEW leaf, deps: serde only — NO dep on any mva-* crate)
├── Project
├── ProjectMetadata
├── AudioTimeline, AudioSource
├── LyricTimeline, LyricTrack, LyricLine, LyricWord, LyricRole
├── Track<T>, Keyframe<T>, Easing, NamedEase   (timeline primitives)
└── Vec2                                         (own definition — §6.2)

mva-timeline  (depends on mva-types)
├── re-exports all mva-types::*  (backward compatible)
├── AnimationTimeline, Layer, LayerKind, TextSource, Transform  (STAYS here)
├── EffectTimeline, EffectInstance  (Phase 3)
└── eval: AnimationTimeline::evaluate(t) → Scene
```

### 6.2 `Vec2` ownership

`mva-types` defines its **own** `Vec2`:

```rust
// mva-types/src/vec2.rs

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 { pub x: f32, pub y: f32; }
```

| Reason | Detail |
|---|---|
| Leaf crate | `mva-types` must not depend on `mva-scene` |
| No new geom crate | A single `Vec2` does not justify `mva-geom` |
| Conversion boundary | `mva-timeline` converts `mva_types::Vec2` → `mva_scene::Vec2` at the eval‑to‑scene boundary |

**Usage:**

- `mva-types::Vec2` — used in `Track<Vec2>`, `Keyframe<Vec2>`, model data.
- `mva-scene::Vec2` — used in `ComputedTransform`, `DrawCommand`, rendering coordinates.
- `mva-timeline::eval` converts between them when building the `Scene`.

### 6.3 What does NOT move in Phase 2

| Type | Why it stays in `mva-timeline` |
|---|---|
| `AnimationTimeline` | Contains `Layer` + `Transform` + `LayerKind` — coupled to evaluation semantics |
| `Layer`, `LayerKind`, `TextSource` | Authoring‑facing; not yet needed by external format consumers |
| `Transform` | Contains `Track<T>` references — coupled to animation model |
| `EffectTimeline`, `EffectInstance` | Phase 3 deliverable |
| `BlendMode` | Already in `mva-scene` (Phase 1.5) |

**Backward compatibility:** `use mva_timeline::model::Project` still works via re‑export.  No downstream code changes.

### 6.4 Full extraction plan

| Phase | Types moved |
|---|---|
| 2 | Project, ProjectMetadata, AudioSource, AudioTimeline, Lyric*, Track/Keyframe/Easing, Vec2 |
| 4 | AnimationTimeline, Layer, LayerKind, Transform (mva‑format writes .mva → needs these) |
| 5+ | EffectTimeline, full asset model (when editor exports .mva) |

---

## 7. Playback State Design

### 7.1 Current state (Phase 1)

```rust
enum PlaybackState { Stopped, Playing, Paused }
```

### 7.2 Phase 2 extended state machine

```
    ┌──────────┐
    │ Stopped  │◄─── no project loaded; explicit user stop
    └────┬─────┘
         │ OpenFile(path)
         ▼
    ┌──────────┐  load succeeds  ┌──────────┐
    │ Loading  │───────────────► │  Ready   │
    │          │                 └────┬─────┘
    └────┬─────┘                      │ Play
         │ load fails                 ▼
         ▼                       ┌──────────┐  Pause   ┌──────────┐
    ┌─────────┐                  │ Playing  │────────► │  Paused  │
    │  Error  │                  └────┬─────┘◄────────└──────────┘
    └────┬────┘                       │  Resume
         │                            │ t >= duration
         │ dismiss / re‑open          ▼
         ▼                       ┌──────────┐  Play (replay)
    ┌──────────┐                 │ Finished │──────────────► Playing
    │ Stopped  │                 └────┬─────┘
    └──────────┘                      │ Stop
                                      ▼
                                 ┌──────────┐
                                 │ Stopped  │
                                 └──────────┘
```

### 7.3 Rust definition

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Loading,
    Ready,
    Playing,
    Paused,
    Finished,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackError {
    FileNotFound,
    DecodeFailed,
    DeviceLost,
    InvalidProject,
    Unknown(String),
}
```

### 7.4 EngineSnapshot additions

```rust
pub struct EngineSnapshot {
    pub state: PlaybackState,
    pub position: f64,
    pub duration: f64,
    pub volume: f32,
    pub active_lyric_index: Option<usize>,
    pub scene: Option<Scene>,
    pub error: Option<PlaybackError>,    // active error for UI display
}
```

### 7.5 Transitions — who triggers them

| Transition | Trigger | Source |
|---|---|---|
| Stopped → Loading | `PlayerCommand::OpenFile` → `EngineEffect::LoadProject` | UI → binary |
| Loading → Ready | `engine.load_project(proj)` called by binary after load succeeds | Binary |
| Loading → Error | `ProjectLoader::load()` fails | Binary → engine |
| Ready → Playing | `PlayerCommand::Play` → `EngineEffect::Audio(Play)` | UI → engine |
| Playing → Paused | `PlayerCommand::Pause` → `EngineEffect::Audio(Pause)` | UI → engine |
| Paused → Playing | `PlayerCommand::Play` | UI → engine |
| Playing → Finished | `engine.update_position(t)` with `t >= duration` | Engine (polled) |
| Finished → Playing | `PlayerCommand::Play` → position reset to 0, `EngineEffect::Audio(Play)` | UI → engine |
| Finished → Stopped | `PlayerCommand::Stop` | UI → engine |
| Error → Stopped | `PlayerCommand::Stop` (user dismiss error) | UI → engine |

---

## 8. Phase 2 Dependency Graph (Final)

```
                          ┌──────────────┐
                          │   mva-scene  │  leaf — Scene, DrawList, EvaluatedLayer, Vec2
                          └──────┬───────┘
                                 │
     ┌───────────────────────────┼────────────────────┐
     ▼                           ▼                    ▼
┌──────────┐  ┌──────────────────────────┐  ┌─────────────────┐
│mva-types │  │      mva-renderer        │  │    mva-core     │
│ leaf     │  │  Scene → DrawList        │  │  Engine         │
│ Project  │  │  (no timeline deps)      │  │  EngineEffect   │
│ Metadata │  └───────────┬──────────────┘  │  AudioCommand   │
│ AudioSrc │              │                  │  AudioController│
│ Lyrics   │              │                  │  ProjectLoader  │
│ Track/KF │              │                  │  PlaybackState  │
│ Vec2     │              │                  │  EngineSnapshot │
└────┬─────┘              │                  │  PlaybackClock  │
     │                    │                  └────────┬────────┘
     ▼                    │                           │
┌────────────┐            │        ┌──────────────────┼──────────┐
│mva-timeline│            │        ▼                  ▼          │
│ evaluator  │            │  ┌───────────┐  ┌──────────────┐    │
│ + anim     │            │  │ mva-audio │  │  mva-lyrics  │    │
│   model    │            │  │AudioCtl   │  │  LRC parser  │    │
└────────────┘            │  │impl       │  └──────────────┘    │
                          │  └─────┬─────┘                      │
                          │        │                            │
                          │        │                            │
     ┌────────────────────┼────────┼────────────────────────────┘
     │                    ▼        ▼
     │             ┌──────────────────┐
     │             │     mva-ui       │  egui / eframe (only here)
     │             │  painter adapter │
     │             └────────┬─────────┘
     │                      │
     │                      ▼
     │             ┌──────────────────┐
     │             │   mva-player     │  binary (composition root)
     │             │  EngineEffect    │  ← dispatch loop
     │             │  wiring          │
     │             └────────┬─────────┘
     │                      │
     │             ┌────────┴─────────┐
     │             │   mva-format     │  impl ProjectLoader
     │             │  MvaLoader       │  loose-file detection
     │             └──────────────────┘
     │
     └── (mva-timeline depends on mva-types; re-exports)
```

### 8.1 Dependency rule compliance

| Rule | Constraint | Satisfied |
|---|---|---|
| `mva-renderer → mva-timeline` | **Forbidden** | ✓ |
| `mva-ui → mva-audio` | **Forbidden** | ✓ (UI only sees `EngineSnapshot`) |
| `mva-core → mva-format` | **Forbidden** | ✓ (core defines `ProjectLoader` trait; format implements it) |
| `mva-core → mva-audio` | **Forbidden** (except trait) | ✓ (core defines `AudioController` trait; audio implements it) |
| `mva-scene → any-business` | **Forbidden** | ✓ (leaf) |
| `mva-types → any-mva-crate` | **Forbidden** | ✓ (leaf; owns its own `Vec2`) |
| `mva-format → mva-player` | **Forbidden** | ✓ (format depends on mva-core, not the binary) |
| `mva-format → mva-core` | **Allowed** | Implementor depends on trait owner |
| Engine is pure state machine | **Required** | ✓ (`EngineEffect` output model) |

### 8.2 New dependency directions (Phase 2)

```
mva-types  → no mva deps     (leaf; own Vec2)
mva-format → mva-core         (implements ProjectLoader trait)
mva-format → mva-types        (uses Project, LyricTimeline)
mva-format → mva-timeline     (for AnimationTimeline re‑exports; future)
mva-format → mva-lyrics       (LRC parsing)
mva-audio  → mva-core         (implements AudioController trait)
mva-player → mva-core         (Engine, EngineEffect dispatch)
mva-player → mva-audio        (Box<dyn AudioController>)
mva-player → mva-format       (MvaLoader construction)
mva-player → mva-ui           (MvaUiApp, painter)
```

---

## 9. Migration Plan (Phase 2 implementation order)

### Step 1: `mva-types` narrow extraction
- Create `crates/mva-types/` with: Project, ProjectMetadata, AudioSource, AudioTimeline, Lyric*, Track/Keyframe/Easing, NamedEase, Vec2.
- Vec2 defined **in** mva-types — no dep on mva-scene.
- `mva-timeline` depends on `mva-types`, re‑exports.
- `mva-timeline::eval` converts `mva_types::Vec2` → `mva_scene::Vec2` at the scene boundary.
- Verify: all existing code compiles unchanged.

### Step 2: `EngineEffect` + `AudioController` trait
- Add `AudioCommand` enum and `EngineEffect` enum in `mva-core`.
- Add `AudioController` trait in `mva-core::audio` (`Send + Sync`).
- `Engine::handle_command` returns `Result<Vec<EngineEffect>, CoreError>`.
- `EngineEffect::LoadProject { path: PathBuf }` — native path, not String.
- `mva-audio` implements `AudioController` for `AudioPlayer`.
- Binary dispatches `EngineEffect::Audio(cmd)` to `audio_controller.apply(cmd)`.
- Verify: volume / play / pause work end‑to‑end.

### Step 3: `PlaybackState` extension
- Replace 3‑state enum with 7‑state enum + `PlaybackError`.
- Engine transitions per §7.5.
- `EngineSnapshot` gains `error: Option<PlaybackError>`.
- `engine.update_position(t)` transitions to `Finished` when `t >= duration`.
- UI reacts to new states.

### Step 4: `ProjectLoader` trait + `mva-format` crate
- Define `ProjectLoader` trait in `mva-core::loader`.
- Create `mva-format` crate with `MvaLoader` + loose‑file loading.
- `Engine::handle_command(OpenFile)` returns `EngineEffect::LoadProject { path }`.
- Binary wires: calls `project_loader.load(&path)`, then `engine.load_project(proj)`.
- Remove `test_project.rs` from binary.  Replace with fixture files in `tests/fixtures/`.
- Verify: open real MP3+LRC directory, play, see synced lyrics.
- **Forbidden:** `mva-format → mva-player` — verified at crate boundaries.

### Step 5: Integration validation
- Run Phase 1.7 acceptance criteria with real files.
- Confirm dependency rule compliance per §8.1.
- Run full test suite.  92 existing tests must still pass.

---

## 10. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| `mva-types` Vec2 vs `mva-scene` Vec2 confusion | Low | Different import paths; `mva-timeline::eval` is the single conversion point |
| `mva-types` extraction breaks re‑exports | Low | Types move verbatim; `use mva_timeline::model::Project` still works |
| `EngineEffect` grows large | Low | Start with 2 variants (`Audio`, `LoadProject`). Phase 6+ adds 1–2 more |
| `ProjectLoader` loading blocks UI thread (I/O) | Low (Phase 2) | MP3+LRC load is sub‑second. Phase 4+: background thread + `EngineEvent::ProjectLoaded` |
| `PlaybackState::Finished` UI confusion | Medium | Finished + Play = restart from 0. Paused + Play = resume. UI labels: Finished → ↺, Paused → ▶ |
| `mva-player` binary grows into god object | Medium | EngineEffect dispatch in small function; wire‑up in `setup.rs`. Limit: < 300 lines |
| `PathBuf` cloning across EngineEffect boundary | Low | Clone once per command; ~0 cost at UI‑frame frequency |
