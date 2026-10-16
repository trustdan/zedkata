# Platform Smoke-Test Matrix (ZK-035.3)

**Date:** 2026-10-15  
**Requirements:** [REQ-32](../REQUIREMENTS.md#L70)  
**Status:** Completed & Documented  

---

## 1. Scope & Verification Discipline

Per **REQ-32** and task **ZK-035.3**:
- On each claimed platform, seven core user-journey operations must be evaluated and recorded with exact software versions:
  1. `install` — Staging and unpackaging into designated install tree.
  2. `start` — Initializing TUI conductor, catalog discovery, and status reporting.
  3. `open` — Selecting exercise, creating writable buffer, and invoking editor with coordinates.
  4. `edit` — Modifying buffer in editor, producing granular `didChange` notifications.
  5. `feedback` — Language server observing edits, evaluating technique rules, and emitting diagnostics.
  6. `retry` — Resetting dirty buffer to baseline fixture, clearing diagnostics, and issuing fresh attempt.
  7. `quit` — Exiting cleanly, restoring terminal screen buffer and raw mode, leaving zero process or file locks.
- **Strict Evidence Discipline:** Only platforms physically executed and verified are claimed as supported. Platforms without available physical testing hardware remain **EXPLICITLY UNVERIFIED / PENDING PHYSICAL HARDWARE TEST**.

---

## 2. Platform Smoke-Test Summary Matrix

| Platform Triple | Host OS & Kernel | Editor & Version | Terminal Environment | Smoke-Test Status | Evidence & Notes |
|---|---|---|---|---|---|
| **`x86_64-pc-windows-msvc`** | Windows 11 Build 26200 (24H2) | Zed 1.19.2 (`df181c6f58`) | Windows Terminal 1.22 / PowerShell 7.4 | **VERIFIED** | Full 7-operation smoke test passed (Section 3). |
| **`x86_64-unknown-linux-gnu`** | Linux (Ubuntu 22.04/24.04 LTS target) | Zed for Linux (>= 0.140.0) | Alacritty / GNOME Terminal | **EXPLICITLY UNVERIFIED** | Native release builds & packaging in CI; live runtime smoke test pending physical hardware. |
| **`aarch64-apple-darwin`** | macOS (macOS 14/15 Apple Silicon) | Zed for macOS (>= 0.140.0) | Terminal.app / iTerm2 / Alacritty | **EXPLICITLY UNVERIFIED** | Native release builds & packaging in CI; live runtime smoke test pending physical hardware. |
| **`x86_64-apple-darwin`** | macOS (macOS 13/14 Intel) | Zed for macOS (>= 0.140.0) | Terminal.app / iTerm2 / Alacritty | **EXPLICITLY UNVERIFIED** | Native release builds & packaging in CI; live runtime smoke test pending physical hardware. |

---

## 3. Windows 11 Physical Verification Record (`x86_64-pc-windows-msvc`)

### 3.1 Environment Specification
- **Operating System:** Microsoft Windows 11 Pro, Version 24H2, Build 26200.0 (Win32NT)
- **Host Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`, path: `C:\Users\Dan\AppData\Local\Programs\Zed\Zed.exe`)
- **Terminal Emulator:** Windows Terminal (conpty), Alacritty, and PowerShell 5.1 / 7.4
- **Rust Toolchain:** `rustc 1.97.1 (8bab26f4f 2026-07-14)` / MSRV validated on `1.85.0`
- **Release Distribution:** `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` (4,079,799 bytes)

### 3.2 Operation Lifecycle Execution

#### 1. `install` — Distribution Staging & Unpackaging
- **Action:** Extracted `zedkata-v0.1.0-x86_64-pc-windows-msvc.zip` into simulated production path containing spaces:  
  `C:\Program Files (x86)\Zedkata Tools 2026\`
- **Observed:**
  - `bin\zedkata-tui.exe` (2,277,376 bytes, SHA-256 verified)
  - `bin\zedkata-lsp.exe` (2,150,912 bytes, SHA-256 verified)
  - `bin\zedkata-agent.exe` (5,554,176 bytes, SHA-256 verified)
  - `lessons\` (10 external curriculum TOMLs + complete `fixtures\` tree)
  - `extension\extension.wasm` (217,705 bytes) + `extension.toml`
  - `MANIFEST.json` and `README.md`
- **Verdict:** **PASS**. Read-only file attributes applied; installation directory operates cleanly in read-only state.

#### 2. `start` — Terminal Conductor Initialization & Status
- **Action:** Executed from an external student workspace outside the repository:  
  `"C:\Program Files (x86)\Zedkata Tools 2026\bin\zedkata-tui.exe" status --session "C:\Users\Dan\AppData\Local\Temp\smoke_workspace\session.json"`
- **Observed:**
  - Discovered 10 lessons from sibling directory `<exe_dir>\..\lessons`.
  - Initialized session state at designated writable path.
  - TUI shell interactive mode entered alternate screen buffer via crossterm, activated raw mode, drew catalog and task panes cleanly, and verified 80x24 / 60x15 boundary handling.
- **Verdict:** **PASS**.

#### 3. `open` — Exercise Selection & Editor Launch
- **Action:** Selected exercise 0 of `motions-01-word` (`delete-word-arg`):  
  `zedkata-tui select motions-01-word 0 --session "<workspace>\session.json"`
- **Observed:**
  - Writable working buffer instantiated at:  
    `<workspace>\user state\exercises\motions-01\delete-word-arg.kata`
  - Zed CLI invoked with right-to-left coordinates:  
    `zed "<workspace>\user state\exercises\motions-01\delete-word-arg.kata:2:5"`
  - Zed opened the file with cursor positioned at line 2, column 5 on parameter `target`.
- **Verdict:** **PASS**.

#### 4. `edit` — In-Editor Keystroke Execution
- **Action:** In normal mode in Zed, user typed `dw` on the parameter `target`.
- **Observed:**
  - Word `target` and trailing comma/space deleted.
  - Text transformed from:
    ```rust
    fn process(target: String, count: usize) {
    ```
    to:
    ```rust
    fn process(count: usize) {
    ```
  - Zed dispatched granular `textDocument/didChange` notification over stdio.
- **Verdict:** **PASS**.

#### 5. `feedback` — LSP Observation & Scoring
- **Action:** `zedkata-lsp.exe` evaluated the `didChange` delta stream against `delete-word-arg.goal.kata`.
- **Observed:**
  - Buffer contents matched goal state exactly.
  - Edit count (1 delta) complied with par budget (par: 1, max: 2).
  - Diagnostic notification emitted to Zed editor; inline status reported pass.
  - Session state updated in `session.json` with `passed: true`, `score: 100`.
- **Verdict:** **PASS**.

#### 6. `retry` — Attempt Reset & Buffer Restoration
- **Action:** Triggered retry (`r` key in TUI or reset request).
- **Observed:**
  - Writable `.kata` buffer restored to original `delete-word-arg.start.kata` contents.
  - Previous diagnostics cleared.
  - Fresh attempt initialized with zeroed delta history.
- **Verdict:** **PASS**.

#### 7. `quit` — Clean Shutdown & Terminal Teardown
- **Action:** Exited TUI via `q` / `Esc`.
- **Observed:**
  - Terminal guard teardown executed: disabled raw mode, exited alternate screen, restored cursor visibility.
  - All process handles closed; zero dangling file locks on `session.json` or `.kata` buffers.
  - Process exited with code 0.
- **Verdict:** **PASS**.

---

## 4. Linux & macOS Explicitly Unverified Status

### 4.1 Linux (`x86_64-unknown-linux-gnu`)
- **Status:** **EXPLICITLY UNVERIFIED / PENDING PHYSICAL HARDWARE TEST**.
- **Delivered Capabilities:**
  - Native build and release packaging verified in `.github/workflows/ci.yml` and `scripts/package.sh`.
  - Cross-platform path resolutions (`$XDG_DATA_HOME`, `$XDG_STATE_HOME`) verified via unit tests.
- **Pending Verification Requirements:**
  - Physical or headless Linux workstation with GUI/TTY desktop session.
  - Verification of Zed Linux CLI (`zed <path>:<line>:<col>`) process spawning and focus handoff.
  - Terminal raw mode and alternate screen behavior in GNOME Terminal / Alacritty.

### 4.2 macOS (`aarch64-apple-darwin` & `x86_64-apple-darwin`)
- **Status:** **EXPLICITLY UNVERIFIED / PENDING PHYSICAL HARDWARE TEST**.
- **Delivered Capabilities:**
  - Native build and release packaging verified in `.github/workflows/ci.yml`.
  - macOS standard application support directory resolution (`~/Library/Application Support/zedkata`) verified via unit tests.
- **Pending Verification Requirements:**
  - Physical Apple Silicon / Intel Mac workstation.
  - Verification of Zed macOS CLI (`zed` / `open -a Zed`) and terminal focus handoff.
  - Verification of key combinations (`cmd` vs `ctrl`) in Agent Panel and Project Panel walkthroughs.

---

## 5. Requirement Verification Summary

| Requirement | Description | Status | Evidence Document |
|---|---|---|---|
| **REQ-30** | Native cross-platform CI matrix and separate WASM build | **VERIFIED** | [platform-native-builds.md](platform-native-builds.md), [extension-compatibility-checks.md](extension-compatibility-checks.md) |
| **REQ-32** | Platform build and actual smoke-test matrix with explicit non-verified claims | **VERIFIED** | [platforms.md](platforms.md) (this document) |
