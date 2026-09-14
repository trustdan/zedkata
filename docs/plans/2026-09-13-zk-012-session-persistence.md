# ZK-012 Session Persistence Implementation Plan

**Date:** 2026-09-13  
**Status:** In execution. Approved by user.

## Scope

Implement versioned session state persistence, atomic safe file replacement on Windows and Unix platforms, attempt lifecycle transitions (start, edit, pass, fail, abandon, retry, switch), and idempotent progress/weakness recording in `crates/zedkata-core/src/session.rs`.

## Tasks

1. **ZK-012.1: Serialize versioned session state**
   - Explicit `CURRENT_SCHEMA_VERSION = 1`.
   - Validate version on load; reject incompatible versions (`version > CURRENT_SCHEMA_VERSION`).
   - Graceful recovery: return default state when file does not exist.
   - Actionable diagnostics on corrupted JSON.

2. **ZK-012.2: Write state through safe file replacement**
   - Platform-standard paths: Windows `%LOCALAPPDATA%\zedkata\session.json`, Unix `$XDG_STATE_HOME/zedkata/session.json` (fallback `~/.local/state/zedkata/session.json`), configurable via `ZEDKATA_SESSION_PATH`.
   - Safe atomic replacement: write to `session.json.<pid>_<timestamp>.tmp` in the same directory, flush/sync, then atomic rename over destination.
   - Clean up temporary files on write/rename failure and preserve existing state intact.

3. **ZK-012.3: Apply attempt lifecycle transitions**
   - State transitions: `start_attempt`, `record_edit`, `finalize_attempt`, `abandon_attempt`, `retry_attempt`, `switch_exercise`.
   - Protect against stale attempt IDs (ignore edits/events from old attempts).
   - Prevent late edits once an attempt is finalized (`Passed` or `Failed`).

4. **ZK-012.4: Persist history and computed progress once**
   - Idempotent finalization: duplicate calls do not add duplicate `history` entries.
   - Concept weakness updates via core on attempt finalization.

## Validation

- `cargo test --workspace --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo +1.85.0 check --workspace --locked`
- `cargo +1.85.0 test --workspace --locked`
