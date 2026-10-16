# Platform Native Builds Evidence (ZK-035.1)

**Date:** 2026-10-15  
**Author:** Pair Programmer  
**Task:** ZK-035.1 — Add macOS and Linux native builds  
**Requirements:** REQ-30, REQ-32  
**Status:** Complete  

---

## 1. Objectives & Declared Targets

Per **REQ-30**, **REQ-32**, and task **ZK-035.1**, native build, test, and packaging jobs must produce release artifacts for declared platform targets alongside Windows in `.github/workflows/ci.yml`. Actual results must be recorded, and failed or untested targets must remain strictly out of verified support claims until concrete execution is observed and recorded.

### 1.1 Declared Platform Targets

| Platform | Target Triple | Runner Environment | Artifact Archive | Support Status |
|----------|---------------|-------------------|------------------|----------------|
| **Windows (x64)** | `x86_64-pc-windows-msvc` | `windows-latest` | `zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` | **VERIFIED** (Local Host & CI) |
| **Linux (x64)** | `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `zedkata-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` | **PENDING HOSTED CI / SMOKE-TEST (ZK-035.3)** |
| **macOS (Apple Silicon)** | `aarch64-apple-darwin` | `macos-latest` | `zedkata-v0.1.0-aarch64-apple-darwin.tar.gz` | **PENDING HOSTED CI / SMOKE-TEST (ZK-035.3)** |
| **macOS (Intel)** | `x86_64-apple-darwin` | `macos-13` | `zedkata-v0.1.0-x86_64-apple-darwin.tar.gz` | **PENDING HOSTED CI / SMOKE-TEST (ZK-035.3)** |

---

## 2. CI Workflow Configuration (`.github/workflows/ci.yml`)

The GitHub Actions workflow was updated to declare and execute locked cross-platform jobs:

1. **Formatting (`fmt`):**
   - Runner: `ubuntu-latest`
   - Command: `cargo fmt --all -- --check`
2. **Clippy Linter (`clippy`):**
   - Runner: `ubuntu-latest`
   - Command: `cargo clippy --workspace --locked --all-targets -- -D warnings`
3. **Tests (`test`):**
   - Matrix across `windows-latest`, `ubuntu-latest`, `macos-latest`, and `macos-13`.
   - Toolchain setup: `dtolnay/rust-toolchain@stable` targeting `${{ matrix.target }}`.
   - Build locked release binaries: `cargo build --workspace --locked --release --target ${{ matrix.target }}`.
   - Stage distribution tree using platform packaging scripts (`pwsh scripts/package.ps1` on Windows, `bash scripts/package.sh` on Unix).
   - Execute locked workspace tests: `cargo test --workspace --locked`.
   - Validate catalog integrity: `cargo test -p zedkata-core --test catalog_validation --locked -- --nocapture`.
4. **Packaging Artifacts (`package-artifacts`):**
   - Matrix across all 4 declared target triples.
   - Builds locked release binaries.
   - Stages distribution and generates archive (`.zip` for Windows, `.tar.gz` for Linux/macOS).
   - Uploads distribution archive and staged tree via `actions/upload-artifact@v4`.
5. **MSRV (`msrv`):**
   - Runner: `ubuntu-latest`
   - Toolchain: `1.85.0`
   - Command: `cargo check --workspace --locked`.

---

## 3. Packaging Scripts Enhancements

Both packaging scripts were upgraded to support automated distribution archiving and standardized manifest generation:

### 3.1 `scripts/package.ps1`
- Added `-Archive` switch.
- Produces self-contained `.zip` file: `dist/zedkata-v<version>-<target>.zip`.
- Computes SHA-256 hashes and file sizes for `MANIFEST.json`.
- Tested locally: created `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` (4,079,799 bytes).

### 3.2 `scripts/package.sh`
- Added `--archive` flag.
- Produces self-contained `.tar.gz` archive: `dist/zedkata-v<version>-<target>.tar.gz`.
- Computes SHA-256 hashes via `sha256sum` or `shasum -a 256` and captures file sizes for `MANIFEST.json`.
- Tested in bash environment: created `dist/zedkata-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` (4,066,106 bytes).

---

## 4. Local Execution & Validation Evidence

### 4.1 Local Windows Checks
Executed on development host (Windows 11, `x86_64-pc-windows-msvc`):
- **Full Workspace Tests:**
  ```text
  cargo test --workspace --locked
  Result: 265 passed; 0 failed; 0 ignored; 0 filtered out
  ```
- **Catalog Validation:**
  ```text
  cargo test -p zedkata-core --test catalog_validation --locked -- --nocapture
  Result: 22 passed; 0 failed; 0 ignored
  ```
- **Code Formatting:**
  ```text
  cargo fmt --all -- --check
  Result: clean (exit code 0)
  ```
- **Clippy Linting:**
  ```text
  cargo clippy --workspace --locked --all-targets -- -D warnings
  Result: clean (exit code 0)
  ```
- **Release Packaging & Archiving:**
  ```text
  powershell -ExecutionPolicy Bypass -File scripts/package.ps1 -Archive
  Result: Successfully created dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip
  ```
- **Installed-Path Discovery & Read-Only Checks:**
  ```text
  powershell -ExecutionPolicy Bypass -File scripts/check-installed-paths.ps1
  Result: PASSED (read-only install tree verified, external working directory with spaces verified)
  ```
- **Bash Packaging Check:**
  ```text
  bash scripts/package.sh --skip-build --archive
  Result: Successfully created dist/zedkata-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
  ```

---

## 5. Support Claims Policy & Scope Discipline

Per project guidelines and acceptance criteria:
- **Verified Support:** Windows 11 (`x86_64-pc-windows-msvc`) is locally verified with end-to-end evidence.
- **Untested / Pending Targets:** Linux (`x86_64-unknown-linux-gnu`) and macOS (`aarch64-apple-darwin`, `x86_64-apple-darwin`) are declared in the CI build and packaging matrix, but are **NOT** claimed as verified runtime environments until:
  1. Hosted GitHub Actions CI completes cleanly on those runners; and
  2. The platform smoke-test matrix is executed and dated evidence is recorded in ZK-035.3 (`docs/evidence/platforms.md`).
- This preserves evidence discipline and avoids declaring support for environments where interactive terminal and editor integration has not yet been directly demonstrated.
