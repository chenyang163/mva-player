# MVA Specification (Draft)

> **Status:** Draft — the MVA format is in the design phase. This document
> outlines the intended direction and is not a finalized specification.

## Overview

MVA (Media Visual Animation) is a format and runtime for timeline-based
media composition. Unlike conventional media containers that encode video
and audio streams, MVA describes how media assets are arranged, animated,
and rendered over time.

## Design Goals

- **Extensibility** — New asset types, effect plugins, and rendering
  backends should be addable without changes to the core format.
- **Timeline-based composition** — All content (audio, text, images,
  effects) is placed on a shared timeline with keyframe-driven animation.
- **Deterministic rendering** — Given the same input, the runtime produces
  the same output on any supported platform.
- **Cross-platform runtime** — The reference implementation targets
  desktop platforms with a portable rendering abstraction.

## Current Status

The format is in active design. The `mva-player` reference implementation
supports:

- Timeline model and pure evaluation engine
- LRC lyrics parsing
- Image layers with transform keyframes
- Effect timeline with parameter keyframes
- Scene-based rendering pipeline
- **Loose `.mva` JSON manifest reading** (experimental): a `.mva` file in
  loose-project form is the §6.2-style JSON manifest referencing audio,
  LRC lyrics, and an `*.anim.json` animation timeline by relative path.
  Readers ignore unknown fields and refuse `format_version` majors ≥ 2.
  See `examples/lyric_demo/demo.mva` for a working document.

Planned for future specification versions:

- `.mva` binary (ZIP) container format reusing the same manifest schema
- Audio/video stream references
- Plugin API contract
- Metadata schema

## Relationship to the Reference Implementation

This specification describes the intended MVA format. The `mva-player`
crate in this repository serves as the reference runtime — validating
the format design through implementation.

## Contributing

Specification discussions are welcome. See [CONTRIBUTING.md](../CONTRIBUTING.md)
for how to participate.

---

*This is a draft. Nothing in this document constitutes a final or binding specification.*
