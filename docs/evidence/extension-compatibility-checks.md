# Extension Compatibility Checks & WASM Pipeline Evidence (ZK-035.2)

**Date:** 2026-10-15  
**Author:** Pair Programmer  
**Task:** ZK-035.2 — Add separate extension and compatibility checks  
**Requirements:** REQ-30, REQ-32  
**Status:** Complete  

---

## 1. Objectives & Architectural Separation

Per **REQ-30**, **REQ-32**, and task **ZK-035.2**:
1. **Isolated WASM CI Pipeline:** The WebAssembly extension check must run independently from native MSRV jobs in `.github/workflows/ci.yml`.
2. **Release Metadata & Compatibility Statements:** `extension/README.md` must state the tested combinations of native OS, extension versions, and Zed editor versions, distinguishing verified combinations from pending platforms.
3. **Binary Lookup Contract:** Document exact lookup expectations and actionable missing-server error behaviors.

---

## 2. Dedicated WASM Extension Job in CI (`.github/workflows/ci.yml`)

The GitHub Actions workflow includes a dedicated `extension` job:

```yaml
  extension:
    name: WASM Extension Check & Build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-wasip2
      - name: Check extension formatting
        run: cargo fmt --manifest-path extension/Cargo.toml -- --check
      - name: Run extension clippy
        run: cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings
      - name: Run extension unit tests
        run: cargo test --manifest-path extension/Cargo.toml
      - name: Build locked release WASM extension
        run: cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release --locked
      - name: Upload compiled WASM artifact
        uses: actions/upload-artifact@v4
        with:
          name: zedkata-extension-wasm
          path: extension/target/wasm32-wasip2/release/zedkata_extension.wasm
          if-no-files-found: error
```

### Separation from Native MSRV:
- The native `msrv` job validates `zedkata-core`, `zedkata-lsp`, `zedkata-tui`, and `zedkata-agent` against Rust 1.85.0.
- The `extension` job validates `zedkata-extension` targeting `wasm32-wasip2` using `zed_extension_api = "=0.7.0"`.

---

## 3. Compatibility Matrix & Release Metadata

The tested environment and platform compatibility matrix has been documented in `extension/README.md`:

| Platform Target | Zed Version | Extension Ver | Extension API | WASM Target | Native Binary | Verification Status |
|-----------------|-------------|---------------|---------------|-------------|---------------|---------------------|
| **Windows x64** (`x86_64-pc-windows-msvc`) | 1.19.2 (verified) | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp.exe` | **VERIFIED** (Live editor & diagnostic loop) |
| **Linux x64** (`x86_64-unknown-linux-gnu`) | >= 0.140.0 | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp` | **PENDING SMOKE-TEST (ZK-035.3)** |
| **macOS Apple Silicon** (`aarch64-apple-darwin`) | >= 0.140.0 | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp` | **PENDING SMOKE-TEST (ZK-035.3)** |
| **macOS Intel** (`x86_64-apple-darwin`) | >= 0.140.0 | 0.1.0 | `0.7.0` | `wasm32-wasip2` | `zedkata-lsp` | **PENDING SMOKE-TEST (ZK-035.3)** |

---

## 4. Binary Lookup Contract & Error Guarantees

Documented and verified in `extension/src/lib.rs` and `extension/README.md`:
1. **Probe Sequence:**
   - Queries `worktree.which("zedkata-lsp")`.
   - On Windows, falls back to `worktree.which("zedkata-lsp.exe")`.
   - Searches active environment PATH (`~/.cargo/bin`, `%USERPROFILE%\.cargo\bin`, or system paths).
2. **Whitespace Safety:**
   - Command construction passes binary path as a structured argument:
     ```rust
     Command { command: binary_path, args: vec!["--stdio".to_string()], env: vec![] }
     ```
   - Verified that paths containing spaces (e.g., `C:\Program Files (x86)\Zedkata Tools\bin\zedkata-lsp.exe`) are invoked without shell splitting.
3. **Actionable Error Messaging:**
   - Produces detailed instruction when `zedkata-lsp` is absent:
     ```text
     Zedkata LSP server binary ('zedkata-lsp.exe' or 'zedkata-lsp') was not found on PATH.
     To install it from the repository, run:
       cargo install --path crates/zedkata-lsp
     Or build it and ensure the binary directory is included in your PATH:
       cargo build -p zedkata-lsp
     Then restart Zed or reload the extension.
     ```

---

## 5. Local Build & Check Results

All extension check and build jobs executed cleanly on the local development host:

- **Extension Formatting:**
  `cargo fmt --manifest-path extension/Cargo.toml -- --check`  
  **Result:** Clean (exit code 0).
- **Extension Clippy:**
  `cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings`  
  **Result:** Clean (0 warnings).
- **Extension Unit Tests:**
  `cargo test --manifest-path extension/Cargo.toml`  
  **Result:** 2 passed; 0 failed.
- **WASM Release Build:**
  `cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release --locked`  
  **Result:** Compiled `extension/target/wasm32-wasip2/release/zedkata_extension.wasm` (217,705 bytes).
- **Staging Verification:**
  `powershell -ExecutionPolicy Bypass -File scripts/package.ps1 -SkipBuild -Archive`  
  `bash scripts/package.sh --skip-build --archive`  
  **Result:** Both packaging scripts detected the compiled WASM binary and staged it as `dist/zedkata-v0.1.0/extension/extension.wasm`.
