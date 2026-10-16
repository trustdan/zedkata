# ZK-034.2 Verify Installed-Path Discovery and Writes Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-31  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-31** and **ZK-034.2**:
- **Acceptance:** Run from outside source checkout and from paths with spaces; read packaged lessons and write state/exercises/generated content only to agreed writable paths; read-only install directory works.
- **Installed-Path Discovery:**
  - Binaries installed in paths with spaces (e.g. `C:\Program Files\Zedkata Course 2026\bin\`) discover their packaged lessons via `<exe_dir>/../lessons` without relying on working directory or hardcoded repository paths.
  - Binaries run from any external working directory (with or without spaces) successfully discover curriculum lessons.
- **Agreed Writable Paths Isolation:**
  - `session.json` state, `exercises/` working buffers, and `lessons/generated/` content are written strictly to agreed writable directories (e.g. `%LOCALAPPDATA%\zedkata` or explicit `--session` / `--data-dir`).
  - Zero files are created or modified in the install directory during runtime.
- **Read-Only Install Directory:**
  - When the installation directory is set to read-only (simulating standard system installation like `C:\Program Files\Zedkata` or `/usr/local/`), the application executes drills, tracks progress, launches buffers, and handles feedback without permission errors.

---

## 2. Architecture & Design

### 2.1 Installed Tree Simulation
Create a test harness in `crates/zedkata-tui/tests/installed_path_tests.rs` (and a companion verification script `scripts/check-installed-paths.ps1`):
1. Creates an installation root with spaces:
   `<temp_dir>/Zedkata Tools 2026/`
2. Copies release binaries to `Zedkata Tools 2026/bin/` and lessons to `Zedkata Tools 2026/lessons/`.
3. Creates a completely separate external student workspace directory with spaces:
   `<temp_dir>/Student Workspace 2026/`
4. Sets the entire `Zedkata Tools 2026/` directory tree to read-only (Windows: ReadOnly file attributes).

### 2.2 Functional Invocations from External Directory
Execute the staged binaries from the external workspace:
1. `zedkata-tui select modal-00-baseline 0 --session "<workspace>/state/session.json"`
   - Verifies packaged lessons are discovered via executable parent.
   - Verifies `session.json` is written to `<workspace>/state/session.json`.
   - Verifies install directory remains unmodified.
2. `zedkata-tui status --session "<workspace>/state/session.json"`
   - Reads session state and reports current status cleanly.
3. Exercise buffer launch via `launcher::launch_exercise` with a mock runner:
   - Verifies writable exercise buffer is created under `<workspace>/state/exercises/`.
   - Verifies right-to-left line:col argument parsing works with paths containing spaces.
   - Verifies zero writes in read-only install directory.
4. `zedkata-lsp --stdio`:
   - Runs LSP feedback session pointing to the writable session path and external exercise buffer.
   - Verifies diagnostic notifications and session updates occur in writable directory.
5. `zedkata-agent walkthrough list`:
   - Lists available walkthroughs when executed from external working directory.

### 2.3 Verification Matrix
- [x] Execution from path with spaces.
- [x] Execution from external working directory outside source checkout.
- [x] Read-only install directory compatibility.
- [x] All state and buffers written only to designated writable path.
- [x] Clean uninstall / temp cleanup leaves zero residual files in install tree.

---

## 3. Implementation Steps

1. **Author Companion Check Script:** Create `scripts/check-installed-paths.ps1` for automated PowerShell verification.
2. **Author Automated Integration Suite:** Create `crates/zedkata-tui/tests/installed_path_tests.rs` exercising all acceptance criteria.
3. **Execute Verification:** Run the tests and PowerShell checks against the release binaries.
4. **Document Results:** Author `docs/evidence/installed-path-discovery.md`, update `docs/DEVELOPMENT.md`, and update `docs/BACKLOG.md`.
