# Release Preparation: License, Versions, and Release Notes (ZK-037.1)

**Date:** 2026-10-16  
**Requirements:** [REQ-04](../REQUIREMENTS.md#L42), [REQ-34](../REQUIREMENTS.md#L72)  
**Status:** Completed & Verified  

---

## 1. Scope & Acceptance Criteria

Per **REQ-04**, **REQ-34**, and task **ZK-037.1**:
1. **Explicit License Choice**: Choose license explicitly; provide standard full-text license files (`LICENSE-MIT`, `LICENSE-APACHE`, and root `LICENSE` dual-license statement), declare them in all Cargo and extension manifests, and state them in `README.md`.
2. **Reconciled Artifact Versions**: Reconcile version declarations across workspace crates, extension manifests, packaging scripts, and binary CLI versions, ensuring zero drift from `0.1.0`.
3. **Document Known Limits & Support Evidence**: Document empirical platform support status (Windows 11 VERIFIED; Linux and macOS EXPLICITLY UNVERIFIED / PENDING PHYSICAL HARDWARE TEST per REQ-32), technique inference limits via LSP `didChange`, and self-reported feature walkthroughs.
4. **Author Release Notes**: Produce a complete, reviewable `RELEASE_NOTES.md` capturing delivered capabilities, architecture, manifests, checksums, and limitations without advertising untested behavior as complete.

---

## 2. License Selection & Rationale

Zedkata adopts the standard dual-licensing model prevalent throughout the Rust open-source ecosystem:
- **MIT License** ([LICENSE-MIT](../../LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](../../LICENSE-APACHE))

### Rationale
- **Ecosystem Compatibility**: Matches Rust standard library and major ecosystem dependencies (`tokio`, `serde`, `clap`, `ratatui`), providing maximum flexibility for downstream developers and contributors.
- **Explicit Patent & Trademark Protection**: Apache 2.0 provides explicit patent grant and termination provisions along with trademark protections, while MIT offers permissive simplicity.
- **Manifest Alignment**: Root `Cargo.toml` and `extension/Cargo.toml` declare `license = "MIT OR Apache-2.0"`. `README.md` and `RELEASE_NOTES.md` explicitly link both full-text licenses.

---

## 3. Reconciled Artifact Versions Matrix

All repository manifests, target definitions, and executable versions have been inspected and reconciled:

| Component / File | Field / Property | Reconciled Value | Status |
|---|---|---|---|
| `Cargo.toml` | `[workspace.package].version` | `"0.1.0"` | Matches |
| `Cargo.toml` | `[workspace.package].license` | `"MIT OR Apache-2.0"` | Matches |
| `Cargo.toml` | `[workspace.package].rust-version` | `"1.85"` | Matches |
| `crates/zedkata-core/Cargo.toml` | `version.workspace` | `true` (`0.1.0`) | Matches |
| `crates/zedkata-lsp/Cargo.toml` | `version.workspace` | `true` (`0.1.0`) | Matches |
| `crates/zedkata-tui/Cargo.toml` | `version.workspace` | `true` (`0.1.0`) | Matches |
| `crates/zedkata-agent/Cargo.toml` | `version.workspace` | `true` (`0.1.0`) | Matches |
| `extension/Cargo.toml` | `[package].version` | `"0.1.0"` | Matches |
| `extension/Cargo.toml` | `[package].license` | `"MIT OR Apache-2.0"` | Matches |
| `extension/extension.toml` | `version` | `"0.1.0"` | Matches |
| `extension/extension.toml` | `schema_version` | `1` | Matches |
| `scripts/package.ps1` | `$Version` default | `"0.1.0"` | Matches |
| `scripts/package.sh` | `VERSION` default | `"0.1.0"` | Matches |
| `zedkata-tui.exe --version` | CLI output | `zedkata-tui 0.1.0` | Matches |
| `zedkata-lsp.exe --version` | CLI output | `zedkata-lsp 0.1.0` | Matches |
| `zedkata-agent.exe --version` | CLI output | `zedkata-agent 0.1.0` | Matches |

Automated reconciliation assertions are permanently enforced in `crates/zedkata-core/tests/release_metadata_tests.rs`.

---

## 4. Documented Limits & Evidence Disciplines

In strict compliance with **REQ-04** ("distinguish assumptions, record ADRs") and **REQ-32** ("keep failed/untested targets out of verified support claims"):

### 4.1 Platform Support Claims
- **Windows 11 (`x86_64-pc-windows-msvc`)**: **VERIFIED**. Tested on Windows 11 Pro 24H2 with Zed 1.19.2 (`df181c6f58`). Full 7-operation smoke test passed (`install`, `start`, `open`, `edit`, `feedback`, `retry`, `quit`). Recorded in [docs/evidence/platforms.md](platforms.md).
- **Linux (`x86_64-unknown-linux-gnu`)**: **DECLARED / UNVERIFIED**. Build and packaging automated in CI, standard XDG path resolution tested, but live runtime smoke test pending physical hardware.
- **macOS (`aarch64-apple-darwin` & `x86_64-apple-darwin`)**: **DECLARED / UNVERIFIED**. Build and packaging automated in CI, macOS Library path resolution tested, but live runtime smoke test pending physical hardware.

### 4.2 Observability Boundaries
- **Keystroke & Cursor Telemetry**: Zed's extension API cannot observe keystrokes or cursor coordinates directly (OQ-04). Technique is inferred from `didChange` text deltas (OQ-01). Unobservable motions (e.g. visual mode) are constrained via Par edit budgets rather than keystroke logging (REQ-10).
- **Walkthrough Self-Reporting**: Non-buffer editor features (LSP actions, project panel netrw, multibuffer search, tasks/debuggers) are guided walkthroughs where student self-reports completion; they are not scored drills (REQ-25, REQ-29).
- **Offline Autonomy**: The core drills, TUI, and LSP observer require no internet, no API keys, and no AI agent processes. The ACP agent is strictly optional.
- **Installation Tree Immutability**: All runtime state, exercise buffers, and generated drills are written to user-specific writable data directories (`%LOCALAPPDATA%\zedkata`), allowing the installation directory to remain read-only (REQ-31).

---

## 5. Automated Verification Results

### 5.1 Release Metadata Integration Tests
Executed `cargo test -p zedkata-core --test release_metadata_tests -- --nocapture`:
```text
running 4 tests
test test_release_notes_present_and_document_known_limits ... ok
test test_packaging_scripts_stage_licenses_and_release_notes ... ok
test test_explicit_license_files_present_and_reconciled ... ok
test test_workspace_and_extension_versions_reconciled ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 5.2 Packaging Script Staging Verification
Executed `scripts/package.ps1 -SkipBuild -Archive`:
```text
[3/5] Assembling staging tree at C:\Users\Dan\zedkata\dist\zedkata-v0.1.0...
  [OK] Extension languages/ definitions packaged
  [OK] Precompiled extension.wasm packaged
  [OK] LICENSE packaged
  [OK] LICENSE-MIT packaged
  [OK] LICENSE-APACHE packaged
  [OK] RELEASE_NOTES.md packaged

[4/5] Generating release manifest...
  [OK] MANIFEST.json written

[5/5] Validating staged artifacts...
  [OK] zedkata-tui.exe --version reports: zedkata-tui 0.1.0
  [OK] zedkata-lsp.exe --version reports: zedkata-lsp 0.1.0
  [OK] zedkata-agent.exe --version reports: zedkata-agent 0.1.0
  [OK] 10 external lesson TOML files verified in staged lessons directory

Creating release archive: zedkata-v0.1.0-x86_64-pc-windows-msvc.zip...
  [OK] Release archive created at: C:\Users\Dan\zedkata\dist\zedkata-v0.1.0-x86_64-pc-windows-msvc.zip
```

### 5.3 Full Workspace Verification
- `cargo test --workspace --locked`: **269 tests passed** (265 prior tests + 4 release metadata tests).
- `cargo clippy --workspace --locked --all-targets -- -D warnings`: **0 warnings**.
- `cargo fmt --all -- --check`: **Clean formatting**.
- `cargo test --manifest-path extension/Cargo.toml`: **2 tests passed**.
