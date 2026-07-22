# MVA Phase 3 Architecture Design

Status: **DESIGN — awaiting approval**
Date: 2026-07-22
Based on: Phase 2 Final Audit (EngineEffect stable, ProjectLoader done, mva-types done, AudioController done).

---

## 1. Goals

Phase 3 milestone statement:

> Validate that MVA is an **extensible visual rendering engine**, not just a lyrics player.
> No feature bloat. One new layer kind. One effect. One image. The architecture proves it
> can grow — the implementation stays minimal.

Metric: at Phase 3 end, a `.json` description of an Image layer with a simple effect
(say, opacity-modulate-by-audio-level) flows correctly through the **full** pipeline:
`mva-types → mva-timeline → mva-scene → mva-renderer → mva-ui → pixel`.

---

## 2. Scope — MUST / WILL NOT / DEFER

| Classification | Item | Rationale |
|---|---|---|
| **MUST** | EffectTimeline data model (mva-types) | Proves effect data can be serialized and evaluated. |
| **MUST** | EffectInstance + EffectParams + Track\<ParamValue\> | Composable with the existing Track/Keyframe engine. |
| **MUST** | LayerKind::Image (mva-types) | Proves non-text layer rendering. |
| **MUST** | EvaluatedLayerKind::Image (mva-scene) | Scene carries resolved image refs. |
| **MUST** | DrawCommand::Image (mva-scene) | Renderer produces image draw commands. |
| **MUST** | DrawCommand::Effect (mva-scene) | Renderer produces effect draw commands. |
| **MUST** | Renderer pipeline: Image layer → DrawCommand::Image | Proves layer dispatch is extensible. |
| **MUST** | Renderer pipeline: Effect ref → DrawCommand::Effect | Proves renderer handles effect metadata without knowing GPU. |
| **MUST** | UI paints DrawCommand::Image (simple file read) | Proves a new DrawCommand variant reaches the pixel. |
| **MUST** | UI has PaintCallback slot for DrawCommand::Effect | Proves the GPU boundary exists; no-op implementation accepted. |
| **MUST** | Plugin trait skeleton (mva-plugin, traits only) | Validates the plugin crate's dependency position. |
| **WILL NOT** | GPU shader execution | Effect DrawCommand produces a placeholder. |
| **WILL NOT** | mva-assets crate | Image loading is simple file I/O in mva-ui for now. |
| **WILL NOT** | Particle system, camera system, shape layers | Defined but not implemented. |
| **WILL NOT** | Plugin loaders (WASM, dylib, ABI) | Traits only; loading deferred to Phase 6. |
| **WILL NOT** | Editor UI | No visual authoring tools. |
| **WILL NOT** | .mva container read/write | Loose files only. |
| **DEFER** | Custom WGSL shaders | Phase 4 (after GPU boundary is proven). |
| **DEFER** | Asset caching / GPU texture atlas | Phase 4–5 (with mva-assets). |
| **DEFER** | Audio-reactive uniform upload (FFT → GPU) | Phase 4. |
| **DEFER** | Layer blending modes beyond Normal | Phase 4. |

---

## 3. Effect System — Layer Analysis

### 3.1 The question

Effect spans four architectural concerns. Where does each concern live?

| Concern | Example | Lives in |
|---|---|---|
| **Persistent definition** — what effects exist on the project | `EffectTimeline` with `EffectInstance[]`, each having `effect_id`, `time_range`, `params: Track<ParamValue>[]` | **mva-types** — it is serializable project data, same tier as `AnimationTimeline`. |
| **Evaluation** — which effects are active at time *t*, what are their resolved params | `EffectTimeline::evaluate(t) -> Vec<ActiveEffect>` — sample each `Track<ParamValue>` at *t*, filter by `time_range` | **mva-timeline** — pure computation; same pattern as `AnimationTimeline::evaluate`. |
| **Scene reference** — the renderer receives "there is an effect X with these resolved params" | `EvaluatedLayer` or scene-level entry carrying `effect_id` + flat params for this frame | **mva-scene** — already holds the render contract between timeline and renderer. |
| **Rendering** — how to execute effect X (GPU shader, post-process) | `DrawCommand::Effect { effect_id, params, clip_rect }` → UI translates to `egui::PaintCallback` or future wgpu pass | Renderer produces the **command** (mva-renderer → mva-scene type). UI **executes** it (mva-ui, GPU). |

