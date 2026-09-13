# Phase 1 Scaffolding Plan

**Date:** 2026-09-13  
**Status:** In execution. Feasibility gate passed (ADR 0003).

## Scope

Establish architectural contracts in `docs/ARCHITECTURE.md` (ZK-006) and scaffold the native Rust workspace for `zedkata-core`, `zedkata-lsp`, and `zedkata-tui` (ZK-007).

## Components & File Layout

```text
Cargo.toml                   # Root workspace manifest
Cargo.lock                   # Pinned dependency lockfile
.gitignore                   # Ignore target/ and temp files
crates/
  zedkata-core/              # Lesson model, verification rules, state schema (sync)
    Cargo.toml
    src/lib.rs
  zedkata-lsp/               # Language server binary (tokio + async)
    Cargo.toml
    src/main.rs
  zedkata-tui/               # Terminal user interface binary (ratatui + crossterm)
    Cargo.toml
    src/main.rs
```

## State & Lifecycle Contracts (ZK-006)

1. **State Ownership (`session.json`)**:
   - **TUI writes**: `current.lesson_id`, `current.exercise_id`, `current.nav_mode`.
   - **LSP writes**: `current.attempt` (attempt_id, start_time, edit_count, elapsed, verdict, message).
   - **Persistence rule**: Writes use atomic replacement (write to `<file>.tmp` then rename). On Windows, replace via temporary file pattern.
2. **Attempt Lifecycle**:
   - `idle` -> `started` -> `editing` -> `passed` | `failed` | `retried`.
3. **Core is Sync**:
   - `zedkata-core` stays pure sync and runtime-agnostic. No Tokio in core.

## Validation

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- CLI validation: `cargo run -p zedkata-lsp -- --help`, `cargo run -p zedkata-tui -- --help`
