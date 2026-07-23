# Lyric Demo — the first MVA showcase

This example proves the core MVA concept end-to-end:

> **Audio + Lyrics + Timeline-based media experience**

A real MP3 plays through the MVA runtime while synchronized lyric
lines — parsed from a standard `.lrc` file — are rendered as an
animated text layer driven by the audio clock.

Complex visual effects are intentionally **out of scope** for this
first demo; correctness of the complete workflow comes first.

## Contents

```
lyric_demo/
├── README.md                          ← this file
├── demo.mva                           ← loose MVA project (JSON manifest)
├── lyrics.lrc                         ← synchronized lyric timeline
└── assets/
    ├── monkeys-spinning-monkeys.mp3   ← music (CC BY 4.0, see below)
    └── lyric.anim.json                ← AnimationTimeline: lyric text layer
```

### `demo.mva` — loose MVA manifest

The loose (unzipped) form of the MVA format: a JSON manifest that
references its media entries by relative path (architecture §6.2).
The same manifest schema will live inside the future `.mva` ZIP
container as `manifest.json`.

```json
{
  "format_version": "1.0",
  "metadata": { "title": "…", "artist": "…", "duration": 125.0 },
  "entries": {
    "audio": "assets/monkeys-spinning-monkeys.mp3",
    "lyrics": ["lyrics.lrc"],
    "animation": "assets/lyric.anim.json"
  }
}
```

### Timeline events

`assets/lyric.anim.json` contains one text layer bound to the active
lyric line (`"source": "lyric_line"`) with basic keyframe timing:

- opacity `0 → 1` over the first 0.8 s (`ease_out_cubic`)
- scale `0.95 → 1.0` over the same window

That is deliberately all — lyric synchronization is the feature
being demonstrated.

## Running the demo

From the repository root:

```
cargo run -p mva-player
```

Then in the player window, enter this path into the **Path** field and
press **Open** (or Enter):

```
examples/lyric_demo/demo.mva
```

Press **Play**. You should hear the music and see the current lyric
line rendered in the viewport, changing in sync with the audio clock.
The seek bar scrubs the timeline deterministically (pure evaluation).

## Automated verification

The demo is covered by tests (no audio device or GPU needed, except
the full-decode check which only uses the CPU decoder):

```
cargo test -p mva-player --test demo_showcase
cargo test -p mva-format
```

These verify: the manifest loads, the MP3 decodes and matches the
declared duration, and every lyric line becomes the active text-layer
content at the right engine-clock time.

## Music license

```
Monkeys Spinning Monkeys Kevin MacLeod (incompetech.com)
Licensed under Creative Commons: By Attribution 4.0 License
https://creativecommons.org/licenses/by/4.0/
```

Download: <https://incompetech.com/music/royalty-free/mp3-royaltyfree/Monkeys%20Spinning%20Monkeys.mp3>
(no account required). Full provenance: [`docs/demo-assets.md`](../../docs/demo-assets.md).

The track is instrumental; `lyrics.lrc` is an original demo lyric
timeline written by the MVA Player project purely to exercise
lyric/timeline synchronization (repository license applies).
