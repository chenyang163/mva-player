# MVA Player — Architecture

Status: **REVISION 2 — awaiting final approval** (review round 1 adjustments applied)
Date: 2026-07-22
Based on: `docs/research.md` (approved technology stack) and `Agent.md` (project rules).

This document defines the system architecture. **No code exists yet.**
Implementation begins only after final approval of this revision.

Revision 2 changes (from review round 1):
1. Clarified `mva-core` vs. the future `mva-types` crate (§3.4).
2. Added plugin API versioning design (§8.1).
3. Added future asset management subsystem (§9).
4. Expanded Phase 1 scope to validate the full rendering pipeline with
   minimal animation (§11).

---

## 1. Architectural Vision: MVA Is a Format, Not Just a Player

MVA Player is one consumer of the MVA format. The system is therefore designed as
**three layers** sharing one engine core:

```
┌───────────────────────────────────────────────────────────────────┐
│ LAYER 3: MVA CREATOR / EDITOR   (binary: mva-editor)              │
│ Authors projects: import audio + lyrics, edit timing,             │
│ typography, animation, effects → exports .mva                     │
├───────────────────────────────────────────────────────────────────┤
│ LAYER 1: PLAYER RUNTIME         (binary: mva-player)              │
│ Opens loose files (mp3 + lrc) or .mva packages,                   │
│ plays audio, syncs lyrics, renders animation/effects,             │
│ hosts plugins, settings, about page                               │
├───────────────────────────────────────────────────────────────────┤
│ LAYER 2: MVA FORMAT ENGINE      (library: mva-format)             │
│ Defines the .mva container: read, write, validate, migrate.       │
│ Pure data + IO. No audio, no UI, no rendering dependencies.       │
│ Shared by BOTH the Player and the Creator.                        │
└───────────────────────────────────────────────────────────────────┘

Both Layer 1 and Layer 3 sit on the same engine crates (timeline,
renderer). Consequence: the editor's preview IS the player's output —
WYSIWYG by construction, not by duplication.
```

Long-term, `mva-format` is publishable as a standalone crate so that
**other players/tools can implement the MVA format** — this is what makes
MVA an *open format* rather than an internal project file.

---

## 2. Crate Map (Cargo Workspace)

```
mva-player/                      (repository root)
├── Cargo.toml                   (workspace definition)
├── config/                      (TOML configuration files — see §10)
├── docs/
├── assets/
├── plugins/                     (plugin drop-in directory, future)
└── crates/
    ├── mva-core                 Core Logic
    ├── mva-audio                Audio Engine
    ├── mva-timeline             Timeline Engine (data model + evaluation)
    ├── mva-lyrics               Lyric parsing (LRC now; TTML/ASS later)
    ├── mva-renderer             Animation Renderer (scene → draw list)
    ├── mva-format               MVA Format Engine (Layer 2)
    ├── mva-plugin               Plugin System (traits + host + loaders)
    ├── mva-ui                   UI Layer (egui) — player UI widgets/pages
    ├── mva-player               Player binary (Layer 1 app shell)
    ├── mva-editor               Creator/Editor binary (Layer 3, future)
    ├── mva-types                (FUTURE — §3.4: public format contract types)
    └── mva-assets               (FUTURE — §9: asset management subsystem)
```

Mapping from `Agent.md`'s recommended module list:

| Agent.md module | Home in this architecture |
|---|---|
| core | `mva-core` |
| audio | `mva-audio` |
| decoder | `mva-audio::decoder` (research decision: rodio bundles symphonia; a separate crate is unjustified) |
| lyrics | `mva-lyrics` |
| renderer | `mva-renderer` (+ `mva-timeline` for evaluation — required by the mandated Timeline Engine separation) |
| format | `mva-format` |
| editor | `mva-editor` |
| plugin | `mva-plugin` |
| ui | `mva-ui` |
| settings | `mva-ui::pages::settings` + `config/settings.toml` |
| about | `mva-ui::pages::about` (a page, not an engine concern) |
| config | `config/*.toml` files + config structs in `mva-core::config` |

---

## 3. Module Responsibilities & Dependency Rules

### 3.1 The six mandated separations

