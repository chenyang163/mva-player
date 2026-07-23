# MVA Player — API Documentation

Generated from source on 2026-07-22.  The source code is the authoritative reference;
this document is a human-readable overview.

---

## Workspace Overview

| Crate | Purpose | Leaf? |
|---|---|---|
| `mva-types` | Pure serde data model — format contract types | ✓ (no mva-* deps) |
| `mva-scene` | Renderer-independent Scene IR shared by timeline eval and renderer | |
| `mva-timeline` | Timeline data model + pure evaluation engine | |
| `mva-core` | Application runtime: state machine, config, commands, clock trait, loader contract | |
| `mva-audio` | Rodio-based audio engine implementing PlaybackClock + AudioController | |
| `mva-renderer` | Scene → DrawList converter (no timeline dep) | |
| `mva-lyrics` | LRC parser → LyricTimeline | |
| `mva-format` | File → Project loader (directory / single-file detection) | |
| `mva-ui` | egui/eframe panels + painter adapter — **only** crate with egui | |
| `mva-player` | Composition root binary — wiring only | |

### Dependency Graph

```
mva-types  ──► mva-scene  ──► mva-timeline  ──► mva-core
                                          │
              mva-renderer ◄── mva-scene   │
                                          │
              mva-lyrics  ◄── mva-types    │
                                          │
              mva-format  ◄── mva-core, mva-timeline, mva-lyrics
                                          │
              mva-ui      ◄── mva-core, mva-renderer, mva-scene
                                          │
              mva-player  ◄── all of the above
```

**Key invariants:**
- `mva-renderer` never depends on `mva-timeline`
- `mva-ui` never depends on `mva-audio` (uses `Box<dyn PlaybackClock>`)
- `mva-ui` is the only crate with `egui`/`eframe`
- `mva-types` is a pure serde leaf — own `Vec2`, no mva-scene dep

---

## Runtime Data Flow

```
Audio File / Sine Wave
      │
      ▼
mva-audio (rodio)
  ├── PlaybackClock::position_seconds()  ← polled per frame
  └── AudioController::apply(cmd)        ← transport commands
      │
      ▼
mva-core (Engine)
  ├── engine.update_position(t)           ← audio clock → engine position
  ├── engine.handle_command(cmd)          ← UI commands → Vec<EngineEffect>
  │     ├── EngineEffect::Audio(Play/Seek/Stop/Vol)
  │     └── EngineEffect::LoadProject { path }
  └── engine.snapshot() → EngineSnapshot
        ├── state: PlaybackState
        ├── position / duration / volume
        ├── active_lyric_index
        ├── scene: Option<Scene>
        └── error: Option<PlaybackError>
      │
      ▼
mva-timeline (evaluator)
  ├── evaluate(animation, lyrics, effects, t) → Scene
  ├── evaluate_effects(effect_timeline, t) → Vec<ActiveEffect>
  └── active_lyric_text / active_lyric_word
      │
      ▼
mva-renderer
  └── Renderer::render(&Scene, &Viewport) → DrawList
        ├── DrawCommand::Text { .. }       (centred layout)
        ├── DrawCommand::Image(ImageDraw)  (asset ref pass-through)
        └── DrawCommand::Effect(EffectDraw) (effect id + params)
      │
      ▼
mva-ui (painter)
  ├── Text → egui::Painter::text()
  ├── Image → fs::read + image::decode → egui texture → painter.image()
  │           (with HashMap<String, TextureHandle> cache)
  └── Effect → debug rect + label "Effect: {id} ({n} params)"
      │
      ▼
pixels @ display fps
```

---

## Public API Reference

### `mva_types` — Pure Data Model

**Location:** `crates/mva-types/src/`

**Deps:** `serde`

All types derive `Serialize + Deserialize` for `.mva` format contract.

