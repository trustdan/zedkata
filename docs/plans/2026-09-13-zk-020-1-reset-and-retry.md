# ZK-020.1 Reset and Retry Active Exercise Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All 114 workspace tests passing, clippy clean, fmt clean.  
**Requirements:** REQ-13, REQ-20.  

## Scope

Implement exercise retry and dirty-buffer reset in `zedkata-tui` and `launcher`, ensuring that retrying an exercise issues a fresh attempt ID, clears prior verdict feedback, restores the writable buffer file to pristine baseline content, and guarantees that late updates from prior attempts are rejected.

## Subtasks

1. **Launcher Reset Implementation (`crates/zedkata-tui/src/launcher.rs`)**:
   - Implement `reset_exercise_attempt(session_path, catalog, exercises_dir_override) -> Result<LaunchResult, LauncherError>`:
     - Resolves active lesson and exercise from `session.current`.
     - Overwrites dirty buffer on disk with the pristine start fixture via `prepare_exercise_buffer`.
     - Generates a new unique attempt ID via `session.retry_attempt(new_id)`.
     - Atomically persists the updated session state to disk.

2. **TUI Shell Integration (`crates/zedkata-tui/src/app.rs`)**:
   - Implement `app.reset_active() -> Result<()>`:
     - Calls `launcher::reset_exercise_attempt` for the active exercise.
     - Refreshes in-memory session state and updates `status_message`.
   - Update `app.refresh_state()` to focus on catalog and session reload from disk.

3. **Key Event Handling & Footer UI (`crates/zedkata-tui/src/event.rs`, `ui.rs`)**:
   - Bind `r` to `app.reset_active()` (Retry active exercise & restore baseline buffer).
   - Bind `R` to `app.refresh_state()` (Reload catalog and session state from disk).
   - Update footer help bar: `[r] Retry  [R] Reload  [s] Sync  [q] Quit`.
   - Update feedback pane actionable hints to prompt `[r]` to retry.

4. **Headless Subcommand Update (`crates/zedkata-tui/src/main.rs`)**:
   - Update `handle_reset` to accept `lessons_dir`, load the catalog, and invoke `reset_exercise_attempt` so that headless `zedkata-tui reset` also restores dirty buffers on disk.

5. **Automated Verification (`crates/zedkata-tui/tests/retry_tests.rs`)**:
   - Test fresh attempt generation and verdict clearance.
   - Test dirty buffer restoration: overwrite buffer with arbitrary edits, call reset, verify file on disk matches baseline fixture.
   - Test late update rejection: verify pre-retry attempt ID edits are rejected by `session.record_edit` and `FeedbackEngine`.
   - Test TUI key handling (`r` and `R`).
   - Test headless `reset` subcommand.

## Validation

- `cargo test -p zedkata-tui --test retry_tests`
- `cargo test --workspace --locked`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