| Subsystem | Crate | Responsibility |
|---|---|---|
| Core Logic | `mva-core` | Application state machine, config structs, playback clock trait, command/event types, error types, common ids/units |
| Audio Engine | `mva-audio` | Decode + output + transport (play/pause/seek/volume), owns the playback position source. Wraps rodio/symphonia. No UI, no lyrics |
| Timeline Engine | `mva-timeline` | The internal data model (§4) and its evaluation at time *t*. Pure computation: no audio, no GPU, no UI |
| Animation Renderer | `mva-renderer` | Converts an evaluated scene into a backend-neutral **DrawList** (§7.3). No egui, no wgpu in this crate |
| UI Layer | `mva-ui` | egui/eframe: window, pages (player, settings, about), widgets, paints DrawLists, forwards user input as commands |
| Plugin System | `mva-plugin` | Plugin traits, manifest, registry/host lifecycle; loaders (built-in now; WASM + native later) |

### 3.2 Dependency graph (acyclic, enforced by Cargo)

```
                        ┌────────────┐
                        │  mva-core  │   config, clock, commands, events, errors
                        └─────┬──────┘
                              │
              ┌───────────────┼────────────────────┐
              ▼               ▼                    ▼
      ┌──────────────┐ ┌─────────────┐     ┌─────────────┐
      │  mva-audio   │ │ mva-timeline│     │ (nothing    │
      │ rodio/cpal   │ │ data model  │     │  else yet)  │
      │ symphonia    │ │ + evaluator │     └─────────────┘
      └──────┬───────┘ └──────┬──────┘
             │                │
             │       ┌────────┼─────────────┐
             │       ▼        ▼             ▼
             │  ┌─────────┐ ┌─────────┐ ┌───────────┐
             │  │mva-     │ │mva-     │ │ mva-      │
             │  │lyrics   │ │format   │ │ renderer  │
             │  │LRC/TTML │ │.mva IO  │ │ DrawList  │
             │  └────┬────┘ └────┬────┘ └─────┬─────┘
             │       └───────────┼────────────┘
             │                   ▼
             │            ┌─────────────┐
             │            │ mva-plugin  │  traits + host
             │            └──────┬──────┘
             │                   │
             ▼                   ▼
                    ┌─────────────────────┐
                    │       mva-ui        │  egui / eframe (wgpu)
                    └─────────┬───────────┘
                              │
                  ┌───────────┴───────────┐
                  ▼                       ▼
          ┌──────────────┐        ┌──────────────┐
          │  mva-player  │        │  mva-editor  │
          │   (binary)   │        │ (bin, future)│
          └──────────────┘        └──────────────┘
```

Rules:
1. `mva-core` depends on **no** other mva crate.  *Exception: Phase 1.3+ core depends on
   `mva-timeline` and `mva-scene` for the snapshot containing the evaluated scene — this is
   permitted by the Phase 1.3 brief and documented in `docs/ui-architecture.md`.*
2. No engine crate (`core`, `audio`, `timeline`, `lyrics`, `format`, `renderer`,
   `plugin`) may depend on `mva-ui` or on egui/eframe/wgpu types.
3. Only `mva-ui` may depend on egui/eframe.  The painter adapter (converting DrawLists
   into egui shapes) lives inside `mva-ui`; a dedicated `mva-render-egui` shim is deferred
   to Phase 2.  For the full per‑frame flow see `docs/ui-architecture.md`.
4. `mva-format` depends only on `mva-timeline` (for the data model) + serde/zip —
   it must stay embeddable in foreign tools.
5. Binaries wire everything together; binaries contain no business logic.
6. `mva-scene` is a **leaf** crate — both `mva-timeline` and `mva-renderer` depend on it,
   but neither depends on the other.  See `docs/ui-architecture.md` §1 for the Phase 1.6
   dependency graph.

### 3.3 Why the UI cannot couple to the engine

- The engine never sees egui types. All communication is:
  **UI → engine:** `PlayerCommand` enum values through a channel (§7.2).
  **engine → UI:** an immutable `EngineSnapshot` polled once per frame (§7.2).
- The renderer outputs a **backend-neutral DrawList** (own enum: text runs with
  font/size/color/transform, paths, images, shader-effect references).
  egui is *one* painter of DrawLists; a future `mva-ui-iced` or `mva-ui-gpui`
  crate could replace `mva-ui` without touching any engine crate.
- Audio position reaches the UI only as `Duration` values, never as rodio types.

### 3.4 `mva-core` vs. the future `mva-types` crate (public format contract)

A deliberate distinction between **application internals** and the
**public format contract**:

