# Post-Audit Remediation Plan (Phase 6 — Epics ZK-039 to ZK-043)

**Date:** 2026-10-31  
**Status:** Completed (All 5 Epics `ZK-039`–`ZK-043` and 10 Execution Cards `ZK-039.1`–`ZK-043.1` implemented and verified).

## Overview

A comprehensive post-v0.1.0 audit of the `zedkata` workspace, curriculum lessons, and documentation identified 10 concrete functional, robustness, convention, and documentation discrepancies across `zedkata-core`, `zedkata-lsp`, `zedkata-tui`, `zedkata-agent`, `lessons/`, and `docs/`.

This plan organizes those findings into **Phase 6 — Post-Audit Hardening & Reconciliation**, consisting of 5 Epics (`ZK-039` through `ZK-043`) and 10 leaf execution cards (`ZK-039.1` through `ZK-043.1`) in `docs/BACKLOG.md`.

---

## Epics & Execution Cards

### Epic ZK-039 — Catalog, Launcher & LSP Discovery Integrity

1. **ZK-039.1 — Fix `Catalog::load_with_generated` default directory resolution**
   - **Touchpoints:** `crates/zedkata-core/src/catalog.rs`, `crates/zedkata-core/tests/`
   - **Problem:** `Catalog::load_with_generated` documents that when `generated_dir` is `None`, it attempts to load from `default_generated_dir()` if it exists, followed by `shipped_dir.join("generated")`. In reality, the `None` branch only checks `shipped_dir.join("generated")`, causing `zedkata-lsp` and `App::load` to miss generated drills in `%LOCALAPPDATA%\zedkata\lessons\generated\` unless `--generated-dir` or `ZEDKATA_GENERATED_DIR` is explicitly passed.
   - **Action:** Check `default_generated_dir()` in the `None` branch of `Catalog::load_with_generated` before checking `shipped_dir.join("generated")` (deduplicating if both resolve to the same canonical path).
   - **Acceptance:** Automated tests verify `Catalog::load_with_generated(&shipped_dir, None)` discovers generated lessons in the platform/env-configured default generated directory.

2. **ZK-039.2 — Unify prerequisite unlock logic in `zedkata-tui::launcher`**
   - **Touchpoints:** `crates/zedkata-tui/src/launcher.rs`, `crates/zedkata-tui/tests/launch_tests.rs`
   - **Problem:** `Catalog::is_lesson_unlocked` requires all exercises in each prerequisite lesson to be passed (`is_lesson_completed`), whereas `launcher::launch_exercise` checks `session.history.iter().any(|h| h.lesson_id == *req && h.passed)`, allowing CLI launches of locked lessons after passing only 1 of 3 exercises in a prerequisite.
   - **Action:** Replace the manual `session.history.iter().any(...)` check in `launch_exercise` with `catalog.is_lesson_unlocked(lesson_id, &session.history)` and report the specific incomplete prerequisite lesson(s) in `LaunchError::PrerequisitesNotMet`.
   - **Acceptance:** Automated test confirms passing 1 of 3 exercises in `motions-01-word` rejects `launch_exercise` on `motions-02-find` with `PrerequisitesNotMet`, and passing all 3 exercises unlocks it.

3. **ZK-039.3 — Harden `FeedbackEngine` catalog reload and filename matching in `zedkata-lsp`**
   - **Touchpoints:** `crates/zedkata-lsp/src/feedback.rs`, `crates/zedkata-lsp/tests/feedback_integration_tests.rs`
   - **Problem:**
     1. `FeedbackEngine::is_active_exercise` fallback rule 3 checks `file_name.starts_with(&exercise.id) || file_name.contains(&exercise.id)` without excluding `.goal.kata` files or verifying the lesson context.
     2. `FeedbackEngine` loads `Catalog` once at startup and never refreshes it when a newly generated `gen-*` drill is activated in `session.json` while `zedkata-lsp` is running.
   - **Action:**
     1. Tighten `is_active_exercise` so `.goal.kata` buffers are never treated as active exercise buffers and filename matching requires exact `<exercise_id>.kata` or `<exercise_id>.start.kata`.
     2. Store the shipped and generated directory roots in `FeedbackEngine` and reload `self.catalog` on `didOpen` / `didChange` if the active `lesson_id` in `session.json` is not present in `self.catalog`.
   - **Acceptance:** Integration tests verify that opening `<exercise_id>.goal.kata` does not reset or evaluate the active attempt, and generating a new `gen-*` drill after `FeedbackEngine` initialization allows immediate live evaluation on `didOpen`/`didChange`.

---

### Epic ZK-040 — Curriculum Walkthrough & Fixture Reconciliation

4. **ZK-040.1 — Reconcile walkthrough IDs between `lessons/*.toml` and `WalkthroughRecord::all()`**
   - **Touchpoints:** `crates/zedkata-agent/src/walkthrough.rs`, `crates/zedkata-agent/tests/walkthrough_tests.rs`
   - **Problem:** Four walkthrough IDs defined in `lessons/*.toml` (`lsp-actions-walkthrough`, `single-line-find-scope`, `exchange-and-surround`, `keymap-contexts`)—and claimed in `ZK-033.1`—are missing from `WalkthroughRecord::all()`, causing `zedkata-agent` to reject them as unknown walkthroughs.
   - **Action:** Add complete `WalkthroughRecord` definitions for `lsp-actions-walkthrough`, `single-line-find-scope`, `exchange-and-surround`, and `keymap-contexts` in `walkthrough.rs` (while preserving `lsp-symbol-rename` and `lsp-code-action` compatibility).
   - **Acceptance:** Cross-crate integration test loads the shipped `Catalog`, iterates through every `[[walkthrough]]` across all 10 curriculum lessons, and asserts `WalkthroughRecord::find(&wt.id)` succeeds with non-empty steps and verification prompts.

5. **ZK-040.2 — Fix stale fixture paths and symbol/line references in `walkthrough.rs`**
   - **Touchpoints:** `crates/zedkata-agent/src/walkthrough.rs`, `crates/zedkata-agent/tests/walkthrough_tests.rs`
   - **Problem:**
     1. `lsp-symbol-rename` and `lsp-code-action` instruct the user to open `lessons/fixtures/walkthroughs/lsp/sample_service.rs` and navigate to `ProcessOrder` on line 12 (and line 8), which does not exist in `sample_service.rs` (which defines `TelemetryClient::record_metric` on line 50).
     2. `diagnostic-navigation` references `lessons/fixtures/modal-00/swap-words.start.kata`, which does not exist on disk (`modal-00-baseline` uses `insert-visibility.start.kata`, `append-semicolon.start.kata`, and `navigate-and-insert.start.kata`).
   - **Action:** Update the walkthrough instructions and `fixture_path` metadata in `walkthrough.rs` to match `sample_service.rs` (`record_metric` on line 50 / `TelemetryClient`) and `lessons/fixtures/modal-00/insert-visibility.start.kata`.
   - **Acceptance:** Automated test verifies that every `fixture_path` in `WalkthroughRecord::all()` exists in the repository `lessons/fixtures/` tree and that referenced symbols exist in the target fixture files.

---

### Epic ZK-041 — ACP Server & Model Provider Fidelity (`zedkata-agent`)

6. **ZK-041.1 — Persist generated drills and pass resolved session/catalog in ACP `AgentServer`**
   - **Touchpoints:** `crates/zedkata-agent/src/server.rs`, `crates/zedkata-agent/tests/`
   - **Problem:**
     1. When handling a drill generation prompt over ACP (`is_generate == true`), `AgentServer::run` calls `generate_candidate_from_session_async` and returns `candidate.format_preview()` without calling `validate_and_store`, so drills generated in Zed's Agent Panel are never saved to disk.
     2. When handling an attempt explanation prompt over ACP (`else` branch), `AgentServer::run` calls `provider.explain_default_session_async(None, ...)` instead of `provider.explain_session_async(&path, catalog.as_ref(), ...)`, ignoring `session_cwd` and omitting `Catalog` metadata (`task_prompt`, `lesson_title`, `concepts`, `par_edits`, `forbid_rules`).
   - **Action:**
     1. Load `Catalog` in `AgentServer` (using `resolve_lessons_dir` / `load_with_generated`).
     2. On ACP generation, validate and persist the candidate via `validate_and_store` into the configured/default generated directory and report the admitted status and file paths in the ACP response.
     3. On ACP explanation, call `provider.explain_session_async(&path, catalog.as_ref(), cancel_flag)` with the resolved `session_path` and loaded `Catalog`.
   - **Acceptance:** Automated ACP integration tests verify that `"generate practice"` persists the `.toml` and `.kata` fixtures to the generated directory with `Status: Admitted`, and `"explain attempt"` uses the session CWD's `session.json` and includes catalog task/par/forbid metadata.

7. **ZK-041.2 — Implement hybrid HTTP transport for `ConfiguredProvider` and scope out `chatgpt-auth`**
   - **Touchpoints:** `crates/zedkata-agent/Cargo.toml`, `crates/zedkata-agent/src/provider.rs`, `crates/zedkata-agent/src/config.rs`, `docs/adr/0005-model-providers-and-credentials.md`, `docs/ARCHITECTURE.md`
   - **Problem:** `ConfiguredProvider` currently ignores `build_attempt_user_prompt`, `build_candidate_user_prompt`, and `parse_candidate_response` in production and performs no network requests for `lm-studio`, `ollama`, `openai`, `anthropic`, or `gemini`. Meanwhile, ADR 0005 mentions Tier 3 `chatgpt-auth`, which is unimplemented in `ProviderKind::from_str`.
   - **Action:**
     1. Wire HTTP request/response execution in `ConfiguredProvider` for local OpenAI-compatible servers (`lm-studio`, `ollama`) and standard API-key providers (`openai`, `anthropic`, `gemini`) using `system_prompt`, `build_attempt_user_prompt`, `build_candidate_user_prompt`, and `parse_candidate_response` (while retaining `Offline` as the default deterministic zero-network provider and supporting mock/loopback HTTP servers in tests).
     2. Explicitly reject `"chatgpt-auth"` in `ProviderKind::from_str` with an actionable error message stating that OAuth subscription auth is out of scope, and update ADR 0005 and `docs/ARCHITECTURE.md` to record that Tier 1 (`lm-studio`, `ollama`) + Tier 2 API keys (`openai`, `anthropic`, `gemini`) + `offline` are supported, while Tier 3 (`chatgpt-auth`) is explicitly scoped out.
   - **Acceptance:** Automated tests against a local loopback HTTP test server verify end-to-end prompt dispatch and response parsing for `explain_attempt` and `generate_candidate` across OpenAI-compatible, Anthropic, and Gemini wire formats, timeout/error handling, and explicit rejection of `chatgpt-auth`.

---

### Epic ZK-042 — UTF-8 Safety & Code Convention Compliance

8. **ZK-042.1 — Fix UTF-8 char-boundary truncation in `context.rs` and percent-decoding in `feedback.rs`**
   - **Touchpoints:** `crates/zedkata-agent/src/context.rs`, `crates/zedkata-lsp/src/feedback.rs`
   - **Problem:**
     1. `bound_edits` in `crates/zedkata-agent/src/context.rs` calls `truncated_text.truncate(max_diff_bytes)` at raw byte index `max_diff_bytes`, which panics if a multi-byte UTF-8 character straddles the boundary.
     2. `urlencoding_decode` in `crates/zedkata-lsp/src/feedback.rs` casts each percent-decoded byte individually via `byte as char` (Latin-1) instead of decoding UTF-8 byte sequences, corrupting non-ASCII file paths.
   - **Action:**
     1. Walk backward from `max_diff_bytes` to the nearest `is_char_boundary` index before truncating `truncated_text` in `bound_edits`.
     2. Collect raw bytes into a `Vec<u8>` in `urlencoding_decode` and decode with `String::from_utf8_lossy`.
   - **Acceptance:** Unit tests verify multi-byte UTF-8 payloads (`🌍`, `é`) straddling `max_diff_bytes` truncate cleanly without panicking, and `uri_to_path` decodes percent-encoded UTF-8 directory/file names accurately.

9. **ZK-042.2 — Eliminate library `unwrap()` calls and remove `tokio` from `zedkata-tui`**
   - **Touchpoints:** `crates/zedkata-core/src/lib.rs`, `crates/zedkata-lsp/src/lib.rs`, `crates/zedkata-lsp/src/feedback.rs`, `crates/zedkata-agent/src/lib.rs`, `crates/zedkata-agent/src/server.rs`, `crates/zedkata-agent/src/provider.rs`, `crates/zedkata-tui/Cargo.toml`, `crates/zedkata-tui/src/lib.rs`, `crates/zedkata-tui/src/main.rs`
   - **Problem:**
     1. `CLAUDE.md` specifies *"No `unwrap()` or `expect()` in library code"*, but `feedback.rs`, `server.rs`, and `provider.rs` use `.unwrap()` on `Option` and `Mutex::lock()`.
     2. `CLAUDE.md` specifies *"Add `tokio` only in the binary crates that need async I/O (likely `zedkata-lsp` and `zedkata-agent`)"*, but `zedkata-tui` depends on `tokio` (`features = ["full"]`) and uses `#[tokio::main]` despite being completely synchronous.
   - **Action:**
     1. Replace `session.attempt.as_mut().unwrap()` / `as_ref().unwrap()` in `feedback.rs` and all `.lock().unwrap()` calls in library code with non-panicking patterns, and enforce `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]` across library crates.
     2. Remove `tokio` from `crates/zedkata-tui/Cargo.toml` and make `main()` in `crates/zedkata-tui/src/main.rs` synchronous.
   - **Acceptance:** `cargo clippy --workspace --locked --all-targets -- -D warnings` and `cargo test --workspace --locked` pass with the lint enforced on all non-test library code and zero `tokio` dependency in `zedkata-tui`.

---

### Epic ZK-043 — Documentation, Backlog & Open-Questions Reconciliation

10. **ZK-043.1 — Reconcile stale execution cards in `docs/BACKLOG.md` and `OQ-11` in `docs/OPEN-QUESTIONS.md`**
    - **Touchpoints:** `docs/BACKLOG.md`, `docs/OPEN-QUESTIONS.md`
    - **Problem:**
      1. `docs/BACKLOG.md` still has a stale `"Next manual action: ZK-004.2..."` banner at line 323, and execution cards `ZK-006.1`–`ZK-006.5` remain marked `in progress` / `blocked` despite `ZK-006` and `docs/ARCHITECTURE.md` being complete.
      2. `OQ-11` in `docs/OPEN-QUESTIONS.md` is still labeled `ASSUMED` ("Not closed by Phase 0") even though `02-dw-repeat.jsonl` captured and verified single-step undo (`u`) inverse edit restoration and `scoring.rs` implements single-step inverse cancellation.
    - **Action:**
      1. Remove the stale `ZK-004.2` banner in `docs/BACKLOG.md` and update `ZK-006.1`–`ZK-006.5` to `Status: done` with completion notes referencing `docs/ARCHITECTURE.md` and `ADR 0004`.
      2. Update `OQ-11` in `docs/OPEN-QUESTIONS.md` to `VERIFIED (single-step inverse restoration) / SCOPED` citing `tests/fixtures/lsp/02-dw-repeat.jsonl` and `crates/zedkata-core/src/scoring.rs`.
    - **Acceptance:** `docs/BACKLOG.md` and `docs/OPEN-QUESTIONS.md` have zero contradictory status banners or unclosed Phase 1 cards.

---

## Validation Gate

Every epic and execution card must pass:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo test --workspace --locked`
- `cargo test -p zedkata-core --test catalog_validation --locked`

