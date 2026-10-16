# Tasks and Debugger Walkthrough Workspace Fixture

This directory serves as the reproducible sample workspace for practicing Zed's native **Tasks** and **DAP Debugger** capabilities under `lessons/zed-04-tasks-and-debugger.toml` and `docs/walkthroughs/tasks-and-debugger.md`.

## Workspace Structure

- `.zed/tasks.json`: Project-specific task configuration demonstrating:
  - `cargo check`: Reusable check task running in the integrated terminal pane.
  - `cargo test current file`: Context-sensitive test task utilizing the `$ZED_STEM` editor variable and `rust-test` runnable tag.
  - `cargo run`: Launching the project binary.
  - `inspect active file`: Demonstrating `$ZED_FILE` and `$ZED_WORKTREE_ROOT` variable substitution.
- `.zed/debug.json`: Project-specific DAP debug configurations:
  - `Build & Debug sample-calculator`: CodeLLDB launcher with pre-launch `cargo build` task.
  - `Build & Debug current test`: CodeLLDB launcher for unit/integration tests with pre-launch compilation.
- `src/lib.rs` & `src/math.rs`: Core calculation and statistics logic with unit tests suitable for setting breakpoints and stepping through loops.
- `src/main.rs`: CLI entry point executing sequential operations suitable for stepping and variable inspection.
- `tests/calculator_test.rs`: Integration test exercising the multi-step calculation pipeline.

## Opening in Zed

Open this directory directly as a Zed workspace:

```bash
zed lessons/fixtures/walkthroughs/tasks_and_debugger
```
