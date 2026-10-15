# ZK-021.1 Progress and Prerequisites Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All 126 workspace tests passing, clippy clean, fmt clean.  
**Requirements:** REQ-11, REQ-20  

## Scope

Implement core-derived history prerequisite unlocking across sessions, concept weakness tracking display, and score/unscored mode presentation in `zedkata-core` and `zedkata-tui`:
1. Core-derived prerequisite satisfaction: A lesson prerequisite is satisfied when all its exercises are completed (passed) in session history. Failed attempts never satisfy prerequisites.
2. Unscored mode vs scored mode display:
   - For goal-only exercises (`par_edits: None`), task pane, live attempt verdict, and history display label the technique as unscored rather than fabricating a par or score.
   - For scored exercises (`par_edits: Some(...)`), full budget, par, actual edits, and scores are rendered.
3. Concept weakness tracking:
   - Session weaknesses computed by core (`HashMap<String, f64>`) are displayed in the TUI feedback pane and headless CLI `status`.
   - Pass, failure, and retry updates to weaknesses are surfaced to the user.
4. Terminal layout preservation: Ensure small terminal (60x15) continues to render cleanly with weakness and score information.
5. Automated verification covering prerequisite unlocking across sessions, unscored mode, and weakness tracking.

## Subtasks

1. **Prerequisite Satisfaction in Core (`crates/zedkata-core/src/catalog.rs`)**:
   - Add `is_lesson_completed(&self, lesson_id: &str, history: &[HistoryEntry]) -> bool`:
     - Checks if a lesson in catalog has all of its exercises passed in history.
   - Add `is_lesson_unlocked(&self, lesson_id: &str, history: &[HistoryEntry]) -> (bool, Vec<String>)`:
     - Evaluates `requires` for the lesson:
       - If required lesson is in catalog, requires `is_lesson_completed`.
       - If required lesson is external / uncataloged, requires at least one passed attempt in history.
       - Returns `(is_unlocked, missing_prerequisites)`.

2. **TUI Shell Prerequisite Integration (`crates/zedkata-tui/src/app.rs`)**:
   - Update `app.is_lesson_locked` to delegate to `catalog.is_lesson_unlocked`.
   - Ensure session history reloads accurately reflect unlocked state across sessions.

3. **Score & Unscored Mode Display (`crates/zedkata-tui/src/ui.rs`, `crates/zedkata-tui/src/main.rs`)**:
   - Update `render_task_pane`:
     - If `par_edits.is_none()`, display `Budget: Goal-only (unscored) | Max: <max>`.
   - Update `render_feedback_pane`:
     - For `Verdict::Passed`, if `par.is_none()`, display `PASSED! <edits> edit(s) (Goal-only / Unscored)`.
     - In Recent attempts: if `par_edits.is_none()`, display `<lesson>/<exercise> (unscored)`.
   - In `main.rs` (`handle_status`):
     - Format history items with `(unscored, edits: <n>)` when `par_edits.is_none()`.

4. **Weakness Tracking Display (`crates/zedkata-tui/src/ui.rs`, `crates/zedkata-tui/src/main.rs`)**:
   - In `render_feedback_pane`:
     - Display a compact weakness summary row:
       - If empty: `Weaknesses: None recorded (clean)`.
       - If present: display concepts with formatted percentage / decimal, sorted by severity (highest first).
   - In `main.rs` (`handle_status`):
     - Print tracked concept weaknesses with severity breakdown.

5. **Automated Verification (`crates/zedkata-tui/tests/progress_prerequisites_tests.rs`)**:
   - Test core prerequisite satisfaction: incomplete exercises block unlock, failed attempts do not unlock, all completed exercises unlock across sessions.
   - Test scored vs unscored mode rendering in TUI and headless status.
   - Test weakness display updates upon pass/fail/retry.
   - Test exact 60x15 boundary layout fit.

## Validation

- `cargo test -p zedkata-core`
- `cargo test -p zedkata-tui --test progress_prerequisites_tests`
- `cargo test --workspace --locked`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
