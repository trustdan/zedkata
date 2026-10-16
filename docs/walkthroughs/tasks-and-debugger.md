# Workspace Tasks & DAP Debugger

**Category:** Guided Walkthrough (Unscored)  
**Lesson:** `zed-04-tasks-and-debugger` (`[[walkthrough]] id = "tasks-and-debugger-walkthrough"`)  
**Target Editor:** Zed 1.19.2+ (Vim mode enabled)  
**Requirements:** REQ-25, REQ-29  
**Sample Fixture Workspace:** [`lessons/fixtures/walkthroughs/tasks_and_debugger/`](../../lessons/fixtures/walkthroughs/tasks_and_debugger/)  
**Prerequisites:** `["zed-03-multibuffer"]`  

---

## 1. Why Tasks and Debugger are Guided Walkthroughs

In ZedKata, drill exercises evaluate student technique through Language Server Protocol (`textDocument/didChange`) incremental notifications within a single `.kata` exercise buffer.

However, **tasks and debugger workflows cannot be represented as scored single-buffer drills** for four architectural reasons:

1. **Tasks Spawn External Terminal Processes:**
   - Tasks execute shell commands (`cargo check`, `cargo test`, `echo`, custom scripts) in Zed's integrated terminal pane or terminal tabs.
   - The stdout, stderr, and process termination are rendered inside Zed's terminal emulator and produce **zero LSP `textDocument/didChange` modifications** on student `.kata` buffers.
2. **Debugger Communicates via Debug Adapter Protocol (DAP):**
   - Zed implements the client side of the Debug Adapter Protocol (DAP), communicating over IPC or TCP with standalone language debug adapters (such as `CodeLLDB` for Rust, C, and C++).
   - Breakpoint registration, stepping, pause/resume, call stack frame inspection, and variable watch expressions take place inside Zed's debugging UI panels and the DAP connection, completely outside document text editing streams.
3. **Absence of Single-Buffer Deterministic Goal States:**
   - Standard drills evaluate a single start buffer reaching an exact target goal buffer within a strict Par edit budget.
   - Tasks compile projects or run external test suites; debugging inspects running runtime memory states and stack frames. Neither produces a single-file text transformation that can be validated by a text comparator.
4. **Task and Debugger Modals are Internal to GPUI:**
   - Opening the task picker (`task: spawn`), typing ad-hoc oneshot commands (`alt-enter`), toggling task summary visibility, setting gutter breakpoints (`editor::ToggleBreakpoint`), and viewing debug variables are internal GPUI state transitions.
   - Per `CLAUDE.md`, REQ-10, and OQ-04, Zed exposes no external API or extension hooks to inspect running tasks, terminal buffer text, breakpoint locations, or active debug adapter states.

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*) and **REQ-29**, workspace tasks and debugger workflows are formally classified as an **unscored guided walkthrough** with concrete reproducible workspace fixtures, verified step-by-step recipes, and self-reported verification criteria.

---

## 2. Zed Tasks Architecture & Verified Semantics

Zed provides a native task execution system capable of running build commands, unit tests, and arbitrary shell scripts directly from the editor.

### 2.1 Task Configuration Scopes
- **Project-Specific (Worktree-Local):** Stored in `.zed/tasks.json` in the workspace root. These tasks are available only when the worktree is open. Open via the command palette action `zed: open project tasks`.
- **Global (User-Wide):** Stored in `~/.config/zed/tasks.json` (macOS/Linux) or `%APPDATA%\Zed\tasks.json` (Windows). Available across all Zed projects. Open via the command palette action `zed: open tasks`.
- **VS Code Task Compatibility:** Zed automatically loads tasks defined in `.vscode/tasks.json`.
- **Oneshot / Ad-hoc Tasks:** Typed directly into the task modal and executed immediately via `alt-enter`. Persist for the duration of the editing session.

### 2.2 Task Template Properties

Tasks are defined as a JSON array of task objects:

```json
[
  {
    "label": "cargo check",
    "command": "cargo",
    "args": ["check", "--all-targets"],
    "env": { "RUST_BACKTRACE": "1" },
    "cwd": "$ZED_WORKTREE_ROOT",
    "use_new_terminal": false,
    "allow_concurrent_runs": false,
    "reveal": "always",
    "hide": "never",
    "shell": "system",
    "show_summary": true,
    "show_command": true,
    "save": "all",
    "tags": ["rust-check"]
  }
]
```

