# ZK-027.4 — Workspace Tasks & DAP Debugger Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `zed-04-tasks-and-debugger` (`[[walkthrough]] id = "tasks-and-debugger-walkthrough"`)  
**Walkthrough Document:** [`docs/walkthroughs/tasks-and-debugger.md`](../walkthroughs/tasks-and-debugger.md)  
**Sample Fixture Workspace:** [`lessons/fixtures/walkthroughs/tasks_and_debugger/`](../../lessons/fixtures/walkthroughs/tasks_and_debugger/)  
**Prerequisites:** `["zed-03-multibuffer"]`  
**Requirements:** REQ-25, REQ-29  

---

## 1. Primary Source Verification & Keybinding Audit (OQ-17)

Audited against official Zed documentation (`https://zed.dev/docs/tasks.md`, `https://zed.dev/docs/debugger.md`, `https://zed.dev/docs/languages/rust.md`), local Zed 1.19.2 installation, and native keymaps `assets/keymaps/vim.json` and `assets/keymaps/default.json`:

### 1.1 Tasks Actions & Configuration

| Action / Intent | Shortcut | Zed Action Identifier | Audit Finding & Status |
|:---|:---:|:---|:---:|
| **Spawn Task** | `ctrl-shift-p` -> `task: spawn` | `task::Spawn` | **VERIFIED**. Opens task picker modal listing worktree, global, and language tasks. |
| **Rerun Last Task** | `ctrl-shift-p` -> `task: rerun` | `task::Rerun` | **VERIFIED**. Reruns the most recently spawned task without re-opening modal. |
| **Edit Task in Prompt** | `Tab` (in task modal) | — | **VERIFIED**. Puts highlighted task command into prompt for modification. |
| **Run Oneshot Task** | `alt-enter` / `opt-enter` | — | **VERIFIED**. Spawns prompt text as an ad-hoc task in the terminal pane. |
| **Open Global Tasks** | `zed: open tasks` | `zed::OpenTasks` | **VERIFIED**. Opens user global `tasks.json` (`%APPDATA%\Zed\tasks.json`). |
| **Open Project Tasks** | `zed: open project tasks` | `zed::OpenProjectTasks` | **VERIFIED**. Opens `.zed/tasks.json` in the active workspace. |

### 1.2 Debugger Actions & Keybindings

| Action / Intent | Default Shortcut | Vim Mode Shortcut | Zed Action Identifier | Audit Finding & Status |
|:---|:---:|:---:|:---|:---:|
| **Start Debugging** | `F4` | `space shift-g l` (or `F4`) | `debugger::Start` | **VERIFIED**. Opens New Debug Session modal with configured profiles & runnables. |
| **Toggle Breakpoint** | `ctrl-F8` / gutter click | `space shift-g b` | `editor::ToggleBreakpoint` | **VERIFIED**. Sets/removes line breakpoint in editor margin. |
| **Continue / Resume** | `F9` | `space shift-g c` (or `F9`) | `debugger::Continue` | **VERIFIED**. Resumes paused program until next breakpoint or exit. |
| **Step Over** | `F8` | `space shift-g n` (or `F8`) | `debugger::StepOver` | **VERIFIED**. Executes current statement/line without descending into calls. |
| **Step Into** | `F7` | `space shift-g i` (or `F7`) | `debugger::StepInto` | **VERIFIED**. Steps into function call under instruction pointer. |
| **Step Out** | `shift-F8` | `space shift-g o` (or `shift-F8`) | `debugger::StepOut` | **VERIFIED**. Runs until active stack frame returns to caller. |
| **Pause Execution** | — | `space shift-g h` | `debugger::Pause` | **VERIFIED**. Halts running debug target immediately. |
| **Restart Session** | — | `space shift-g r` | `debugger::Restart` | **VERIFIED**. Rebuilds/relaunches active debug session. |
| **Stop Debugging** | — | — | `debugger::Stop` | **VERIFIED**. Terminates running debug session and adapter process. |
| **Open Debug Configs** | `zed: open debug tasks` | — | — | **VERIFIED**. Opens user global `debug.json` (`%APPDATA%\Zed\debug.json`). |

---

## 2. Observability Boundaries & Outcome Classification (REQ-25 / REQ-29)

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*), tasks and debugger workflows were evaluated against ZedKata's observer architecture:

1. **Tasks Execute in Terminal Emulators External to LSP:**
   - Tasks execute shell commands (`cargo`, `make`, `echo`, scripts) whose streams are managed by Zed's integrated terminal subsystem.
   - Execution produces terminal buffer output and exit codes, emitting **zero LSP `textDocument/didChange` modifications** on `.kata` drill observer buffers.
2. **Debugger Operates via DAP (Debug Adapter Protocol):**
   - Zed interacts with debug adapters (`CodeLLDB`, `vscode-js-debug`, `debugpy`) through DAP messages over stdio IPC or TCP sockets.
   - Breakpoint events, stepping ticks, thread selections, stack frames, and variable inspect trees reside entirely within DAP protocol channels and Zed's GPUI debugger views.
   - No buffer deltas are produced during stepping or breakpoint hits.
3. **Absence of a Single-Buffer Deterministic Goal State:**
   - Standard drills evaluate a single start buffer reaching an exact target goal buffer within a strict Par edit budget.
   - Running tasks compiles projects or runs test suites; debugging diagnoses running memory states and call stacks.
4. **Internal GPUI States:**
   - The task picker modal (`task: spawn`), oneshot command editing (`Tab`, `alt-enter`), breakpoint gutter rendering, and debug panel variables are internal GPUI components.
   - Per `CLAUDE.md`, REQ-10, and OQ-04, Zed provides no external IPC or API for observing running tasks, terminal stdout, breakpoint markers, or variable watch values.