| Crate | Role | Audience | Stability discipline |
|---|---|---|---|
| `mva-core` | **Application runtime glue**: app state machine, config structs, `PlayerCommand`/`EngineEvent`/`EngineSnapshot`, playback clock trait, error types | Internal only (our own crates + binaries) | Free to change between app releases |
| `mva-types` *(future)* | **Pure serde data model of the MVA format**: `Project`, metadata, all timeline/track/keyframe/layer types, `AssetRef` — types exactly as serialized in `.mva` files | Public: `mva-format`, our engine, and **external ecosystem tools** (validators, converters, third-party players) | Versioned together with `format_version`; breaking change = format major bump |
| `mva-timeline` | **Evaluation engine** over those types (`evaluate(t)`, sampling, lyric lookup) + our ergonomic extensions | Public but engine-facing | Semver; may evolve faster than the format |
| `mva-format` | Container IO (ZIP/manifest), validation, migration | Public SDK | Semver |

**Is `mva-types` needed? Yes eventually — but not yet.**
Rationale for deferring: today the data model has exactly one writer and one
reader (our engine), so splitting it would only add crate-graph churn to
Phase 1. The model therefore starts inside `mva-timeline`, **with one hard
rule from day one: every type that appears in a serialized artifact
(`*.anim.json`, `manifest.json`, future `.mva`) must be defined in a single,
clearly-marked module (`mva_timeline::model`) containing only plain serde
data types — no engine logic, no egui/audio/plugin types.**

Extraction trigger (planned, Phase 5–6): the moment any external tool or the
plugin API needs the model without the engine, `model` is moved verbatim
into `mva-types`; `mva-timeline` re-exports it (no downstream code changes);
`mva-format` switches its dependency to `mva-types`. `mva-core` is never
extracted — commands, events, and config are our application's private
business and must never leak into the format contract.

Dependency rule added: no type in the future `model` module may reference
types from `mva-core`.

---

## 4. Internal Data Model (defined BEFORE implementation)

All model types live in `mva-timeline` (pure data + serde). Time is `f64`
**seconds**, continuous, defined by the audio clock — the model is
frame-rate independent (render at any fps).

```
Project                         ← one loaded song / one .mva document
├── ProjectMetadata
├── AudioTimeline
├── LyricTimeline    (0..* lyric tracks: original, translation, romanization)
├── AnimationTimeline
└── EffectTimeline
```

### 4.1 ProjectMetadata
```
title, artist, album, duration, cover_image (ref),
language(s), author_of_project, created_with (app+version),
format_version (semver), unique_id, custom: Map<String, String>
```

### 4.2 AudioTimeline — the master clock domain
```
source: AudioSource            Embedded(entry_path) | ExternalFile(path)
duration: f64
sample_rate: u32, channels: u8
volume_envelope: Option<Track<f32>>   (editor feature: fade in/out)
```
All other timelines are synchronized against the audio clock.
Playback position *t* ∈ [0, duration].

### 4.3 LyricTimeline
```
LyricTrack
├── language / role (Original | Translation | Romanization)
├── offset: f64                 (global shift, from LRC `offset` tag)
└── lines: Vec<LyricLine>
        ├── start: f64, end: Option<f64>
        ├── text: String
        └── words: Option<Vec<LyricWord>>   (karaoke: word + start/end)
```
Phase 1 uses line-level only (`words = None`). Word-level fills the same
structure without model changes.

### 4.4 AnimationTimeline — motion-graphics model (see §5)
```
AnimationTimeline
└── layers: Vec<Layer>          (z-ordered stack, bottom first)
```

### 4.5 EffectTimeline
```
effects: Vec<EffectInstance>
├── time_range: (f64, f64)
├── effect_id: String           (e.g. "bloom", "spectrum", "particles.snow")
├── target: EffectTarget        (WholeScene | Layer(layer_id) | Background)
└── parameters: Vec<Track<ParamValue>>   (animated effect parameters)
```

### 4.6 Track, Keyframe, Easing (shared primitives)
```
Track<T>                        one animated property lane
└── keyframes: Vec<Keyframe<T>> (sorted by time)

Keyframe<T>
├── time: f64
├── value: T
└── easing: Easing              (how we LEAVE this keyframe toward the next)

Easing
├── Hold                        (step)
├── Linear
├── Named(NamedEase)            (ease_in_quad, ease_out_cubic, … from simple-easing)
└── CubicBezier(x1, y1, x2, y2) (AE/CSS-style custom curve)

Interpolation = determined by the outgoing keyframe's Easing.
Sampling: value_at(track, t) -> T   (binary search keyframes, ease, lerp)
```

