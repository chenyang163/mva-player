# MVA Player — UI Architecture (Phase 1.6)

Status: **DESIGN — awaiting approval**
Date: 2026-07-22
Based on: `docs/architecture.md` §3.2, §7.2, §7.3 and Phase 1.5 renderer.

This document defines the `mva-ui` crate and the `mva-player` binary
shell before any code is written.  Implementation proceeds only after
approval.

---

## 1. Dependency Graph (Phase 1.6 target)

```
                    ┌──────────────┐
                    │  mva-scene   │   IR types (Vec2, Scene, …)
                    └──────┬───────┘
                           │
              ┌────────────┼──────────────┐
              │            │              │
              ▼            ▼              ▼
      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
      │ mva-timeline │ │ mva-renderer │ │   mva-core   │
      │ (eval)       │ │ (DrawList)   │ │ (engine,     │
      └──────────────┘ └──────┬───────┘ │  clock trait)│
                              │         └──────┬───────┘
                              │                │
                              │     ┌──────────┘
                              │     │
                              ▼     ▼
                      ┌───────────────────┐
                      │     mva-ui        │  egui/eframe (wgpu)
                      │  (MvaUiApp)       │
                      └────────┬──────────┘
                               │
                      ┌────────┴──────────┐
                      │   mva-player      │  binary shell — wires
                      │   (main.rs)       │  everything together
                      └───────────────────┘

mva-audio ──► mva-core     (audio engine implements PlaybackClock;
                            NOT visible to mva-ui)
```

**Hard rules:**

| Rule | Reason |
|---|---|
| `mva-ui` depends on `mva-core` | To read `EngineSnapshot` and `PlaybackClock` trait |
| `mva-ui` depends on `mva-renderer` | To convert `DrawList` into egui shapes |
| `mva-ui` depends on `mva-scene` (transitively) | For layer types (via snapshot) |
| `mva-ui` must **NOT** depend on `mva-audio` | Audio engine is opaque behind `PlaybackClock` |
| `mva-ui` must **NOT** depend on `mva-timeline` | Timeline evaluation is opaque behind `EngineSnapshot` |
| `mva-player` binary only wires — no business logic | Architecture §3.2 rule 5 |

---

## 2. eframe Lifecycle

eframe's `run_native` blocks the main function and calls
`MvaUiApp::update(&mut self, ctx, _frame)` every frame (vsync).

The per-frame sequence inside `update()`:

```text
┌─ MvaUiApp::update() ──────────────────────────────────────────┐
│                                                                │
│  1. let t = self.clock.position_seconds();                     │
│     // PlaybackClock trait — the audio engine is behind this.  │
│                                                                │
│  2. self.engine.update_position(t);                            │
│     // Advances the core engine to the current audio time.     │
│                                                                │
│  3. let snap = self.engine.snapshot();                         │
│     // Immutable snapshot: state, position, lyric index,       │
│     // evaluated Scene.                                        │
│                                                                │
│  4. let viewport = Viewport::from_egui(&ctx);                  │
│     // Runtime window size (may change on resize).             │
│                                                                │
│  5. if let Some(ref scene) = snap.scene {                      │
│         let draw_list = self.renderer.render(scene, &viewport); │
│         paint_draw_list(ctx, &draw_list);                      │
│     }                                                          │
│                                                                │
│  6. ui_panels(ctx, &snap);  // controls, seek bar, settings    │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### 2.1 Why the engine does NOT own the clock

- `Engine` stays a pure state machine — no thread‑safety concern,
  testable without audio.
- `Box<dyn PlaybackClock>` lives in `MvaUiApp`.  The binary creates
  the `AudioPlayer`, boxes it, and hands it to the UI.
- `mva-ui` sees only `Box<dyn PlaybackClock>` — never
  `mva_audio::AudioPlayer`.  Dependency direction preserved.

### 2.2 Position update is NOT a PlayerCommand

`PlayerCommand::UpdatePosition` was already removed in Phase 1.3.
The audio clock position feeds directly into
`Engine::update_position(pos: f64)` — a plain method, not a command
channel.  This keeps the transport-control commands (Play / Pause /
Stop / Seek) separate from the continuous clock feed.

---

## 3. Renderer API — Stateless Viewport

### 3.1 Problem

The Phase 1.5 `Renderer` held a `RendererConfig` containing the
viewport size.  When the user resizes the egui window, the viewport
changes — but mutating `RendererConfig` conflates **startup
configuration** with **runtime window state**.

### 3.2 New API

```rust
/// Runtime window geometry — changes on resize.
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Renderer {
    pub fn new(config: RendererConfig) -> Self { … }

    /// Produce a DrawList from a scene at the current viewport size.
    ///
    /// `viewport` carries the runtime window dimensions (may change
    /// every frame on resize).  `RendererConfig` only holds static
    /// settings (e.g. future: quality level, AA samples, …).
    pub fn render(&self, scene: &Scene, viewport: &Viewport) -> DrawList { … }
}
```

| Was | Now |
|---|---|
| `render(&Scene)` — viewport baked into `RendererConfig` | `render(&Scene, &Viewport)` — viewport per frame |
| `RendererConfig { viewport_width, viewport_height }` | `RendererConfig { … }` — static only; viewport dimensions removed |
| Layout used `self.config.viewport_*` | Layout uses `viewport.width` / `viewport.height` |

### 3.3 `RendererConfig` — static settings only

```rust
/// Static renderer configuration loaded at startup.
///
/// **Does not** contain runtime viewport dimensions — those are
/// passed per‑frame via [`Viewport`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RendererConfig {
    // Future: render quality, AA samples, effect presets, …
    // Intentionally empty in Phase 1.6 — all runtime state is in
    // Viewport.
}

