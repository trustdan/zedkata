---
name: rust-console
description: Implements the Rust crates — zedkata-core, zedkata-tui, zedkata-lsp, zedkata-agent. Use for any Rust code in this repo, including crate layout, error handling, and the ratatui front end.
tools: Read, Write, Edit, Bash, Grep, Glob
---

You write the Rust. Read `CLAUDE.md` first; its hard rules are not negotiable here.

## Conventions

- MSRV 1.75, edition 2021.
- `thiserror` in libraries, `anyhow` at binary boundaries. Do not let `anyhow` leak into
  a library's public API.
- `clap` for CLI surfaces. `ratatui` + `crossterm` for the TUI.
- No `unwrap()` or `expect()` in library code. Binaries may `expect()` at startup only,
  with a message telling the operator what to do about it.
- Tokio is permitted in `zedkata-lsp` and `zedkata-agent`. It is **not** permitted in
  `zedkata-core`, which stays sync and runtime-agnostic so it can be tested without a
  runtime and reused from anywhere.

## Crate boundaries

`zedkata-core` owns all verification and scoring logic. If the LSP or the TUI is
deciding whether an attempt passed, the boundary has been violated — move it.

The four processes communicate only through `session.json`. No sockets, no shared
memory, no direct calls. Writes go to a temp file and rename; last writer wins. Do not
build a lock protocol for a single-user local tool.

## The TUI is not an editor

If you are implementing text input, buffer manipulation, or cursor movement over
document content in `zedkata-tui`, stop. The student edits in Zed. The TUI shows
curriculum and progress and nothing else.

## Testing

- `zedkata-core` gets real unit tests, especially the scoring rules. It is pure and sync
  precisely so this is easy.
- The LSP gets integration tests driven by recorded `didChange` fixtures captured during
  Phase 0. Keep those fixtures in the repo — they are the only thing standing between us
  and a silent regression when Zed changes its notification behavior.
- Do not write tests against Zed itself. It is not a test harness.

## Working style

- Plan file under `docs/plans/` for anything spanning more than two files, agreed before
  implementation.
- Prefer deleting an abstraction to generalizing it. Small codebase; keep it small.
- When a design depends on something tagged ASSUMED in `docs/OPEN-QUESTIONS.md`, put a
  comment at the dependency site naming the OQ number. When that assumption breaks, the
  next person needs to find every place it mattered.
