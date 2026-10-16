# ZK-035.3 Run Platform Smoke-Test Matrix Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-32  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-32** and task **ZK-035.3**:
- **Acceptance:** On each claimed platform, install/start/open/edit/feedback/retry/quit checks are recorded with versions; unavailable machines stay explicitly unverified.
- **Seven Core Operations:**
  1. `install`: Distribution extraction into destination install tree (preserving read-only layout and lessons).
  2. `start`: TUI conductor / CLI invocation, terminal guard initialization, catalog loading.
  3. `open`: Exercise selection, writable buffer generation, Zed editor launch with coordinates.
  4. `edit`: Buffer modification, keystroke execution in editor.
  5. `feedback`: LSP observation of `didChange` deltas, verification evaluation, diagnostic emission, and session persistence.
  6. `retry`: Attempt reset, dirty buffer restoration from start fixture, diagnostic clearance.
  7. `quit`: Clean process shutdown, terminal teardown, zero file locks or orphan processes.
- **Evidence Discipline:**
  - Complete, dated evidence recorded for Windows 11 (`x86_64-pc-windows-msvc`).
  - Linux and macOS targets explicitly designated as **UNVERIFIED / PENDING PHYSICAL HARDWARE TESTING**; keeping them strictly out of verified support claims until concrete runtime testing on actual hardware is executed.

---

## 2. Implementation Steps

1. **Plan Formulation:** Author this implementation plan.
2. **Execute Local Smoke-Test Harness:**
   - Execute the 7-step smoke test lifecycle against staged Windows distribution (`dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` and staged tree).
   - Validate each operation: install, start, open, edit, feedback, retry, quit.
   - Capture exact toolchain, OS, terminal, and editor versions.
3. **Author Evidence Document (`docs/evidence/platforms.md`):**
   - Record the full matrix of claimed vs. unverified platforms.
   - Record detailed evidence for the 7 operations on Windows 11.
   - Record explicit pending status for Linux and macOS.
4. **Update Project Documentation:**
   - Update `docs/DEVELOPMENT.md`.
   - Update `docs/BACKLOG.md` marking ZK-035.3 and Epic ZK-035 complete, and advancing to ZK-036.1.