impl RendererConfig {
    pub fn from_toml(toml_str: &str) -> Result<Self, ???> { … }
}
```

The `config/renderer.toml` file moves its `[viewport]` section into
the runtime `Viewport` construction (read by the binary).
`RendererConfig` TOML holds only static knobs (presently empty).

---

## 4. NO `AudioConfig` type

There is no `AudioConfig` type anywhere in the current codebase, and
the architecture does not define one.  Audio engine configuration
(`config/audio.toml`) will be added when `mva-audio` gains adjustable
parameters (buffer size, output device selection — beyond Phase 1.6).

`mva-player` binary does NOT reference an `AudioConfig` during wiring.

---

## 5. `mva-player` Binary Shell

```rust
fn main() {
    // 1. Load configs
    let app_config  = AppConfig::from_toml(…);
    let anim_config = AnimationConfig::from_toml(…);
    let rend_config = RendererConfig::from_toml(…);

    // 2. Create engine (core runtime)
    let mut engine = Engine::new(app_config, anim_config);

    // 3. Create audio player (implements PlaybackClock)
    let audio = AudioPlayer::new().expect("audio device");
    let clock: Box<dyn PlaybackClock> = Box::new(audio);

    // 4. Create renderer
    let renderer = Renderer::new(rend_config);

    // 5. Launch UI
    eframe::run_native(
        "MVA Player",
        native_options(),
        Box::new(move |cc| {
            Ok(Box::new(MvaUiApp::new(cc, engine, clock, renderer)))
        }),
    );
}
```

The binary contains **zero business logic** — it only constructs
objects and wires them together (architecture §3.2 rule 5).

---

## 6. `MvaUiApp` Struct (conceptual)

```rust
pub struct MvaUiApp {
    engine: Engine,
    clock: Box<dyn PlaybackClock>,
    renderer: Renderer,
    // egui‑specific state (seek bar position, open‑file dialog, …)
}

impl eframe::App for MvaUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // per §2 above
    }
}
```

---

## 7. Architecture Constraints — Confirmation

| Constraint | Satisfied? | How |
|---|---|---|
| `mva-ui` never sees `mva-audio` types | ✓ | `Box<dyn PlaybackClock>` erased type |
| `mva-ui` never sees `mva-timeline` types | ✓ | `EngineSnapshot` is the contract boundary |
| `mva-renderer` never sees `mva-timeline` | ✓ | Already enforced — renderer consumes `mva_scene::Scene` |
| `mva-audio` never sees `mva-timeline` | ✓ | Audio is pure time source |
| Renderer is stateless | ✓ | `Viewport` per frame; `RendererConfig` immutable at startup |
| No `AudioConfig` created | ✓ | — |
| Renderer never reads files | ✓ | `from_toml()` parses a string; binary handles IO |
| Binary = wiring only | ✓ | No business logic in `main.rs` |
| Configuration‑First | ✓ | Every subsystem has its TOML + struct |

---

## 8. Phase 1.6 Implementation Checklist

- [ ] Add `Viewport` struct to `mva-renderer`
- [ ] Change `Renderer::render()` signature to `(&Scene, &Viewport)`
- [ ] Move viewport dimensions out of `RendererConfig`
- [ ] Add `RendererConfig::from_toml()`
- [ ] Update layout / cull to use `Viewport`
- [ ] Update renderer tests
- [ ] Create `crates/mva-ui/`
- [ ] Implement `MvaUiApp` with per‑frame flow (§2)
- [ ] Create `crates/mva-player/` binary shell
- [ ] Wire: config load → engine → audio → renderer → eframe
- [ ] Run fmt / clippy / full test suite
