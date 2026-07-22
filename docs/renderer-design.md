# MVA Phase 1.5 — Renderer Architecture Design

Status: **DESIGN — awaiting approval**
Date: 2026-07-22
Context: `mva-timeline` evaluates into `Scene`, `mva-core` publishes `EngineSnapshot`, `mva-audio` provides `PlaybackClock`.
Goal: design how `mva-renderer` converts `Scene` into paintable output without depending on the timeline engine.

**Hard constraints (from review):**
1. Renderer does NOT depend on `mva-timeline`.
2. Renderer does NOT compute time.
3. Renderer does NOT sync to audio.
4. Renderer ONLY consumes `Scene` / produces `DrawList`.
5. Design must accommodate future: egui editor, GPU backend, video export, image/text/effect layers.

---

## A. Crate Structure — Dependency Inversion via `mva-scene`

**Problem:** current architecture (§3.2) has `mva-renderer` below `mva-timeline` in the graph — it imports `Scene` directly, violating constraint #1.

**Solution:** extract `Scene`, `DrawList`, and all intermediate renderer types into a new **shared-ir crate `mva-scene`**. Both producer and consumer depend on it; neither depends on the other.

### New dependency graph (relevant excerpt)

```
                    ┌────────────┐
                    │  mva-core  │   config, clock, commands, events, errors
                    └─────┬──────┘
                          │
          ┌───────────────┼────────────┐
          ▼               ▼            │
  ┌──────────────┐ ┌─────────────┐     │
  │  mva-audio   │ │ mva-timeline│     │
  │ rodio/cpal   │ │ data model  │     │
  │ symphonia    │ │ + evaluator │     │
  └──────────────┘ └──────┬──────┘     │
                          │            │
                          ▼            │
                   ┌───────────┐       │
                   │ mva-scene │  ◄────┘  (SHARED — pure types, no logic)
                   │ Scene     │           also depended on by: mva-format,
                   │ DrawList  │           mva-plugin, mva-renderer
                   │ TextRun   │
                   └─────┬─────┘
                         │
           ┌─────────────┼──────────────────┐
           ▼             ▼                  ▼
    ┌────────────┐ ┌───────────┐    ┌─────────────┐
    │ mva-format │ │mva-       │    │ mva-plugin  │
    │ .mva IO    │ │renderer   │    │ traits+host │
    │            │ │Scene→     │    └──────┬──────┘
    │            │ │DrawList   │           │
    └────────────┘ └─────┬─────┘           │
                         │                 │
                         ▼                 ▼
                    ┌──────────────┐
                    │    mva-ui    │   egui / eframe (wgpu)
                    └──────┬───────┘
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
      ┌──────────────┐          ┌──────────────┐
      │  mva-player  │          │  mva-editor  │
      └──────────────┘          └──────────────┘
```

Key changes from original architecture §3.2:
- `mva-scene` is a **new leaf crate** depending only on `mva-core` (for `Duration`, `Colors`, ids). It can be `no_std`-friendly for WASM future.
- `mva-timeline` depends on `mva-scene` (to *produce* `Scene`).
- `mva-renderer` depends on `mva-scene` (to *consume* `Scene`). **Zero dependency on `mva-timeline`** — constraint #1 satisfied.
- `mva-format`, `mva-plugin`, `mva-ui` all depend on `mva-scene` as the shared IR, not on the timeline engine.

### Crate responsibility table (updated)

| Crate | Status | Responsibility |
|---|---|---|
| `mva-core` | existing | App glue: config, commands, events, clock trait, errors, ids |
| `mva-scene` | **NEW** | Shared IR: `Scene`, `EvaluatedLayer`, `DrawList`, `TextRun`, `Path`, `Image`, `EffectRef` — pure data, all `Clone + Debug + PartialEq` + (opt) `serde`. No engine logic, no GPU, no font loading. |
| `mva-timeline` | existing | Data model + evaluation: produces `Scene` from `Project + t`. Depends on `mva-scene`. |
| `mva-renderer` | existing | Scene → DrawList pipeline: resolves styles, performs text layout (minimal), culls off-screen layers, sorts by z. Depends on `mva-scene` + `mva-core`. Zero timeline awareness. |
| `mva-ui` | existing | Paints `DrawList` to egui. Depends on `mva-scene` + `mva-renderer`. |
| `mva-format` | future | Reads/writes `.mva`; uses `mva-scene` types for `*.anim.json` serialization. |

