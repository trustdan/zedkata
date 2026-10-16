# ZK-035.2 Add Separate Extension and Compatibility Checks Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-30, REQ-32  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-30**, **REQ-32**, and task **ZK-035.2**:
- **Acceptance:** WASM extension check runs separately from native MSRV jobs; release metadata states tested native/extension/Zed combinations and binary lookup expectations.
- **Workflow Mechanics:**
  - Dedicated CI job (`extension`) running independently of native MSRV jobs.
  - Installs `wasm32-wasip2` target.
  - Verifies extension formatting (`cargo fmt`), clippy linting (`cargo clippy --target wasm32-wasip2`), unit tests (`cargo test`), and locked release compilation (`cargo build --target wasm32-wasip2 --release --locked`).
  - Publishes `extension.wasm` artifact via `actions/upload-artifact@v4`.
- **Compatibility & Discovery Documentation:**
  - Update `extension/README.md` to specify:
    1. Tested native/extension/Zed combinations matrix (identifying verified vs. pending targets).
    2. Pinned extension API version (`zed_extension_api = "=0.7.0"`) and WASM target (`wasm32-wasip2`).
    3. Binary lookup expectations: exact probe sequence (`worktree.which`), platform naming (`zedkata-lsp` vs `zedkata-lsp.exe`), argument handling for paths with spaces, and actionable remediation instructions when absent.
- **Packaging Integration:**
  - Update `scripts/package.ps1` and `scripts/package.sh` to stage precompiled `extension.wasm` when present.

---

## 2. Implementation Steps

1. **Plan Formulation:** Author this plan document.
2. **Update CI Workflow (`.github/workflows/ci.yml`):**
   - Add `extension` job targeting `ubuntu-latest` with `wasm32-wasip2`.
   - Run fmt, clippy, test, and release build steps with `--locked`.
   - Upload compiled WASM artifact.
3. **Enhance Packaging Scripts:**
   - Update `scripts/package.ps1` and `scripts/package.sh` to include `extension.wasm` in `dist/zedkata-v<version>/extension/`.
4. **Update Extension Documentation (`extension/README.md`):**
   - Add compatibility matrix and detailed binary lookup specification.
5. **Local Validation:**
   - Run extension formatting, clippy, unit tests, and locked release build.
   - Run packaging scripts to verify `extension.wasm` staging.
   - Run full workspace tests to guarantee zero regressions.
6. **Evidence & Backlog Documentation:**
   - Author `docs/evidence/extension-compatibility-checks.md`.
   - Update `docs/DEVELOPMENT.md` and `docs/BACKLOG.md`.
