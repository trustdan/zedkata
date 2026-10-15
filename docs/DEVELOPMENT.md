# Development setup

Checked 2026-09-13 on Dan's Windows machine. Start with the
[backlog](BACKLOG.md); Phase 0 still gates production code.

## Observed tools

| Tool | Observed version or state |
|------|---------------------------|
| Zed | 1.19.2, commit `df181c6f58d02677b385fa947d6bfde6d3530078` |
| Rust compiler | 1.97.1 (`8bab26f4f`, 2026-07-14) |
| Cargo | 1.97.1 (`c980f4866`, 2026-06-30) |
| Active toolchain | `stable-x86_64-pc-windows-msvc` |
| Installed Rust targets | `x86_64-pc-windows-msvc` only |
| Node.js | 22.21.1 |
| npm | 10.9.4 |
| pnpm | 11.20.0; optional, not selected for the probe |
| Git | 2.55.0.windows.3 |
| Visual Studio | 2022 Community; `vswhere` reports the x86/x64 C++ tools component |

Zed resolves to `%LOCALAPPDATA%\Programs\Zed\bin\zed.exe`, Rust tools to
`%USERPROFILE%\.cargo\bin`, and Node/npm to `C:\nvm4w\nodejs` on this machine.
Use PATH discovery rather than embedding these paths in shipped code.

## Recheck the setup

Run from the repository root in PowerShell:

```powershell
Get-Command zed,rustc,cargo,rustup,node,npm.cmd,git
zed --version
rustc --version
cargo --version
rustup show active-toolchain
rustup toolchain list
rustup target list --installed
node --version
npm.cmd --version
git --version
```

Use `npm.cmd` to invoke npm explicitly from PowerShell. The probe will use npm and
a committed lockfile; package selection and installation belong to ZK-002.

## Zed CLI: opening a file at a position

Verified 2026-09-14 on Zed 1.19.2. ZK-018.2 uses this to launch an exercise buffer.

```powershell
zed -n "<path>:<line>:<column>"
```

- **Line and column are 1-based.** `:3:12` puts the caret on the 12th character of
  line 3 — confirmed character-exact by observation.
- **Quote the whole argument.** The position suffix is appended to the path, and the
  CLI splits it from the right, so paths containing spaces work as a single quoted
  argument. Do not split the path and position into separate arguments.
- Useful flags: `-n`/`--new` (new workspace), `-e`/`--existing` (reuse a window),
  `-a`/`--add` (add to current workspace), `-w`/`--wait` (block until closed).

Discover the binary through PATH rather than hardcoding it; on this machine it
resolves to `%LOCALAPPDATA%\Programs\Zed\bin\zed.exe`.

This positions the caret **at launch only.** Zed exposes no way to read the cursor
afterward (OQ-04) — see [CLI cursor placement](evidence/2026-09-14-cli-cursor-placement.md).

## Remaining prerequisites

- `wasm32-wasip2` was installed during ZK-003 on 2026-09-13 with
  `rustup target add wasm32-wasip2`; the disposable extension release build passed.
- A grammar-bearing extension may need wasi-sdk; Zed documents automatic download.
  Grammar choice and the actual extension build remain for ZK-003.
- Rust 1.85 is verified as the native MSRV (checked 2026-09-13). Rust 1.75 is
  incompatible because modern 2026 dependencies use `edition2024` (stabilized in 1.85).
  Native workspace compilation and unit tests pass on both 1.85.0 and 1.97.1.
- Native CI workflow (`.github/workflows/ci.yml`) runs formatting, clippy, tests, and
  MSRV checks on Windows.

