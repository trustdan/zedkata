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

## Task handoff

ZK-001 through ZK-018 (exercise launch and writable buffer preparation) are complete. Next is **ZK-019: Live state and connection health** ([ZK-019.1](BACKLOG.md#task-zk-019-1) to poll and display live session feedback in the TUI).


