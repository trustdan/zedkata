# Zedkata v0.1.0 Release Notes

**Release Candidate:** v0.1.0  
**Date:** 2026-10-16  
**License:** Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)  
**Supported Host Editor:** Zed >= 0.140.0 (Empirically verified on Zed 1.19.2)

---

## 1. Overview

Zedkata is an interactive, drill-based training system for Zed's Vim mode and editor features.
Unlike standalone tutors or web apps, Zedkata runs drills **in real Zed buffers** so students build genuine muscle memory in their actual editing environment.

Zedkata consists of four coordinated components communicating through shared state and standard protocols:
1. **`zedkata-tui`**: Terminal-based curriculum conductor, progress tracker, and drill launcher.
2. **`zedkata-lsp`**: Language server observer evaluating incremental edits and publishing inline diagnostics.
3. **`zedkata-agent`**: Conversational ACP (Agent Client Protocol) agent for Zed's Agent Panel.
4. **`zedkata-extension`**: WebAssembly extension registering `.kata` files and connecting the observer.

---

## 2. Reconciled Artifact Versions & Manifests

All workspace packages, native binaries, and extension manifests are synchronized to version `0.1.0`:

| Artifact | Type | Manifest Location | Version | License | Target Toolchain |
|---|---|---|---|---|---|
| **`zedkata` (root workspace)** | Cargo Workspace | `Cargo.toml` | `0.1.0` | MIT OR Apache-2.0 | Rust 1.85+ |
| **`zedkata-core`** | Rust Library | `crates/zedkata-core/Cargo.toml` | `0.1.0` | MIT OR Apache-2.0 | Rust 1.85+ |
| **`zedkata-lsp`** | Native Binary | `crates/zedkata-lsp/Cargo.toml` | `0.1.0` | MIT OR Apache-2.0 | Rust 1.85+ |
| **`zedkata-tui`** | Native Binary | `crates/zedkata-tui/Cargo.toml` | `0.1.0` | MIT OR Apache-2.0 | Rust 1.85+ |
| **`zedkata-agent`** | Native Binary | `crates/zedkata-agent/Cargo.toml` | `0.1.0` | MIT OR Apache-2.0 | Rust 1.85+ |
| **`zedkata-extension`** | WASM Extension | `extension/Cargo.toml` | `0.1.0` | MIT OR Apache-2.0 | `wasm32-wasip2` |
| **Zed Extension Metadata** | Extension Manifest | `extension/extension.toml` | `0.1.0` | — (`schema_version = 1`) | `zed_extension_api = "=0.7.0"` |

---

## 3. Features & Delivered Milestones

### Interactive TUI Conductor (`zedkata-tui`)
- **Terminal Shell**: Built with `ratatui` and `crossterm`. Features dual-pane navigation (catalog, task, hint, and weakness tracking).
- **Vim Navigation**: Full normal-mode keybindings (`j`/`k`, `enter` to launch, `r` to retry, `n` to advance, `h` to toggle hints, `q` to quit).
- **Responsive Layout**: Dynamic resizing with boundary enforcement (warnings when viewport is smaller than 60x15).
- **Safe Terminal Teardown**: Guarantees restoration of cooked mode, alternate screen buffer exit, and cursor visibility on exit or panic.
- **Headless CLI Commands**: Supports headless workflow automation (`status`, `select`, `next`, `reset`, `launch`).

### Precision LSP Drill Observer (`zedkata-lsp`)
- **Delta Stream Replay**: Reconstructs student buffers from incremental `textDocument/didChange` UTF-16 deltas over stdio JSON-RPC.
- **Goal Verification**: Normalized buffer matching handling CRLF/LF line endings and intra-line whitespace.
- **Technique Scoring**: Evaluates Par edit budgets and forbid rules (e.g. `backspace_run`) in sub-millisecond dispatch time (<1 ms latency).
- **Inline Diagnostics**: Emits actionable pass/fail feedback directly into Zed's editor gutter and hover popovers.

### Conversational ACP Agent (`zedkata-agent`)
- **Zed Agent Panel Integration**: Implements Agent Client Protocol 3.3 over stdio child process pipes.
- **Grounded Attempt Explanations**: Extracts bounded context (max 20 edits, 2 KB text diffs, 24h window) and strictly separates objective buffer observations from inferred Vim technique.
- **Targeted Practice Generation**: Generates custom modal exercises targeting recorded student weaknesses and persists them cleanly to `lessons/generated/`.
- **Offline Autonomy**: Operates 100% offline with built-in heuristics, or connects to local models (LM Studio, Ollama) and cloud providers.