Target and SDK behavior comes from the current, unversioned
[extension development guide](https://zed.dev/docs/extensions/developing-extensions),
checked 2026-09-13; validate it against installed Zed during ZK-003.

## Planned probe connection

Use the local dev extension, with the following files now prepared in ZK-003 under
`spikes/lsp-probe/extension/`:

```text
extension.toml
Cargo.toml
src/lib.rs
languages/zedkata/config.toml
```

Define the language name and `.kata` suffix in language metadata; register a server
for that same language name in the manifest. Implement `language_server_command`
to launch Node with the probe's script path and stdio argument as separate arguments.
The spike pins extension API 0.7.0 and references the Rust grammar; actual loading
in Zed remains pending. See the [exact install/checklist](../spikes/lsp-probe/extension/README.md).
The registration mechanism is documented in
[Language Extensions](https://zed.dev/docs/extensions/languages#language-servers).

Once those files and the probe exist:

1. Open the repository with `zed .`.
2. Run `zed: install dev extension` in the command palette and select the extension
   directory above. Publishing is unnecessary for this development route.
3. Open the probe's `.kata` fixture and verify its selected language.
4. Confirm the capture contains initialization and document-open messages, edit the
   file, and check that a change arrives and the probe diagnostic renders.
5. Use `zed: open log` to investigate registration, build, or startup failures.

These are preparation instructions, **not completed runtime checks**. ZK-002 creates
the probe; ZK-003 validates the connection; ZK-004 measures actual traffic.

The settings documentation covers file associations and binary overrides for known
servers; it directs new language support to extensions. It does not establish a
settings-only registration route for our new language/server pair.
[Configuring Languages](https://zed.dev/docs/configuring-languages)

## Production dev extension (ZK-015)

The production Zed extension lives in `extension/`. It registers the `Zedkata` language
for `.kata` drill buffers, reuses the built-in `rust` grammar for highlighting, and discovers
and launches the native `zedkata-lsp` binary.

### Build the extension and native server

```powershell
# 1. Build and install native LSP server onto PATH
cargo install --path crates/zedkata-lsp

# 2. Build the WASM extension
cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release

# 3. Run extension checks and tests
cargo fmt --manifest-path extension/Cargo.toml --check
cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings
cargo test --manifest-path extension/Cargo.toml
```

### Install in Zed

1. Open Zed (`zed .` from repository root).
2. Open Command Palette (`Ctrl+Shift+P`).
3. Select `zed: install dev extension` and choose `C:\Users\Dan\zedkata\extension`.
4. Open any `.kata` exercise file (e.g. `lessons/fixtures/motions-01/delete-to-word-end.start.kata`).
5. Confirm the language indicator displays **Zedkata** and editing triggers live diagnostic feedback.

See [extension/README.md](../extension/README.md) for full details and troubleshooting.

## Terminal TUI Shell (ZK-017)

Launch the interactive Ratatui / Crossterm TUI shell:

```powershell
# Launch interactive TUI shell
cargo run -p zedkata-tui

# Verify terminal restoration on error exit
cargo run -p zedkata-tui -- --test-error

# Verify terminal restoration on panic exit
cargo run -p zedkata-tui -- --test-panic
```

Inside the TUI:
- `j` / `k` (or Down/Up): Navigate between curriculum lessons.
- `h` / `l` (or Left/Right): Navigate exercises within the active lesson.
- `Enter`: Select and activate the highlighted exercise in `session.json`.
- `?`: Toggle category hint display.
- `r`: Refresh session state and curriculum catalog from disk.
- `q`, `Esc`, or `Ctrl-C`: Exit cleanly and restore cooked terminal state.
- Resizing updates layout dynamically; shrinking below 60x15 displays a centered small-terminal warning overlay.

## Headless exercise selection (ZK-016)

Exercises can also be selected and inspected without launching the interactive TUI using `zedkata-tui`:

```powershell
# Select an active exercise in session state
cargo run -p zedkata-tui -- select motions-01-word 0

# Check active session status and history
cargo run -p zedkata-tui -- status

# Reset active attempt for current exercise
cargo run -p zedkata-tui -- reset
```

## Capturing raw LSP traffic

`zedkata-lsp` appends every message it reads and writes to a `.jsonl` file when
`ZEDKATA_CAPTURE` names an output path. Unset, it is inert. This exists to answer open
questions in [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md) with reproducible wire evidence.

Records use the same `{sequence, at, direction, message}` shape as the Phase 0 probe
captures, so a new capture replays through the golden-fixture tests in
`crates/zedkata-lsp/tests/`.

**The variable must reach the Zed GUI process.** The extension launches the server with
`env: vec![]`, so the server inherits Zed's environment. Two things defeat this on
Windows, both silently:

1. Zed already running. The variable only reaches processes started afterward.
2. **Launching via `zed`.** Verified 2026-09-14: `zed` on PATH resolves to
   `...\Programs\Zed\bin\zed.exe`, a CLI shim that delegates to a *separate* GUI binary
   at `...\Programs\Zed\Zed.exe` and exits. The variable goes into the shim and dies
   with it. Launch the GUI binary directly instead.

```powershell
# 1. Rebuild and reinstall the server that the extension finds on PATH
cargo install --path crates/zedkata-lsp --force

# 2. Quit Zed completely first, then confirm nothing is running
Get-Process -Name Zed -ErrorAction SilentlyContinue | Stop-Process

# 3. Set the capture path and launch the GUI binary directly (NOT `zed .`)
$env:ZEDKATA_CAPTURE = "C:\Users\Dan\zedkata\captures\undo.jsonl"
New-Item -ItemType Directory -Force (Split-Path $env:ZEDKATA_CAPTURE) | Out-Null
& "C:\Users\Dan\AppData\Local\Programs\Zed\Zed.exe" "C:\Users\Dan\zedkata"

# 4. Perform the edits under investigation, then close the buffer and quit Zed.
```

Confirm capture is active by checking Zed's language server logs (`debug: open language
server logs`) for `[zedkata-lsp] capturing traffic to ...` on stderr. If that line is
absent the variable did not reach the process.

Captures may contain buffer contents. Review one before retaining it under
`tests/fixtures/lsp/`.

## Connection Health, Stale-State Recovery, and Troubleshooting (ZK-019.2)

The TUI communicates with Zed and the LSP observer exclusively through `session.json`. The TUI automatically monitors connection health and provides clear, actionable feedback for connection issues and desynchronized state:

### 1. Waiting for Zed / Stopped LSP
- **Indicator:** `Connection: WAITING FOR ZED — Buffer not yet open in editor`
- **Cause:** An exercise has been selected in the TUI, but Zed has not opened the `.kata` buffer, or `zedkata-lsp` has crashed or stopped.
- **Actionable Recovery:**
  - Press `[o]` in the TUI to launch the buffer directly into Zed at the agreed coordinates.
  - Or manually open the buffer: `zed <buffer_path>`.
  - If Zed is already running but the server is stopped, open the Command Palette in Zed (`Ctrl-Shift-P` on Windows/Linux or `Cmd-Shift-P` on macOS) and run:
    ```
    workspace: restart language server
    ```
  - As soon as Zed emits `textDocument/didOpen`, `zedkata-lsp` initializes the active attempt, and the TUI automatically transitions to `CONNECTED TO ZED`.

### 2. Mismatched Session (Browsing While Editing)
- **Indicator:** `Connection: MISMATCHED VIEW — Viewing <A> while Zed has <B> active`
- **Cause:** The student is navigating through lessons/exercises in the TUI while a different drill is active in Zed.
- **Actionable Recovery:**
  - Press `[s]` to synchronize the TUI view back to the active drill in Zed.
  - Or press `[Enter]` to switch the session's active drill to the currently viewed exercise.

### 3. Stale Buffer Recovery
- **Indicator:** `Connection: STALE BUFFER — Exercise buffer file does not exist on disk`
- **Cause:** The temporary buffer file was deleted, cleaned up by the OS, or moved.
- **Actionable Recovery:**
  - Press `[o]` in the TUI. The launcher will regenerate the pristine fixture into a fresh writable buffer and relaunch it in Zed.

### 4. Simulating and Testing Connection Scenarios
Automated verification is covered in `crates/zedkata-tui/tests/connection_health_tests.rs`:
```powershell
cargo test -p zedkata-tui --test connection_health_tests
```

To test manually:
1. Launch an exercise in the TUI (`cargo run -p zedkata-tui` and press `o`).
2. Close the `.kata` tab in Zed. Observe the TUI status change.
3. In Zed, trigger `workspace: restart language server`. Re-open the buffer and confirm the live attempt immediately reconnects.

## Resume and Recovery After Restart (ZK-021.2)

Session persistence and restart lifecycle guarantees are tested in `crates/zedkata-tui/tests/restart_recovery_tests.rs`:

```powershell
cargo test -p zedkata-tui --test restart_recovery_tests
```

### 1. Manual Quit / Restart Restoring Completion
- When the student quits (`q`, `Esc`, or process exit) and relaunches `zedkata-tui` (or runs headless CLI commands), the application reloads `session.json`.
- Completed exercises and technique scores in `history` are preserved and reloaded.
- Core-derived prerequisite unlocks are re-evaluated: lessons whose required completed exercises are fulfilled remain unlocked, and completed lessons display `[✓]`.
- Navigation and active drill focus restore directly to the session's active `current.lesson_id` and `current.exercise_index`.

### 2. Unfinished Attempt Handling
- If an attempt was in progress (`verdict: in_progress` or unfinalized) when the student quit:
  - The attempt identity (`id`), recorded edit count, and buffer path are retained.
  - An unfinished attempt is **never counted as a failure in history** until explicitly abandoned or finalized.
  - If the buffer remains on disk, re-opening Zed or pressing `[o]` seamlessly reconnects.
  - If the buffer was deleted or cleaned up while the app was closed, the TUI detects `Connection: STALE BUFFER` with actionable recovery: press `[o]` to regenerate the pristine fixture, or press `[r]` to reset.

### 3. Corrupt and Incompatible State Recovery (No Silent Overwrite)
- If `session.json` is corrupted (syntax error, truncated file) or has an unsupported future schema version (`version > 1`):
  - The application **never silently overwrites** the file with default state.
  - `SessionState::load_or_recover` automatically renames and preserves the unparseable file to `session.json.corrupt.<nanos>.bak`.
  - A clean default state is initialized in memory, and an actionable warning is surfaced in the TUI header and feedback pane:
    `Recovery: Preserved corrupt session at "session.json.corrupt.<nanos>.bak"`
  - Subsequent session writes create a valid `session.json` without destroying the backed-up file, allowing manual inspection and recovery.

### 4. Missing Fixture Safeguards
- If a curriculum lesson references a start or goal fixture that is missing or deleted from disk:
  - `App::current_exercise_fixture_status` detects `MissingStart` or `MissingGoal`.
  - The TUI surfaces a high-visibility `Fixture Error: Start fixture missing at <path>` notice in the Task pane.
  - `launch_selected` and `reset_active` safely abort without spawning invalid Zed processes or corrupting session state.

## Supported Terminal Hosting Routes (ZK-023.2, OQ-08)

The `zedkata-tui` drill conductor requires an interactive terminal emulator supporting raw mode, alternate screen buffers (`crossterm::terminal::EnterAlternateScreen`), and responsive geometry reflow. Following the empirical verification in [ZK-023.2](BACKLOG.md#task-zk-023-2) (recorded in `docs/evidence/terminal-hosting-routes.md`), ZedKata advertises and officially supports three hosting routes:

### 1. Standard Zed Terminal Pane (Officially Supported & Recommended)
- **How to launch:** Run command `workspace: new terminal` (`Ctrl-~` or bottom dock toggle).
- **Execution:** Run `cargo run -p zedkata-tui` or the compiled `zedkata-tui.exe` binary.
- **Behavior:** ConPTY and Zed's internal VTE terminal cleanly engage alternate screen mode, hide the cursor, and capture vim navigation keystrokes without conflicting with Zed editor keymaps.
- **Fullscreen / Zoom:** The dock can be zoomed to full editor height using `Shift+Esc` or double-clicking the dock tab header.

### 2. Zed Center Terminal Tab (Officially Supported & Recommended)
- **How to launch:** Run command `workspace: new center terminal`.
- **Execution:** Opens a terminal session as a standard editor tab in Zed's central workspace.
- **Side-by-Side Split:** Can be dragged to a vertical split next to the active exercise buffer (e.g., student buffer on the left, `zedkata-tui` drill conductor on the right). This provides the highest ergonomic workflow inside a single Zed window.

### 3. External Terminal Emulator (Officially Supported & Recommended)
- **How to launch:** Windows Terminal, Alacritty, WezTerm, PowerShell, or macOS/Linux native terminals.
- **Execution:** Ideal for multi-monitor workstations where the student dedicates one screen to Zed and an adjacent screen to the TUI drill conductor.

### Non-Recommended Route: Zed Terminal Threads
- **Evaluation:** Zed's Assistant / Agent Panel supports a `Terminal` thread route (`agent::NewTerminalThread`), designed primarily for streaming AI CLI agents (Claude Code, Amp, etc.).
- **Why not advertised:** The Agent Panel defaults to a narrow sidebar dock (~30–45 columns), which immediately triggers the `< 60x15` minimum dimension overlay. Furthermore, Terminal Threads cannot be split or zoomed as standard workspace tabs, and their thread container remains retained in Zed's agent metadata store upon process exit. Students should use standard terminal panes or center terminal tabs instead.

### Terminal Sizing & Restoration Guarantees
- **Minimum Geometry:** The TUI requires a minimum of **60 columns by 15 rows**. When the terminal is resized below this threshold, a centered warning overlay (`Terminal size too small (WxH). Minimum size is 60x15.`) is displayed until adequate space is restored.
- **Restoration:** `zedkata-tui` incorporates an RAII `TerminalGuard` and a custom `std::panic::set_hook`. On exit (`q`, `Esc`), interrupt (`Ctrl-C`), unhandled error (`--test-error`), or panic (`--test-panic`), the terminal is guaranteed to leave the alternate screen, disable raw mode, and restore cursor visibility, leaving the shell prompt completely clean.

## Task handoff

ZK-001 through ZK-022, **ZK-023.1** (Accept the starter-course user journey), and **ZK-023.2** (Check supported terminal hosting routes) are complete. Local MVP milestone **ZK-023** is fully accepted! Full evidence is recorded in `docs/evidence/local-mvp.md` and `docs/evidence/terminal-hosting-routes.md`.

Next is **Phase 3 (Reference Curriculum & Full Release)** starting with **ZK-024.1** ([ZK-024.1](BACKLOG.md#task-zk-024-1) — Audit capabilities and settle module ordering).