---

## 5. Animation System (inspired by professional motion graphics)

Concept mapping (After Effects → MVA):

| AE concept | MVA concept |
|---|---|
| Composition | `AnimationTimeline` (+ its duration = AudioTimeline duration) |
| Layer | `Layer` — a visual object with its own local time & transform |
| Transform properties (P/S/R/O/A) | `Layer.transform`: `Track<Vec2>` position/scale, `Track<f32>` rotation/opacity, `Track<Vec2>` anchor |
| Keyframe | `Keyframe<T>` (§4.6) |
| Easy Ease / graph editor | `Easing::CubicBezier` (per-keyframe tangents; graph editor is an editor-UI feature) |
| Parenting | `Layer.parent: Option<LayerId>` (transform inheritance) |
| Text animator (per-word/per-char) | `TextSource` binding + per-word evaluation (below) |
| Effects & presets | `EffectInstance` + named preset files (Phase 3) |
| Expressions | **Property bindings** (Phase 3+): a property may reference data, e.g. lyric progress or audio level, instead of keyframes |

```
Layer
├── id: LayerId, name: String
├── kind: LayerKind
│     ├── Text   { source: TextSource, style: TextStyle }
│     │            TextSource = Static(String)
│     │                       | LyricLine          (binds to active line)
│     │                       | LyricWord          (binds to active word)
│     ├── Image  { asset: AssetRef }
│     ├── Shape  { path/shape params }
│     └── ParticleEmitter { params }               (Phase 3)
├── transform: Transform        (animated Tracks, see above)
├── visible_range: (f64, f64)
├── parent: Option<LayerId>
└── blend_mode: BlendMode
```

**Evaluation (Timeline Engine core operation):**
```
AnimationTimeline::evaluate(t) -> Scene

Scene = Vec<EvaluatedLayer>   (flat, z-ordered, world transforms resolved)
EvaluatedLayer = kind + resolved content (text string for t)
               + computed transform/opacity + computed visibility
```
The evaluator is **pure**: same `t` in → same `Scene` out. No hidden state.
This gives us: deterministic tests, editor scrubbing (evaluate any t on demand),
and cheap seeking (nothing to "catch up").

Data-driven rule (Agent.md): layers/keyframes/effects are **data**
(serde JSON inside .mva / standalone `.anim.json`), never hardcoded.

---

## 6. The .mva Container Format (Layer 2 specification)

### 6.1 Format choice
Decision: **ZIP archive with a JSON manifest** (same proven pattern as
.docx/.apk/.vsix).

Rationale: standard tooling (inspect with any unzip), per-entry compression,
random access to entries, mature Rust support (`zip` crate), trivially
streamable entries. A directory with the same layout ("loose project") is also
valid input — this is the editor's working form, and Phase 1's
`mp3 + lrc in one folder` convention is forward-compatible with it.

Alternatives rejected: custom chunked binary (tooling burden, no benefit at
our scale); single JSON with base64 payloads (wasteful, unstreamable);
TAR (poor random access).

### 6.2 Layout
```
song.mva        (ZIP, extension .mva)
├── manifest.json               REQUIRED — the single entry point
├── audio/main.mp3|.flac|.wav   REQUIRED (exactly one)
├── lyrics/main.lrc             optional
├── lyrics/<name>.lrc|.ttml     optional (translations…)
├── animation/main.anim.json    optional (AnimationTimeline, serde JSON)
├── effects/*.effect.json       optional (EffectTimeline fragments/presets)
├── shaders/*.wgsl              optional (custom effect shaders)
├── assets/images/*             optional (covers, layer images)
├── assets/fonts/*              optional (bundled fonts)
└── metadata/cover.jpg          optional (conventional location)
```

`manifest.json` (conceptual):
```json
{
  "format_version": "1.0",
  "generator": "mva-editor 0.1.0",
  "id": "uuid",
  "metadata": { "title": "…", "artist": "…", "duration": 213.5 },
  "entries": {
    "audio": "audio/main.mp3",
    "lyrics": ["lyrics/main.lrc"],
    "animation": "animation/main.anim.json",
    "effects": []
  }
}
```

### 6.3 Versioning & backward compatibility
- `format_version` is semver. Reader rule: accept any `1.x`; refuse `≥2.0`
  with a clear error; **ignore unknown manifest fields and unknown ZIP
  entries** (forward tolerance).
