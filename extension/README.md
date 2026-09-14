# Zedkata Production Extension

This directory contains the production Zed extension for Zedkata. It registers the **Zedkata** language for `.kata` drill buffers, associates syntax highlighting, and discovers and launches the native `zedkata-lsp` observer binary.

## Architectural Boundaries

- **WASM Component:** Compiles to WebAssembly (`wasm32-wasip2`) using the pinned `zed_extension_api` `0.7.0`.
- **Language Registration:** Registers the `Zedkata` language with `.kata` file extension.
- **Grammar Reuse:** Reuses Zed's built-in `rust` grammar for fast syntax highlighting without external tree-sitter or wasi-sdk build dependencies.
- **Server Discovery:** Automatically locates the `zedkata-lsp` native binary on the system PATH via Zed's worktree API (supporting both Unix and Windows `.exe` conventions, as well as paths with spaces).
- **Actionable Errors:** If `zedkata-lsp` is absent from PATH, the extension returns a clear error guiding the developer to install it.

---

## Build & Test

The extension is maintained as an isolated Cargo project (with its own `[workspace]` declaration) so it does not interfere with the native workspace toolchain.

### Prerequisites

Ensure the `wasm32-wasip2` target is installed:

```powershell
rustup target add wasm32-wasip2
```

### Verification Commands

From the repository root:

```powershell
# 1. Check formatting
cargo fmt --manifest-path extension/Cargo.toml --check

# 2. Run Clippy linter against WASM target
cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings

# 3. Run unit tests
cargo test --manifest-path extension/Cargo.toml

# 4. Build release WASM artifact
cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release
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

## Tested Environment

- **Zed Editor:** 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)
- **OS:** Windows 11
- **Toolchain:** Rust 1.97.1 (`stable-x86_64-pc-windows-msvc`), target `wasm32-wasip2`
- **Extension API:** `zed_extension_api = "=0.7.0"`
