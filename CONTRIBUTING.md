# Contributing to MVA Player

Contributions are welcome. MVA Player is in early development — your input
can help shape the architecture and direction of the project.

## Ways to Contribute

- **Bug Report** — Found something broken? Open an issue.
- **Architecture Discussion** — Ideas about crate boundaries, data flow, or
  future design.
- **Feature Proposal** — Suggest a new capability or improvement.
- **Code Review** — Review open PRs and provide constructive feedback.

## Issue Types

When opening an issue, please choose the appropriate type:

| Type | Use For |
|------|---------|
| **Bug Report** | Unexpected behavior, crashes, rendering errors, test failures |
| **Feature Request** | New functionality, enhancements, quality-of-life improvements |
| **Architecture Discussion** | Proposals affecting crate boundaries, API design, data model, or dependency rules |
| **License Concern** | Potential copyright or licensing issues (see README § Code Similarity and Licensing Notice) |

## Issue Guidelines

- Search existing issues before opening a new one.
- For bug reports, include: Rust version, OS, reproduction steps.
- For architecture discussions, reference the relevant section of
  `docs/architecture.md`.

## Pull Request Guidelines

- Keep PRs focused on a single change.
- Ensure `cargo test --workspace` passes.
- Ensure `cargo clippy --workspace` is clean.
- Follow the existing code style and crate dependency rules.
- Update relevant documentation in `docs/`.
- No business logic in `mva-player` (binary shell is wiring only).
- `mva-format` must not depend on `mva-player`.

## Code of Conduct

Be respectful and constructive. Assume good intent.