### External Curriculum Catalog (`lessons/`)
- **10 Curriculum Modules**: Includes 15 calibrated Par 1 drills and 10 guided feature walkthroughs:
  - `modal-00-baseline`: Insert/normal mode transitions, line appends, and word replacements.
  - `motions-01-word`: Word motions (`w`, `e`, `b`, `ge`, uppercase variants).
  - `motions-02-find`: In-line find motions (`f`, `t`, `F`, `T`, `;`, `,`).
  - `motions-03-textobjects`: Delimiter objects (`i"`, `a"`, `i(`, `a(`, `i{`, `a{`).
  - `operators-01-edit`: Native operators (`r`, `d<motion>`, counted deletes, indent shifts).
  - Feature walkthroughs: LSP actions, project panel netrw, multibuffers, workspace tasks & DAP debugger, and Agent Panel workflows.
- **Zero-Recompile Architecture**: Lessons are parsed dynamically from external TOML files and fixtures. Changes require no code recompilation.
- **Catalog Validation**: Prerequisite DAG acyclicity and fixture completeness verified in automated CI tests.

### Atomic Session Persistence
- **Single-Writer Safety**: All progress, attempt history, and weakness tallies persist to `session.json` (`CURRENT_SCHEMA_VERSION = 1`).
- **Atomic File Swapping**: Written via atomic temporary-file replacement to prevent partial writes.
- **Crash & Corruption Recovery**: Incompatible future versions or malformed files trigger automatic non-destructive backup creation (`session.json.corrupt.<nanos>.bak`).

---

## 4. Platform Support & Empirical Evidence Discipline

In strict accordance with **REQ-04** and **REQ-32**, Zedkata distinguishes between verified test evidence and declared targets:

| Target Platform Triple | Operating System | Host Editor | Support Status | Verification Basis |
|---|---|---|---|---|
| **`x86_64-pc-windows-msvc`** | Windows 11 Build 26200 (24H2) | Zed 1.19.2 (`df181c6f58`) | **VERIFIED** | Physical 7-operation smoke test passed (`install`, `start`, `open`, `edit`, `feedback`, `retry`, `quit`). See [docs/evidence/platforms.md](docs/evidence/platforms.md). |
| **`x86_64-unknown-linux-gnu`** | Linux (glibc >= 2.31) | Zed for Linux (>= 0.140.0) | **DECLARED / UNVERIFIED** | CI matrix build, packaging, and XDG path tests pass; physical desktop smoke test pending. |
| **`aarch64-apple-darwin`** | macOS 14+ (Apple Silicon) | Zed for macOS (>= 0.140.0) | **DECLARED / UNVERIFIED** | CI matrix build, packaging, and Library path tests pass; physical Apple Silicon smoke test pending. |
| **`x86_64-apple-darwin`** | macOS 13+ (Intel) | Zed for macOS (>= 0.140.0) | **DECLARED / UNVERIFIED** | CI matrix build, packaging, and Library path tests pass; physical Intel Mac smoke test pending. |

*Untested platforms are not advertised as supported until physical smoke-test evidence is recorded.*

---

## 5. Observability Boundaries & Known Limitations

1. **Keystroke & Cursor Inspection**:
   - Zed's extension API does not expose raw keystrokes, active caret positions, or visual selections to extension code.
   - Technique verification relies strictly on granular `textDocument/didChange` deltas emitted over stdio.
   - Actions producing identical buffer states without distinct deltas are governed by Par edit budgets rather than keystroke logging.
2. **Editor Feature Walkthroughs**:
   - Workflows involving project panels, multibuffer search, debugger panes, and task execution operate entirely in Zed's GPUI windowing subsystem.
   - These modules are designed as **guided walkthroughs** with verified step-by-step instructions where completion is self-reported, not automatically scored.
3. **Offline Autonomy**:
   - The TUI conductor and LSP drill observer run 100% locally and offline without external network access.
   - The conversational ACP agent is strictly optional; absence of AI credentials or local LLMs does not block curriculum drills.
4. **Data Isolation**:
   - Zedkata operates cleanly from read-only installation trees (e.g. `C:\Program Files (x86)\...`).
   - Session states and generated drills are written exclusively to standard user directories (`%LOCALAPPDATA%\zedkata` on Windows, `$XDG_DATA_HOME/zedkata` on Linux, `~/Library/Application Support/zedkata` on macOS).

---

## 6. Packaging & Distribution Artifacts

Staged release archives are packaged via `scripts/package.ps1` and `scripts/package.sh`:

- **Windows (`x86_64-pc-windows-msvc`)**: `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip`
- **Linux (`x86_64-unknown-linux-gnu`)**: `dist/zedkata-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- **Distribution Directory Contents**:
  - `bin/`: `zedkata-tui`, `zedkata-lsp`, `zedkata-agent`
  - `lessons/`: 10 TOML lesson files and complete `fixtures/` directory
  - `extension/`: `extension.toml`, `README.md`, `languages/`, and compiled `extension.wasm`
  - Root documents: `README.md`, `RELEASE_NOTES.md`, `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`, `MANIFEST.json`
