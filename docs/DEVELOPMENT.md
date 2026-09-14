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

## Task handoff

ZK-001 through ZK-017 (TUI shell and pane rendering) are complete. Next is **ZK-018: Exercise launch** ([ZK-018.1](BACKLOG.md#task-zk-018-1) to verify CLI positioning and fallback, followed by [ZK-018.2](BACKLOG.md#task-zk-018-2) to launch writable exercise copies into Zed). Alternatively, [ZK-019.1](BACKLOG.md#task-zk-019-1) (poll and display live session feedback) is also unblocked.