| Type | Variants / Fields |
|---|---|
| `Vec2` | `{ x: f32, y: f32 }` — own definition (no mva-scene dep) |
| `Track<T>` | `{ keyframes: Vec<Keyframe<T>> }` — one animated property lane |
| `Keyframe<T>` | `{ time: f64, value: T, easing: Easing }` — single keyed value |
| `Easing` | `Hold` \| `Linear` \| `Named(NamedEase)` — 18 Penner curves |
| `AudioTimeline` | `{ source: AudioSource, duration, sample_rate, channels, volume_envelope: Option<Track<f32>> }` |
| `AudioSource` | `Embedded { entry_path }` \| `ExternalFile { path }` |
| `LyricTimeline` | `{ tracks: Vec<LyricTrack> }` |
| `LyricTrack` | `{ role, language, offset, lines: Vec<LyricLine> }` |
| `LyricLine` | `{ start, end, text, words: Option<Vec<LyricWord>> }` |
| `ProjectMetadata` | `{ title, artist, album, duration, cover_image, languages, author, created_with, format_version, id, custom: BTreeMap }` |
| `EffectTimeline` | `{ effects: Vec<EffectInstance> }` |
| `EffectInstance` | `{ time_range, effect_id, target: EffectTarget, parameters: Vec<EffectParam> }` |
| `EffectTarget` | `WholeScene` \| `Layer { layer_id }` \| `Background` |
| `EffectParam` | `{ name: String, track: Track<ParamValue> }` |
| `ParamValue` | `Float { value: f32 }` \| `Bool { value: bool }` \| `Int { value: i32 }` (default: `Float(0.0)`) |
| `AssetRef` | `File { path }` \| `Pkg { path }` — stable resource identifier |

### `mva_scene` — Scene IR

**Location:** `crates/mva-scene/src/`

**Deps:** `mva-types`, `serde`

| Type | Purpose |
|---|---|
| `Scene` | Evaluated frame: `{ layers: Vec<EvaluatedLayer>, effects: Vec<ActiveEffect> }` |
| `Scene::empty()` | Returns `Scene { layers: vec![], effects: vec![] }` |
| `EvaluatedLayer` | `{ id, name, layer_index, kind, transform, visible, blend_mode }` |
| `EvaluatedLayerKind` | `Text { text, style }` \| `Image { asset }` |
| `ComputedTransform` | `{ position, scale, rotation, opacity, anchor }` — reset values at time `t` |
| `ActiveEffect` | `{ effect_id, params: Vec<(String, ParamValue)>, target: EffectTarget }` |
| `Rgba` | `{ r, g, b, a: u8 }` — `Rgba::WHITE` constant, `From<[u8;4]>` |
| `LayerId` | `pub String` newtype — transparent serde |

### `mva_timeline` — Model + Evaluation

**Location:** `crates/mva-timeline/src/`

**Deps:** `mva-types`, `mva-scene`

#### `model` module

Re-exports: `Vec2`, `Track`, `Keyframe`, `Easing`, `NamedEase`, `AudioTimeline`, `AudioSource`, `LyricTimeline`, `LyricTrack`, `LyricLine`, `LyricWord`, `LyricRole`, `ProjectMetadata`, `EffectTimeline`, `BlendMode`, `LayerId`, `TextStyle`

Additional model types:

| Type | Purpose |
|---|---|
| `Project` | Root document: `{ metadata, audio, lyrics, animation, effect_timeline }` |
| `AnimationTimeline` | Z-ordered layer stack: `{ layers: Vec<Layer> }` |
| `Layer` | Visual object: `{ id, name, kind, transform, visible_range, parent, blend_mode }` |
| `LayerKind` | `Text { source, style }` \| `Image { asset }` |
| `TextSource` | `Static { text }` \| `LyricLine` \| `LyricWord` |
| `Transform` | `{ position, scale, rotation, opacity, anchor }` — each is `Track<Vec2>` or `Track<f32>` |

#### `eval` module

