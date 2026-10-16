# ZK-035.1 Add macOS and Linux Native Builds Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-30, REQ-32  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-30**, **REQ-32**, and **ZK-035.1**:
- **Acceptance:** Locked native build/test jobs produce artifacts for declared targets alongside Windows in `.github/workflows/ci.yml`; record actual results and keep failed/untested targets out of verified support claims.
- **Declared Targets:**
  - Windows: `x86_64-pc-windows-msvc` (`windows-latest`)
  - Linux: `x86_64-unknown-linux-gnu` (`ubuntu-latest`)
  - macOS Apple Silicon: `aarch64-apple-darwin` (`macos-latest`)
  - macOS Intel: `x86_64-apple-darwin` (`macos-13`)
- **Workflow Mechanics:**
  - Automated CI test matrix executing locked tests (`cargo test --workspace --locked` and catalog validation) across declared targets.
  - Automated release packaging job producing packaged distribution archives (`.zip` for Windows, `.tar.gz` for Linux/macOS) and publishing them via `actions/upload-artifact@v4`.
- **Support Claims Discipline:**
  - Record actual execution results.
  - Keep untested or hosted-dependent platforms (Linux, macOS) explicitly marked as **PENDING HOSTED CI / SMOKE-TEST MATRIX (ZK-035.3)** and out of verified support claims until concrete runtime verification is captured.

---

## 2. Implementation Plan

### 2.1 Enhance Packaging Scripts (`scripts/package.ps1` & `scripts/package.sh`)
- **`scripts/package.ps1`:**
  - Add `-Archive` switch.
  - Compress staged tree into `dist/zedkata-v<version>-<target>.zip`.
- **`scripts/package.sh`:**
  - Add `--archive` flag.
  - Compute SHA-256 hashes and sizes for binaries in `MANIFEST.json`.
  - Create `dist/zedkata-v<version>-<target>.tar.gz`.

### 2.2 Refine Test Harness for Cross-Platform Cleanups (`crates/zedkata-tui/tests/installed_path_tests.rs`)
- In `set_dir_readonly`, ensure that when restoring write permissions (`readonly == false`), the directory itself is made writable before modifying its entries, ensuring clean deletion on Unix filesystems.

### 2.3 Expand GitHub Actions CI (`.github/workflows/ci.yml`)
- Update `test` job to run across a platform matrix:
  - `windows-latest` (`x86_64-pc-windows-msvc`)
  - `ubuntu-latest` (`x86_64-unknown-linux-gnu`)
  - `macos-latest` (`aarch64-apple-darwin`)
  - `macos-13` (`x86_64-apple-darwin`)
- Pre-stage distribution tree prior to `cargo test` so that staged and installed path integration suites execute.
- Add `package-artifacts` job that builds locked release binaries, stages release directory, creates distribution archive, and uploads workflow artifacts.

### 2.4 Document Evidence & Platform Status
- Author `docs/evidence/platform-native-builds.md` recording:
  - Target matrix and build definitions.
  - Local verification results on Windows host (all 265 tests passing, locked release binaries built, archive packaged and validated).
  - Explicit distinction between verified platform support (Windows) and pending validation (Linux & macOS).
- Update `docs/DEVELOPMENT.md` and `docs/BACKLOG.md`.
