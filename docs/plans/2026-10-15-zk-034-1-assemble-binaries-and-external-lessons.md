# ZK-034.1 Assemble Binaries and External Lesson Assets Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-08, REQ-31  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-08** and **REQ-31**:
- **Acceptance:** Staged install includes versioned native binaries and complete external lesson data; changing lesson files needs no recompile and optional agent setup does not block local drills.
- **Packaging layout:** Native release binaries (`zedkata-tui`, `zedkata-lsp`, `zedkata-agent`) and external lessons with complete fixtures staged in a clean distribution tree outside the source development tree.
- **External lesson contract:** Shipped lessons reside as external TOML and fixture files; editing any lesson or fixture requires zero binary re-compilation.
- **Agent independence:** The core local drill loop (TUI, Zed launch, LSP observer) functions completely offline and independently; absence of agent credentials or offline/unconfigured agent setup never impedes local practice.

---

## 2. Architecture & Design

### 2.1 Release Staging Layout (`dist/zedkata-v<version>/`)

The packaging scripts produce a self-contained release tree:

```text
dist/zedkata-v0.1.0/
├── bin/
│   ├── zedkata-tui.exe (or zedkata-tui)
│   ├── zedkata-lsp.exe (or zedkata-lsp)
│   └── zedkata-agent.exe (or zedkata-agent)
├── lessons/
│   ├── README.md
│   ├── modal-00-baseline.toml
│   ├── motions-01-word.toml
│   ├── motions-02-find.toml
│   ├── motions-03-textobjects.toml
│   ├── operators-01-edit.toml
│   ├── zed-01-lsp-actions.toml
│   ├── zed-02-project-panel.toml
│   ├── zed-03-multibuffer.toml
│   ├── zed-04-tasks-and-debugger.toml
│   ├── zed-05-agent-panel.toml
│   └── fixtures/
│       ├── baseline/
│       ├── motions-01/
│       ├── motions-02/
│       ├── motions-03/
│       └── operators-01/
├── extension/
│   ├── extension.toml
│   └── README.md
├── README.md
└── MANIFEST.json
```

### 2.2 Unified Installed Lessons Discovery (`crates/zedkata-core/src/catalog.rs`)

To ensure binaries discover external lessons seamlessly when executed from any location:
Add `default_lessons_dir()` and `resolve_lessons_dir(cli_override: Option<&Path>) -> Result<PathBuf, CoreError>` in `zedkata_core`:
1. Explicit CLI override (`--lessons-dir`).
2. Environment variable `ZEDKATA_LESSONS_DIR`.
3. Executable-relative lookup (`std::env::current_exe()`):
   - `<exe_dir>/../lessons` (standard staged layout: `bin/` alongside `lessons/`)
   - `<exe_dir>/lessons` (flat/portable layout: executable side-by-side with `lessons/`)
   - `<exe_dir>/../share/zedkata/lessons` (Unix FHS installation: `/usr/bin` to `/usr/share/zedkata/lessons`)
4. Working directory / repository checkout fallback:
   - `./lessons`
   - `../../lessons`
5. Platform standard user/data paths:
   - `%LOCALAPPDATA%\zedkata\lessons` (Windows)
   - `$XDG_DATA_HOME/zedkata/lessons` / `~/.local/share/zedkata/lessons` (Linux)
   - `~/Library/Application Support/zedkata/lessons` (macOS)

### 2.3 Binary Updates

Update `crates/zedkata-tui/src/main.rs` and `crates/zedkata-lsp/src/main.rs` to consume `zedkata_core::resolve_lessons_dir` uniformly.

### 2.4 Packaging Scripts

Provide production packaging scripts:
- `scripts/package.ps1` for Windows PowerShell environments.
- `scripts/package.sh` for POSIX environments.
Scripts support:
- Compiling release binaries via `cargo build --workspace --release`
- Creating the staging layout
- Copying lessons, fixtures, extension metadata, and documentation
- Generating `MANIFEST.json` containing version, target, timestamp, binary SHA-256 hashes, and catalog stats
- Verifying binary versions (`--version`) and catalog validation (`validate_shipped_catalog`)

### 2.5 Automated Verification Suite (`tests/staged_install_tests.rs`)

Write comprehensive integration tests verifying:
1. Staging directory structure and `MANIFEST.json` integrity.
2. Version matching across all three binaries (`zedkata-tui`, `zedkata-lsp`, `zedkata-agent`).
3. Shipped catalog validity: all 10 lessons load, 0 drafts present, prerequisite graph acyclic, all fixtures present.
4. No-recompile hot-reloading: mutating staged lesson content updates runtime catalog without rebuilding code.
5. Optional agent independence: TUI app and LSP feedback operate completely standalone without agent presence or credentials.

---

## 3. Implementation Steps

1. **Implement Lesson Discovery in Core:** Add `default_lessons_dir` and `resolve_lessons_dir` in `crates/zedkata-core/src/catalog.rs` and export in `lib.rs`.
2. **Update TUI and LSP Entrypoints:** Wire `resolve_lessons_dir` in `crates/zedkata-tui/src/main.rs` and `crates/zedkata-lsp/src/main.rs`.
3. **Create Packaging Scripts:** Write `scripts/package.ps1` and `scripts/package.sh`.
4. **Author Integration Tests:** Create `tests/staged_install_tests.rs` covering all acceptance criteria.
5. **Run Packaging & Test:** Assemble the release staging tree in `dist/` and run the full test suite, clippy, and rustfmt.
6. **Documentation & Evidence:** Document results in `docs/evidence/staged-install-assembly.md`, update `docs/DEVELOPMENT.md`, and update `docs/BACKLOG.md`.