| Function | Signature | Purpose |
|---|---|---|
| `evaluate` | `(&AnimationTimeline, &LyricTimeline, &EffectTimeline, f64) → Scene` | Full scene evaluation at time `t` |
| `value_at` | `<T: Interpolate>(&Track<T>, f64, T) → T` | Sample a track at `t` |
| `active_lyric_text` | `(&LyricTimeline, f64) → Option<&str>` | Active lyric line |
| `active_lyric_index` | `(&LyricTimeline, f64) → Option<usize>` | Line index for UI highlight |
| `evaluate_effects` | `(&EffectTimeline, f64) → Vec<ActiveEffect>` | Active effects at `t` |

| Trait | Method | Implementors |
|---|---|---|
| `Interpolate: Clone` | `fn lerp(&self, &Self, f64) → Self` | `f32`, `Vec2`, `ParamValue` |

### `mva_core` — Application Runtime

**Location:** `crates/mva-core/src/`

**Deps:** `mva-timeline`, `toml`, `thiserror`

#### `Engine`

```rust
pub struct Engine { /* fields private */ }

impl Engine {
    pub fn new(app: AppConfig, anim: AnimationConfig) -> Self;
    pub fn handle_command(&mut self, cmd: PlayerCommand)
        -> Result<Vec<EngineEffect>, CoreError>;
    pub fn update_position(&mut self, position: f64);
    pub fn snapshot(&self) -> EngineSnapshot;
    pub fn set_state(&mut self, state: PlaybackState);
    pub fn set_error(&mut self, error: PlaybackError);
}
```

Pure state machine — produces `EngineEffect`s for the composition root.

#### Commands & Effects

| Type | Variants |
|---|---|
| `PlayerCommand` | `Play`, `Pause`, `Stop`, `Seek(f64)`, `SetVolume(f32)`, `LoadProject(Box<Project>)`, `OpenFile(PathBuf)` |
| `EngineEffect` | `Audio(AudioCommand)` \| `LoadProject { path: PathBuf }` |
| `AudioCommand` | `Play`, `Pause`, `Stop`, `Seek(f64)`, `SetVolume(f32)` |

#### State

| Type | Variants |
|---|---|
| `PlaybackState` | `Stopped`, `Loading`, `Ready`, `Playing`, `Paused`, `Finished`, `Error` |
| `PlaybackError` | `AudioDeviceUnavailable`, `DecodeFailed`, `ProjectLoadFailed`, `Unknown(String)` |
| `EngineSnapshot` | `{ state, position, duration, volume, active_lyric_index, scene, error }` |

#### Traits

| Trait | Method | Location |
|---|---|---|
| `PlaybackClock` | `fn position_seconds(&self) -> f64` | `clock.rs` |
| `AudioController: Send + Sync` | `fn apply(&self, cmd: AudioCommand) -> Result<(), AudioError>` | `audio.rs` |
| `ProjectLoader: Send + Sync` | `fn load(&self, path: &Path) -> Result<Project, ProjectLoadError>` + `fn supported_extensions(&self) -> &[&str]` | `loader.rs` |

#### Config

| Struct | Source TOML |
|---|---|
| `AppConfig` | `config/app.toml` |
| `AudioConfig` | `config/audio.toml` |
| `AnimationConfig` | `config/animation.toml` |
| `LyricsConfig` | `config/lyrics.toml` |

All support `from_toml(&str) -> Result<Self, CoreError>`.

### `mva-audio` — Audio Engine

**Location:** `crates/mva-audio/src/`

**Deps:** `mva-core`, `rodio`

| Type | Purpose |
|---|---|
| `AudioPlayer` | Rodio transport: `new()`, `load_file()`, `load_source()`, `play()`, `pause()`, `stop()`, `set_volume()` |
| `SharedAudioPlayer` | `Arc<AudioPlayer>` newtype — implements `PlaybackClock` + `AudioController` for shared ownership |
| `AudioError` | `NoSource` \| `InvalidState(String)` \| `Backend(String)` \| `Decode(String)` \| `Io(std::io::Error)` |