- `label` *(string)*: Unique display name shown in the task picker modal.
- `command` *(string)*: Executable program name or script path.
- `args` *(array of strings)*: Command-line arguments passed to the program.
- `env` *(object)*: Key-value environment variable overrides.
- `cwd` *(string)*: Working directory for execution (defaults to `$ZED_WORKTREE_ROOT`).
- `use_new_terminal` *(boolean)*: Whether to create a fresh terminal tab (`true`) or reuse the existing task tab (`false`, default).
- `allow_concurrent_runs` *(boolean)*: Whether to allow parallel runs of the same task (`true`) or wait/cancel prior run (`false`, default).
- `reveal` *(string)*: Terminal pane display mode when started:
  - `"always"` (default): Show terminal pane and focus task tab.
  - `"no_focus"`: Show terminal pane and add tab, but retain focus in editor.
  - `"never"`: Do not alter focus; add/reuse tab in pane.
- `hide` *(string)*: Terminal pane behavior upon process completion:
  - `"never"` (default): Leave terminal pane open.
  - `"always"`: Automatically close the tab when process exits.
  - `"on_success"`: Close tab only if process exits with code 0; retain tab on error.
- `save` *(string)*: Buffer saving behavior before spawning:
  - `"none"` (default): Do not save buffers.
  - `"current"`: Save only the active editor buffer.
  - `"all"`: Save all dirty project buffers.
- `tags` *(array of strings)*: Category tags (e.g. `"rust-test"`, `"bash-script"`). When tagged, tasks can override inline runnable indicators in the editor gutter or code actions dropdown (`g .` / `ctrl-.`).

### 2.3 Built-in Dynamic Context Variables

Zed expands dynamic variables at the moment a task is invoked:

| Variable | Description | Example Value |
|:---|:---|:---|
| `$ZED_FILE` | Absolute path of the active file | `/home/user/repo/src/main.rs` |
| `$ZED_FILENAME` | File name with extension | `main.rs` |
| `$ZED_DIRNAME` | Parent directory of the active file | `/home/user/repo/src` |
| `$ZED_RELATIVE_FILE` | Path relative to workspace root | `src/main.rs` |
| `$ZED_RELATIVE_DIR` | Directory relative to workspace root | `src` |
| `$ZED_STEM` | File name without extension | `main` |
| `$ZED_SYMBOL` | Symbol currently under cursor | `fn test_execute_basic_arithmetic` |
| `$ZED_SELECTED_TEXT` | Currently highlighted text in editor | `execute_operation` |
| `$ZED_LANGUAGE` | Language identifier of active buffer | `Rust`, `Python`, `JSON` |
| `$ZED_WORKTREE_ROOT` | Absolute path to current workspace root | `/home/user/repo` |
| `$ZED_MAIN_GIT_WORKTREE` | Absolute path to main git worktree | `/home/user/repo` |
| `$ZED_CUSTOM_RUST_PACKAGE` | Cargo package name enclosing active file | `sample-calculator` |
| `$ZED_ROW` / `$ZED_COLUMN` | 1-based cursor coordinates in active file | `42` / `15` |

> [!TIP]
> **Variable Fallbacks & Quoting:** Use `${ZED_SELECTED_TEXT:default_value}` to provide a fallback when a variable is empty. Always quote variables or place them in the `args` array to prevent shell splitting when paths contain spaces.

---

## 3. Zed DAP Debugger Architecture & Verified Keybindings

Zed includes a built-in client for the **Debug Adapter Protocol (DAP)**. DAP enables a standardized interface between Zed's graphical user interface and underlying debug engines.

### 3.1 Supported Adapters
- **Rust / C / C++:** `CodeLLDB` (built-in; uses LLDB) or `GDB` (Linux).
- **JavaScript / TypeScript:** `vscode-js-debug` (built-in; Node.js and Chrome).
- **Python:** `debugpy` (built-in).
- **Go:** `delve` (built-in).
- **PHP:** `php-debug` (built-in).

### 3.2 Debug Configuration Scopes (`debug.json`)
- **Project-Specific:** Stored in `.zed/debug.json` in the workspace root.
- **Global:** Stored in `%APPDATA%\Zed\debug.json` (Windows), `~/Library/Application Support/Zed/debug.json` (macOS), or `~/.config/zed/debug.json` (Linux). Open via `zed: open debug tasks`.
- **VS Code Launch Compatibility:** Zed automatically loads launch configurations from `.vscode/launch.json` if `.zed/debug.json` is absent.

### 3.3 Configuration Schema (`.zed/debug.json`)

```json
[
  {
    "label": "Build & Debug sample-calculator",
    "adapter": "CodeLLDB",
    "request": "launch",
    "build": {
      "command": "cargo",
      "args": ["build"]
    },
    "program": "$ZED_WORKTREE_ROOT/target/debug/sample-calculator",
    "sourceLanguages": ["rust"],
    "env": { "RUST_LOG": "debug" }
  }
]
```

