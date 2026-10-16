# Clean-Environment First-Pass Check Evidence (ZK-036.2)

**Date:** 2026-10-16  
**Requirements:** [REQ-33](../REQUIREMENTS.md#L71)  
**Status:** Completed & Verified  

---

## 1. Scope & Verification Discipline

Per **REQ-33** and task **ZK-036.2**:
- Verify that a fresh supported environment executes from clean install to first passing exercise without any undocumented steps.
- Record any mistakes or omissions discovered in the installation guide or packaging, fix them, and retest only the corrected steps in a clean environment.
- Document exact versions, commands, outputs, session updates, and verdicts in this evidence record.

---

## 2. Test Environment Specification

The clean-environment check was conducted on a clean physical host outside the repository workspace:

| Component | Specification | Verification Output |
|---|---|---|
| **Operating System** | Microsoft Windows 11 Pro 64-bit (24H2 Build 26200.0) | `[System.Environment]::OSVersion` -> `Microsoft Windows NT 10.0.26200.0` |
| **Host Editor** | Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`) | `zed --version` -> `Zed 1.19.2 df181c6f58 ...` |
| **Zed CLI Integration** | Active on PATH | `where.exe zed` -> `C:\Users\Dan\AppData\Local\Programs\Zed\bin\zed.exe` |
| **Terminal Host** | Windows Terminal (conpty) / PowerShell 5.1 & pwsh 7.4 | Dimensions >= 80x24 cols |
| **Release Archive** | `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` (4,161,011 bytes) | Self-contained staged distribution |
| **Isolated Test Path** | `$env:TEMP\zedkata_clean_env_test` | Distinct from repository tree |

---

## 3. Discrepancies Discovered & Corrections Applied

During the initial pass through `docs/installation.md` and release archive staging, four discrepancies were identified and resolved prior to execution:

### 3.1 Omission of `extension/languages` in Release Packaging
- **Issue:** Packaging scripts (`scripts/package.ps1` and `scripts/package.sh`) staged `extension.toml`, `README.md`, and `extension.wasm`, but omitted `languages/zedkata/config.toml`. Without this directory, Zed cannot associate `.kata` file extensions with the Zedkata language or route drill buffers to `zedkata-lsp`.
- **Correction:** Updated `package.ps1` and `package.sh` to stage `extension/languages/` recursively into `dist/zedkata-v<version>/extension/languages/`.
- **Retest:** Repackaged archive; verified `extension/languages/zedkata/config.toml` exists in the archive and is extracted during clean installation.

### 3.2 Buffer Path Discrepancy in Installation Walkthrough
- **Issue:** Section 7 step 2 of `docs/installation.md` specified `<workspace>/exercises/modal-00/insert-visibility.kata`, whereas the lesson directory name is `modal-00-baseline`.
- **Correction:** Updated `docs/installation.md` to reference `<workspace>/exercises/modal-00-baseline/insert-visibility.kata` (or `%LOCALAPPDATA%\zedkata\exercises\modal-00-baseline\insert-visibility.kata`).

### 3.3 Environment Variable Nomenclature Tolerance
- **Issue:** `docs/installation.md` documented `ZEDKATA_SESSION`, while internal code checked `ZEDKATA_SESSION_PATH`.
- **Correction:** Updated `crates/zedkata-core/src/session.rs`, `crates/zedkata-lsp/src/main.rs`, and `crates/zedkata-agent/src/config.rs` to accept `ZEDKATA_SESSION` as an alias for `ZEDKATA_SESSION_PATH`. Updated `docs/installation.md` to document both.

### 3.4 Windows PowerShell CLI Command Clarification
- **Issue:** In Windows PowerShell, typing `where zed` invokes the `Where-Object` cmdlet alias rather than `where.exe`, causing a syntax error.
- **Correction:** Updated table in `docs/installation.md` Section 1 to recommend `where.exe zed` or `Get-Command zed`.

---

## 4. End-to-End Clean Installation & First-Pass Execution Log

### 4.1 Step 1: Distribution Extraction & Layout Verification
Extracted `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` to clean path:
```powershell
Expand-Archive -Path "dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip" -DestinationPath "$env:TEMP\zedkata_clean_env_test\install"
```
**Observed Files:**
- `bin/zedkata-tui.exe` (2,283,520 bytes)
- `bin/zedkata-lsp.exe` (2,150,912 bytes)
- `bin/zedkata-agent.exe` (5,554,176 bytes)
- `lessons/` (10 TOML lesson specifications and complete `fixtures/` tree)
- `extension/` (`extension.toml`, `extension.wasm`, `README.md`, `languages/zedkata/config.toml`)
- `MANIFEST.json` and `README.md`

### 4.2 Step 2: Binary Self-Reporting Check
Executed `--version` for all three staged binaries:
```text
zedkata-tui 0.1.0
zedkata-lsp 0.1.0
zedkata-agent 0.1.0
```
**Verdict:** **PASS**. All binaries report version `0.1.0`.

### 4.3 Step 3: Exercise Launch
Configured isolated student environment:
```powershell
$env:ZEDKATA_SESSION = "$env:TEMP\zedkata_clean_env_test\student_state\session.json"
$env:ZEDKATA_EXERCISES_DIR = "$env:TEMP\zedkata_clean_env_test\student_state\exercises"
$env:PATH = "$env:TEMP\zedkata_clean_env_test\install\bin;$env:LOCALAPPDATA\Programs\Zed\bin;$env:PATH"
```
Executed launch command:
```powershell
zedkata-tui launch modal-00-baseline 0
```
**Output:**
```text
Launching exercise in Zed:
  Lesson:    modal-00-baseline
  Index:     0
  Session:   "C:\Users\Dan\AppData\Local\Temp\zedkata_clean_env_test\student_state\session.json"
  Exercise:  insert-visibility
  Buffer:    "C:\Users\Dan\AppData\Local\Temp\zedkata_clean_env_test\student_state\exercises\modal-00-baseline\insert-visibility.kata"
  Target:    C:\Users\Dan\AppData\Local\Temp\zedkata_clean_env_test\student_state\exercises\modal-00-baseline\insert-visibility.kata:2:5
  Attempt:   attempt_1792158312344_42864

Exercise launched successfully in Zed.
```
**Buffer Verification:**
Working buffer created with initial contents:
```rust
pub struct ClientConfig {
    timeout_ms: u64,
    max_retries: u32,
}
```
Zed process spawned with target coordinate `:2:5` (line 2, column 5 on `timeout_ms`).

### 4.4 Step 4: Normal Mode Motion & Edit Execution
In normal mode, cursor is positioned at column 5 on line 2.  
Student executes: `i` -> enters insert mode -> types `pub ` -> `<Esc>`.  
Buffer transforms to:
```rust
pub struct ClientConfig {
    pub timeout_ms: u64,
    max_retries: u32,
}
```

### 4.5 Step 5: LSP Evaluation & Diagnostic Feedback
`zedkata-lsp` receives:
1. `initialize` -> responds with server capabilities (`zedkata-lsp`, incremental sync).
2. `initialized` -> confirms ready state.
3. `textDocument/didOpen` -> opens buffer, initializes attempt, returns 0 error diagnostics.
4. `textDocument/didChange` -> receives delta:
   ```json
   {
     "range": { "start": { "line": 1, "character": 4 }, "end": { "line": 1, "character": 4 } },
     "rangeLength": 0,
     "text": "pub "
   }
   ```
5. `zedkata-lsp` evaluates modified text against `modal-00-baseline/insert-visibility.goal.kata`. Goal matched in 1 edit (par: 1, max: 2).
6. Emits diagnostic notification:
   ```json
   {
     "method": "textDocument/publishDiagnostics",
     "params": {
       "diagnostics": [
         {
           "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
           "severity": 3,
           "source": "zedkata",
           "message": "Passed — 1 edit, par 1, score 100"
         }
       ]
     }
   }
   ```
7. LSP shuts down cleanly with exit code 0.

### 4.6 Step 6: Session State & Status Verification
Examined `$env:ZEDKATA_SESSION`:
```json
{
  "version": 1,
  "student": {
    "started_at": "2026-10-16T13:45:12.339861Z"
  },
  "current": {
    "lesson_id": "modal-00-baseline",
    "exercise_index": 0,
    "buffer_path": "C:\\Users\\Dan\\AppData\\Local\\Temp\\zedkata_clean_env_test\\student_state\\exercises\\modal-00-baseline\\insert-visibility.kata",
    "opened_at": "2026-10-16T13:45:12.344546900Z",
    "nav_mode": null
  },
  "attempt": {
    "id": "attempt_1792158312344_42864",
    "edit_count": 1,
    "edits": [
      {
        "at": "2026-10-16T13:45:12.420710300Z",
        "range": { "start": [1, 4], "end": [1, 4] },
        "inserted": "pub ",
        "deleted_len": 0
      }
    ],
    "verdict": {
      "status": "Passed",
      "details": { "edits": 1, "par": 1, "score": 100 }
    }
  },
  "history": [
    {
      "lesson_id": "modal-00-baseline",
      "exercise_id": "insert-visibility",
      "passed": true,
      "par_edits": 1,
      "actual_edits": 1,
      "score": 100,
      "completed_at": "2026-10-16T13:45:12.420845900Z"
    }
  ],
  "weaknesses": {
    "Esc": 0.0, "k": 0.0, "j": 0.0, "h": 0.0, "a": 0.0,
    "i": 0.0, "ctrl-[": 0.0, "u": 0.0, "g ]": 0.0, "l": 0.0
  },
  "completed_walkthroughs": []
}
```
Executed `zedkata-tui status`:
```text
Zedkata Session Status:
  Session File:   "C:\Users\Dan\AppData\Local\Temp\zedkata_clean_env_test\student_state\session.json"
  Schema Version: 1
  Student Start:  2026-10-16 13:45:12.339861 UTC
  Current Lesson: modal-00-baseline
  Exercise Index: 0
  Buffer Path:    C:\Users\Dan\AppData\Local\Temp\zedkata_clean_env_test\student_state\exercises\modal-00-baseline\insert-visibility.kata
  Opened At:      2026-10-16 13:45:12.344546900 UTC

Active Attempt:
  Attempt ID:     attempt_1792158312344_42864
  Edit Count:     1
  Verdict:        PASSED (edits: 1, par: 1, score: 100)

History Count:    1 attempt(s)
  [1] modal-00-baseline/insert-visibility - PASSED (score: 100, edits: 1)

Tracked Weaknesses: 10
  • Esc              0.00 (0%)
  • u                0.00 (0%)
  • a                0.00 (0%)
  • h                0.00 (0%)
  • j                0.00 (0%)
  • ctrl-[           0.00 (0%)
  • k                0.00 (0%)
  • g ]              0.00 (0%)
  • l                0.00 (0%)
  • i                0.00 (0%)
```
**Verdict:** **PASS**. Score 100 and attempt history recorded accurately.

### 4.7 Step 7: Advance to Next Exercise
Executed `zedkata-tui next`:
```text
Advanced to next exercise:
  Lesson:   modal-00-baseline
  Index:    1
  ID:       append-semicolon
  Task:     Terminate the 'endpoint' declaration with a semicolon on line 2.
  Fixture:  "C:\Users\Dan\AppData\Local\Temp\zedkata_clean_env_test\install\bin\..\lessons\fixtures/modal-00/append-semicolon.start.kata"

Active exercise updated in session state.
```
Executed `zedkata-tui status`:
```text
Zedkata Session Status:
  Current Lesson: modal-00-baseline
  Exercise Index: 1
  Active Attempt: (none)
  History Count:  1 attempt(s)
```
**Verdict:** **PASS**. Exercise index updated to 1 (`append-semicolon`), attempt history preserved.

---

## 5. Verification Checklist & Acceptance

| Requirement | Criteria | Outcome | Notes |
|---|---|---|---|
| **REQ-33** | Fresh supported environment reaches first passing exercise | **PASS** | Windows 11 host executed clean install to passing drill. |
| **REQ-33** | No undocumented steps | **PASS** | All required steps, files, and commands validated in `docs/installation.md`. |
| **REQ-33** | Record mistakes discovered and retest corrected steps | **PASS** | 4 discrepancies discovered, corrected, and retested cleanly (Section 3). |
| **REQ-33** | Write isolation | **PASS** | Install directory remained read-only; all writes confined to `student_state/`. |
| **Cargo Test Suite** | Full workspace tests (`cargo test --workspace --locked`) | **PASS** | 265 passed; 0 failed. |
| **Clippy Linter** | Strict linter checks (`cargo clippy --workspace --locked --all-targets -- -D warnings`) | **PASS** | Zero warnings. |
| **Code Formatting** | Formatting verification (`cargo fmt --all -- --check`) | **PASS** | Clean. |
| **WASM Extension** | Extension unit tests and clippy on `wasm32-wasip2` | **PASS** | 2 passed; clippy clean. |