### 3.2 No new crate

Effect is **not** a new crate. It is a concern that threads through four existing crates at their respective layers, using the same `Track<f32>` and `Keyframe` primitives already defined in mva-types (§4.6). The evaluator in mva-timeline already knows how to sample `Track<T>` at time *t* — `EffectTimeline::evaluate` is a thin composition of existing primitives.

```
mva-types:         EffectTimeline { instances: Vec<EffectInstance> }
                        │
mva-timeline:      EffectTimeline::evaluate(t) -> Vec<ActiveEffect>
                        │
mva-scene:         Scene { effects: Vec<ActiveEffect> }
                        │
mva-renderer:      for each ActiveEffect → DrawCommand::Effect
                        │
mva-ui:            match DrawCommand::Effect → egui::PaintCallback (placeholder)
```

### 3.3 Why not a separate mva-effect crate?

A separate crate would require:
- Its own dependency on mva-types (for the model), mva-timeline (for Track evaluation), and mva-scene (for the Scene contract).
- A new trait or interface layer between mva-effect and the rest.

This is justified when: (a) effects have their own complex runtime (shader compilation, GPU resource management), or (b) third-party plugins author effects through a stable ABI. Phase 3 has neither. A separate crate at this stage would be premature abstraction — the effect data model is one struct, and the evaluator is a for-loop.

**Recommendation: no mva-effect crate. Effect is a cross-cutting concern spread across existing crates at their correct layers.**

---

## 4. Image Layer Design

### 4.1 Where does image data live?

The image **reference** (a path or URI) belongs in the persistent model (`LayerKind::Image` in mva-types). The image **pixels** belong to the rendering subsystem (future mva-assets, currently simple file reading in mva-ui).

| Concern | Home | Type |
|---|---|---|
| Layer definition (what image?) | mva-types | `LayerKind::Image { asset: AssetRef }` |
| Evaluated layer (at time t) | mva-scene | `EvaluatedLayerKind::Image { asset: AssetRef }` |
| Draw command | mva-scene | `DrawCommand::Image { asset: AssetRef, transform, opacity }` |
| Pixel loading + painting | mva-ui | Read file → egui texture → paint |

### 4.2 AssetRef placement

`AssetRef` is used by both `mva-types` (persistent model) and `mva-scene` (render IR). Both are leaf crates. The shared leaf they both already depend on is **mva-core**.

```
mva-core::asset::AssetRef   — a new type in mva-core (shared infrastructure)
    ├── mva-types depends on mva-core  →  uses AssetRef in LayerKind::Image
    └── mva-scene depends on mva-core  →  uses AssetRef in DrawCommand::Image
```

Phase 3 definition (minimal — formalized when mva-assets arrives):

```rust
// mva-core (conceptual, not code)
pub enum AssetRef {
    /// File path relative to the project directory (loose directory mode).
    File(String),
    /// Placeholder for future pkg:// URIs (inside .mva container).
    Pkg(String),
}
```

No `Builtin` or `Plugin` variants yet — those arrive with mva-assets (Phase 5).

### 4.3 Image loading — pragmatic Phase 3 approach

mva-ui reads the file:

```
DrawCommand::Image { asset, transform, opacity } →
    1. if let AssetRef::File(path) = asset:
       a. let bytes = std::fs::read(project_dir.join(path))?
       b. let image = image::load_from_memory(&bytes)?  (crate: "image")
       c. let texture = egui_ctx.load_texture(name, image);
       d. emit egui::Shape::image(texture_id, rect, uv);
    2. apply transform + opacity as usual.
```

This is **not** the final architecture — it ties image loading to the UI crate. It is **deliberately temporary**, to be replaced by mva-assets in Phase 5. The interface (`DrawCommand::Image` with `AssetRef`) stays stable; only the loading moves.

---

## 5. Asset Architecture — Re-evaluated

### 5.1 Current state (Phase 2)

No asset subsystem. `AssetRef` is a concept, not a type.

### 5.2 Phase 3 decision: defer mva-assets

**Do not create mva-assets now.** Reasons:

1. Phase 3 has exactly one asset type (images) with one consumer (UI painter). A dedicated crate with registry, cache, and GPU atlasing would be a 300-line crate serving one call site — premature abstraction.

2. The architectural boundary that matters — `mva-format` never touches assets, `mva-renderer` never touches files — is already satisfied. mva-ui loading images directly does NOT violate any separation rule (UI is the terminal layer; it has always been allowed to read files).

