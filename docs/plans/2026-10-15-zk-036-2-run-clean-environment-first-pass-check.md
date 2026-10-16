# ZK-036.2 Run Clean-Environment First-Pass Check Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-33  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-33** and task **ZK-036.2**:
- **Acceptance:** Fresh supported environment reaches first passing exercise without undocumented steps; record mistakes discovered and retest only corrected steps.
- **Files/components:**
  - `docs/evidence/clean-install.md` (new comprehensive evidence record).
  - `scripts/package.ps1` and `scripts/package.sh` (package `extension/languages` so dev extension has filetype association).
  - `crates/zedkata-core/src/session.rs`, `crates/zedkata-lsp/src/main.rs`, and `crates/zedkata-agent/src/config.rs` (support both `ZEDKATA_SESSION_PATH` and `ZEDKATA_SESSION` env vars).
  - `docs/installation.md` (correct PowerShell `where.exe zed`, `modal-00-baseline` exercise buffer path, env var names, and staging details).
  - `docs/BACKLOG.md` and `docs/DEVELOPMENT.md` (task completion tracking).

---

## 2. Planned Execution Phases

### Phase 1: Fix Discrepancies Discovered During Inspection
1. **Extension packaging:** Ensure `extension/languages/` is staged into `dist/zedkata-v<version>/extension/languages/` in both `scripts/package.ps1` and `scripts/package.sh`.
2. **Environment variable tolerance:** Allow `ZEDKATA_SESSION` in addition to `ZEDKATA_SESSION_PATH` across `zedkata-core`, `zedkata-lsp`, and `zedkata-agent`.
3. **Documentation accuracy in `docs/installation.md`:**
   - Update verification command from `where zed` to `where.exe zed` / `Get-Command zed`.
   - Update buffer path in walkthrough from `exercises/modal-00/...` to `exercises/modal-00-baseline/...`.
   - Clarify `ZEDKATA_SESSION_PATH` (with `ZEDKATA_SESSION` alias) and `ZEDKATA_EXERCISES_DIR`.

### Phase 2: Clean Environment Packaging & Staging
1. Re-run `package.ps1 -Clean -Archive` to produce an updated, complete distribution zip archive in `dist/`.
2. Verify staged contents including `extension/languages/zedkata/config.toml`.

### Phase 3: Clean-Environment Walkthrough Execution
1. Create an isolated student workspace in `$env:TEMP/zedkata_clean_env_test` completely isolated from repo sources.
2. Unpack release archive into simulated installation directory `$CleanDir\install`.
3. Set temporary `PATH` to include only clean install binaries and Zed editor.
4. Execute verification commands:
   - `zedkata-tui --version`
   - `zedkata-lsp --version`
   - `zedkata-agent --version`
5. Execute first passing exercise workflow:
   - Launch `modal-00-baseline` exercise 0 (`insert-visibility`).
   - Verify buffer creation at `$StateDir\exercises\modal-00-baseline\insert-visibility.kata`.
   - Verify initial contents and cursor coordinates `[2, 5]`.
   - Execute the target edit: insert `pub ` on line 2 at column 5 (`    pub timeout_ms: u64,`).
   - Run `zedkata-lsp` evaluation to verify clean pass, diagnostics emission, and session record update.
   - Verify `session.json` recorded `passed: true`, `score: 100`, edit count 1, par 1.
   - Advance to next exercise (`append-semicolon`) via `zedkata-tui next`.

### Phase 4: Evidence Recording & Backlog Tracking
1. Author comprehensive evidence report in `docs/evidence/clean-install.md` detailing:
   - Environment specifications (OS, Zed, Terminal, Rust).
   - Mistakes/discrepancies discovered and corrected re-test results.
   - Exact commands, output, and execution logs from clean install to first pass.
2. Run full workspace test suite, clippy, and formatting check.
3. Update `docs/BACKLOG.md` and `docs/DEVELOPMENT.md` advancing to ZK-036.3.
