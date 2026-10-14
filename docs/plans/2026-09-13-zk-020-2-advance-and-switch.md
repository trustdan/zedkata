# ZK-020.2 Advance and Switch Exercises Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All 122 workspace tests passing, clippy clean, fmt clean.  
**Requirements:** REQ-20  

## Scope

Implement exercise advancement and switching in `zedkata-tui` and headless CLI:
1. Advance within lesson or across unlocked lessons (`n` / `N` key and `advance_and_activate`).
2. Update task description, concepts, budget, and forbid rules immediately upon exercise selection/advance.
3. Accurate progress reporting using unique completed exercises to prevent repeated completions from inflating progress.
4. Guarantee that repeated completions and late edits from prior exercise buffers cannot skip exercises, duplicate history entries, or corrupt session state.
5. Provide headless `zedkata-tui next` CLI subcommand.

## Subtasks

1. **TUI Shell Methods (`crates/zedkata-tui/src/app.rs`)**:
   - `advance_exercise(&mut self) -> bool`:
     - Advances to the next exercise within the current lesson.
     - If at the end of the lesson, checks if the subsequent lesson is unlocked via `is_lesson_locked`.
     - If unlocked, advances to the next lesson (exercise 0).
     - If locked, halts and reports actionable prerequisite block in `status_message`.
     - If at the end of the catalog, halts and reports "Course completed!".
   - `advance_and_activate(&mut self) -> Result<bool>`:
     - Calls `advance_exercise()`, then calls `activate_selected()` to atomically write to `session.json`.
   - Update `app.next_exercise()` to advance across lessons when at the end of the current lesson if unlocked.

2. **Accurate Unique Progress Reporting (`crates/zedkata-tui/src/ui.rs`)**:
   - Update `render_feedback_pane` to calculate `passed_count` as the number of unique `(lesson_id, exercise_id)` pairs passed in `session.history`.
   - Ensure retrying or repeatedly passing the same exercise never inflates the completion count or percentage beyond total exercises.
   - Update pass prompt to include `[n]` for direct advance: `PASSED! Press [n] for next exercise, or [o] to launch`.

3. **Key Event Handling & Footer UI (`crates/zedkata-tui/src/event.rs`, `ui.rs`)**:
   - Bind `n` and `N` to `app.advance_and_activate()`.
   - Update footer help bar to display `[n] Next`.

4. **Headless Subcommand (`crates/zedkata-tui/src/main.rs`)**:
   - Add `Next` subcommand to `Commands` enum in `main.rs`.
   - Implement `handle_next` to advance the active exercise to the next unlocked exercise and save session state.

5. **Automated Verification (`crates/zedkata-tui/tests/navigation_tests.rs`)**:
   - Test advancing within lesson updates task, concepts, and budget.
   - Test advancing across lesson boundaries when unlocked.
   - Test advancing blocked when next lesson has unsatisfied prerequisites.
   - Test advancing gracefully halts at end of curriculum.
   - Test repeated completion does not corrupt progress or duplicate history.
   - Test late edits from old buffer / old attempt ID are rejected and do not skip exercises or modify new attempt state.
   - Test TUI key `n` triggers advance and activate.
   - Test headless `next` command.

## Validation

- `cargo test -p zedkata-tui --test navigation_tests`
- `cargo test --workspace --locked`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
