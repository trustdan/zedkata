# PR Draft: Add Zedkata Extension

**Repository:** `zed-industries/extensions`  
**Base Branch:** `main`  
**Title:** `Add Zedkata extension (v0.1.0)`  

---

### Description

Adds the **Zedkata** extension (`zedkata`), providing language support, syntax highlighting, and drill observer integration for Zedkata modal-editing drills (`.kata` files).

- **Extension ID:** `zedkata`
- **Version:** `0.1.0`
- **Repository:** `https://github.com/trustdan/zedkata`
- **Subdirectory (`path`):** `extension`
- **License:** Dual-licensed under MIT and Apache-2.0 (licenses included in extension root and repository root)

---

### Capabilities & Architecture

- **Language Registration:** Registers the `Zedkata` language for `*.kata` drill buffers.
- **Syntax Highlighting:** Reuses the built-in `rust` grammar without requiring external grammar downloads or wasi-sdk compilation.
- **Language Server Discovery:** Locates the native `zedkata-lsp` observer binary on the user's system `PATH` via `worktree.which("zedkata-lsp")` (with `zedkata-lsp.exe` fallback on Windows).
- **Lightweight WASM:** In accordance with Zed guidelines, the extension binary does not bundle heavy native binaries. If `zedkata-lsp` is missing from `PATH`, the extension displays an actionable error message with installation instructions (`cargo install --path crates/zedkata-lsp` or downloading precompiled release binaries).
- **API Version:** Pinned to `zed_extension_api = "=0.7.0"`, targeting `wasm32-wasip2`.

---

### Verification & Testing

- [x] Verified WASM release compilation: `cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release --locked`
- [x] Verified unit tests: `cargo test --manifest-path extension/Cargo.toml`
- [x] Verified linter: `cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings`
- [x] Verified code formatting: `cargo fmt --manifest-path extension/Cargo.toml -- --check`
- [x] Tested live in Zed 1.19.2 via `zed: install dev extension`:
  - Verified language indicator displays **Zedkata** for `.kata` files.
  - Verified syntax highlighting operates cleanly.
  - Verified actionable error notification when `zedkata-lsp` is absent.
  - Verified bidirectional LSP communication (initialize, open, incremental change, diagnostics) when `zedkata-lsp` is present.
