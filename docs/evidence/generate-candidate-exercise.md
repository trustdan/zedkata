# ZK-032.1 — Generate Complete Candidate Exercise Evidence

- **Task:** [ZK-032.1](../BACKLOG.md#task-zk-032-1) (Generate a complete candidate exercise)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-28](../REQUIREMENTS.md#L66)
- **Architecture Reference:** [ADR 0005: Model Providers and Credentials Architecture](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Components:**
  - `crates/zedkata-agent/prompts/generator_system.md` (Pedagogical system prompt enforcing modal micro-drills and JSON output schema)
  - `crates/zedkata-agent/src/generator.rs` (`WeaknessContext`, `CandidateExercise`, `CandidateStatus`, `build_candidate_user_prompt`, `parse_candidate_response`, `synthesize_default_candidate`)
  - `crates/zedkata-agent/src/context.rs` (`extract_weakness_context` helper)
  - `crates/zedkata-agent/src/provider.rs` (`ExerciseGenerator` trait, `FakeProvider` and `ConfiguredProvider` integration with async timeout and cancellation)
  - `crates/zedkata-agent/src/server.rs` (ACP server prompt dispatch detecting practice generation requests)
  - `crates/zedkata-agent/src/main.rs` (CLI `--generate` flag and `generate` subcommand)
  - `crates/zedkata-agent/tests/candidate_generator_tests.rs` (10 automated integration tests)

---

## 1. Executive Summary

Task **ZK-032.1** implements targeted modal practice generation targeting identified weaknesses under [REQ-28](../REQUIREMENTS.md#L66) and [ADR 0005](../adr/0005-model-providers-and-credentials.md).

All acceptance criteria are satisfied:
1. **Agreed Weakness Context Extraction:**
   - Extracts concept weakness metrics (`HashMap<String, f64>`) and failure history from `session.json`.
   - Highest-severity weakness is automatically selected as `target_concept` (e.g. `dw` with severity 0.85).
   - Captures all active weaknesses sorted descending by severity and correlates recent failed attempts.
2. **Missing / Unscored Weakness Data Contract:**
   - If `session.json` is missing, corrupt, has empty weaknesses, or consists solely of unscored attempts, it follows the contract gracefully without failing.
   - Falls back to `DEFAULT_FALLBACK_CONCEPT` (`"word-motions"`), sets severity to `None`, and marks `is_unscored_or_clean = true`.
   - Generates foundational review practice rather than erroring out.
3. **Complete Lesson Metadata and Both Fixtures:**
   - Generates complete candidate metadata: timestamped lesson ID (`gen-<concept>-<timestamp>`), title, concepts, prose guidance, exercise ID, task goal, par budget (Par 1), max edits (2), 1-based cursor coordinates, hint, and forbid rules (`backspace_run = 3`).
   - Generates relative fixture paths ending with `.kata` under the lesson subfolder (`fixtures/gen-<concept>-<timestamp>/<exercise_id>.start.kata` and `.goal.kata`).
   - Generates both fixture buffer contents (`start_fixture_content` and `goal_fixture_content`), guaranteeing that the buffers differ by the target motion.
   - Generates complete, valid lesson TOML matching the `zedkata-core::Lesson` schema.
4. **Candidate-Only Output Preservation:**
   - The generated output is held strictly as a `CandidateExercise` with `status: CandidateStatus::Candidate`.
   - No files are written to disk and no catalog mutations occur during ZK-032.1.
   - Asset validation, disk writes to writable user data paths, and catalog merging are safely decoupled for [ZK-032.2](../BACKLOG.md#task-zk-032-2).
5. **Provider Integration & Robustness:**
   - Supports local models (LM Studio / Ollama), cloud APIs, fake test doubles, and offline degraded mode.
   - Respects async request timeouts and client cancellation (`tokio::sync::watch`), leaving session state intact on cancellation or network error.

---

## 2. Weakness Context Contract

```rust
pub struct WeaknessContext {
    pub target_concept: String,
    pub severity: Option<f64>,
    pub all_weaknesses: Vec<(String, f64)>,
    pub is_unscored_or_clean: bool,
    pub recent_failures: Vec<String>,
    pub session_path: Option<PathBuf>,
}
```

### Fallback Contract Matrix

| Session State | Target Concept | Severity | `is_unscored_or_clean` | Behavior |
|---|---|---|---|---|
| Active weaknesses (e.g. `dw: 0.85, ci: 0.35`) | Top weakness (`"dw"`) | `Some(0.85)` | `false` | Targets specific mistake pattern with failure context |
| Clean session (empty weaknesses map) | `"word-motions"` | `None` | `true` | Generates foundational word motion practice |
| Unscored attempts only | `"word-motions"` | `None` | `true` | Generates foundational review practice |
| Missing or unreadable session file | `"word-motions"` | `None` | `true` | Fallback without error |
| Custom concept requested via CLI (`--concept ci`) | `"ci"` | `Some(...)` or `None` | Per session | Generates targeted drill for requested concept |

---

## 3. Candidate Exercise Data Model

```rust
pub struct CandidateExercise {
    pub lesson_id: String,
    pub title: String,
    pub module: Option<u32>,
    pub requires: Vec<String>,
    pub concepts: Vec<String>,
    pub draft: bool,
    pub prose: String,
    pub exercise_id: String,
    pub task: String,
    pub start_fixture_relative_path: String,
    pub start_fixture_content: String,
    pub goal_fixture_relative_path: String,
    pub goal_fixture_content: String,
    pub par_edits: Option<u32>,
    pub max_edits: Option<u32>,
    pub cursor: Option<[u32; 2]>,
    pub hint: Option<String>,
    pub forbid_backspace_run: Option<u32>,
    pub raw_toml: String,
    pub status: CandidateStatus,
}
```

All generated candidates validate against `zedkata_core::lesson::Lesson::from_toml`, ensuring 100% compatibility with curriculum loaders.

---

## 4. Verification Evidence

### Automated Integration Tests (`candidate_generator_tests.rs`)

```text
running 10 tests
test test_fake_provider_with_canned_candidate ... ok
test test_parse_candidate_response_malformed_returns_error ... ok
test test_weakness_context_extraction_with_scored_weaknesses ... ok
test test_weakness_context_missing_or_clean_follows_contract ... ok
test test_candidate_exercise_generation_structure_and_fixtures ... ok
test test_parse_candidate_response_markdown_fences_and_slugification ... ok
test test_candidate_generation_with_missing_unscored_weakness ... ok
test test_candidate_exercise_raw_toml_validity_with_core_lesson_parser ... ok
test test_configured_provider_async_generation_cancellation ... ok
test test_configured_provider_async_generation_and_timeout ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
```

### CLI Verification (`zedkata-agent generate --concept dw`)

```text
### Candidate Practice Drill: Generated Practice: Delete word (dw) (`gen-dw-1792110428`)

**Status**: Candidate
**Concepts**: dw
**Task**: Delete the unused parameter name 'temp' on line 2.
**Par Budget**: 1 edit(s)
**Starting Cursor**: Line 1, Column 14
**Hint**: Position cursor on 'temp' and press 'dw'.

#### Pedagogical Guidance
The 'dw' operator deletes from the cursor through the next word boundary in Normal mode.
Practice cleanly eliminating redundant identifiers without backspacing.

#### Buffer Comparison
```start
fn calculate(temp: i32, value: i32) -> i32 {
    value * 2
}
```

```goal
fn calculate(value: i32) -> i32 {
    value * 2
}
```
```

All 233 workspace tests pass cleanly. `cargo clippy --all-targets -- -D warnings` and `cargo fmt -- --check` pass with zero warnings.

Task **ZK-032.1** is complete and verified. Unblocks **ZK-032.2** (Validate and store generated assets).
