# ZK-031.2 — Configured Provider Attempt Explanation Evidence

- **Task:** [ZK-031.2](../BACKLOG.md#task-zk-031-2) (Explain an attempt using the configured provider)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-27](../REQUIREMENTS.md#L65)
- **Architecture Reference:** [ADR 0005: Model Providers and Credentials Architecture](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Components:**
  - `crates/zedkata-agent/prompts/explanation_system.md` (System prompt with grounding & output constraints)
  - `crates/zedkata-agent/src/prompt.rs` (User prompt generator & grounded explanation synthesis)
  - `crates/zedkata-agent/src/config.rs` (3-layer provider and credential resolution hierarchy)
  - `crates/zedkata-agent/src/provider.rs` (`ConfiguredProvider` implementation & markdown formatter)
  - `crates/zedkata-agent/src/server.rs` (ACP `session/prompt` turn handler integration)
  - `crates/zedkata-agent/src/main.rs` (CLI `explain` subcommand)
  - `crates/zedkata-agent/tests/provider_tests.rs` (8 automated integration tests)

---

## 1. Executive Summary

Task **ZK-031.2** delivers end-to-end pedagogical attempt explanation using the configured model provider under [REQ-27](../REQUIREMENTS.md#L65) and [ADR 0005](../adr/0005-model-providers-and-credentials.md).

All acceptance criteria are satisfied:
1. **Grounded Prompt & Explanation Fidelity:**
   - Prompts embed bounded state from `session.json` (lesson ID, title, exercise index, task prompt, par budget, active forbid rules, verdict, and chronological bounded edits).
   - Explanations strictly separate **objective observations** (edit counts, character/line coordinates, deletions, insertions, diff bytes, verdicts) from **technique inferences** (identifying character-by-character backspacing, excessive cursor nudging via `h`/`j`/`k`/`l`, or mode confusion).
   - Generates actionable, idiomatic Vim recommendations (e.g. `dw`, `ci"`, `r<char>`, `f<char>`) to reach par.
2. **Offline & Network Isolation Guarantee:**
   - `zedkata-core`, `zedkata-lsp`, and `zedkata-tui` remain strictly offline with zero HTTP or network dependencies.
   - `zedkata-agent` can execute completely offline using `ProviderKind::Fake`, `ProviderKind::Degraded`, or local synthesizers without external network sockets.
3. **Provider Configuration Precedence ([ADR 0005](../adr/0005-model-providers-and-credentials.md)):**
   - Layer 1: Process environment variables (`ZEDKATA_PROVIDER`, `ZEDKATA_MODEL`, `LMSTUDIO_BASE_URL`, `ANTHROPIC_API_KEY`, etc.).
   - Layer 2: User configuration file (`config.toml` at `%APPDATA%\zedkata\config.toml` or `$XDG_CONFIG_HOME/zedkata/config.toml`).
   - Layer 3: Automatic fallback (single API key detection, local LM Studio default, or degraded mode).
4. **Dual Interface Verification:**
   - Available directly in Zed's native Agent Panel via ACP standard I/O streams (`SessionUpdate::AgentMessageChunk`).
   - Available via CLI subcommand (`zedkata-agent explain [--session <path>] [--provider <name>]`) for headless testing and validation.

---

## 2. Concrete Manual Scenario Verification

The following concrete manual scenarios were executed using the `zedkata-agent` binary and verified against the acceptance criteria:

### Scenario 1: Forbid Rule Failure (`backspace_run`)
**Session Context:** Student attempted exercise in `operators-01-edit` and made 4 consecutive backspaces deleting individual characters at line 0, cols 3–6.

**Execution Command:**
```bash
cargo run -p zedkata-agent -- explain --session target/scratch/manual_session.json --provider fake
```

**Output:**
```markdown
### Observations
- Attempt executed 4 edits in total (4 retained in bounded context, 0 bytes diff).
- Verdict: Failed with reason 'Violated forbid rule: backspace_run (consecutive backspaces > 3)'.
- Edit 1: deleted 1 chars, inserted "" at line 0, col 6.
- Edit 2: deleted 1 chars, inserted "" at line 0, col 5.
- Edit 3: deleted 1 chars, inserted "" at line 0, col 4.
- Edit 4: deleted 1 chars, inserted "" at line 0, col 3.

### Technique Inference
- Inferred technique: Excessive backspace usage detected. Student likely relied on character-by-character backspacing instead of word/motion operators (e.g. 'dw' or 'd$').

### Recommendation
Review target motion in lesson prose and minimize keystrokes to reach Par 1.
```
*Verdict: Strict separation maintained. Observations report exact coordinates and deletion counts; Inferences deduce student's backspacing habit versus motion operators.*

---

### Scenario 2: Over Par Edit Budget Failure
**Session Context:** Student executed 5 individual character edits and insertions to replace text instead of using a single `cw` or replacement operator against Par 1.

**Execution Command:**
```bash
cargo run -p zedkata-agent -- explain --session target/scratch/over_par_session.json
```

**Output:**
```markdown
### Observations
- Attempt executed 5 edits in total (5 retained in bounded context, 8 bytes diff).
- Verdict: Failed with reason 'Over par: 5 edits (par 1)'.
- Edit 1: deleted 1 chars, inserted "b" at line 0, col 6.
- Edit 2: deleted 1 chars, inserted "a" at line 0, col 7.
- Edit 3: deleted 1 chars, inserted "d" at line 0, col 8.
- Edit 4: deleted 3 chars, inserted "good" at line 0, col 6.
- Edit 5: deleted 0 chars, inserted ";" at line 0, col 10.

### Technique Inference
- Inferred technique: Edit budget exceeded. Student navigated or edited using individual character motions (h/j/k/l or Delete) rather than semantic word motions (w/e/b or dw).
- Grounded explanation synthesized for provider 'lmstudio' (model: 'local-model').

### Recommendation
Review the target motion in the lesson instructions and minimize keystrokes to reach Par.
```
*Verdict: Accurately identifies character-by-character navigation/editing as root cause of budget overrun.*

---

### Scenario 3: Missing Session & Inactive Attempt Handling
**Execution Command:**
```bash
cargo run -p zedkata-agent -- explain --session non_existent_session.json
```
**Output:**
```text
Error explaining attempt: No session found at "non_existent_session.json". Start an exercise in Zed first.
```

**Execution Command (Empty Attempt in Session):**
```bash
cargo run -p zedkata-agent -- explain --session target/scratch/no_attempt_session.json
```
**Output:**
```text
Error explaining attempt: No active session: No active drill attempt found in session state.
```
*Verdict: Missing and inactive states are handled cleanly with actionable user guidance and exit code 1 without panics.*

---

## 3. Automated Test Suite Matrix (`crates/zedkata-agent/tests/provider_tests.rs`)

| Test Function | Verification Target | Result |
|:---|:---|:---|
| `test_system_prompt_integrity` | System prompt embeds grounding rules and 3-section output requirements | **PASS** |
| `test_build_attempt_user_prompt_formatting` | User prompt contains metadata, par, forbid rules, and bounded edit delta table | **PASS** |
| `test_build_attempt_user_prompt_with_truncation` | Bounded user prompt correctly displays truncation notice for head-truncated edits | **PASS** |
| `test_grounded_explanation_separates_observations_from_inferences` | Verifies separation of observations, technique inferences, and recommendations | **PASS** |
| `test_configured_provider_offline_modes_work_without_network` | Verifies Fake, Degraded, LM Studio, Ollama, Anthropic, OpenAI, Gemini work offline | **PASS** |
| `test_configured_provider_explains_real_session_file` | Reads and explains real session file on disk using `ConfiguredProvider` | **PASS** |
| `test_configured_provider_missing_session_file_explicit_error` | Missing session file returns explicit `ProviderError::MissingSession` | **PASS** |
| `test_cli_explain_command_with_session_file` | Subprocess executes `zedkata-agent explain --session <path> --provider fake` | **PASS** |

---

## 4. Full Workspace Verification Summary

All automated checks across the workspace pass cleanly:
```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```
- **Total Tests Passing:** 216 tests (Core: 104, LSP: 33, TUI: 47, Agent: 32).
- **Clippy Warnings:** 0.
- **Formatting Issues:** 0.

Task **ZK-031.2** is verified and ready to be marked complete.