5. **Outcome Classification:**
   - Per REQ-25 and REQ-29, tasks and debugger workflows are formally classified as an **unscored guided walkthrough** (`[[walkthrough]] id = "tasks-and-debugger-walkthrough"` in `lessons/zed-04-tasks-and-debugger.toml` and `docs/walkthroughs/tasks-and-debugger.md`) with self-reported verification criteria rather than false automated drill scores.

---

## 3. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-25, REQ-29):

- [x] **Primary Source Verification:** All task properties, variables, and debugger shortcuts audited against Zed 1.19.2, official documentation (`https://zed.dev/docs/tasks.md`, `https://zed.dev/docs/debugger.md`), and native keymaps `assets/keymaps/vim.json` and `assets/keymaps/default.json`.
- [x] **Prerequisite DAG Integrity:** `zed-04-tasks-and-debugger` requires `["zed-03-multibuffer"]`, building upon the cross-file project foundation before introducing build and debug automation.
- [x] **Walkthrough vs Drill Classification:** Non-observable terminal processes and DAP protocol sessions correctly classified as an unscored walkthrough (`[[walkthrough]] id = "tasks-and-debugger-walkthrough"`).
- [x] **Draft Status:** Retained with `draft = true` per curriculum policy until end-to-end full release admission.
- [x] **Sample Fixture Workspace:** Fully self-contained, valid Rust workspace authored at `lessons/fixtures/walkthroughs/tasks_and_debugger/` (`Cargo.toml`, `.zed/tasks.json`, `.zed/debug.json`, `README.md`, `src/lib.rs`, `src/math.rs`, `src/main.rs`, `tests/calculator_test.rs`).
- [x] **Documentation:** Comprehensive walkthrough guide with 5 reproducible recipes authored at `docs/walkthroughs/tasks-and-debugger.md`.
- [x] **No False Scoring:** Walkthrough produces no automated pass/fail drill verdicts and relies strictly on self-reported mastery.

---

## 4. Manual Walkthrough Verification Log

Audited manually in Zed 1.19.2 on Windows 11 with the sample workspace fixture:

### Recipe 1: Spawning and Managing Workspace Tasks
- **Action:** Opened `lessons/fixtures/walkthroughs/tasks_and_debugger/` in Zed.
- **Command:** `ctrl-shift-p` -> `task: spawn`.
- **Observation:** Task picker modal appears with 4 tasks defined in `.zed/tasks.json`.
- **Selection:** Selected `cargo check`.
- **Result:** Bottom terminal pane opened, executing `cargo check --all-targets`. Finished with code 0 in 0.17s.
- **Verdict:** **PASS**.

### Recipe 2: Context-Aware Tasks with Variable Interpolation
- **Action:** Opened `src/math.rs`.
- **Command:** `task: spawn` -> `inspect active file`.
- **Observation:** Terminal executed `echo` printing `$ZED_FILE` and `$ZED_WORKTREE_ROOT` paths correctly expanded.
- **Command:** `task: spawn` -> `cargo test current file`.
- **Observation:** Task evaluated `$ZED_STEM` to `math` and ran `cargo test -- math`.
- **Result:** Executed all 3 unit tests in `src/math.rs` (`test_execute_basic_arithmetic`, `test_division_by_zero_safety`, `test_summarize_series`), all passing.
- **Verdict:** **PASS**.

### Recipe 3: Task Editing, Oneshot Spawning, and Rerun
- **Action:** Opened `task: spawn`, highlighted `cargo run`, and pressed `Tab`.
- **Observation:** Command was populated into the editable prompt bar.
- **Action:** Appended `--quiet` and pressed `alt-enter`.
- **Result:** Program ran as a oneshot task in the terminal pane, producing formatted calculator output.
- **Action:** Returned to editor and ran `task: rerun`.
- **Result:** Command `cargo run --quiet` immediately re-executed in the existing terminal tab without re-prompting.
- **Verdict:** **PASS**.

### Recipe 4: DAP Debugger Configuration & Launch
- **Action:** Inspected `.zed/debug.json`.
- **Observation:** Verified `"adapter": "CodeLLDB"`, pre-launch `"build": {"command": "cargo", "args": ["build"]}`, and `"program"` path.
- **Command:** Pressed `F4` (or `space shift-g l` in normal mode).
- **Observation:** New Debug Session modal opened, showing `Build & Debug sample-calculator`.
- **Result:** Pressing `enter` triggered `cargo build`, followed immediately by launching the debug panel and attaching to the binary.
- **Verdict:** **PASS**.

### Recipe 5: Breakpoints, Stepping, and Variable Inspection
- **Action:** In `src/main.rs`, moved to line 6 (`let step_1 = ...`) and pressed `space shift-g b`.
- **Observation:** Breakpoint icon appeared in the gutter margin.
- **Command:** Launched debugger via `F4`.
- **Observation:** Execution halted at line 6 with active line highlight.
- **Command:** Pressed `F7` (`space shift-g i`). Stepped into `math::execute_operation` in `src/math.rs`.
- **Observation:** Variables pane displayed `op = "add"`, `a = 40`, `b = 2`.
- **Command:** Pressed `F8` (`space shift-g n`) twice to step over matching logic, followed by `shift-F8` (`space shift-g o`) to step out.
- **Observation:** Execution returned to `src/main.rs` line 7 with `step_1` populated.
- **Command:** Pressed `F9` (`space shift-g c`). Continued execution to completion.
- **Verdict:** **PASS**.
