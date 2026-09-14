# ZK-014 Feedback Integration Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All 87 workspace tests passing, clippy clean, fmt clean, MSRV 1.85.0 verified.

## Scope

Connect the `zedkata-lsp` observer to the curriculum catalog, `session.json` persistence, `zedkata-core` verification, and Language Server Protocol inline diagnostics (`textDocument/publishDiagnostics`).

## Tasks

1. **ZK-014.1: Resolve active exercises and apply verdicts**
   - Active exercise resolution: Match opened document against `session.current` and curriculum catalog.
   - Ignore unrelated documents: Do not verify, mutate state, or emit feedback for unrelated buffers.
   - State synchronization:
     - On `didOpen`: start fresh attempt in `session.json` with unique attempt ID and save state.
     - On `didChange`: record edits to session attempt, evaluate buffer against goal via `verify_attempt`.
     - On `Passed` / `Failed`: finalize attempt in session, update history and weaknesses, and persist to disk via atomic safe file replacement (`save_to_file`).
     - Reject late edits on finalized attempts and stale attempt IDs.

2. **ZK-014.2: Publish and clear feedback**
   - On attempt pass: push `textDocument/publishDiagnostics` with `Information` severity and congratulatory score message.
   - On attempt fail: push `textDocument/publishDiagnostics` with `Error` severity and actionable failure reason (budget or forbid rule).
   - On retry / reset / `didClose`: push empty diagnostics list (`diagnostics: []`) to clear editor feedback.
   - Integration test verifying exactly one completion and zero stale diagnostics.

## Validation

- `cargo test --workspace --locked`
- `cargo test -p zedkata-lsp --test feedback_integration_tests`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo +1.85.0 check --workspace --locked`
- `cargo +1.85.0 test --workspace --locked`