`mva-renderer` crate layout:
```
mva-renderer/
├── Cargo.toml          (depends on: mva-core, mva-scene; NO mva-timeline)
├── lib.rs              (re-export public API)
├── pipeline.rs          Pipeline: Scene → DrawList orchestration
├── layer/
│   ├── text.rs          TextLayer layout: resolves TextStyle → TextRun list
│   ├── mod.rs           Layer dispatch (match LayerKind { Text => … })
├── layout/
│   ├── simple.rs        Phase 1.5 simple text layout (no shaping engine)
│   └── mod.rs           LayoutEngine trait (pluggable: simple / harfbuzz / cosmic-text later)
├── style.rs             Style resolution: TextStyle → computed font, color, size, alignment
├── cull.rs              Viewport culling + z-sort
└── draw_list.rs         DrawList builder + dedup/coalesce
```

---

## B. Data Flow — SRP boundaries enforced

```
Per rendered frame (UI thread, ~60fps):

  mva-audio::PlaybackClock
       │
       ▼  t: Duration (atomic, no lock)
  engine / mva-timeline
       │
       ▼
  Scene ───────────────────────────────────┐
  (mva-scene type, flat EvaluatedLayer[])   │  ← evaluator is pure f(t)
       │                                    │
       ▼                                    │
  ┌──────────────────────────────────┐      │
  │ Renderer (mva-renderer)          │      │
  │                                  │      │
  │ ● receives: Scene (all layers)   │      │
  │ ● culls off-screen layers        │      │
  │ ● sorts by z                     │      │
  │ ● for each layer:                │      │
  │     match LayerKind:             │      │
  │       Text → layout → TextRun[]  │      │
  │       Image → ImageRef           │      │
  │       Shape → Path               │      │
  │       (etc. — future)            │      │
  │     apply transform              │      │
  │ ● produces: DrawList             │      │
  └──────────┬───────────────────────┘      │
             │                              │
             ▼                              │
  DrawList ─────────────────────────────────┘
  (mva-scene type, sorted Vec of DrawItem)
       │
       ▼
  ┌─────────────────────────┐
  │ Painter backend         │
  │ (mva-ui, egui impl)      │
  │ ● TextRun → egui::Shape │
  │ ● Path → egui::Shape    │
  │ ● EffectRef → PaintCall │
  └──────────┬──────────────┘
             ▼
         pixels
```

**Why this separation matters:**

| Boundary | Enforces |
|---|---|
| Renderer takes Scene, not Project | Renderer never evaluates time, never runs keyframe sampling — constraint #2 |
| Renderer knows nothing about PlaybackClock | Renderer cannot sync audio — constraint #3 |
| Renderer depends on mva-scene, not mva-timeline | Renderer can be unit-tested with hand-built Scenes (no engine needed) |
| Scene is `Clone + Debug + PartialEq` | Snapshot testing: assert_eq!(renderer.render(scene_golden), expected_drawlist) |
| DrawList is `mva-scene` type | Video exporter, editor preview, or a different UI framework all share the same DrawList type |

---

## C. Scene → DrawList Design

### Scene (defined in `mva-scene`, produced by `mva-timeline::evaluate`)

```rust
// Conceptual — not code, type sketch only

/// The fully-evaluated output of the timeline engine at time t.
/// No animations, no keyframes — every property is a concrete resolved value.
/// Cloneable and comparable for deterministic testing.
pub struct Scene {
    pub time: f64,                     // the t this was evaluated at (read-only, for logging/sync-check)
    pub dims: (f32, f32),             // viewport/canvas size (from config/renderer.toml)
    pub layers: Vec<EvaluatedLayer>,   // flat, z-ordered (high z = on top)
}

pub struct EvaluatedLayer {
    pub id: LayerId,
    pub kind: EvaluatedLayerKind,      // resolved content
    pub transform: EvaluatedTransform, // world-space transform (parenting resolved, rotation in radians)
    pub opacity: f32,                  // 0..1
    pub visible: bool,                 // false if t outside visible_range
    pub blend_mode: BlendMode,         // passed through to GPU later
}

pub enum EvaluatedLayerKind {
    Text {
        content: String,               // already resolved (static text or active lyric line/word)
    },
    Image {
        asset: AssetRef,               // URI from §9 asset system (future: pkg://…)
    },
    Shape {
        path: ShapePath,               // (future)
    },
    // … future: ParticleEmitterSnapshot, VideoFrame, etc.
}

pub struct EvaluatedTransform {
    pub pos: (f32, f32),
    pub scale: (f32, f32),
    pub rotation: f32,
    pub anchor: (f32, f32),            // scale/rotation origin (normalized 0..1 per axis)
}
```

