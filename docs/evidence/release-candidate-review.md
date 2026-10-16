# Zedkata v0.1.0 Release Candidate Review (ZK-037.3)

**Date:** 2026-10-16  
**Target Version:** `v0.1.0` (Release Candidate)  
**Requirements:** [REQ-04](../REQUIREMENTS.md#L42), [REQ-34](../REQUIREMENTS.md#L72)  
**Status:** Reviewed, Validated, and Accepted  

---

## 1. Executive Summary

This document records the formal release candidate review for **Zedkata v0.1.0**.
All required tasks across Phases 0 through 5 have been completed, verified with automated test suites, and evidenced through dated physical and synthetic observation records.

All release artifacts, platform claims, extension submission drafts, and documentation agree with repository state and architectural contracts. In accordance with **REQ-34**, **external publication remains an independent, user-authorized action**.

---

## 2. Milestone & Task Audit

Every milestone defined in [docs/BACKLOG.md](../BACKLOG.md) has been audited:

| Phase / Epic | Scope | Tasks | Audit Outcome |
|---|---|---|---|
| **Phase 0** | Feasibility Spike (Gate) | ZK-001 – ZK-005 | **100% Complete**. Granular `didChange` notifications confirmed in Zed 1.19.2; sub-millisecond dispatch measured; Finding 1 accepted in [ADR 0003](../adr/0003-phase-0-feasibility-gate.md). |
| **Phase 1** | Scaffold & Core Loop | ZK-006 – ZK-016 | **100% Complete**. Scaffolding, Windows CI, TOML lesson loading, verification, evidence-based edit counting, atomic session persistence, production LSP observer, inline diagnostics, and one-exercise demonstration in Zed. |
| **Phase 2** | Usable Local MVP | ZK-017 – ZK-023 | **100% Complete**. Ratatui TUI shell, 1-based CLI exercise buffer launch, live polling/health, retry & navigation, restart recovery, starter course, and local MVP acceptance across 3 terminal routes. |
| **Phase 3** | Curriculum Expansion | ZK-024 – ZK-028 | **100% Complete**. Text objects audit, baseline/motion lessons, operator drills, GPUI keymap context walkthroughs, editor-feature modules, and full catalog DAG validation in CI. |
| **Phase 4** | Conversational Tutor | ZK-029 – ZK-033 | **100% Complete**. ACP provider choice ([ADR 0004](../adr/0004-acp-sdk-and-custom-agent-integration.md), [ADR 0005](../adr/0005-model-providers-and-credentials.md)), stdio ACP agent scaffold, attempt explanation with strict observation/inference separation, targeted weakness generation, and guided feature walkthroughs. |
| **Phase 5** | Distribution & Packaging | ZK-034 – ZK-037 | **100% Complete**. Staged native/external layout, installed-path write isolation, multi-platform CI & smoke matrix, clean install/update/uninstall, dual licensing, release notes, extension submission artifacts, and release candidate audit. |
| **Optional / Post-MVP** | Inline Task Hints | ZK-038 (REQ-16) | **Deferred Beyond MVP**. Recorded in [ADR 0001](../adr/0001-record-architecture-decisions.md) and backlog as non-blocking. |
| **Deliberately Not Doing** | Scope Exclusions | REQ-35 | **Excluded**. No web UI, no telemetry, no leaderboards, no multiplayer, no teaching Vim from zero. |

---

## 3. Artifact Reconciliation & Integrity Audit

All release artifacts have been assembled, inspected, and validated:

### 3.1 Binaries & WASM Component
- `bin/zedkata-tui.exe` (2,283,520 bytes): Version `0.1.0` verified via `--version`.
- `bin/zedkata-lsp.exe` (2,151,936 bytes): Version `0.1.0` verified via `--version`.
- `bin/zedkata-agent.exe` (5,552,640 bytes): Version `0.1.0` verified via `--version`.
- `extension/extension.wasm` (217,705 bytes): Compiled against `wasm32-wasip2`, pinned to `zed_extension_api = "=0.7.0"`.

### 3.2 External Curriculum Assets
- 10 TOML lessons staged in `lessons/` (`modal-00-baseline.toml`, `motions-01-word.toml`, `motions-02-find.toml`, `motions-03-textobjects.toml`, `operators-01-edit.toml`, `zed-01-lsp-actions.toml`, `zed-02-project-panel.toml`, `zed-03-multibuffer.toml`, `zed-04-tasks-and-debugger.toml`, `zed-05-agent-panel.toml`).
- Complete start/goal fixtures tree in `lessons/fixtures/`.
- Verified zero code recompilation required when editing external curriculum files.

### 3.3 Packaging Manifest & Distribution Archive
- Staged release tree: `dist/zedkata-v0.1.0/`
- Release archive: `dist/zedkata-v0.1.0-x86_64-pc-windows-msvc.zip`
- Manifest: `dist/zedkata-v0.1.0/MANIFEST.json` matches SHA-256 binary checksums, lesson counts, and layout.

### 3.4 Licensing & Release Documentation
- Root and extension directory contain: `LICENSE` (dual notice), `LICENSE-MIT`, and `LICENSE-APACHE`.
- `README.md` and `RELEASE_NOTES.md` declare dual MIT OR Apache-2.0 licensing.
- `RELEASE_NOTES.md` accurately documents delivered features, platform statuses, and limitations.

---

## 4. Platform Results & Evidence Consistency

Strict compliance with **REQ-04** and **REQ-32** has been verified:

1. **Microsoft Windows 11 (`x86_64-pc-windows-msvc`)**:
   - Status: **VERIFIED**.
   - Tested on Windows 11 Pro 24H2 with Zed 1.19.2 (`df181c6f58`).
   - All 7 operations (`install`, `start`, `open`, `edit`, `feedback`, `retry`, `quit`) executed and documented in [docs/evidence/platforms.md](platforms.md) and [docs/evidence/clean-install.md](clean-install.md).
2. **Linux (`x86_64-unknown-linux-gnu`)**:
   - Status: **DECLARED / UNVERIFIED**.
   - Native compilation and release packaging automated in CI matrix and `scripts/package.sh`.
   - Explicitly listed as pending physical hardware testing in public documentation.
3. **macOS Apple Silicon & Intel (`aarch64-apple-darwin`, `x86_64-apple-darwin`)**:
   - Status: **DECLARED / UNVERIFIED**.
   - Native compilation and release packaging automated in CI matrix.
   - Explicitly listed as pending physical hardware testing in public documentation.

---

## 5. Extension Submission Review

The extension submission artifacts prepared in `docs/submission/` are ready for upstream pull request submission:
- [extensions.toml.patch](../submission/extensions.toml.patch): Verified unified diff registering `[zedkata]` in Zed's official registry.
- [PR_DRAFT.md](../submission/PR_DRAFT.md): Complete PR title, description, architecture explanation, and test checklist.
- [SUBMISSION_CHECKLIST.md](../submission/SUBMISSION_CHECKLIST.md): Step-by-step instructions for maintainers.

---

## 6. Verification Test Summary

All automated test suites and validation scripts were executed and passed cleanly:

| Test Suite / Command | Scope | Result |
|---|---|---|
| `cargo test --workspace --locked` | Core, LSP, TUI, Agent unit & integration tests | **270 passed; 0 failed** |
| `cargo test --manifest-path extension/Cargo.toml` | Extension WASM unit tests | **2 passed; 0 failed** |
| `cargo test -p zedkata-core --test catalog_validation` | Curriculum DAG acyclicity & fixtures | **22 passed; 0 failed** |
| `cargo test -p zedkata-core --test release_metadata_tests` | Versions, licenses & limits assertions | **5 passed; 0 failed** |
| `cargo clippy --workspace --locked --all-targets -- -D warnings` | Workspace linting | **0 warnings** |
| `cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings` | Extension WASM linting | **0 warnings** |
| `cargo fmt --all -- --check` | Formatting consistency | **Clean** |
| `scripts/check-installed-paths.ps1` | Read-only install tree & write isolation | **PASSED** |
| `scripts/package.ps1 -Clean -Archive` | Clean release packaging & archive verification | **PASSED** |

---

## 7. Sign-Off & Publication Separation

- **Release Readiness Verdict:** **ACCEPTED FOR v0.1.0 RELEASE**.
- **Publication Gate:**
  In accordance with project guidelines and REQ-34:
  - No external Git tags or GitHub releases have been pushed.
  - No crates have been published to crates.io.
  - No pull request has been submitted to `zed-industries/extensions`.
  - All artifacts are retained locally and in version control, awaiting explicit user authorization for external distribution.