- Adding new optional entries never breaks old players.
- `mva-format` exposes `read() -> Project`, `write(Project) -> .mva`,
  `validate()`, and later `migrate()`.
- Phase 1 does not implement the container; the data model above is the
  contract the container will serialize.

---

## 7. Data Flow

### 7.1 Runtime playback pipeline (per rendered frame)

```
 MP3/FLAC/WAV file
       │
       ▼
 ┌─────────────┐   PCM (f32)   ┌──────────────┐
 │   Decoder   │ ────────────► │ Audio Output │  (rodio/symphonia,
 │ (symphonia) │               │ (cpal/WASAPI)│   own audio thread)
 └──────┬──────┘               └──────────────┘
        │ samples consumed
        ▼
 ┌──────────────┐  t (Duration, atomic read — lock-free)
 │ PlaybackClock│
 └──────┬───────┘
        ▼
 ┌────────────────┐   lyric state for t   ┌──────────────────┐
 │ Timeline Engine│ ────────────────────► │ Scene (evaluated │◄── AnimationTimeline::evaluate(t)
 │  (mva-timeline)│                       │  layers @ t)     │
 └────────────────┘                       └────────┬─────────┘
                                                   ▼
                                          ┌──────────────────┐
                                          │ Animation        │
                                          │ Renderer         │──► DrawList (backend-neutral)
                                          │ (mva-renderer)   │
                                          └────────┬─────────┘
                                                   ▼
                                          ┌──────────────────┐
                                          │ UI (egui painter │──► pixels @ display fps
                                          │  + wgpu effects) │
                                          └──────────────────┘
```

### 7.2 Command / event flow (UI ⇄ engine — the decoupling contract)

```
        user input                    ┌────────────────────────────┐
              │                       │  Engine (UI thread, Phase1)│
              ▼                       │                            │
 ┌────────────────────┐  PlayerCommand│  AudioEngine (rodio handle)│
 │ mva-ui (egui pages)│ ────────────► │  TimelineEngine            │
 │                    │  OpenFile     │  PluginHost                │
 │                    │  Play/Pause   └────────────┬───────────────┘
 │  poll once/frame   │  Seek/Volume…              │
 │ ◄──────────────────┤                            │ EngineEvent (channel):
 │   EngineSnapshot   │   immutable snapshot:      │ TrackEnded / Error /
 │   { state, pos,    │   { playback state,        │ ProjectLoaded
 │     duration, vol, │     position, duration,    │
 │     lyric_index }  │     volume, lyric index }  │
 └────────────────────┘                            ▼
                                          audio thread (rodio internal;
                                          position published via atomics)
```

Rationale for poll-based snapshots (instead of push events for position):
egui is immediate-mode and repaints every frame anyway; snapshots are
lock-free, allocation-light, and impossible to miss/queue-flood. Rare events
(track end, errors) use a channel.

### 7.3 Rendering boundary

```
Scene ──► mva-renderer ──► DrawList ──► painter adapter ──► egui/wgpu
           (layout text,      enum:      (inside mva-ui)      pixels
            resolve styles,   TextRun,
            z-sort, cull)     Path/Image/
                              EffectRef)
```
Phase 1: DrawList ≈ styled text lines; painter adapter = thin egui mapping.
Phase 2+: EffectRef entries route to `egui::PaintCallback` (wgpu shaders).

### 7.4 Threading model

| Thread | Owns | Rules |
|---|---|---|
| Main (UI) | eframe, Engine, Timeline eval, rendering | Never blocks on disk/decode in frame path |
| Audio (rodio internal) | PCM mixing/output | Never touched directly by us; position via `AtomicU64` (sample counter) |
| (Future) decode-ahead | gapless next track | Communicates via channels |
| (Future) analysis | FFT/waveform for visualizers | Lock-free ring buffer to renderer |

### 7.5 MVA authoring flow (Layers 2+3)

```
audio file + lyric file ──► mva-editor ──► edit: timing / layers /
                                               keyframes / effects
                                    │
                                    ▼
                            mva-format::write(Project)
                                    │
                                    ▼
                              song.mva ──► mva-player ──► playback (7.1)
                              (also: loose folder, for version control)
```

---

## 8. Plugin System Architecture

Phase 1–2: **in-tree traits only** (all plugins compiled in). This defines the
seams with zero ABI risk. Loaders come later (research §7 conclusion).

