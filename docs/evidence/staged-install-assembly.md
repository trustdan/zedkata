# Staged Install Assembly and External Lesson Assets (ZK-034.1)

**Date:** 2026-10-15  
**Requirements:** [REQ-08](../REQUIREMENTS.md#L46), [REQ-31](../REQUIREMENTS.md#L69)  
**Status:** Completed & Verified  

---

## 1. Overview & Objectives

In accordance with **REQ-08** and **REQ-31**, `zedkata` must package its native executables alongside external curriculum lesson files, ensuring:
1. **Self-contained distribution:** Versioned native binaries and complete external lesson data are staged outside the development tree in a production directory structure.
2. **External lesson hot-reloading:** Lesson data and fixtures reside as external data files; modifying or adding lessons requires zero binary re-compilation.
3. **Optional agent independence:** The core local drill loop (TUI, Zed launch, LSP observer) functions completely offline and independently without requiring agent configuration, API keys, or active agent processes.

---

## 2. Release Staging Layout (`dist/zedkata-v<version>/`)

The packaging scripts produce the following clean, self-contained distribution layout:

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

### Manifest Metadata (`MANIFEST.json`)
The generated `MANIFEST.json` documents package provenance and integrity:
- `name`: `"zedkata"`
- `version`: `"0.1.0"`
- `packaged_at`: ISO-8601 UTC timestamp
- `target`: compilation target or host architecture
- `binaries`: list of binaries with relative paths, sizes in bytes, and SHA-256 checksums
- `curriculum`: total lesson count (10) and presence of `fixtures/`
- `layout`: directory mapping for `bin`, `lessons`, and `extension`

---

## 3. Unified Installed Lessons Discovery

To allow binaries to discover external lessons without hardcoding source checkout paths, `crates/zedkata-core/src/catalog.rs` provides `default_lessons_dir()` and `resolve_lessons_dir(explicit: Option<&Path>)`:

1. **Explicit CLI override:** `--lessons-dir <PATH>` takes precedence.
2. **Environment variable:** `ZEDKATA_LESSONS_DIR` if set and non-empty.
3. **Executable-relative lookup:** Inspects `std::env::current_exe()`:
   - `<exe_dir>/../lessons` (standard staged distribution layout: `bin/` sibling to `lessons/`)
   - `<exe_dir>/lessons` (flat/portable installation layout)
   - `<exe_dir>/../share/zedkata/lessons` (Unix FHS layout: `/usr/bin` to `/usr/share/zedkata/lessons`)
4. **Current working directory / source checkout:**
   - `./lessons`
   - `../../lessons`
5. **Platform standard data directory:**
   - Windows: `%LOCALAPPDATA%\zedkata\lessons`
   - Linux: `$XDG_DATA_HOME/zedkata/lessons` (`~/.local/share/zedkata/lessons`)
   - macOS: `~/Library/Application Support/zedkata/lessons`

Both `crates/zedkata-tui/src/main.rs` and `crates/zedkata-lsp/src/main.rs` consume `zedkata_core::resolve_lessons_dir` for uniform resolution.

---

## 4. Production Packaging Scripts

Two production packaging scripts are provided in `scripts/`:
- `scripts/package.ps1`: Windows PowerShell script supporting `-OutputDir`, `-Target`, `-SkipBuild`, and `-Clean`.
- `scripts/package.sh`: POSIX bash script supporting `--output-dir`, `--target`, `--skip-build`, and `--clean`.

Both scripts:
1. Extract the release version from root `Cargo.toml`.
2. Build native release binaries (`cargo build --workspace --release`).
3. Assemble the distribution tree into `dist/zedkata-v<version>/`.
4. Compute SHA-256 checksums and write `MANIFEST.json` (UTF-8 without BOM).
5. Validate staged binaries (`--version`) and external lesson completeness.

---

## 5. Verification & Acceptance Evidence

Automated test suite `crates/zedkata-tui/tests/staged_install_tests.rs` validates all acceptance criteria:

| Test Case | Acceptance Criteria Verified | Result |
|---|---|---|
| `test_staged_install_tree_and_manifest_structure` | Layout correctness, binary SHA-256 checksums, `MANIFEST.json` schema, extension metadata | **PASS** |
| `test_staged_native_binaries_version_and_help` | All 3 binaries (`zedkata-tui`, `zedkata-lsp`, `zedkata-agent`) report matching version `0.1.0` and `--help` text | **PASS** |
| `test_staged_external_lessons_catalog_validation` | Staged catalog passes `validate_shipped_catalog()`; 0 drafts in runnable drill lessons; all start and goal fixtures resolve on disk | **PASS** |
| `test_changing_external_lesson_files_needs_no_recompile` | External TOML modification is loaded immediately without binary recompilation | **PASS** |
| `test_optional_agent_does_not_block_local_drills` | Standalone local drill loop (TUI + LSP observer) completes successfully with zero agent configuration or presence | **PASS** |

Execution sample:
```text
running 5 tests
test test_staged_install_tree_and_manifest_structure ... ok
test test_staged_external_lessons_catalog_validation ... ok
test test_optional_agent_does_not_block_local_drills ... ok
test test_staged_native_binaries_version_and_help ... ok
test test_changing_external_lesson_files_needs_no_recompile ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```
