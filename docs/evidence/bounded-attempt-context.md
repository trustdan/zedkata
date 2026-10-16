# ZK-031.1 — Bounded Last-Attempt Context Verification Evidence

- **Task:** [ZK-031.1](../BACKLOG.md#task-zk-031-1) (Build bounded last-attempt context from `session.json`)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-27](../REQUIREMENTS.md#L65)
- **Primary Sources & Specifications:**
  - [ADR 0005: Model Providers and Credentials Architecture](../adr/0005-model-providers-and-credentials.md) (Section 3: Bounded Context Contract)
  - `crates/zedkata-agent/src/context.rs` (Bounded context extractor)
  - `crates/zedkata-agent/src/provider.rs` (`AttemptExplainer` trait and `FakeProvider` implementation)
  - `crates/zedkata-agent/tests/context_tests.rs` (12 automated integration tests)

---

## 1. Executive Summary

This report documents the verification of extracting bounded student drill attempt context from `session.json` in `zedkata-agent` and supplying it to an attempt explanation provider.

All acceptance criteria are satisfied:
1. **Bounded Extraction:** Read `session.json` and active attempt edit history adhering strictly to the contract defined in [REQ-27](../REQUIREMENTS.md#L65) and [ADR 0005](../adr/0005-model-providers-and-credentials.md):
   - Active lesson ID, title, concepts, exercise ID, index, and task goal prompt.
   - Par edit budget and forbid rules (e.g. `backspace_run`).
   - Edit history clamped to at most the **last 20 operations** or **2 KB total text diff payload**.
   - Chronological order preserved for the retained tail window; older head edits truncated with explicit counts and diff byte metrics.
2. **Explicit Edge-Case Handling:**
   - **Missing session file:** Returns `ContextError::MissingSessionFile` and `ProviderError::MissingSession` without crashing.
   - **Missing active attempt:** Returns `ContextError::NoActiveAttempt` and `ProviderError::MissingSession`.
   - **Corrupt session file:** Preserves the corrupt file on disk without modification or deletion, returning actionable `ContextError::CorruptSessionFile` and `ProviderError::CorruptSession`.
   - **Incompatible schema version:** Rejects `version > CURRENT_SCHEMA_VERSION` (version 1) explicitly as `ContextError::CorruptSessionFile`.
   - **Stale attempt:** Detects attempts whose last edit/activity exceeds the staleness threshold (configurable, default 24h) and returns `ContextError::StaleAttempt`.
   - **Oversized state:** Clamps attempts with >20 edits or >2 KB diff gracefully, populating `is_truncated = true`, `truncated_edits_count`, and truncation notices.
3. **Fake Provider Verification:**
   - `FakeProvider` receives exact `AttemptContext` payloads without network overhead.
   - Generates grounded explanations that strictly separate **objective observations** (edits, counts, diff coordinates, verdicts) from **technique inferences** (e.g., character-by-character backspacing vs. word operators).
   - Generates actionable pedagogical recommendations (e.g., `dw`, `cw`, `f<char>`).
   - Supports canned responses, injected errors, and inspectable history.
4. **Automated Verification:** 12 integration tests in `crates/zedkata-agent/tests/context_tests.rs` pass cleanly; formatting and clippy checks pass across all 208 workspace tests.

---

## 2. Bounded Context Specification

| Parameter | Limit | Enforcement Mechanism | Failure/Truncation Behavior |
|:---|:---|:---|:---|
| **Max Edit Operations** | 20 edits | Head truncation (`count_clamped`) | Retains newest 20 edits; records `truncated_edits_count` |
| **Max Diff Bytes** | 2048 bytes (2 KB) | Reverse byte accumulation | Drops older edits in window or clamps individual oversized edits; sets `is_truncated = true` |
| **Default Max Age** | 86,400s (24 hours) | Timestamp delta against `Utc::now()` | Emits explicit `ContextError::StaleAttempt` |
| **Catalog Enrichment** | Optional | Cross-references `session.current` with `Catalog` | Gracefully falls back to session-only state if catalog is absent |
| **Corrupt State** | Zero overwrite | Read-only parse verification | Leaves damaged file intact for student recovery; reports explicit error |

---

## 3. Test Suite Matrix (`crates/zedkata-agent/tests/context_tests.rs`)

| Test Function | Target Scenario | Result |
|:---|:---|:---|
| `test_extract_bounded_context_golden_normal` | Normal attempt with 2 edits, catalog enrichment, failed verdict | **PASS** |
| `test_extract_oversized_edits_capped_to_20` | Attempt with 45 edits; verifies 20 retained, 25 truncated, `is_truncated = true` | **PASS** |
| `test_extract_oversized_diff_capped_to_2kb` | Attempt with 5 x 600-byte edits (3000 bytes); verifies byte clamp <= 2048 | **PASS** |
| `test_single_giant_edit_capped_to_2kb` | Single 5000-byte edit; verifies clamped to 2048 bytes | **PASS** |
| `test_missing_session_file` | Non-existent path returns `MissingSessionFile` / `MissingSession` | **PASS** |
| `test_no_active_attempt_in_session` | Session with `attempt = None` returns `NoActiveAttempt` / `MissingSession` | **PASS** |
| `test_empty_lesson_id_treated_as_no_active_attempt` | Session with empty `current.lesson_id` returns `NoActiveAttempt` | **PASS** |
| `test_corrupt_session_file_explicit_error` | Malformed JSON returns `CorruptSessionFile`; original file preserved | **PASS** |
| `test_incompatible_schema_version_explicit_error` | `version = 999` returns `CorruptSessionFile` / `CorruptSession` | **PASS** |
| `test_stale_old_attempt_detection` | Attempt 48 hours old against 1-hour max staleness returns `StaleAttempt` | **PASS** |
| `test_fake_provider_receives_context_and_explains_grounded` | E2E fake provider receives context, separates observations from inferences | **PASS** |
| `test_fake_provider_error_injection` | Simulated upstream rate limit / provider rejection | **PASS** |

---

## 4. Grounded Explanation Example

When an attempt failing `forbid_rule: backspace_run` is passed to `FakeProvider::explain_session`, the generated `Explanation` strictly separates observation from inference:

```json
{
  "observations": [
    "Attempt executed 2 edits in total (2 retained in bounded context, 0 bytes diff).",
    "Verdict: Failed with reason 'Violated forbid rule: backspace_run'.",
    "Edit 1: deleted 0 chars, inserted \"dw\" at line 0, col 0.",
    "Edit 2: deleted 4 chars, inserted \"\" at line 0, col 2."
  ],
  "inferences": [
    "Inferred technique: Excessive backspace usage detected. Student likely relied on character-by-character backspacing instead of word/motion operators (e.g. 'dw' or 'd$')."
  ],
  "recommendation": "To accomplish 'Delete the word under cursor', consider using motion operators like 'dw', 'cw', or 'f<char>' to reach Par 1.",
  "is_bounded": true,
  "truncation_note": null
}
```

---

## 5. Verification Commands

```bash
# 1. Run context and provider test suite
cargo test -p zedkata-agent --test context_tests -- --nocapture

# 2. Run all workspace tests (208 tests)
cargo test

# 3. Format and clippy validation
cargo fmt --check
cargo clippy --all-targets
```
All commands execute cleanly with exit code 0.
