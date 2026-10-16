# ZK-036.3 Check Update and Uninstall Behavior Implementation Plan

**Date:** 2026-10-16  
**Requirements:** REQ-33  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-33** and task **ZK-036.3**:
- **Acceptance:** Document version/schema incompatibility recovery and distinguish application removal from intentional progress removal; manually verify update and uninstall on a supported platform.
- **Components to Update & Validate:**
  1. `docs/installation.md`:
     - Add Section 9: "Updating Zedkata" covering in-place binary extraction over existing install, zero-recompile external lesson addition/updates, and schema version forward/backward compatibility with automatic corruption backup recovery (`session.json.corrupt.<timestamp>.bak`).
     - Add Section 10: "Uninstalling & Data Retention" clearly separating application removal (binaries + extension) from user progress/history (`session.json` + `exercises/`), providing exact commands for both clean application uninstall and full data purge across Windows, Linux, and macOS.
  2. Manual Verification on Windows 11 (`x86_64-pc-windows-msvc`):
     - **Scenario 1: Update in place:**
       - Start with existing installation and active student progress.
       - Simulate an update by updating binaries and staging an updated/new lesson asset without recompiling.
       - Verify student progress and session state are preserved intact across the update.
     - **Scenario 2: Version / Schema Incompatibility Recovery:**
       - Simulate a session file with an unsupported future schema version (`version: 99`).
       - Run `zedkata-tui status` / start conductor; verify automatic creation of timestamped backup (`session.json.corrupt.<nanos>.bak`) and initialization of clean session with zero data destruction.
     - **Scenario 3: Application Removal (Preserving Student Data):**
       - Remove application install directory (`bin/`, `lessons/`, `extension/`).
       - Verify user state (`session.json` and `exercises/`) remains completely untouched in user directories.
     - **Scenario 4: Intentional Complete Data Purge:**
       - Execute explicit progress removal commands.
       - Verify complete cleanup without orphan file locks.
  3. Evidence Document:
     - Record all commands, observations, outputs, and verdicts in `docs/evidence/update-and-uninstall.md`.
  4. Backlog Tracking:
     - Update `docs/BACKLOG.md` and `docs/DEVELOPMENT.md` completing ZK-036.3 and Epic ZK-036, and advancing to ZK-037.1 (Prepare license, versions and release notes).

---

## 2. Execution Steps

1. **Formulate Plan:** Author this document under `docs/plans/2026-10-16-zk-036-3-check-update-and-uninstall-behavior.md`.
2. **Update `docs/installation.md`:** Add Section 9 (Updating Zedkata) and Section 10 (Uninstalling & Data Retention).
3. **Execute Verification Scenarios:**
   - Execute the 4 concrete scenarios using isolated paths on Windows 11.
   - Verify zero regression on existing automated tests.
4. **Author Evidence Document:** Record findings in `docs/evidence/update-and-uninstall.md`.
5. **Update Backlog:** Update `docs/BACKLOG.md` marking ZK-036.3 and Epic ZK-036 complete, and advancing to ZK-037.1.
