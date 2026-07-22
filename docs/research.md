# MVA Player — Technical Research

Research date: 2026-07-22
Method: live data from crates.io API, GitHub API, official documentation, and project READMEs.
All version numbers, licenses, and activity dates were verified against those sources on the research date.

Scope: technology selection for a Windows-first, cross-platform Rust music visual player
(audio + synchronized lyrics + text animation + GPU effects + plugins + editor).

---

## 1. Rust Audio Playback Libraries

| Library | Latest version (date) | License | Maintenance | Downloads (total) |
|---|---|---|---|---|
| rodio | 0.22.2 (2026-03) | MIT OR Apache-2.0 | Very active (pushed 2026-07) | ~9.3M |
| cpal | 0.18.1 (2026-06) | Apache-2.0 | Very active | ~16.3M |
| kira | 0.12.2 (2026-07) | MIT OR Apache-2.0 | Very active | ~826k |
| oddio | 0.7.4 (2023-10) | MIT OR Apache-2.0 | Unmaintained | ~53k |
| alto (OpenAL) | 3.0.4 (2019-05) | MIT/Apache-2.0 | Unmaintained since 2019 | ~152k |
| sdl2 (+mixer) | 0.38.0 (2025-07) | MIT | Active, needs native SDL2 DLLs | ~4.8M |
| openal | 0.2.2 (2016) | WTFPL | Abandoned | ~11k |

### rodio
High-level playback engine built on cpal. Since 0.21, **Symphonia is the default decoder
for all formats** (mp3, flac, mp4/aac, vorbis, wav enabled by default). f32 sample pipeline,
`Player::try_seek()`, `Player::get_pos()` (sample-counted, speed-aware position —
suitable for lyric sync), volume/speed control, mixing, effects (fade, crossfade, filters).
Pure Rust on Windows (WASAPI via cpal), zero native dependencies.

- Advantages: simplest complete API; pure-Rust decode chain; actively developed; position
  tracking and seek built in; everything Phase 1 needs out of the box.
- Disadvantages: frequent breaking changes between minor versions (0.20 → 0.21 → 0.22);
  upgrade churn must be expected.

### cpal
Low-level cross-platform audio I/O (WASAPI/ASIO on Windows, CoreAudio, ALSA/PipeWire/JACK,
AAudio, Web Audio). Full control over buffers and latency, but **no** mixer, decoder,
resampler, or position tracking — we would have to build everything ourselves.

- Advantages: maximum control; the foundation under rodio and kira.
- Disadvantages: far too low-level for a music player frontend on its own.

### kira
Game-oriented audio engine on top of cpal with Symphonia decoding. Smooth tweens/fades,
mixer sub-tracks, effect chains, a clock system for precisely timed events,
`seek_to()`/`position()` (seconds, f64) on handles. Most testing happens on Windows.

- Advantages: built-in smooth parameter fades; precise event scheduling; clean position API.
- Disadvantages: game-centric (no track queue concept); smaller community than rodio.

### oddio / alto / sdl2 / openal
- oddio: unmaintained since 2023, low-level, no decoding — rejected.
- alto/openal: abandoned, require OpenAL runtime — rejected.
- sdl2: maintained, but requires shipping SDL2/SDL2_mixer DLLs on Windows and offers a
  basic audio API — rejected (native dependency friction).

**Conclusion (playback): rodio 0.22** is the mainstream, actively maintained choice with
decoding, seek, and position tracking included. kira is the credible alternative if we ever
need sample-scheduled effects. cpal only if we outgrow both.

---

## 2. Rust Audio Decoding Libraries

| Library | Latest version (date) | License | Maintenance | Scope |
|---|---|---|---|---|
| symphonia | 0.6.0 (2026-05) | MPL-2.0 | Very active | MP1/2/3, FLAC, AAC, ALAC, Vorbis, PCM, ADPCM + OGG/MKV/MP4/WAV/AIFF/CAF containers |
| minimp3 | 0.6.1 (2025-08) | MIT | Light | MP3 only (C library bindings) |
| hound | 3.5.1 (2023-09) | Apache-2.0 | Stable | WAV only |
| claxon | 0.4.3 (2020-08) | Apache-2.0 | Dormant | FLAC only |
| lewton | 0.10.2 (2021-01) | MIT/Apache-2.0 | Largely unmaintained | Vorbis only |
| audrey | 0.3.0 (2021-01) | MIT/Apache-2.0 | Unmaintained | Wrapper over claxon/lewton/hound |
| ffmpeg-next | 8.1.0 (2026-03) | WTFPL (FFmpeg itself LGPL/GPL) | Very active | Everything FFmpeg decodes |