3. Creating mva-assets now would force us to design its API before we know what Phase 4–5 actually needs (font atlases for cosmic-text, WGSL compilation, particle texture atlases). Designing against unknown requirements creates the wrong abstraction.

**Trigger for mva-assets (Phase 4–5):** the second asset type (fonts) or the second consumer of images (GPU texture cache for effects). Until then, mva-ui owns image loading.

### 5.3 Dependency constraints maintained

| Constraint | How Phase 3 satisfies it |
|---|---|
| `mva-format → renderer` forbidden | `mva-format` only writes `AssetRef::File(String)` to `LayerKind::Image`. No renderer dep. |
| `mva-core → asset loader` forbidden | `mva-core` defines `AssetRef` (a type, not a loader). No file I/O in core. |
| `mva-renderer` no file I/O | Renderer emits `DrawCommand::Image { asset: AssetRef, ... }`. Never reads the file. |

---

## 6. DrawList Changes

### 6.1 Current DrawCommand (Phase 1.7)

```rust
// mva-scene (conceptual)
pub enum DrawCommand {
    Text(TextDraw),
}
pub struct TextDraw { pub text: String; pub font_family: String; pub font_size: f32; pub transform: EvaluatedTransform; pub color: Color; pub alignment: TextAlignment; }
```

### 6.2 Phase 3 extensions

```rust
// mva-scene (conceptual — three new variants)
pub enum DrawCommand {
    Text(TextDraw),                              // existing
    Image(ImageDraw),                            // NEW
    Effect(EffectDraw),                          // NEW
}

pub struct ImageDraw {
    pub asset: AssetRef,                          // from mva-core
    pub transform: EvaluatedTransform,            // where + how to draw
    pub opacity: f32,                             // 0..1
    pub source_rect: Option<Rect>,                // sub-region of image (future: sprites)
}

pub struct EffectDraw {
    pub effect_id: String,                        // e.g. "mva.opacity_modulate"
    pub params: Vec<(String, f32)>,               // resolved key-value params for this frame
    pub target_rect: Rect,                        // region of the framebuffer affected
    pub input_texture: Option<TextureId>,         // future: previous render pass output (chain effects)
}
```

### 6.3 Renderer pipeline update

```
Scene { layers: Vec<EvaluatedLayer>, effects: Vec<ActiveEffect> }
        │
        ▼
Renderer::render(scene) -> DrawList
        │
        ├─ for layer in scene.layers:
        │     match layer.kind:
        │       Text { content, style } → DrawCommand::Text(...)
        │       Image { asset }          → DrawCommand::Image { asset, transform, opacity }
        │
        └─ for effect in scene.effects:
               DrawCommand::Effect { effect_id, params: effect.resolved_params, target_rect: viewport }
```

The renderer does NOT evaluate effects — it only translates `ActiveEffect` structs (already evaluated by the timeline) into draw commands. The renderer has zero knowledge of what `"mva.opacity_modulate"` does.

### 6.4 UI execution (mva-ui)

```
for cmd in drawlist.commands {
    match cmd {
        DrawCommand::Text(t)   => /* existing: egui text shapes */,
        DrawCommand::Image(i)  => /* NEW: file → egui texture → Image shape */,
        DrawCommand::Effect(e) => /* NEW: placeholder PaintCallback for Phase 3 */,
    }
}
```

Effect placeholder:

```rust
// Phase 3: shape only, no GPU work
DrawCommand::Effect(e) => {
    egui::Shape::Noop  // or a debug rect showing the effect is "registered"
}
```

This is intentionally trivial. It proves the **data path** (EffectTimeline → evaluator → Scene → Renderer → DrawCommand → UI) works. The GPU execution arrives in Phase 4.

---

## 7. GPU Strategy

### 7.1 Three approaches

| | A: Renderer manages GPU | B: UI via PaintCallback (recommended) | C: New mva-gpu crate |
|---|---|---|---|
| **Where wgpu lives** | mva-renderer | mva-ui (eframe already owns wgpu) | mva-gpu |
| **Renderer deps** | +wgpu (heavy) | unchanged | unchanged |
| **Cross-platform** | Renderer must handle all backends | Inherited from eframe | Own implementation |
| **UI-framework independence** | Broken (renderer couples to wgpu) | Maintained (renderer produces DrawCommand::Effect; UI interprets) | Maintained (renderer → mva-gpu trait → wgpu) |
| **Code to write now** | Full GPU pipeline | Zero (placeholder) | New crate + trait + wgpu backend |
| **Phase 3 fit** | Overkill, premature | Perfect — proves boundary exists without building it | Overkill, premature |

