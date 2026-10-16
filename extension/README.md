# Zedkata Production Extension

This directory contains the production Zed extension for Zedkata. It registers the **Zedkata** language for `.kata` drill buffers, associates syntax highlighting, and discovers and launches the native `zedkata-lsp` observer binary.

## Architectural Boundaries

- **WASM Component:** Compiles to WebAssembly (`wasm32-wasip2`) using the pinned `zed_extension_api` `0.7.0`.
- **Language Registration:** Registers the `Zedkata` language with `.kata` file extension.
- **Grammar Reuse:** Reuses Zed's built-in `rust` grammar for fast syntax highlighting without external tree-sitter or wasi-sdk build dependencies.
- **Server Discovery:** Automatically locates the `zedkata-lsp` native binary on the system PATH via Zed's worktree API (supporting both Unix and Windows `.exe` conventions, as well as paths with spaces).
- **Actionable Errors:** If `zedkata-lsp` is absent from PATH, the extension returns a clear error guiding the developer to install it.

---

## Compatibility & Release Metadata Matrix

Per **REQ-30** and **REQ-32**, supported native/extension/Zed combinations and actual verification statuses are tracked explicitly:

| Platform Target | Zed Version | Extension Ver | Extension API | WASM Target | Native Binary | Verification Status |
|-----------------|-------------|---------------|---------------|-------------|---------------|---------------------|
| **Windows x64** (`x86_64-pc-windows-msvc`) | 1.19.2 (verified) | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp.exe` | **VERIFIED** (Live editor & diagnostic loop) |
| **Linux x64** (`x86_64-unknown-linux-gnu`) | >= 0.140.0 | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp` | **PENDING SMOKE-TEST (ZK-035.3)** |
| **macOS Apple Silicon** (`aarch64-apple-darwin`) | >= 0.140.0 | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp` | **PENDING SMOKE-TEST (ZK-035.3)** |
| **macOS Intel** (`x86_64-apple-darwin`) | >= 0.140.0 | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp` | **PENDING SMOKE-TEST (ZK-035.3)** |

> [!NOTE]
> Zed extensions run in a sandboxed WebAssembly runtime (`wasm32-wasip2` or `wasm32-wasip1`). The extension WASM artifact itself is architecture-independent; however, the native language server binary it launches is compiled for the host OS and architecture.

---

## Binary Lookup Expectations

The extension relies on Zed's worktree host environment to locate and execute `zedkata-lsp`. The discovery contract operates as follows:

1. **Discovery Order:**
   - The extension queries `worktree.which("zedkata-lsp")`.
   - On Windows, if `zedkata-lsp` without extension is not found, it falls back to `worktree.which("zedkata-lsp.exe")`.
   - Zed resolves this call against the user's active PATH environment (including `%USERPROFILE%\.cargo\bin` on Windows or `~/.cargo/bin` on Unix, or installed application directories).
2. **Whitespace and Path Safety:**
   - The resolved absolute binary path is encapsulated directly in `zed_extension_api::Command { command: binary_path, args: vec!["--stdio".to_string()], env: vec![] }`.
   - Arguments are passed through structured arrays without shell interpolation, ensuring paths with spaces (e.g., `C:\Program Files (x86)\Zedkata Tools\bin\zedkata-lsp.exe`) execute without string truncation or quote splitting.
3. **Missing Binary Recovery:**
   - If neither binary name is discovered on PATH, the extension produces an actionable, platform-specific error message:
     ```text
     Zedkata LSP server binary ('zedkata-lsp.exe' or 'zedkata-lsp') was not found on PATH.
     To install it from the repository, run:
       cargo install --path crates/zedkata-lsp
     Or build it and ensure the binary directory is included in your PATH:
       cargo build -p zedkata-lsp
     Then restart Zed or reload the extension.
     ```
   - This failure message is propagated to Zed's UI and extension log without crashing the extension host.

---

## Build & Test

The extension is maintained as an isolated Cargo project (with its own `[workspace]` declaration) so it does not interfere with the native workspace toolchain or MSRV requirements.

### Prerequisites

Ensure the `wasm32-wasip2` target is installed:

```powershell
rustup target add wasm32-wasip2
```

### Verification Commands

From the repository root:

```powershell
# 1. Check formatting
cargo fmt --manifest-path extension/Cargo.toml -- --check

# 2. Run Clippy linter against WASM target
cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings

# 3. Run unit tests
cargo test --manifest-path extension/Cargo.toml

# 4. Build locked release WASM artifact
cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release --locked
```

The compiled WebAssembly binary is located at:
`extension/target/wasm32-wasip2/release/zedkata_extension.wasm`.

---

## Installing into Zed for Local Development

1. **Build and install the native LSP binary:**
   Ensure `zedkata-lsp` is built and available on your system `PATH`:
   ```powershell
   cargo install --path crates/zedkata-lsp
   ```
   *(Alternatively, build with `cargo build -p zedkata-lsp` and add `target/debug` to your system `PATH`.)*

2. **Install the Dev Extension in Zed:**
   - Launch Zed (e.g., `zed .` from the repository root).
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) to open the Command Palette.
   - Run `zed: install dev extension`.
   - Browse to and select the `extension/` directory (e.g. `C:\Users\Dan\zedkata\extension`).
   - Zed will compile and activate the extension.

3. **Verify Language Association & Server Startup:**
   - Open any `.kata` file (for example, `lessons/fixtures/motions-01/delete-to-word-end.start.kata`).
   - Look at the bottom right of the Zed window: the language selector should display **Zedkata**.
   - If `zedkata-lsp` is running, hover over or edit the buffer to see live feedback diagnostics.

---

## Troubleshooting

- **Missing Server Binary:**
  If Zed displays an error when opening a `.kata` file:
  `Zedkata LSP server binary ('zedkata-lsp.exe') was not found on PATH.`
  Verify that `%USERPROFILE%\.cargo\bin` (or your local binary folder) is in your system `PATH`, and restart Zed so it inherits the updated environment.
- **Inspect Extension Host Logs:**
  Open the Command Palette in Zed (`Ctrl+Shift+P`) and select `zed: open log`. Any compilation, grammar, or LSP process startup errors will be logged here.
- **Rebuilding after changes:**
  Whenever you modify `extension/`, re-run `zed: install dev extension` and select the directory again, or run `zed: reload extensions`.

---

## Tested Environment Reference

- **Zed Editor:** 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)
- **OS:** Windows 11 Version 24H2
- **Toolchain:** Rust 1.97.1 (`stable-x86_64-pc-windows-msvc`), target `wasm32-wasip2`
- **Extension API:** `zed_extension_api = "=0.7.0"`