### symphonia
Pure Rust media container + audio decoding. Accurate and coarse seek modes, gapless support,
metadata (ID3v1/v2, APE, Vorbis comments). It is the default decoder inside both rodio and
kira. MPL-2.0 is file-level copyleft — compatible with our use (with attribution).

- Advantages: one crate covers every format in scope (MP3/FLAC/WAV and beyond); pure Rust;
  no native dependencies; actively maintained; already included via rodio.
- Disadvantages: decode-only (no encoding); MPL-2.0 slightly more restrictive than MIT.

### Single-format crates (minimp3, hound, claxon, lewton, audrey)
All are superseded by symphonia for our purposes (hound remains relevant only if we ever
write WAV files, e.g. export). audrey is dead. Rejected as redundant.

### ffmpeg-next
Safe FFmpeg bindings. Decodes virtually anything, but requires FFmpeg dev libraries at build
time and FFmpeg DLLs at runtime — a major packaging burden on Windows, plus LGPL/GPL
considerations for the linked FFmpeg build.

- Advantages: universal format coverage.
- Disadvantages: build/deployment complexity; licensing care; overkill while symphonia
  covers our formats.

**Conclusion (decoding): symphonia 0.6**, consumed through rodio (which already bundles it).
Direct symphonia use stays available behind our decoder abstraction if we need
sample-accurate control (e.g. for a future visualizer FFT tap). ffmpeg-next is the documented
fallback for exotic formats.

---

## 3. Rust UI Frameworks

| Framework | Latest (date) | License | Rendering | Windows | Custom GPU shaders | Text quality | Built-in animation | Backing |
|---|---|---|---|---|---|---|---|---|
| Slint | 1.17.1 (2026-07) | GPLv3 / Royalty-free / Commercial | Skia, FemtoVG, software | Excellent | GL/wgpu interop only | parley/swash, no glyph API | Yes, declarative | SixtyFPS GmbH |
| Dioxus Desktop | 0.7.9 (2026-05) | MIT/Apache | WebView2/webkit | Excellent | WebGL/WebGPU in webview | Browser-grade (best) | CSS/WAAPI | Dioxus Labs |
| Iced | 0.14.0 (2025-12) | MIT | wgpu + tiny-skia fallback | Good–very good | Yes (wgpu shader widget) | cosmic-text | Yes (new in 0.14) | hecrj + Kraken; COSMIC |
| egui/eframe | 0.35.0 (2026-06) | MIT/Apache | wgpu (default) or glow | Very good | Yes (PaintCallback) | harfrust shaping; glyph positions via Galley | Helpers + immediate mode | emilk + Rerun |
| Tauri | 2.11.5 (2026-07) | MIT/Apache | WebView2/webkit | Excellent | WebGL/WebGPU | Browser-grade | CSS/WAAPI | Tauri Foundation |
| floem | 0.2.0 (2024-11; git active) | MIT | wgpu (vger/vello) | OK | Partial | parley/fontique | Transitions + keyframes | Lapce (1 maintainer) |
| GPUI | 0.2.2 (2025-10) + git | Apache-2.0 | Custom GPU, DirectWrite text | Production via Zed | Partial | Editor-grade | Yes | Zed Industries |

### Candidates assessment

**Slint** — mature stable 1.x API, excellent declarative animation system, tiny footprint,
broadest platform support. Disadvantages: triple license (GPL-3.0 / royalty-free /
commercial) creates friction for a project whose license we have not fixed; no public
glyph-level text API (karaoke-style per-word effects need workarounds); custom shaders only
via GL/wgpu texture interop.

**Dioxus Desktop** — browser-grade text/animation (ideal for karaoke lyrics), Rust RSX
codebase, hot reload. Disadvantages: pre-1.0 churn; depends on the WebView2 runtime; UI
runs in a webview (not native GPU); UI↔Rust bridge needs care at 30–60 Hz lyric updates.

**Iced** — pure-Rust Elm architecture, MIT, cosmic-text shaping, first-class custom wgpu
shader widget (official `custom_shader` example), new `Animation` API in 0.14, tiny-skia
software fallback. Disadvantages: 0.x breaking changes, ~1 release/year, sparse docs.
(Notable proof-of-concept: `Rustle`, an Apple-Music-style lyrics player built on iced.)

