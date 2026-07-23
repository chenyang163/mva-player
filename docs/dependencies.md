# MVA Player — Third-Party Dependencies

Every third-party dependency decision is recorded here (Agent.md rule).

Format:
- **crate-name**: repository / license / used-by / purpose / reason

## Runtime dependencies

### serde
- Repository: https://github.com/serde-rs/serde
- License: MIT OR Apache-2.0
- Used by: `mva-timeline` (and eventually every crate with serializable types)
- Purpose: serialization / deserialization framework (derive) for the data model and config structs
- Reason: mandated by the approved stack (research §9; architecture §3.4, §4). De-facto Rust standard.

### simple-easing
- Repository: https://github.com/julianknodt/simple-easing
- License: MIT OR Apache-2.0
- Used by: `mva-timeline` (eval engine)
- Purpose: named easing evaluation (Penner curves: sine_in, quad_out, cubic_in_out, …)
- Reason: approved by research §6. Phase 1.2 timeline evaluation maps [`Easing::Named`] variants to these functions.

### toml
- Repository: https://github.com/toml-rs/toml
- License: MIT OR Apache-2.0
- Used by: `mva-core` (config loading)
- Purpose: parse `config/*.toml` files into config structs
- Reason: approved by research §9 ("Config | serde + TOML"); Configuration-First rule (Agent.md) requires TOML as the config format.

### thiserror
- Repository: https://github.com/dtolnay/thiserror
- License: MIT OR Apache-2.0
- Used by: `mva-core` (error types)
- Purpose: derive `std::error::Error` for crate-level error enums
- Reason: approved by research §9 ("Error handling | thiserror (libs)"). Each crate exposes one `thiserror` enum per architecture §11.

### mva-scene (internal)
- Used by: `mva-timeline` (eval output), `mva-renderer` (render input)
- Purpose: renderer-independent intermediate representation — Scene, EvaluatedLayer, ComputedTransform, shared types (Vec2, LayerId, BlendMode, TextStyle, Rgba)
- Reason: architecture Phase 1.5 — decouples timeline evaluation from rendering; both crates depend on mva-scene but mva-renderer never depends on mva-timeline.

### rodio
- Repository: https://github.com/RustAudio/rodio
- License: MIT OR Apache-2.0
- Used by: `mva-audio`
- Purpose: audio playback engine (decoding via Symphonia, transport via cpal/WASAPI)
- Reason: approved by research §1. Pure Rust, built-in Symphonia decode chain for MP3/FLAC/WAV, `Player` for play/pause/stop/seek/position. Rodio 0.22+ API (`DeviceSinkBuilder` / `Player` / `Mixer`).

### serde_json
- Repository: https://github.com/serde-rs/json
- License: MIT OR Apache-2.0
- Used by: `mva-format` (`.mva` manifest + `*.anim.json` reading); `mva-timeline` (tests)
- Purpose: canonical serde JSON backend for all serialized MVA artifacts
- Reason: architecture §5, §6 define serialized artifacts as serde JSON (`*.anim.json`, `manifest.json`). Promoted from dev-only to a runtime dependency when `mva-format` gained the loose `.mva` manifest reader (Phase 4 start, first demo).

## Dev dependencies

(none beyond workspace-internal crates)

### clap
- Repository: https://github.com/clap-rs/clap
- License: MIT OR Apache-2.0
- Used by: `mva-player` (binary crate only, not workspace-wide)
- Purpose: CLI argument parsing (positional path, `--demo`, `--help`, `--version`) via derive macros
- Reason: clap 4 is the de‑facto standard Rust CLI parser. The `derive` API produces a compact, type‑safe struct without manual matching. `PathBuf` value parsing uses `OsStr` internally, keeping Windows non‑UTF‑8 paths intact. A hand‑written parser was rejected per Rule 1 (research §4.2; `docs/phase4-architecture.md` §4.2).

## Planned (not yet introduced)

| Crate | Approved by | Milestone | Purpose |
|---|---|---|---|
| `bezier_easing` | research §6 | Phase 2+ | CSS cubic-bezier() evaluation for `Easing::CubicBezier` |