### DrawList (defined in `mva-scene`, produced by `mva-renderer`, consumed by any painter)

```rust
/// Sorted, batched list of GPU-agnostic draw primitives.
/// The painter translates each variant into its platform (egui shapes, wgpu commands, video frames).
pub struct DrawList {
    pub items: Vec<DrawItem>,
}

pub struct DrawItem {
    pub z: i32,
    pub primitive: Primitive,
}

pub enum Primitive {
    /// A run of text with uniform style. Font family/size/style are keys into
    /// the painter's font atlas; the painter handles rasterization.
    TextRun {
        text: String,
        font_family: String,        // e.g. "Noto Sans SC", "default"
        font_size: f32,
        font_weight: FontWeight,    // Normal, Bold (Light/ExtraBold later)
        color: Color,               // RGBA f32
        transform: EvaluatedTransform,  // where + how to draw this run
        alignment: TextAlignment,   // Left, Center, Right
    },
    /// An image blitted to the canvas (future).
    Image {
        asset: AssetRef,
        source_rect: Option<(f32, f32, f32, f32)>,  // u,v,w,h
        transform: EvaluatedTransform,
    },
    /// A filled/stroked 2D path (future).
    Path {
        shape: ShapePath,
        fill: Option<Color>,
        stroke: Option<Stroke>,
        transform: EvaluatedTransform,
    },
    /// Opaque reference to a GPU effect pass (future: PaintCallback).
    Effect {
        effect_id: String,          // matches EffectTimeline::effect_id
        uniforms: Vec<u8>,          // serialized per-effect params for this frame
        clip_rect: (f32, f32, f32, f32),
    },
}
```

### Pipeline: `Renderer::render(Scene, TextStyle) -> DrawList`