- `label` *(string, required)*: Identifier shown in the debug session modal.
- `adapter` *(string, required)*: The DAP adapter identifier (e.g. `"CodeLLDB"`, `"JavaScript"`, `"Python"`).
- `request` *(string)*: `"launch"` (start a new process) or `"attach"` (connect to an existing process).
- `program` *(string)*: Path to the target binary or entry script.
- `build` *(object or string)*: Pre-launch build step. Can be an inline task definition (`{"command": "cargo", "args": ["build"]}`) or a string referencing an existing task label from `tasks.json`.
- `sourceLanguages` *(array)*: Required for `CodeLLDB` when debugging Rust (`["rust"]`).

### 3.4 Verified Debugger & Stepping Keybindings Table

Audited against Zed 1.19.2 (`assets/keymaps/vim.json`, `assets/keymaps/default.json`, and official docs):

| Action / Intent | Default Shortcut | Vim Mode Shortcut | Zed Action Identifier |
|:---|:---:|:---:|:---|
| **Start Debugging** | `F4` | `space shift-g l` (or `F4`) | `debugger::Start` |
| **Toggle Breakpoint** | `ctrl-F8` / gutter click | `space shift-g b` | `editor::ToggleBreakpoint` |
| **Continue / Resume** | `F9` | `space shift-g c` (or `F9`) | `debugger::Continue` |
| **Step Over** | `F8` | `space shift-g n` (or `F8`) | `debugger::StepOver` |
| **Step Into** | `F7` | `space shift-g i` (or `F7`) | `debugger::StepInto` |
| **Step Out** | `shift-F8` | `space shift-g o` (or `shift-F8`) | `debugger::StepOut` |
| **Pause Execution** | — | `space shift-g h` | `debugger::Pause` |
| **Restart Session** | — | `space shift-g r` | `debugger::Restart` |
| **Stop Debugging** | — | — | `debugger::Stop` |

---

## 4. Concrete Step-by-Step Walkthrough Recipes

Open the sample workspace in Zed:
```bash
zed lessons/fixtures/walkthroughs/tasks_and_debugger
```

### Recipe 1: Spawning and Managing Workspace Tasks

**Objective:** Launch a project task from `.zed/tasks.json` and inspect its execution in Zed's integrated terminal pane.

1. Open the command palette with `ctrl-shift-p` (or `cmd-shift-p` on macOS).
2. Type `task: spawn` and press `enter`.
   - The Task Picker modal appears, listing the tasks configured in `.zed/tasks.json`:
     - `cargo check`
     - `cargo test current file`
     - `cargo run`
     - `inspect active file`
3. Highlight `cargo check` and press `enter`.
4. Observe the integrated terminal pane opening at the bottom of the workspace:
   - Notice the task summary banner displaying the command: `cargo check --all-targets`.
   - The build output stream shows Cargo checking packages and exiting with code `0`.
5. Press `escape` to return focus to the editor buffer.

---

### Recipe 2: Context-Aware Tasks with Variable Interpolation

**Objective:** Execute a task that dynamically references the active editor file using `$ZED_STEM` and `$ZED_FILE`.

1. Open `src/math.rs` in the editor.
2. Open the command palette (`ctrl-shift-p`), run `task: spawn`, and select `inspect active file`.
   - The terminal runs `echo` and outputs:
     ```text
     Inspecting active file: .../src/math.rs in worktree: .../tasks_and_debugger
     ```
3. Run `task: spawn` again and select `cargo test current file`.
   - Notice how `$ZED_STEM` evaluates to `math`.
   - Cargo executes `cargo test -- math`, specifically running the unit tests inside `src/math.rs` (`test_execute_basic_arithmetic`, `test_division_by_zero_safety`, `test_summarize_series`).
   - All tests pass in the terminal pane.

---

### Recipe 3: Task Editing, Oneshot Spawning, and Rerun

**Objective:** Customize a task on the fly, execute an ad-hoc oneshot shell command (`alt-enter`), and rerun it with `task: rerun`.

1. Open the command palette (`ctrl-shift-p`) and select `task: spawn`.
2. Highlight `cargo run`. Instead of pressing `enter`, press `Tab`.
   - The modal switches into an editable prompt pre-filled with `cargo run`.
3. Edit the command line to append `--quiet`:
   ```bash
   cargo run --quiet
   ```
