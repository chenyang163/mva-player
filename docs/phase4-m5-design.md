# MVA Phase 4 M5 — Native File Dialog

**Status:** IMPLEMENTED
**Prerequisite:** Phase 4 M1/M2/M3/M4 completed.

## 1. Goal

Replace the no-op "Open…" button in the File menu with real native file-browse
dialogs.  Users can now open MVA projects without manually typing a path.

## 2. Implemented Changes

| File | Change |
|---|---|
| `crates/mva-ui/Cargo.toml` | Added `rfd = "0.15"` (MIT, native OS dialogs) |
| `crates/mva-ui/src/panels/settings.rs` | Replaced empty "Open…" with "Open File…" and "Open Folder…" |

**File menu structure:**

```
File
├── Open File…    → rfd::FileDialog (filters: *.mva, *.mp3/*.flac/*.wav)
└── Open Folder…  → rfd::FileDialog (directory picker)
```

Both emit `PlayerCommand::OpenFile(PathBuf)`.  Cancel (`None`) produces no
command and no error.  The existing text-path input and "Open" button are
preserved as an accessibility fallback.

### Unmodified crates

| Crate / module | Reason |
|---|---|
| `mva-core` (engine, state, command, effect, config) | Existing `OpenFile` pipeline unchanged |
| `mva-player` (main, startup, cli) | Composition root unchanged |
| `mva-format`, `mva-audio`, `mva-renderer`, `mva-timeline`, `mva-scene` | M5 changes only the path source |

## 3. Architecture Flow

```
File Menu  ──click──→  rfd::FileDialog  ──returns──→  PathBuf (or None)
                                                           │
                                              PlayerCommand::OpenFile(path)
                                                           │
                                                        Engine
                                                           │
                                                 LoadProject Effect
                                                           │
                                                   activate_project
                                                           │
                                                    Ready / Playing
```

M5 **only changes the path source**.  The entire downstream pipeline
(`PlayerCommand::OpenFile` → `LoadProject` effect → `activate_project` →
`Ready`/`Playing`) is unchanged from M2/M3.

## 4. Design Decisions

### `rfd` choice

- MIT license — compatible with project licensing.
- Native OS dialog (Windows: IFileDialog; Linux: GTK/Zenity; macOS: NSOpenPanel).
- Cross-platform — no platform-specific file-browser code.
- Avoids building a custom file browser widget.

### Open Folder included

- Phase 2 help text already promised "open the folder" to users.
- `MvaLoader` already supports directory input (scans for audio + lyrics).
- No additional pipeline work — folder path is just another `PathBuf` source.

### Lock blocking (accepted)

`rfd::pick_file()` / `rfd::pick_folder()` are **synchronous calls** executed
while the Engine lock is held (the settings panel renders inside an
`engine.lock()` scope in `app.rs`).

This is an **accepted design decision** for M5:

- No background task currently requires the Engine lock.
- The audio thread (`rodio`) does not depend on the Engine lock.
- Modal native dialogs naturally block UI interaction — desktop users expect
  this behaviour.

**Future:** If async loading or background analysis requires lock-free UI
interaction, the dialog invocation can be moved outside the lock scope via a
delayed command queue.  Not implemented in M5.

## 5. Testing

### Automated

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS (190 tests) |

### Manual checklist

| Scenario | Expected |
|---|---|
| File → Open File… → select `demo.mva` | Project loads through existing pipeline |
| File → Open Folder… → select folder with mp3 + lrc | Project loads through existing pipeline |
| Cancel file dialog | No command, no error, no crash |
| Cancel folder dialog | No command, no error, no crash |
| Text path input + Open button | Still works as before |

## 6. Known Limitations

| Limitation | Status |
|---|---|
| No recent-files list | Deferred |
| No drag-and-drop file opening | Deferred |
| No file-association integration | Deferred |
| Synchronous dialog (blocks Engine lock) | Documented accepted design (§4) |
| File-type filters are hardcoded | Deferred (configuration-driven filters) |