**egui/eframe** — immediate mode, MIT/Apache, most-used pure-Rust GUI (~20M downloads),
very frequent releases, wgpu backend by default with software fallback path, first-class
custom GPU drawing via `PaintCallback` (wgpu/glow), text shaping upgraded to
harfrust/skrifa in 0.35, laid-out `Galley`s expose glyph positions for custom per-glyph
painting, huge third-party ecosystem (node graphs, timeline widgets, file dialogs).
Immediate mode maps naturally onto "recompute lyric highlight and visualizer every frame",
and is the dominant paradigm for editor-style tooling (Rerun etc.) — which matters for the
future MVA Creator (After-Effects-style timeline). Disadvantages: non-native look;
breaking changes between releases; more UI polish is hand-rolled than with Slint/web tech.

**Tauri** — most mature "web UI + Rust core" option, but the frontend is JavaScript/TypeScript.
Agent.md mandates Rust for core functions; a JS frontend would also split lyric-timing logic
across an IPC boundary. Rejected for this project (documented as the industry-common
alternative — Nuclear, Audion, WaveFlow use it).

**floem** — attractive on paper (signals + built-in animations + parley text), but no
crates.io release since 2024-11, tiny community, single-maintainer bus-factor. Rejected (high risk).

**GPUI** — outstanding text/performance DNA (DirectWrite on Windows, proven by Zed), but not
productized: requires git dependencies, minimal docs, API tracks Zed's needs. Watchlist —
re-evaluate in 6–12 months.

**Microsoft / windows-rs** — no mature Microsoft Rust UI framework exists. windows-rs offers
raw Win32/Direct2D/DirectWrite bindings; WinUI 3 has no viable Rust projection. Hand-rolling
a toolkit is rejected.

**Conclusion (UI): egui + eframe (wgpu backend).**
Rationale: (1) immediate mode is the natural fit for per-frame lyric animation and
visualizers; (2) first-class custom wgpu shader integration covers the effects roadmap;
(3) glyph-position access enables word-level karaoke rendering later; (4) the editor use
case (timeline, keyframes) is egui's strongest domain; (5) permissive license, largest
community, fastest iteration. Iced is the documented runner-up (retained mode, MIT,
cosmic-text) if egui's non-native look becomes a problem.

---

## 4. Lyrics / Subtitle Parsing Libraries

| Crate | Latest (date) | License | Maintenance | Notes |
|---|---|---|---|---|
| lrc | 0.2.0 (2026-07) | MIT | Active again (revived) | Standard line-timed LRC + ID tags; no word-level |
| lrc_rs | 0.1.3 (2026-06) | MIT | Brand new | A2 extension (word-by-word `<mm:ss.xx>`); unproven |
| lrc-nom | 0.3.0 (2024-04) | MIT | Dormant | Zero-copy standard LRC |
| amll-lyric | 0.3.0 (2025-07) | **GPL-3.0** | Active | LRC/ESLRC/TTML/YRC/QRC/LYS/ASS, word-level — GPL is a blocker |
| ttml_processor | 0.3.3 (2026-05) | MIT | Very active | TTML lyrics (Apple Music / AMLL dialects), parse + generate |
| lyrics-helper | 0.1.5 (2026-07) | Apache-2.0 | Brand new | Parse/decrypt/convert/search many providers (NetEase, QQ, Apple, LRCLIB, Spotify…) |
| subtitler | 2.6.1 (2026-07) | Apache-2.0 | Very active | 15 caption formats incl. SRT/VTT/ASS/TTML/LRC, cue-level |
| ass_parser | 0.2.3 (2025-04) | MIT/Apache-2.0 | Moderate | ASS parse/edit, no rendering |
| ass-editor (+ass-core) | 0.1.2 (2026-07) | MIT | Active | Ergonomic ASS editing layer, young |
| libass-sys | 0.1.2 (2020-11) | ISC | Unmaintained | FFI to C libass (CPU rasterizer) |
| srtparse / vtt / subrip | various | MIT | Various | SRT/VTT only |

Findings:
- Standard line-level LRC is well covered by `lrc` (MIT, simple, maintained again).
- Word-level (karaoke) lyrics are the project's differentiator, but the ecosystem is young:
  `lrc_rs` and `lyrics-helper` are weeks old; the most complete implementation
  (`amll-lyric`) is GPL-3.0 and cannot be used.