4. Press `alt-enter` (or `option-enter` on macOS) to execute it as a **oneshot task**.
   - The task runs immediately in the terminal, executing `main.rs` and displaying:
     ```text
     === Zed Tasks & Debugger Sample Calculator ===
     Step 1: add = 42
     Step 2: mul = 126
     Summary: count=6, sum=271, mean=45, min=12, max=89
     Calculator execution completed successfully.
     ```
5. Return to the editor, make a small change or navigate lines.
6. Open the command palette and run `task: rerun`.
   - Zed immediately re-executes the most recently spawned oneshot command (`cargo run --quiet`) without re-opening the picker modal.

---

### Recipe 4: Configuring and Launching a DAP Debug Session

**Objective:** Inspect `.zed/debug.json` and trigger a pre-compiled debugging session using `CodeLLDB`.

1. Inspect `.zed/debug.json` in the workspace root:
   - Notice the configuration `"Build & Debug sample-calculator"`.
   - It specifies `"adapter": "CodeLLDB"`, `"sourceLanguages": ["rust"]`, and an automatic `"build"` step running `cargo build`.
2. Open `src/main.rs`.
3. Press `F4` (or in Vim normal mode press `space shift-g l`, or run `debugger: start` from the command palette).
   - The **New Debug Session** modal opens.
   - It displays `"Build & Debug sample-calculator"`, alongside any automatically detected Rust test/binary targets.
4. Select `"Build & Debug sample-calculator"` and press `enter`.
   - Zed automatically triggers the pre-launch build step (`cargo build`).
   - Once compiled, the debugger panel opens docking at the bottom or side, attaching to the binary.

---

### Recipe 5: Gutter Breakpoints, Stepping, and Variable Inspection

**Objective:** Set breakpoints using Vim keybindings, step through arithmetic logic, and inspect local variables.

1. In `src/main.rs`, navigate cursor to line 6:
   ```rust
   let step_1 = math::execute_operation("add", 40, 2);
   ```
2. Set a breakpoint:
   - Press `space shift-g b` in normal mode (or click in the gutter margin next to line 6).
   - A distinct breakpoint circle icon appears in the gutter on line 6.
3. Set a second breakpoint on line 12 inside the `if let Some(summary)` block:
   ```rust
   println!("Summary: count={}, ...", summary.sample_count, ...);
   ```
4. Launch the debug session via `F4` (or `space shift-g l`).
   - The program launches and immediately halts at line 6.
   - The active execution line is highlighted with an accent color.
5. In the **Variables** pane of the Debugger panel:
   - Note the initialized locals (`operations`).
6. Press `F7` (or `space shift-g i`) to **Step Into** `math::execute_operation` in `src/math.rs`.
   - The active line jumps to `execute_operation` on line 21.
   - Observe arguments `op = "add"`, `a = 40`, `b = 2` in the Variables inspector.
7. Press `F8` (or `space shift-g n`) twice to **Step Over** the `match op` statement.
   - Observe `value = 42` assigned.
8. Press `shift-F8` (or `space shift-g o`) to **Step Out** back to `src/main.rs`.
   - Cursors returns to line 6 in `main.rs`, and stepping advances to line 7.
   - Observe local variable `step_1` populated with `CalculationResult { operation: "add", operand_a: 40, operand_b: 2, value: 42 }`.
9. Press `F9` (or `space shift-g c`) to **Continue** execution.
   - The debugger runs forward and hits the second breakpoint on line 12.
   - Inspect `summary` in the Variables pane: verify `sample_count = 6`, `sum = 271`, `mean = 45`, `min = 12`, `max = 89`.
10. Press `F9` (or `space shift-g c`) to resume to program completion.
    - Process exits with code `0`. The debugger status indicates session ended.

---

## 5. Self-Reported Completion Checklist

Because tasks and debugger interactions execute outside student `.kata` buffer streams (REQ-25, REQ-29), students verify their mastery using this self-report checklist:

- [ ] **Task Spawning:** Launched `cargo check` using `task: spawn` and confirmed the build output in Zed's integrated terminal pane.
- [ ] **Variable Expansion:** Ran `cargo test current file` from an active Rust buffer and verified that `$ZED_STEM` correctly filtered test execution to that file.
- [ ] **Oneshot Execution:** Used `Tab` in the task modal to edit a command, spawned it with `alt-enter`, and subsequently reran it with `task: rerun`.
- [ ] **Debug Configuration:** Validated the structure of `.zed/debug.json`, including adapter specification (`CodeLLDB`), pre-launch `build` tasks, and path resolution.
- [ ] **Breakpoints & Stepping:** Set a breakpoint via `space shift-g b` / gutter click, launched via `F4` / `space shift-g l`, stepped into and over functions with `F7`/`F8` (`space shift-g i`/`n`), and confirmed variable states in the Variables pane.
