# Plan: Open Exercises in Same Zed Window & Convert Curriculum to `vimtutor`-Style Plain Text

## Context & Goal
1. **Same-Window Zed Launch**: Pressing `[o]` in the TUI currently spawns `zed.exe -n <path:line:col>`, where `-n` (`--new`) forces Zed to open a brand-new window. Using `-e` (`--existing`) opens the `.kata` exercise buffer as a tab in the existing Zed window instead of creating a new window.
2. **Language-Agnostic `vimtutor` Experience**:
   - Replace Rust code in `.kata` exercise fixtures (`lessons/fixtures/**`) with everyday plain-text notes, checklists, memos, and structured plain text (using quotes, parentheses, braces, and colons where needed for find/till/text-object drills).
   - Rewrite all lesson tasks, hints, and prose in `lessons/*.toml` so every instruction is literal, self-contained, and requires zero programming language knowledge.
   - Surface a `Target:` line preview and lesson guide (`prose`) in the TUI's `Active Exercise Task` pane (`crates/zedkata-tui/src/ui.rs`) so learners can immediately see the exact edit goal without guessing.
   - Update `zedkata-agent` fallback drill templates (`crates/zedkata-agent/src/generator.rs`), system prompt (`crates/zedkata-agent/prompts/generator_system.md`), and walkthrough explanations (`crates/zedkata-agent/src/walkthrough.rs`) to use plain text instead of Rust syntax.

## Files to Modify
1. `crates/zedkata-tui/src/launcher.rs` & `crates/zedkata-tui/tests/launch_tests.rs` — Switch `-n` to `-e` so Zed reuses the existing window.
2. `crates/zedkata-tui/src/app.rs` & `crates/zedkata-tui/src/ui.rs` — Load and display a compact `Target:` preview line (comparing `.start.kata` and `.goal.kata`) and lesson guide (`prose` when vertical space allows) in `render_task_pane`.
3. `lessons/*.toml` & `lessons/fixtures/**/*.kata` — Convert all 15 runnable drill fixtures and lesson TOML descriptions/tasks/hints to `vimtutor`-style plain English text.
4. `crates/zedkata-agent/src/generator.rs`, `crates/zedkata-agent/prompts/generator_system.md`, & `crates/zedkata-agent/src/walkthrough.rs` — Convert generated exercises and walkthroughs to language-agnostic plain text.
5. `crates/zedkata-lsp/tests/feedback_integration_tests.rs`, `crates/zedkata-tui/tests/starter_journey_tests.rs`, & `crates/zedkata-tui/tests/navigation_tests.rs` — Update test fixtures/assertions to match the new plain-text buffers and tasks.