```
mva-plugin
├── PluginManifest  (plugin.toml: id, name, version, capabilities)
├── trait Plugin               (lifecycle: on_load/on_unload, metadata)
├── capability traits:
│     ├── DecoderPlugin        (new audio formats → PCM source)
│     ├── LyricParserPlugin    (new lyric formats → LyricTrack)
│     ├── LyricProviderPlugin  (online fetch: LRCLIB, NetEase…)
│     ├── VisualizerPlugin     (audio-reactive scenes)
│     ├── EffectPlugin         (new effect_ids for EffectTimeline)
│     └── ThemePlugin          (UI themes)
├── PluginHost / PluginRegistry (capability lookup, lifecycle)
└── loaders/  (Phase 3+: builtin | wasm(extism) | native(libloading))
```

Boundary rules: plugins never get raw engine internals — only narrow,
versioned capability APIs. WASM plugins: no real-time audio path (research
conclusion). Native visualizer plugins: musikcube-style C-ABI, opt-in,
explicitly trusted.

### 8.1 Plugin API versioning (defined now, enforced when loaders land)

Three independent version lines exist in this project — do not conflate them:

| Version line | Lives in | Governs |
|---|---|---|
| App semver | `crates/*/Cargo.toml` | Player/editor releases |
| `format_version` | `.mva` manifest.json (§6.3) | Container/file compatibility |
| **`api_version`** | plugin manifest + `mva-plugin` host | Plugin↔host compatibility |

**Plugin manifest** (`plugin.toml`, shipped inside every plugin):
```toml
id = "com.example.snowfall"
name = "Snowfall Particles"
plugin_version = "1.2.0"        # the plugin's own semver
api_version = "1.0"             # plugin API it was built against  ← MAJOR.MINOR
capabilities = ["effect/v1", "theme/v1"]   # per-capability interface versions
```

**Compatibility strategy:**
- `api_version` is `MAJOR.MINOR`. The host declares the range it supports,
  e.g. `>=1.0, <2.0`.
- Host **adds** fields/functions with default behavior → MINOR bump; old
  plugins keep working (backward compatible within MAJOR).
- Host **changes/removes** anything → MAJOR bump; plugins of the old MAJOR
  are refused at load time with a clear, user-visible error (never a silent
  failure, never UB — critical for the future native-dylib loader).
- Capability traits carry their **own** version (`effect/v1`, `decoder/v2`):
  a plugin can implement `decoder/v1` while another implements `decoder/v2`;
  the host may support several capability generations simultaneously during
  deprecation windows. Deprecations are announced ≥1 MINOR cycle ahead and
  recorded in the plugin API changelog (`docs/plugin.md`, per Agent.md).
- **Version negotiation happens at load**: host reads manifest → checks
  `api_version` range → checks each declared capability version → only then
  runs `on_load`. A failed check never reaches plugin code.
- WASM future: capability versions map onto WIT world names
  (`mva:effect@1.0.0`), giving typed, versioned interfaces for free
  (Component Model direction from research §7).
- Phase 1–2: only built-in plugins exist; they are compiled against the
  in-tree traits, which constitute **plugin API v0.x — unstable, internal**.
  `api_version` becomes a public commitment (v1.0) when the first external
  loader ships (Phase 6+).

---

## 9. Asset Management (future subsystem — designed now, built Phase 2+)

Visual projects accumulate binary resources. Unmanaged paths inside
timelines would break portability (a `.mva` moved to another machine must
still find its images/fonts). A dedicated subsystem — crate **`mva-assets`**
— will own this.

### 9.1 Asset categories
| Category | Examples | Source |
|---|---|---|
| Images | covers, layer textures, sprite sheets | packaged in `.mva`, loose project dir, or external user files |
| Fonts | lyric fonts, title fonts | packaged, system-installed, user-imported |
| Shaders | WGSL effect shaders | packaged `shaders/`, plugin-provided, built-in |
| Particle resources | emitter presets, particle textures | packaged, plugin-provided |
| External assets | anything the user references outside the project | absolute/relative filesystem paths |

### 9.2 Design
- **Stable references:** timelines never store filesystem paths. They store
  `AssetRef` URIs, resolved by the subsystem at load:
  - `pkg://assets/images/cover.jpg` — inside the current `.mva` / loose project
  - `builtin://fonts/default` — compiled into the player
  - `file:///…` — external user file (editor warns before packaging;
    `mva-format` can *collect* externals into the package on export)
  - `plugin://<plugin-id>/<name>` — plugin-provided resources