1. **Cull** — remove `!visible` layers and layers with `opacity == 0`.
2. **Z-sort** — stable sort by `layer.transform.pos.z` (projected; `z` = depth for stacking).
3. **Per-layer dispatch** (for Phase 1.5, only Text):
   - `EvaluatedLayerKind::Text { content }`:
     a. Resolve `TextStyle` from config defaults (font, size, color from `config/renderer.toml` or per-project `animation.json` style block — for Phase 1.5, project-level styles aren't implemented, so use config defaults).
     b. Text layout: break `content` into lines (split on `\n`), compute per-line bounding rect using simple metrics (line_height = font_size * 1.2; advance = glyph count * approximate glyph width).
     c. Apply alignment: shift each line within the layer's anchor box.
     d. Emit one `TextRun` per line, with the layer's `transform * alignment_offset` as the final world transform.
4. **Ordering:** Text runs inherit the `EvaluatedLayer`'s z-order.
5. **Return** final `DrawList`.

---

## D. Font Rendering Scheme

### Principle: renderer decides WHAT to draw; the painter decides HOW to rasterize.

| Concern | Who owns it |
|---|---|
| Font selection (family, size, weight) | Renderer (resolves `TextStyle` per layer) |
| Text layout (line breaks, alignment, baseline) | Renderer (phase-dependent engine — simple or shaping) |
| Glyph shaping (complex scripts, kerning, BiDi) | Layout engine (future: cosmic-text / harfbuzz) |
| Glyph rasterization (glyph → pixels) | Painter backend (egui Fonts, GPU atlas) |
| Font file loading (`.ttf/.otf`) | Future `mva-assets` → registered into painter's font atlas |
| Caching | Painter backend (texture atlas, already handled by egui) |

### Phase 1.5: Simple Layout Engine

Since Phase 1.5 renders **one lyric line per frame** (a single short string), full text shaping is unnecessary:

- Monospace approximation: glyph advance = `font_size * 0.6` (empirical avg Latin/CJK width).
- Line height = `font_size * 1.4`.
- Alignment: center, left, or right within the viewport width.
- No wrapping, no complex script shaping.

This produces correct-enough results for Latin and CJK lyrics at any scale, and is trivial to replace later.

### Phase 2+: Pluggable Layout Engine

```rust
pub trait LayoutEngine {
    /// Given text + style, return positioned glyph runs.
    fn layout(&self, text: &str, style: &TextStyle, viewport: (f32, f32)) -> Vec<PositionedGlyph>;
}
```

Implementations:
- **SimpleLayout** (Phase 1.5) — monospace approx
- **CosmicLayout** (Phase 2) — wraps cosmic-text for full shaping
- **ParleyLayout** (future) — wraps parley if we adopt the Vello stack

The renderer selects the engine via a `LayoutEngine` trait object set at construction — no timeline dependency.

---

## E. GPU Backend Selection

### Current: Phase 1.5 delegates to egui's built-in wgpu painter

eframe already owns a wgpu instance. egui's `Shape::Text` handles glyph rasterization and texture atlasing. Our `DrawList → egui` conversion in `mva-ui` is thin:

```
DrawList.items ──► match Primitive {
    TextRun { text, font_family, font_size, color, transform, .. } =>
        egui::Shape::text(
            &egui_fonts,
            egui::pos2(transform.pos.0, transform.pos.1),
            color,
            0, // galley index
            egui::text::LayoutJob::simple(text, font_id, color, ..),
        ),
    // Effect { effect_id, .. } => egui::PaintCallback { ... }
    // Image { asset, .. } => egui::Shape::image(...)
}
```

The video exporter future would also consume DrawList — that's the point of the neutral IR.

### Long-term evolution

| Milestone | Backend | Trigger |
|---|---|---|
| Phase 1.5 | egui wgpu painter (built-in) | Phase 1 deliverable |
| Phase 2 | egui PaintCallback for wgpu Effects | Custom particle / spectrum shaders |
| Phase 3+ | Full custom wgpu renderer behind `RenderBackend` trait | Performance: draw batching, custom font atlas, GPU text (glyphon) |
| Phase 5 | Video export backend (DrawList → CPU raster → encoder frames) | Editor export |
| Phase 6 | Vello backend (DrawList → Vello scene) | Vello stable 1.x + GPU text via parley; alternative to wgpu for declarative 2D |

The architecture requires no changes to reach any of these — each is a new implementation of the same painter contract, consuming the same `DrawList`.

---

## F. Phase 1.5 Minimum Implementation Scope

### New crate: `mva-scene`
- `Scene`, `EvaluatedLayer`, `EvaluatedLayerKind`, `EvaluatedTransform`, `BlendMode`
- `DrawList`, `DrawItem`, `Primitive::{TextRun, Image, Path, Effect}`
- `TextStyle` (font family, size, weight, color, alignment)
- `Color`, `FontWeight`, `TextAlignment`
- Depends: `mva-core` (for `LayerId`, `AssetRef` placeholder, time types)

### Update: `mva-timeline`
- `Cargo.toml`: add `mva-scene` dependency
- `evaluate(t) -> mva_scene::Scene` (previously returned own `Scene` type)
- Move existing `Scene` model bits into `mva-scene` if they were there; implement `From`/`Into` if needed

### Update: `mva-renderer` (Phase 1.5 implementation)
- `Cargo.toml`: depends on `mva-scene`, `mva-core`; **remove `mva-timeline` dep**
- `renderer.rs`: public fn `render(scene: &Scene, style: &TextStyle, config: &RendererConfig) -> DrawList`
- `layer/text.rs`: Text layer → `TextRun` (simple layout engine inline)
- `cull.rs`: visibility + opacity cull
- `draw_list.rs`: z-sort + build DrawList

### New config file: `config/renderer.toml`
```toml
[viewport]
width = 800
height = 600

[defaults]
font_family = "default"
font_size = 42.0

[lyric_text]
color = [1.0, 1.0, 1.0, 1.0]       # RGBA
alignment = "center"
line_height_ratio = 1.4
```

Phase 1 `config/animation.toml` already covers fade/scale parameters; the renderer reads `RendererConfig` for static text styling and `AnimationConfig` for the animated properties (which the timeline evaluates into the `EvaluatedLayer`).

### Acceptance criteria for Phase 1.5 renderer
1. `mva-renderer` compiles with **zero** `use mva_timeline` — constraint #1 proven.
2. `Scene` is produced by a pure function (no clock, no audio) — constraint #2 proven.
3. Unit test: hand-build a `Scene` with one text layer → `render()` returns expected `DrawList.TextRun` with correct transform (position + scale from `EvaluatedTransform`).
4. Unit test: viewport cull drops an off-screen layer.
5. Integration: engine produces `Scene` at `t`, renderer gives `DrawList`, UI paints it (Phase 1 full pipeline test).
6. No unwrap in library code.
