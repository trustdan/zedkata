# ZK-019.2 Connection Health & Stale State Recovery Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All 110 workspace tests passing, clippy clean, fmt clean.  
**Requirement:** REQ-20.  

## Scope

Implement connection health diagnostics and stale/mismatched session detection in `zedkata-tui`, ensuring the UI clearly distinguishes between waiting, connected, mismatched, and stale/stopped states, with actionable recovery steps and documented manual restart procedures.

## Subtasks

1. **Connection & Sync Health Model (`crates/zedkata-tui/src/app.rs`)**:
   - Define `ConnectionStatus` enum:
     - `Idle`: No active drill in session.
     - `WaitingForZed`: Exercise selected/launched, waiting for Zed buffer open or LSP didOpen.
     - `Connected`: Buffer open in Zed, active attempt attached, live LSP feedback.
     - `MismatchedSession`: TUI selection desynchronized from Zed's active session.
     - `StaleBuffer`: Buffer path in session missing on disk or unreadable.
   - Implement `app.connection_status() -> ConnectionStatus`.
   - Implement `app.sync_to_active()`: navigates TUI selection to currently active session drill (bound to key `s`).

2. **Actionable Feedback & Recovery UI (`crates/zedkata-tui/src/ui.rs`)**:
   - Render connection status banner and specific actionable recovery hints in `render_feedback_pane`:
     - Waiting: displays expected buffer path and instructs student how to open in Zed or restart server (`workspace: restart language server`).
     - Mismatched: displays both viewed and active exercises, with instructions to press `[Enter]` to activate or `[s]` to sync.
     - Connected: displays connected status, buffer path, live attempt verdict, and next-step actions.
     - Stale: warns of missing buffer and instructs student to press `[o]` to relaunch.
   - Update header status line with connection health indicator.
   - Add `[s] Sync` shortcut to footer help bar.

3. **Key Event Handling (`crates/zedkata-tui/src/event.rs`)**:
   - Map `s` / `S` to `app.sync_to_active()`.

4. **Integration & Recovery Tests (`crates/zedkata-tui/tests/connection_health_tests.rs`)**:
   - Test `WaitingForZed` state when drill is selected but buffer unopened.
   - Test transition from `WaitingForZed` to `Connected` when LSP records `didOpen`.
   - Test `MismatchedSession` when student navigates away from active drill, and recovery via `sync_to_active()`.
   - Test `StaleBuffer` when referenced buffer file is missing on disk.
   - Test UI rendering of all connection health states without panic.

5. **Troubleshooting & Manual Recovery Guide (`docs/DEVELOPMENT.md`)**:
   - Document concrete manual scenarios:
     - Simulating stopped/killed LSP server and verifying TUI "waiting / disconnected" feedback.
     - Restarting the language server in Zed (`debug: restart language server` / `workspace: restart language server`).
     - Stale state recovery after unexpected editor exit.

## Validation

- `cargo test -p zedkata-tui --test connection_health_tests`
- `cargo test --workspace --locked`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