`SharedAudioPlayer` is the primary type used by the binary — it allows one `AudioPlayer` to be shared behind both `Box<dyn PlaybackClock>` (for position polling) and `Box<dyn AudioController>` (for transport dispatch).

### `mva-renderer` — Animation Renderer

**Location:** `crates/mva-renderer/src/`

**Deps:** `mva-scene`, `mva-types`

| Type | Purpose |
|---|---|
| `Renderer` | `new(config) → Self`, `render(&Scene, &Viewport) → DrawList` |
| `RendererConfig` | Static settings (empty in Phase 3; `from_toml()`) |
| `Viewport` | Runtime window geometry `{ width: f32, height: f32 }` |
| `DrawList` | `{ commands: Vec<DrawCommand> }` — `empty()` sentinel |
| `DrawCommand` | `Text { .. }` \| `Image(ImageDraw)` \| `Effect(EffectDraw)` |
| `ImageDraw` | `{ asset: AssetRef, x, y, width, height, opacity }` |
| `EffectDraw` | `{ effect_id, params, target_rect: (f32, f32, f32, f32) }` |

**Pipeline:** layers → cull (invisible/off-screen) → layout (text centre, image position) → commands.
Effects → `target_rect` per `ActiveEffect` → `DrawCommand::Effect`.

### `mva-lyrics` — LRC Parser

**Location:** `crates/mva-lyrics/src/`

**Deps:** `mva-types`, `lrc`

| Function | Purpose |
|---|---|
| `parse_lrc(content: &str) -> Result<LyricTimeline, LyricParseError>` | Parse `.lrc` text into single `LyricTrack` |

**Status: Phase 2** — line-level only (word timings deferred).

### `mva-ui` — UI Layer

**Location:** `crates/mva-ui/src/`

**Deps:** `mva-core`, `mva-renderer`, `mva-scene`, `mva-types`, `image`, `egui`, `eframe`

#### Public

| Type | Purpose |
|---|---|
| `MvaUiApp` | `eframe::App` implementation — per-frame state machine |

#### Internal (not public API, documented for contributors)

| Module | Role |
|---|---|
| `painter` | `paint_draw_list(cache, ctx, painter, draw_list)` — Text → `painter.text()`, Image → `fs::read` + `image::decode` + texture cache + `painter.image()`, Effect → debug rect/label |
| `panels::controls` | Play/Pause/Replay/Stop buttons, seek slider, volume slider → `PlayerCommand` |
| `panels::info` | Top bar: state, position/duration, lyric index, error display |
| `panels::viewport` | Central panel: delegates to painter |
| `panels::settings` | File menu, path input, Help/About |

#### Image Loading

Only in `mva-ui` — `AssetRef::File { path }` → `std::fs::read` → `image::load_from_memory` → `egui::ColorImage` → `ctx.load_texture`.  Textures are cached in `HashMap<String, TextureHandle>` scoped to `MvaUiApp`.

`AssetRef::Pkg { .. }` displays a grey placeholder (Phase 4+).

---

## Planned / Not Yet Implemented

| Feature | Phase | Status |
|---|---|---|
| `.mva` ZIP container read/write | 4 | Not implemented |
| `mva-types` full extraction (AnimationTimeline, Layer) | 4 | Partial — Phase 2 narrow extraction done |
| `mva-assets` crate (AssetRef resolution, font loading) | 4 | Not implemented |
| Editor / Creator application | 5 | Not implemented |
| Plugin system (traits) | 6 | Not implemented |
| WASM / native plugin loaders | 6+ | Not implemented |
| Word-level (karaoke) LRC parsing | 2 | Not implemented |
| GPU effect execution (wgpu shaders) | 4 | Not implemented — Phase 3 debug visualization only |
| `AudioConfig` wire-up | 2 | Struct exists; not wired to audio engine |
| `LyricsConfig` wire-up | 2 | Struct exists; not wired to lyric parser |
| File dialog (native `rfd` crate) | Future | Text input field used as workaround |
| Online lyric providers | 6+ | Not implemented |