### 7.2 Recommendation: Approach B

1. **Phase 3 (now):** `DrawCommand::Effect` reaches the UI. UI produces an egui `PaintCallback` with the effect_id and params stored. The callback does nothing (or draws a debug outline). This validates the entire architectural pipeline without writing a line of GPU code.

2. **Phase 4 (next):** Implement one real effect (e.g., opacity modulation) as a wgpu compute/render pass inside the `PaintCallback`. This is a single wgpu pipeline behind a well-defined interface — the renderer doesn't change.

3. **Phase 5+ (future):** If multiple effects with chaining/compositing become complex, extract the GPU layer into `mva-gpu` behind a `GpuBackend` trait. The commitment point is when we need: multi-pass rendering, render-to-texture, or custom shader hot-reload that the UI crate shouldn't own.

**Cross-platform note:** eframe's wgpu backend already abstracts DX12 (Windows), Vulkan (Linux), Metal (macOS), and OpenGL (fallback). We get cross-platform for free by staying inside the eframe/PaintCallback boundary. No new backends to maintain.

---

## 8. Plugin Strategy — Skeleton Only

### 8.1 What Phase 3 creates

A new `mva-plugin` crate containing **only trait definitions**. No loading. No ABI. No WASM. No dylib.

```rust
// mva-plugin/lib.rs (conceptual)
// depends on: mva-core (for PluginId, AssetRef)

pub trait EffectPlugin: Send + Sync {
    fn plugin_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn effect_id(&self) -> &str;         // which effect this plugin provides
    fn param_schema(&self) -> Vec<EffectParamDef>;  // semantic (name, type, range, default)
}

pub trait VisualizerPlugin: Send + Sync {
    fn plugin_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn generate_scene(&self, t: f64, audio_data: &AudioFrame) -> Scene;
}

pub struct EffectParamDef {
    pub name: String,
    pub param_type: ParamType,           // Float, Int, Bool, Color, Enum
    pub default: ParamValue,
    pub range: Option<(f32, f32)>,
}
```

### 8.2 Why now

The plugin crate exists to validate one architectural claim: **can a third-party concept (an "effect plugin") be expressed entirely through the existing crate boundaries without creating forbidden dependencies?**

Answer: yes. `mva-plugin` depends only on `mva-core` + `mva-scene` (for `Scene`). Plugins produce `Scene` (the same type the timeline produces). The engine can consume plugin-generated scenes alongside timeline-evaluated scenes. No new coupling.

### 8.3 What it does NOT do

