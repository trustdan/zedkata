# Installed-Path Discovery and Writable Writes Verification (ZK-034.2)

**Date:** 2026-10-15  
**Requirements:** [REQ-31](../REQUIREMENTS.md#L69)  
**Status:** Completed & Verified  

---

## 1. Overview & Acceptance Requirements

Per **REQ-31** and **ZK-034.2**:
- **Acceptance:** Run from outside source checkout and from paths with spaces; read packaged lessons and write state/exercises/generated content only to agreed writable paths; read-only install directory works.
- **Installed Tree Discovery:**
  - Binaries installed in directories containing spaces (e.g. `C:\Program Files (x86)\Zedkata Tools 2026\bin\`) discover external curriculum lessons via `<exe_dir>/../lessons` without relying on working directory or hardcoded repository paths.
  - Binaries run from any external working directory (with or without spaces) successfully discover curriculum lessons.
- **Agreed Writable Paths Isolation:**
  - Student progress (`session.json`), exercise working buffers (`exercises/`), and generated practice drills (`lessons/generated/`) are written strictly to agreed user writable paths (`%LOCALAPPDATA%\zedkata` or explicit `--session` / `--data-dir`).
  - Zero files are created, mutated, or deleted inside the installation directory during execution.
- **Read-Only Install Directory Compatibility:**
  - When the installation directory is set to read-only (simulating standard system installation like `C:\Program Files\Zedkata` or `/usr/local/`), the application executes drills, tracks progress, launches buffers, and handles feedback without permission errors.

---

## 2. Test Environment & Simulation Matrix

A simulated production deployment was executed both via PowerShell check script ([scripts/check-installed-paths.ps1](file:///c:/Users/Dan/zedkata/scripts/check-installed-paths.ps1)) and automated Rust integration suite ([crates/zedkata-tui/tests/installed_path_tests.rs](file:///c:/Users/Dan/zedkata/crates/zedkata-tui/tests/installed_path_tests.rs)):

| Dimension | Path / Configuration | Verification Status |
|---|---|---|
| **Install Directory (with spaces)** | `C:\Users\Dan\AppData\Local\Temp\...\Program Files (x86)\Zedkata Tools 2026\` | Verified |
| **Install Permissions** | Read-only attributes applied to all files and directories | Verified |
| **External Working Directory (with spaces)** | `C:\Users\Dan\AppData\Local\Temp\...\My Student Workspace\Projects 2026\` | Verified |
| **Writable State Directory (with spaces)** | `...\My Student Workspace\Projects 2026\user state\session.json` | Verified |
| **Writable Exercises Directory (with spaces)** | `...\My Student Workspace\Projects 2026\user state\exercises\` | Verified |

---

## 3. Verification Details & Test Cases

Automated test suite `crates/zedkata-tui/tests/installed_path_tests.rs` tests the complete installed runtime lifecycle:

### 3.1 Binary Discovery from External Working Directory with Spaces
- **Test:** `test_installed_tree_binary_discovery_from_external_cwd_with_spaces`
- **Actions:**
  - Invokes `zedkata-lsp --version` from external workspace.
  - Invokes `zedkata-tui select modal-00-baseline 0 --session "<workspace>\user state\session.json"`.
- **Observations:**
  - `zedkata-tui` and `zedkata-lsp` successfully discover curriculum lessons from `<exe_dir>/../lessons`.
  - Session state created at `<workspace>\user state\session.json` with active lesson `modal-00-baseline`.
  - Exit code 0.

### 3.2 Read-Only Install Tree Integrity
- **Test:** `test_read_only_install_tree_integrity`
- **Actions:**
  - Applies strict read-only file attributes across the entire installed tree.
  - Invokes `select motions-01-word 0` and `status` from external cwd.
  - Compares pre-run and post-run file inventories across the install directory.
- **Observations:**
  - Commands succeed without permission errors or crash.
  - Diff comparison confirms zero files added, modified, or deleted in the install tree.

### 3.3 Exercise Buffer Launch in Writable Path with Spaces
- **Test:** `test_exercise_buffer_written_to_agreed_writable_path_with_spaces`
- **Actions:**
  - Invokes `launcher::launch_exercise` with a mock runner.
- **Observations:**
  - Exercise buffer created at `<workspace>\user state\exercises\modal-00\insert-visibility.kata`.
  - CLI target argument formatted with right-to-left line:col parsing (`:2:5`).
  - Session state records active buffer path.
  - Zero writes to install tree.

### 3.4 LSP Feedback Engine with Read-Only Install and External Paths
- **Test:** `test_lsp_feedback_engine_with_read_only_install_and_external_paths`
- **Actions:**
  - Initializes `FeedbackEngine` pointing to external session and read-only catalog.
  - Emulates buffer open (`on_document_open`) and student goal completion (`on_document_change`).
- **Observations:**
  - Diagnostic notification emitted.
  - Session state updated in writable directory with passed verdict and history entry.
  - Zero writes to read-only install tree.

### 3.5 Agent CLI from External Working Directory with Spaces
- **Test:** `test_agent_cli_from_external_cwd_with_spaces`
- **Actions:**
  - Invokes `zedkata-agent walkthrough list --session "<workspace>\user state\session.json"`.
- **Observations:**
  - Outputs full list of 10 curriculum walkthroughs.
  - Exit code 0.

---

## 4. Execution Output

### Rust Integration Test Suite (`cargo test -p zedkata-tui --test installed_path_tests`):
```text
running 5 tests
test test_agent_cli_from_external_cwd_with_spaces ... ok
test test_exercise_buffer_written_to_agreed_writable_path_with_spaces ... ok
test test_lsp_feedback_engine_with_read_only_install_and_external_paths ... ok
test test_read_only_install_tree_integrity ... ok
test test_installed_tree_binary_discovery_from_external_cwd_with_spaces ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
```

### PowerShell Verification Script (`scripts/check-installed-paths.ps1`):
```text
==================================================
 Zedkata Installed-Path & Writable Writes Check   
==================================================

[1/5] Creating simulated install and workspace paths...
  Install Tree: C:\Users\Dan\AppData\Local\Temp\zedkata_install_test_c5b98de5\Program Files (x86)\Zedkata Tools 2026
  Workspace:    C:\Users\Dan\AppData\Local\Temp\zedkata_install_test_c5b98de5\My Student Workspace\Projects 2026

[2/5] Copying staged distribution into install tree...

[3/5] Marking install directory strictly read-only...
  [OK] Read-only file attributes applied

[4/5] Executing binaries from external workspace...
  [OK] zedkata-tui --version: zedkata-tui 0.1.0
  [OK] zedkata-lsp --version: zedkata-lsp 0.1.0
  [OK] zedkata-agent --version: zedkata-agent 0.1.0
  Running 'select modal-00-baseline 0'...
  [OK] Exercise selected successfully
  Running 'status'...
  [OK] Status reported cleanly
  Running 'walkthrough list'...
  [OK] Walkthrough list reported cleanly

[5/5] Verifying write isolation and read-only install tree...
  [OK] State verified in writable path: ...\user state\session.json
  [OK] Verified zero files added or modified in read-only install tree

==================================================
 Installed-Path & Write Isolation Check: PASSED!  
==================================================
```
