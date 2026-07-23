# MVA Player — Project Status

Date: 2026-07-24

## Overall

| Metric | Value |
|--------|-------|
| **Version** | v0.2.0 |
| **Phase** | Phase 4 Complete |
| **Architecture Score** | 9.5 / 10 |
| **Tests** | 190 passing |
| **Clippy** | Clean |

## Phase Status

| Phase | Status |
|-------|--------|
| Phase 1 — Workspace & Core |   Complete |
| Phase 2 — Timeline & Lyrics |   Complete |
| Phase 3 — Images & Effects |   Complete |
| Phase 4 — Application Workflow & Project Loading |   Complete |

## Architecture Health

| Criterion | Score | Notes |
|-----------|-------|-------|
| Crate boundaries | 10/10 | Strict acyclic dependency graph, leaf crate at base |
| Dependency rules | 10/10 | No forbidden dependencies; binary shell has zero business logic |
| Trait abstraction | 9/10 | `PlaybackClock`, `ProjectLoader`, `AudioController` — clean separation |
| Data model | 10/10 | Pure serde types, no serialization to runtime state coupling |
| Renderer separation | 9/10 | Scene IR cleanly decoupled from timeline and UI |
| Testability | 9/10 | Pure evaluation engine, injectable clock, contract tests |
| **Overall** | **9.5/10** | |

## Crate Inventory

| Crate | Lines (approx.) | Tests |
|-------|-----------------|-------|
| `mva-types` | ~300 | 8 |
| `mva-timeline` | ~1200 | 69 |
| `mva-core` | ~1100 | 26 |
| `mva-audio` | ~200 | 9 |
| `mva-lyrics` | ~200 | 0 |
| `mva-scene` | ~400 | 12 |
| `mva-renderer` | ~350 | 15 |
| `mva-ui` | ~600 | 0 (UI tested manually) |
| `mva-format` | ~320 | 7 |
| `mva-player` | ~350 | 8 |

## Phase 4 Complete

- CLI parsing with clap 4 (Empty / Demo / OpenProject modes)
- Composition-root refactor: thin `main.rs`, `startup.rs` bootstrap + runtime services
- Unified project loading: `activate_project` with prepare/activate two-phase boundary
- Audio device failure graceful exit (error window + stderr + exit code 1)
- `autoplay_on_open` configuration (file-driven after M4)
- `config/app.toml` loading with dual-pass parsing and unknown-key warnings
- Native file dialog: File → Open File / Open Folder
- Configuration warning reporting in UI (status bar) + stderr

---

*Refer to `docs/roadmap.md` for detailed implementation tracking.*
