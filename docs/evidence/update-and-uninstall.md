# Update & Uninstall Lifecycle Verification Evidence (ZK-036.3)

**Date:** 2026-10-16  
**Requirements:** [REQ-33](../REQUIREMENTS.md#L71)  
**Status:** Completed & Verified  

---

## 1. Scope & Verification Discipline

Per **REQ-33** and task **ZK-036.3**:
- Document version/schema incompatibility recovery and distinguish application removal from intentional progress removal in [docs/installation.md](../installation.md).
- Manually execute and verify the four core update, schema recovery, application uninstall, and intentional progress purge scenarios on a supported platform (Windows 11).
- Record exact commands, outputs, session states, and verification verdicts in this evidence document.

---

## 2. Environment Specification

| Component | Specification | Verification Output |
|---|---|---|
| **Operating System** | Microsoft Windows 11 Pro 64-bit (24H2 Build 26200.0) | `[System.Environment]::OSVersion` -> `Microsoft Windows NT 10.0.26200.0` |
| **Host Editor** | Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`) | `zed --version` -> `Zed 1.19.2 df181c6f58 ...` |
| **Terminal Host** | Windows Terminal (conpty) / PowerShell 5.1 & pwsh 7.4 | Dimensions >= 80x24 cols |
| **Distribution Archive** | `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` (4,161,011 bytes) | Self-contained staged distribution |
| **Isolated Test Path** | `$env:TEMP\zedkata_update_uninstall_test` | Distinct from repository tree |

---

## 3. Concrete Verification Scenarios & Results

### 3.1 Scenario 1: In-Place Update & Zero-Recompile Lesson Addition

#### Procedure
1. Staged initial release distribution into `$InstallDir`.
2. Created active student progress in `$StateDir\session.json` with a passed attempt:
   - Lesson: `modal-00-baseline`
   - Exercise: `insert-visibility`
   - Status: `Passed`, `score: 100`, `edits: 1`, `par: 1`
   - Tracked weaknesses: `i: 0.0`, `Esc: 0.0`
3. Simulated in-place update by re-extracting release archive over `$InstallDir`:
   ```powershell
   Expand-Archive -Path $ZipArchive -DestinationPath $InstallDir -Force
   ```
4. Added new external curriculum lesson `lessons\update-99-new.toml` directly into the staged lessons tree without recompiling code:
   ```toml
   [lesson]
   id = "update-99-new"
   title = "Post-Update Features"
   module = 99
   requires = ["modal-00-baseline"]
   concepts = ["update"]
   prose = "Demonstrating external lesson loading after application update."
   ```
5. Executed `zedkata-tui status`.

#### Observations
- Conductor started cleanly, discovering existing lessons plus the new `update-99-new` lesson dynamically.
- `session.json` history was preserved 100% intact:
  ```text
  History Count:    1 attempt(s)
    [1] modal-00-baseline/insert-visibility - PASSED (score: 100, edits: 1)
  Tracked Weaknesses: 2
    • Esc              0.00 (0%)
    • i                0.00 (0%)
  ```
- **Verdict:** **PASS**. Binary update in-place preserves user progress; curriculum changes require zero recompile.

---

### 3.2 Scenario 2: Incompatible Schema Version Recovery

#### Procedure
1. Simulated a downgrade scenario or future schema version by writing `version: 99` into `$StateDir\session.json`:
   ```json
   {
     "version": 99,
     "student": { "started_at": "2026-10-16T13:00:00Z" },
     "current": { "lesson_id": "future-lesson", "exercise_index": 0 },
     "history": [
       { "lesson_id": "future-lesson", "exercise_id": "quantum-motion", "passed": true, "score": 100 }
     ],
     "weaknesses": {},
     "completed_walkthroughs": []
   }
   ```
2. Triggered recovery using `zedkata-tui reset`.

#### Observations
- TUI detected the incompatible version (`version 99 > CURRENT_SCHEMA_VERSION 1`).
- Displayed actionable recovery notification:
  ```text
  Warning: Corrupted session state preserved at "...\session.json.corrupt.1792158656731016400.bak". Clean state initialized.
  Reset active attempt for lesson '' (index 0).
  ```
- Verified that timestamped backup file exists and preserves the `version: 99` data with zero data loss:
  ```powershell
  $Backup = Get-Content $BackupPath | ConvertFrom-Json
  $Backup.version # -> 99
  $Backup.history[0].exercise_id # -> quantum-motion
  ```
- Verified that active `session.json` was recovered to standard supported schema (`version: 1`).
- **Verdict:** **PASS**. Incompatible schema is preserved to non-destructive timestamped backup; runtime recovers cleanly.

---

### 3.3 Scenario 3: Application Removal (Progress Retention)

#### Procedure
1. Deleted application installation directory:
   ```powershell
   Remove-Item -Recurse -Force $InstallDir
   ```
2. Verified removal of application directory.
3. Inspected student data directory `$StateDir`.

#### Observations
- Application directory `$InstallDir` completely removed.
- Student data directory `$StateDir` and all progress files remained intact:
  - `session.json` (active session) preserved on disk.
  - `session.json.corrupt.*.bak` (historical backup) preserved on disk.
- **Verdict:** **PASS**. Application removal does not delete or alter user progress.

---

### 3.4 Scenario 4: Intentional Complete User Data Purge

#### Procedure
1. Executed explicit student progress purge:
   ```powershell
   Remove-Item -Recurse -Force $StateDir
   ```
2. Inspected filesystem for residual locks or orphan files.

#### Observations
- Directory `$StateDir` and all child directories (`exercises/`, backups, active sessions) completely removed.
- Zero open process handles or file locks encountered; return code 0.
- **Verdict:** **PASS**. Intentional progress purge executes cleanly.

---

## 4. Requirement Verification Summary

| Requirement | Acceptance Criteria | Result | Notes |
|---|---|---|---|
| **REQ-33** | Document version/schema incompatibility recovery | **PASS** | Detailed in [docs/installation.md](../installation.md) Section 9.3. |
| **REQ-33** | Distinguish application removal from intentional progress removal | **PASS** | Detailed in [docs/installation.md](../installation.md) Section 10.1 vs 10.2. |
| **REQ-33** | Verify in-place update and zero-recompile lesson additions | **PASS** | Validated in Scenario 1. |
| **REQ-33** | Verify schema incompatibility backup and clean recovery | **PASS** | Validated in Scenario 2. |
| **REQ-33** | Verify application uninstall with progress retention | **PASS** | Validated in Scenario 3. |
| **REQ-33** | Verify intentional complete progress purge | **PASS** | Validated in Scenario 4. |
