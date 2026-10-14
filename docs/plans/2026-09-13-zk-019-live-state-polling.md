# ZK-019.1 Live State Polling Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All 104 workspace tests passing, clippy clean, fmt clean.  
**Requirement:** REQ-20.  

## Scope

Implement bounded session state polling in `zedkata-tui` to display real-time attempt feedback, edit counts, verdicts, and progress as the student edits in Zed, with resilient handling of missing, corrupt, or partially replaced session state.

## Subtasks

1. **State Polling Core (`crates/zedkata-tui/src/app.rs`)**:
   - Add fields to `App`:
     - `last_session_poll: std::time::Instant`
     - `last_session_mtime: Option<std::time::SystemTime>`
     - `session_warning: Option<String>`
     - `poll_interval: std::time::Duration` (default 100ms)
   - Implement `poll_session(&mut self) -> bool` with rate-limiting guard (`poll_interval`).
   - Implement `force_poll_session(&mut self) -> bool` bypassing interval/mtime checks.
   - Filesystem check via `mtime`: only re-parse JSON when file modification time has changed.
   - Resilient error handling:
     - Missing file: no panic, no error spam, maintains existing state.
     - Corrupt / malformed JSON: catches error, retains previous valid `SessionState`, sets actionable `session_warning`.
     - Transient I/O error (e.g. atomic rename window): catches error gracefully without crashing.
     - Valid state reload: clears `session_warning` and applies new attempt/history/verdict.

2. **TUI Event Loop Integration (`crates/zedkata-tui/src/terminal.rs`)**:
   - Call `app.poll_session()` in the main loop before `terminal.draw(...)`.
   - Ensure the loop does not busy-loop: `event::poll_and_handle_events` provides 50ms bounded timeout, keeping idle CPU near zero.

3. **UI Feedback Display (`crates/zedkata-tui/src/ui.rs`)**:
   - In `render_feedback_pane`, render `session_warning` if present (e.g. notifying student if session file was corrupted or unreadable).
   - In `render_header`, show warning badge if session state has an active warning.

4. **Automated Verification (`crates/zedkata-tui/tests/live_state_tests.rs`)**:
   - Test controlled session updates: mock external writer updating `session.json` with edits, verdict (InProgress -> Passed), history, and verify `App` updates attempt and progress.
   - Test missing state: polling a nonexistent path does not crash or corrupt existing state.
   - Test corrupt state: writing invalid JSON triggers `session_warning` without crashing or discarding prior valid state.
   - Test self-healing: replacing corrupt JSON with valid JSON clears `session_warning` and loads new state.
   - Test bounded polling throttling: rapid successive calls do not re-read disk unless interval elapsed or forced.

## Validation

- `cargo test -p zedkata-tui --test live_state_tests`
- `cargo test --workspace --locked`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