- **Registry + cache:** `AssetRegistry` maps `AssetRef → AssetId`; an
  `AssetCache` holds decoded/CPU data; GPU-facing caches (texture atlas,
  font atlas, compiled WGSL modules) live behind the renderer backend and
  are keyed by `AssetId`. Lifetime is tied to the loaded `Project`; eviction
  is LRU with a configured budget (`config/renderer.toml`).
- **Fonts:** registered into the text stack (egui `FontDefinitions` now;
  cosmic-text/parley later) under a project-scoped family name so lyric
  styling can reference packaged fonts deterministically.
- **Shaders:** WGSL validated at load; editor offers hot-reload; invalid
  shaders degrade to a fallback effect + a validation warning, never a crash.
- **Missing assets:** `mva-format::validate()` reports unresolved refs;
  runtime substitutes placeholders (checkerboard image, default font,
  no-op effect) and surfaces warnings — a broken asset must never stop
  playback.
- **Loading policy:** lazy, off the frame path (background thread + channel,
  same pattern as §7.4); only the placeholder may be touched synchronously.

Phase 1 scope: none — `AssetRef` exists only as a documented concept;
Phase 1 renders text with the default UI font. The subsystem arrives with
image/font support in Phase 2–3 and is **mandatory** before `mva-editor`
export (Phase 5), since packaging = collecting every referenced asset.

---

## 10. Configuration-First Mapping (Agent.md rule)

Every subsystem reads its TOML; structs live in `mva-core::config`.

| File | Struct | Consumed by | Example keys |
|---|---|---|---|
| `config/app.toml` | `AppConfig` | mva-player | language, last_dir, window size |
| `config/audio.toml` | `AudioConfig` | mva-audio | volume, buffer_size, decoder_threads, output_device, gapless |
| `config/renderer.toml` | `RendererConfig` | mva-renderer/ui | fps_limit, vsync, effect_quality |
| `config/lyrics.toml` | `LyricsConfig` | mva-lyrics | font, offset_adjust, encoding_fallback |
| `config/animation.toml` | `AnimationConfig` | mva-timeline/renderer | default lyric-layer preset: fade_in_ms, fade_out_ms, scale_from, scale_to, easing |
| `config/plugin.toml` | `PluginConfig` | mva-plugin | enabled, plugin_dir, allow_native |
| `config/ui.toml` | `UiConfig` | mva-ui | theme, accent_color, lyric_layout |
| `config/editor.toml` | `EditorConfig` | mva-editor | autosave, snap, preview_quality |

Phase 1 creates only the files it needs (`app.toml`, `audio.toml`,
`lyrics.toml`, `animation.toml`) plus structs; the rest arrive with their
subsystem. The Phase 1 lyric fade/scale animation reads its parameters from
`animation.toml` — animated values are never hardcoded (Agent.md rule).

---

## 11. Error Handling, Testing, Phase Plan

Error handling: each crate exposes one `thiserror` enum (`AudioError`,
`LyricError`, `FormatError`, …); the binary aggregates into a user-facing
error type; engine failures surface as `EngineEvent::Error`, never panics
across threads; no `unwrap()` in library code (Agent.md rule).

Testing: parsers = fixture files; timeline evaluator = golden sampling at
fixed `t` values (purity makes this trivial); format = round-trip property
tests; audio = smoke-tested behind a trait with a null sink for CI.

### Expansion plan (phases)
| Phase | Deliverable | Crates touched/added |
|---|---|---|
| **1** | Minimal player **validating the full pipeline**: open MP3, play, load LRC, synced lyric line rendered as an animated **text layer** (fade + scale + simple easing) — proves AudioClock → Timeline Engine → Renderer → UI end to end | core, audio, lyrics, timeline (LyricTimeline + minimal §5 primitives: `Track<f32>`, `Keyframe`, `Easing`(Hold/Linear/Named), `Layer::Text`, `evaluate`), renderer (text + transform + opacity DrawList), ui, player bin |
| 2 | Animated lyrics v2: word-level LRC parser, richer layer kinds (image/shape), transitions, asset loading (fonts/images via mva-assets v1) | timeline (full §5), renderer, assets, ui |
| 3 | Effects + plugins: EffectTimeline, wgpu effects, PluginHost v1, settings/about pages | plugin, renderer (wgpu path), ui |
| 4 | MVA format v1: manifest + ZIP read; player opens `.mva` | format |
| 5 | MVA Creator v1: import audio+lyrics, timing editor, export `.mva` (asset collection) | editor, format (write), assets |
| 6+ | Visual keyframe/graph editor, plugin loaders (WASM/native, api_version v1.0 public), plugin store UX, online lyric providers, OS media integration (souvlaki), `mva-types` extraction on first external consumer, localization | per feature |