- No `PluginHost` / `PluginRegistry` (that's runtime loading, Phase 6).
- No `on_load` / `on_unload` lifecycle (needs loader abstraction).
- No manifest parsing (`plugin.toml` — needs I/O and format crate).
- No WASM bridge (needs extism or wasmtime).
- No native dylib support (needs libloading + ABI design).

The traits exist as an **anchor point** — a statement of intent that future loading infrastructure will satisfy. Phase 3 doesn't use them at runtime. They compile; that's the acceptance criterion.

---

## 9. Phase 3 Dependency Graph

```
                                   ┌────────────┐
                                   │  mva-core  │  config, commands, events,
                                   │            │  errors, ids, AssetRef,
                                   │            │  AudioController(trait),
                                   │            │  ProjectLoader(trait)
                                   └─────┬──────┘
                                         │
           ┌──────────────┬──────────────┼──────────────────────┬───────────────┐
           ▼              ▼              ▼                      ▼               ▼
    ┌───────────┐  ┌───────────┐  ┌───────────┐        ┌───────────┐   ┌───────────┐
    │ mva-types │  │ mva-scene │  │ mva-audio │        │ mva-lyrics│   │ mva-plugin│
    │ Project   │  │ Scene     │  │ rodio     │        │ LRC       │   │ traits    │
    │ Timeline  │  │ DrawCmd   │  │ impl      │        │ parser    │   │ only      │
    │ EffectDef │  │ EvalLayer │  └─────┬─────┘        └─────┬─────┘   └───────────┘
    │ LayerDef  │  └─────┬─────┘        │                    │
    └─────┬─────┘        │              │                    │
          │              │              │                    │
    ┌─────┴──────┐       │              │                    │
    ▼            ▼       │              │                    │
┌─────────┐ ┌────────┐   │              │                    │
│mva-     │ │mva-    │   │              │                    │
│timeline │ │format  │   │              │                    │
│eval     │ │loader  │   │              │                    │
│+extend  │ └───┬────┘   │              │                    │
└────┬─────┘    │        │              │                    │
     │          │        │              │                    │
     │          │  ┌─────┴──────────────┴────────┐           │
     │          │  │        mva-renderer          │           │
     │          │  │ Scene→DrawList               │           │
     │          │  │ (NO mva-timeline dep ✓)      │           │
     │          │  └─────────────┬────────────────┘           │
     │          │                │                            │
     │          │                ▼                            │
     │          │  ┌─────────────────────────────┐            │
     │          │  │           mva-ui            │            │
     │          │  │  egui/eframe + wgpu backend │            │
     │          │  │  paints Text/Image/Effect   │            │
     │          │  │  image loading (Phase 3)    │            │
     │          │  └──────────────┬──────────────┘            │
     │          │                 │                           │
     │          │                 ▼                           │
     │          │  ┌──────────────────────────────┐           │
     │          │  │       mva-player (binary)    │           │
     │          │  │   wires AudioController      │           │
     │          │  │   wires ProjectLoader        │           │
     │          │  │   creates Engine             │           │
     │          │  └──────────────────────────────┘           │
     │          │                                             │
     └──────────┴─────────────────────────────────────────────┘
```

### 9.1 Forbidden-dependency compliance

| Rule | Constraint | Satisfied? | Proof |
|---|---|---|---|
| 1 | `mva-renderer → mva-timeline` | ✓ | Renderer imports only `mva-scene::Scene` (+ `mva-core` types). Zero `use mva_timeline`. |
| 2 | `mva-ui → mva-format` | ✓ | UI never touches file formats. Images loaded as raw bytes in UI, not via format crate. |
| 3 | `mva-core → mva-assets` | ✓ | `mva-assets` doesn't exist. `AssetRef` in core is a plain enum, no file I/O. |
| 4 | `mva-format → mva-player` | ✓ | Format is a library. Binary depends on format, not vice versa. |
| 5 | `mva-scene → any-business-crate` | ✓ | mva-scene depends only on mva-core. No timeline/audio/format/plugin deps. |
| 6 | `mva-types → mva-scene` | ✓ | mva-types depends only on mva-core (for AssetRef, Color, ids). Serialization types stay pure. |
| 7 | Cycle-free | ✓ | Acyclic: mva-core ← everything else; mva-types ← timeline/format; mva-scene ← renderer/timeline/plugin; mva-ui ← renderer; mva-player ← ui + format + plugin. |

### 9.2 New dependencies introduced by Phase 3

| Crate | New dependency | Reason |
|---|---|---|
| mva-core | `serde` (optional for AssetRef) | AssetRef may need serde if mva-types serializes LayerKind::Image that contains it. |
| mva-types | (none new) | Already depends on mva-core. Uses AssetRef from there. |
| mva-scene | (none new) | Already depends on mva-core. ImageDraw and EffectDraw use existing EvaluatedTransform, Color, AssetRef from core. |
| mva-renderer | (none new) | Image/Effect are new match arms on existing Scene→DrawList code. |
| mva-ui | `image` crate (optional feature) | Phase 3 image loading: decode PNG/JPG to RGBA for egui texture upload. Feature-gated: `default = ["image-loading"]`. |
| mva-plugin | `mva-core` + `mva-scene` | Traits reference PluginId (core), Scene (scene), ParamValue (core). |

---

## 10. Implementation Plan

Each step is small, testable, and independently committable.

### Step 1: Data model extension (mva-types)

**What:** Add `EffectTimeline`, `EffectInstance`, `ParamValue` enum, `LayerKind::Image`.

**Files touched:** mva-types only.

**Test:** serde round-trip on `EffectTimeline` (serialize → deserialize → assert_eq!). No runtime behavior changed.

**Rollback:** revert one crate. No other code depends on new types until Step 3.

### Step 2: mva-scene extensions

**What:** Add `EvaluatedLayerKind::Image`, `ActiveEffect` struct to Scene, `DrawCommand::Image`, `DrawCommand::Effect`.

**Files touched:** mva-scene only.

**Test:** `Scene` with `ActiveEffect` entries → serialize/deserialize (if serde enabled). No renderer change yet.

**Rollback:** revert mva-scene. Renderer won't compile until Step 3 (match arms added).

### Step 3: Renderer pipeline

**What:** Extend renderer to handle `Image` layers (emit `DrawCommand::Image`) and `ActiveEffect` entries (emit `DrawCommand::Effect`).

**Files touched:** mva-renderer only.

**Test:** Unit test with hand-built `Scene` containing an Image layer → assert `DrawList` contains exactly one `Image` variant with correct transform. Test with one `ActiveEffect` → assert `DrawList` contains one `Effect` variant with correct params.

**Rollback:** revert renderer. Back to Phase 2 behavior (only Text layers rendered).

### Step 4: Timeline evaluation of effects

**What:** `EffectTimeline::evaluate(t) -> Vec<ActiveEffect>` in mva-timeline. Samples each effect's param tracks at *t*, filters by time_range.

**Files touched:** mva-timeline only.

**Test:** Build `EffectTimeline` with one effect at [10, 20] with animated param. Evaluate at t=15 → assert one `ActiveEffect` with interpolated param value. Evaluate at t=5 → assert empty.

**Rollback:** revert timeline. Effects won't appear in Scenes; renderer is unaffected (zero effects = zero Effect draw commands).

### Step 5: UI — Image painting

**What:** mva-ui: match `DrawCommand::Image` → decode file with `image` crate → upload to egui texture → emit egui `Image` shape with transform + opacity.

**Files touched:** mva-ui only. New dependency: `image` crate (behind feature flag `image-loading`).

**Test:** Manual — open a project with an Image layer, see it painted. Automated: unit test with `DrawCommand::Image` referencing a test fixture PNG → assert egui shapes produced correctly.

**Rollback:** revert UI. Image draw commands are ignored (rendered as nothing).

### Step 6: UI — Effect placeholder

**What:** mva-ui: match `DrawCommand::Effect` → emit an egui `Shape::Rect` with a debug color (proving the path works), or pass to a no-op `PaintCallback`. Add a debug overlay showing "Effect: {effect_id}" in the corner.

**Files touched:** mva-ui only.

**Test:** Manual — open a project with an effect active, see debug rect. Automated: unit test with `DrawCommand::Effect` → assert UI produces the expected placeholder shape.

**Rollback:** revert UI. Effect draw commands are silently dropped.

### Step 7: mva-plugin skeleton

**What:** Create `crates/mva-plugin/` with `EffectPlugin` and `VisualizerPlugin` traits. Cargo.toml: depends on mva-core + mva-scene.

**Files touched:** New crate + workspace `Cargo.toml` (add member).

**Test:** `cargo check` — compiles. No runtime use. No impact on existing crates (nothing depends on mva-plugin yet).

**Rollback:** remove crate from workspace members list + directory.

### Step 8: Integration validation

**What:** mva-player binary: build a test project with one Text layer, one Image layer, and one Effect instance. Run the full pipeline. Verify all three draw commands appear in debug output.

**Test:** Integration test in mva-player. Not a unit test — validates the complete Phase 3 pipeline.

---

## 11. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `AssetRef` design changes when mva-assets arrives | Medium | Low — backward-compat layer | Keep `AssetRef` minimal (2 variants). Migration = add variants, never remove. |
| Image loading in mva-ui couples UI to I/O | Low | Low (acceptable for now) | Already documented as temporary. mva-ui already touches the filesystem (loading fonts via egui). |
| Effect placeholder creates false confidence | Medium | Medium | Acceptance criteria explicitly test that the **data path** works, not the GPU path. Placeholder is clearly labeled "no-op" in code. |
| `mva-plugin` traits too abstract — unused for 3 phases | High | Low | The cost of an unused crate is near-zero (compiles, CI checks). Having it early prevents us from designing the plugin seam AFTER the codebase hardens around a different shape. |
| Renderer match arms grow with each new LayerKind | Low | Medium | Current count: 2 (Text + Image). Growth to ~5 (Shape, Particle, Video) over Phase 4–6 is linear and non-complex. Extract to per-kind modules if >3. |

---

## Phase 3 Architecture Status: **DESIGN — awaiting approval**

No code written. All design decisions above are proposals for review. After approval, implementation proceeds in the 8 steps defined in §10.