- ASS karaoke effects (`\k`, `\kf`, transforms) are only fully rendered by C libass
  (stale FFI bindings, CPU rasterization) — it does not fit our GPU pipeline. Pure-Rust ASS
  crates parse but do not render.
- LRCLIB (lrclib.net) is the de-facto free synced-lyrics API (used by Strawberry and
  several 2025-2026 Rust players) — relevant for a future online-lyrics feature, not Phase 1.

**Conclusion (lyrics):**
- Define our own format-neutral `LyricTrack` data model (lines + optional word-level timing)
  with a `LyricParser` trait per format (this also future-proofs the plugin system).
- Use the `lrc` crate for standard LRC in Phase 1.
- Implement enhanced (word-level) LRC parsing ourselves later behind the same trait — the
  format is trivial, and no mature permissively-licensed crate exists (documented exception
  to the "don't reinvent" rule, justified by ecosystem gaps above).
- ASS: evaluate `ass-editor`/`ass-core` when ASS support lands; rendering will be our own
  GPU text pipeline, not libass.
- TTML: adopt `ttml_processor` (MIT) when Apple-Music-style lyrics are scheduled.

---

## 5. GPU Rendering Solutions

| Library | Latest (date) | License | Maintenance | Role |
|---|---|---|---|---|
| wgpu | 30.0.0 (2026-07) | MIT/Apache-2.0 | Very active | De-facto safe GPU API (Vulkan/Metal/DX12/GL/Web) |
| glyphon | 0.12.0 (2026-07) | MIT/Apache/Zlib | Active | Text rendering on wgpu (cosmic-text + swash atlas) |
| cosmic-text | 0.19.0 (2026-04) | MIT/Apache-2.0 | Active | Pure-Rust text shaping/layout, BiDi |
| parley | 0.11.0 (2026-06) | Apache/MIT | Very active | Linebender rich-text layout |
| vello | 0.9.0 (2026-05) | Apache/MIT | Very active | Experimental GPU 2D renderer on wgpu |
| femtovg | 0.26.0 (2026-07) | MIT/Apache-2.0 | Active | Canvas-like 2D on OpenGL (wgpu backend optional) |
| tiny-skia | 0.12.0 (2026-02) | BSD-3-Clause | Stable | CPU-only 2D rasterization (fallback) |
| skia-safe | 0.99.0 (2026-06) | MIT | Very active | Full Skia bindings (incl. Lottie via skottie); heavy C++ build |

Findings:
- eframe already embeds wgpu: custom visualizer shaders integrate via `egui::PaintCallback`
  without adding a new windowing/rendering stack.
- egui's own painter + `Galley` glyph positions cover Phase 1–2 lyric text.
- If/when lyric text outgrows egui (complex per-glyph effects at scale), the documented
  upgrade path is wgpu + glyphon/cosmic-text, or the Linebender stack (parley + vello)
  — both pair with the wgpu instance we already own.
- skia-safe is the most complete single library (text + effects + Lottie) but brings an
  enormous C++ build and binary weight — rejected for now.
- tiny-skia: CPU fallback only. femtovg: nice canvas API, but redundant once we speak wgpu.

**Conclusion (GPU): wgpu via eframe + `egui::PaintCallback` for effects; egui painter for
text initially; glyphon/cosmic-text (or parley/vello) as the documented upgrade path.**

---

## 6. Animation Engine Solutions

| Library | Latest (date) | License | Maintenance | Suitability |
|---|---|---|---|---|
| keyframe | 1.1.1 (2022-07) | MIT | Dormant (~4 yrs) | Generic keyframe sequences + easings; proven, but stale |
| tween | 2.2.0 (2026-02) | MIT/Apache-2.0 | Recently revived | Game-oriented tweener, renderer-agnostic |
| simple-easing | 1.0.2 (2026-02) | MIT/Apache-2.0 | Updated | Minimal Penner easing set |
| easer | 0.3.0 (2022-08) | MIT | Dormant but complete | Penner easing functions, generic |
| interpolation | 0.3.0 (2023-08) | MIT | Dormant | Lerp/ease primitives (Piston) |
| bezier_easing | 0.3.0 (2026-05) | Unknown | Active | CSS cubic-bezier() port |
| bevy_tweening / bevy_easings / bevy_tween | active | MIT/Apache | Active | Hard-bound to Bevy ECS — not usable outside Bevy |
| spanda / animato | 2026 | MIT/Apache | Brand new | Tween+timeline newcomers, unproven |
| rive-rs | no crate (git, 2025-07) | MIT | Stalled | Rive runtime; not practically usable from crates.io |
| dotlottie-rs | git (crates.io stuck at 0.1.0-alpha) | MIT | Very active | Lottie/dotLottie player (ThorVG core), CPU frames |
| velato | 0.11.0 (2026-07) | Apache/MIT | Active | Lottie player rendering through vello (GPU) |
| `fluster` (linebender) | — | — | — | Verified: does not exist (404) |