Each phase independently shippable; engine/UI separation means Phases 2–5
never rewrite Phase 1 code, only extend.

**Phase 1 acceptance criteria (updated per review):**
1. Open an MP3 file → audio plays with working transport (play/pause/seek/volume).
2. Load a same-named `.lrc` → lines parse into `LyricTimeline`.
3. Every rendered frame: playback clock position → timeline evaluation →
   the active lyric line is a `Layer::Text` whose opacity **fades in** and
   whose scale **eases from `scale_from` to `scale_to`** (parameters from
   `config/animation.toml`), painted via DrawList → egui.
4. The evaluation is pure: scrubbing the seek bar to any `t` instantly shows
   the correctly-evaluated frame (proves determinism of `evaluate(t)`).

---

## 12. Key Decisions Summary (for review)

1. **Three layers** (Player Runtime / MVA Format Engine / Creator) sharing
   engine crates → editor preview = player output.
2. **Workspace of 10 crates (+2 planned: `mva-types`, `mva-assets`)**,
   acyclic deps; egui confined to `mva-ui`;
   engine speaks `PlayerCommand`/`EngineSnapshot`/`DrawList` only.
3. **One data model** (`Project`) as the single contract between parser,
   format, engine, renderer, and editor; continuous `f64` seconds timebase.
4. **AE-inspired animation**: Timeline→Layer→Track→Keyframe + Easing
   (Hold/Linear/Named/CubicBezier); pure `evaluate(t) -> Scene`.
5. **.mva = ZIP + manifest.json**, semver-versioned, forward-tolerant readers,
   loose-folder mode supported from the start.
6. **Poll-based snapshots** for per-frame state; channel events for rare
   occurrences; atomics for the audio clock.
7. **Plugin seams now, loaders later**: traits in Phase 1–2; extism (WASM)
   and libloading in Phase 6+ per `docs/research.md` §7.
8. **Phase 1 scope is deliberately thin** but every boundary it crosses is
   the final boundary — no throwaway code.
9. **Data model = format contract**: serialized types live in one marked
   plain-serde module inside `mva-timeline` today, extractable verbatim into
   a public `mva-types` crate when the first external consumer appears;
   `mva-core` stays private application glue forever (§3.4).
10. **Three version lines, never conflated**: app semver, `format_version`
    (.mva files), `api_version` (plugins) with per-capability versions and
    load-time negotiation (§8.1).
11. **Assets are referenced, never pathed**: future `mva-assets` subsystem
    owns `AssetRef` URIs, registry/cache, font+shader handling, and
    export-time collection; missing assets degrade, never crash (§9).
12. **Phase 1 proves the whole pipeline**: audio clock → timeline engine →
    renderer → UI, demonstrated by a lyric text layer with config-driven
    fade + scale + easing — not just audio + static lyrics (§11).

---

## 13. Phase 1.7 Demo Bootstrap Boundary

The `mva-player` binary contains a module `test_project.rs` that
provides a **synthetic demo project** and a **synthetic audio source**
(440 Hz sine wave).  This module exists **only** to validate the
Phase 1 pipeline end‑to‑end without external media files.

### What it is

- `make_test_project()` — builds an in‑memory `Project` with 4 timed
  lyric lines, one `LayerKind::Text` layer bound to `LyricLine`, and
  animated opacity + scale keyframes.
- `make_test_sine()` — returns a 30 s, 440 Hz `rodio::SineWave`.

### What it is NOT

- **Not** a production loading path.
- **Not** an API contract — these helpers will be removed or replaced
  when `mva-format` (Phase 4) provides real Project deserialisation.
- **Not** a substitute for `mva-lyrics` (LRC parsing) or
  `mva-format` (`.mva` container).

### Replacement plan

| Milestone | Replacement |
|---|---|
| Phase 1.x | Demo bootstrap runs at startup; integration test validates pipeline |
| Phase 2 | Demo removed; project assembled from loose mp3 + lrc |
| Phase 4 | `mva-format::read()` produces `Project` natively |

### Markers

All items in `test_project.rs` carry the doc comment:
> This is Phase 1.7 demo bootstrap code.
> It will be replaced by real project loading (Phase 4, `mva-format`).