Findings:
- No mature, renderer-agnostic Rust "timeline animation engine" (After-Effects-style tracks
  with keyframes on position/scale/rotation/opacity) exists outside the Bevy ecosystem.
- The animation model is core domain IP of MVA Player (Agent.md: animation must be
  data-driven, e.g. `animation.json` with time/text/effect). A timeline evaluator
  (sample keyframes → ease → interpolate) is small, well-understood, and must match our
  data model exactly — a justified self-implementation.
- Easing curves and cubic-bezier evaluation are commodity: reuse a maintained easing crate
  instead of writing easing math.

**Conclusion (animation): self-implement the data-driven timeline/track/keyframe evaluator
(small, domain-specific), reuse `simple-easing` (or `bezier_easing` for CSS-style curves)
for easing functions. Optionally support Lottie assets later via velato (if we adopt vello)
or dotlottie-rs. Rive: not viable in Rust today.**

---

## 7. Plugin Architecture Solutions

The fundamental problem: **Rust has no stable ABI.** Native `.dll` plugins compiled with a
different rustc can crash the host. Every solution works around this differently.

| Solution | Latest | License | Maintenance | Sandboxing | Cross-language | Notes |
|---|---|---|---|---|---|---|
| libloading (+ hand C ABI) | 0.9.0 | ISC | Active | None | Yes (any C-ABI language) | Zero-overhead loader; ABI design is on us |
| abi_stable | 0.11.3 | MIT/Apache | **Stalled since 2023-10** | None | No (Rust↔Rust) | Ergonomic, but maintenance risk |
| stabby | 72.1.16 | EPL-2.0/Apache-2.0 | Very active (ZettaScale) | None | Rust-focused | Modern abi_stable alternative |
| extism | 1.30.0 (host SDK) | BSD-3-Clause | Active (Dylibso) | **Yes (WASM)** | **Yes (Rust/JS/Go/C/C#/Zig/Haskell/AssemblyScript PDKs)** | Fastest path to sandboxed cross-language plugins |
| wasmtime | 47.0.2 | Apache-2.0 | Extremely active (Bytecode Alliance) | Yes | Yes (any wasm32) | Full control; Component Model + WIT host embedding |
| wasmer | 7.2.0 | MIT | Active | Yes | Yes | Alternative runtime; ecosystem centers on wasmtime |
| mlua (Lua) | 0.12.0 | MIT | Active | Capability-based | No | Themes, keybindings, lyric-provider scripts |
| rhai | 1.25.1 | MIT/Apache-2.0 | Active | Capability-based + resource limits | No | Rust-native scripting (cf. fooyin's FooScript) |
| boa (JS) | 0.21.1 | MIT | Very active | Capability-based | No | Pure-Rust JS engine, no JIT |
| deno_core (V8) | 0.408.0 | MIT | Crate maintained in deno monorepo | Yes | No | Huge build/binary cost — rejected |
| WASM Component Model + WIT | WASI 0.3 ratified 2026-06 | Apache-2.0 | Rapidly maturing | Yes | Yes (typed WIT interfaces) | The long-term "official" answer; WASI 0.2 is today's safe baseline |

Findings from existing players:
- musikcube (C++) is the canonical native-plugin player: plugins for outputs, decoders,
  DSP, tag readers, visualizations — the interface taxonomy maps 1:1 onto our plugin goals.
- fooyin (C++/Qt) adds embedded scripting (FooScript) to a plugin architecture.
- Nuclear (Tauri) shows the best distribution UX: a plugin store + JS SDK.
- No mature Rust player ships a binary/WASM plugin system today — open territory.
- termusic demonstrates out-of-process plugins/components via gRPC (crash isolation).

**Conclusion (plugins): phased hybrid.**
1. **Now (Phase 1–2):** in-tree trait abstractions (`DecoderPlugin`, `LyricProvider`,
   `Visualizer`, `Effect`) compiled in — the termusic/ncspot pattern. Defines the
   interfaces with zero ABI risk.
2. **Untrusted community plugins (lyrics, metadata, themes, tools): WASM via extism**
   (or wasmtime + WIT once the Component Model surface settles) — sandboxed,
   cross-language, distributable as `.wasm` files (Nuclear-style store UX later).
3. **Native performance plugins (visualizers, DSP): C-ABI dylibs via libloading**
   (musikcube pattern), or `stabby` for Rust↔Rust (abi_stable rejected: stalled).
4. **Themes/automation:** declarative files + rhai or mlua scripting.
5. The real-time audio path never crosses WASM/FFI boundaries at sample rate.

---

## 8. Existing Music Player Projects

| Project | Lang/UI | Audio backend | Lyrics | Plugins | License | Stars | Lessons |
|---|---|---|---|---|---|---|---|
| termusic | Rust, ratatui TUI | symphonia + rodio (optional GStreamer/mpv), gRPC TUI↔server split | Yes (download + embed) | No (compile-time backends) | MIT/GPLv3 | 2.2k | Backend-trait pattern; our exact audio stack in production |
| psst | Rust, druid (own fork) | symphonia + cpal | No | No | MIT | 9.4k | Decode→cpal pipeline; warning: dormant GUI = maintain a fork |
| ncspot | Rust, cursive TUI | librespot (rodio) | No | No | BSD-2 | 6.7k | Feature-gated backends |
| Amberol | Rust, GTK4 | GStreamer (gst-play), lofty tags | No | No | GPL-3.0 | 112 | Modern Rust-GNOME stack |
| Audion | Tauri + Svelte | unverified | Synced | Claims themes+plugins | none | 479 | Closest feature analog on Tauri |
| Rustle | Rust, **iced** | unverified | Apple-Music-style | No | AGPL-3.0 | 19 | Lyric-player pattern in pure Rust |
| musikcube | C++, curses | **C-ABI plugin architecture** (outputs/decoders/DSP/tags/visualizers) | Unknown | **Yes** | BSD-3 | 4.8k | Plugin interface taxonomy blueprint |
| fooyin | C++, Qt6 | FFmpeg | Yes (search/edit/sync) | Yes + FooScript | GPL-3.0 | 2.1k | Extensibility spiritual analog |
| Nuclear | TS, Tauri + React | web sources | Unknown | **Yes, store + JS SDK** | AGPL-3.0 | 18k | Plugin distribution UX |
| supersonic | Go, Fyne | libmpv | Yes (dedicated widget) | No | GPL-3.0 | 2.3k | libmpv fallback option |
| Strawberry | C++, Qt6 | GStreamer | Yes, 8 providers incl. LRCLIB | No | GPL-3.0 | 3.8k | Lyric-provider fallback chain |

Reusable component findings: `lofty` (audio metadata/tags — used by termusic, Amberol),
`souvlaki`/`mpris-server` (OS media integration), LRCLIB (free synced-lyrics API).

---

## 9. Final Recommendation — MVA Player Technology Stack

| Concern | Choice | Why (one line) |
|---|---|---|
| Language | Rust (edition 2024 toolchain) | Project mandate |
| Audio playback | **rodio 0.22** | Pure Rust, decoding+seek+position built in, active |
| Audio decoding | **symphonia 0.6** (via rodio) | Every target format, zero native deps |
| Audio metadata (later) | lofty | Ecosystem standard (termusic, Amberol) |
| UI framework | **egui 0.35 + eframe (wgpu backend)** | Immediate mode = per-frame lyric FX; PaintCallback for shaders; editor-friendly; MIT/Apache; biggest community |
| GPU effects | wgpu (inside eframe) via `egui::PaintCallback` | No extra stack; WGSL shaders |
| Lyric text (Phase 1–2) | egui painter (+ `Galley` glyph positions later) | Sufficient for line sync + simple animations |
| Lyric text (upgrade path) | glyphon / cosmic-text (or parley + vello) | GPU text when per-glyph effects demand it |
| Lyrics parsing | `lrc` crate now; `LyricParser` trait + own enhanced-LRC later; `ttml_processor` for TTML later | Only permissive, maintainable path (amll-lyric is GPL) |
| Animation | Own data-driven timeline evaluator + `simple-easing` | Core domain IP; no mature engine exists outside Bevy |
| Plugins | Traits now → extism (WASM) for community plugins + libloading for native visualizers later | Sandboxing where untrusted; zero overhead where hot |
| Scripting (themes/automation, later) | rhai or mlua | Proven embedded scripting |
| Config | serde + TOML (`toml` crate) | Configuration-first rule (Agent.md) |
| Error handling | thiserror (libs) / anyhow (app) | Rust best practice |
| OS media keys / MPRIS (later) | souvlaki | Used by psst/termusic |

### Rejected (with reasons)
- **cpal alone** — too low-level (build our own mixer/decoder/tracking).
- **kira** — credible, but game-centric; rodio's queue/position model fits better.
- **ffmpeg-next** — native build/deploy burden; symphonia covers our formats.
- **Slint** — license friction (GPL/royalty-free/commercial), no glyph-level text API.
- **Dioxus/Tauri** — webview dependency; Tauri needs a JS frontend (violates the
  Rust-for-core rule); pre-1.0 churn (Dioxus).
- **Iced** — strong runner-up; slower release cadence, sparser docs than egui.
- **floem** — single-maintainer risk, no crates release since 2024-11.
- **GPUI** — not productized (git deps, docs minimal); watchlist.
- **windows-rs / hand-rolled Win32** — no mature Microsoft Rust UI framework exists.
- **amll-lyric** — GPL-3.0 incompatible with our undecided license.
- **libass (FFI)** — stale bindings, CPU rasterizer, conflicts with GPU pipeline.
- **abi_stable** — maintenance stalled since 2023-10 (use stabby if Rust↔Rust needed).
- **rive-rs** — stalled, not on crates.io.
- **Bevy** (and bevy_tweening) — full game engine is the wrong frame for a desktop app.

### Risks / watchlist
1. rodio breaking-change churn → pin versions, read UPGRADE.md on bumps.
2. egui breaking changes between releases → same policy; upgrade deliberately.
3. Word-level lyric parsing (enhanced LRC) will be self-built → keep behind `LyricParser`
   trait; if `lrc_rs`/`lyrics-helper` mature, adopt.
4. WASM Component Model (WASI 0.3 ratified 2026-06) may displace extism's ABI →
   revisit at plugin milestone.
5. GPUI productization → re-evaluate UI choice in 6–12 months (only if egui hits a wall).
6. Slint's royalty-free license is actually desktop-friendly — if declarative UI + stable
   API ever outweighs egui's flexibility, it remains the safest commercial-grade fallback.

### Sources (primary)
- crates.io API: rodio, cpal, kira, symphonia, minimp3, hound, claxon, lewton, audrey,
  ffmpeg-next, slint, dioxus, iced, egui/eframe, tauri, floem, gpui, windows, lrc, lrc_rs,
  lrc-nom, amll-lyric, ttml_processor, lyrics-helper, subtitler, ass_parser, ass-editor,
  libass-sys, wgpu, glyphon, cosmic-text, parley, vello, femtovg, tiny-skia, skia-safe,
  keyframe, tween, simple-easing, easer, bezier_easing, bevy_tweening, velato, rive-rs,
  libloading, abi_stable, stabby, extism, wasmtime, wasmer, mlua, rhai, rune, boa.
- GitHub: RustAudio/rodio, RustAudio/cpal, tesselode/kira, pdeljanov/Symphonia,
  slint-ui/slint, DioxusLabs/dioxus, iced-rs/iced, emilk/egui, tauri-apps/tauri,
  lapce/floem, zed-industries/zed, microsoft/windows-rs, gfx-rs/wgpu, linebender/*,
  femtovg/femtovg, rust-skia/rust-skia, grovesNL/glyphon, pop-os/cosmic-text,
  nagisa/rust_libloading, rodrimati1992/abi_stable_crates, ZettaScaleLabs/stabby,
  extism/extism, bytecodealliance/wasmtime, wasmerio/wasmer, tramhao/termusic,
  jpochyla/psst, hrkfdn/ncspot, clangen/musikcube, fooyin/fooyin, nukeop/nuclear,
  strawberrymusicplayer/strawberry, dweymouth/supersonic, LottieFiles/dotlottie-rs.
- WASI 0.3.0 release notes (github.com/WebAssembly/WASI), Bytecode Alliance blog.
