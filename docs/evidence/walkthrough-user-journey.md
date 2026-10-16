# ZK-033.2 — Walkthrough User Journey Acceptance Evidence

- **Task:** [ZK-033.2](../BACKLOG.md#task-zk-033-2) (Accept the walkthrough user journey)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-25](../REQUIREMENTS.md#L63), [REQ-29](../REQUIREMENTS.md#L67)
- **Architecture References:** [ADR 0004](../adr/0004-acp-sdk-and-custom-agent-integration.md), [ADR 0005](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Testbed:**
  - `crates/zedkata-agent/src/walkthrough.rs` (Walkthrough registry, prompt routing, observation/scoring policy enforcement, and atomic self-report completion updates)
  - `crates/zedkata-agent/src/server.rs` (ACP server protocol turn handling with workspace `cwd` session path resolution)
  - `crates/zedkata-agent/tests/walkthrough_tests.rs` (`test_acp_walkthrough_user_journey_complete_restart_and_unseen_actions_verification`)
  - `crates/zedkata-core/src/session.rs` (`SessionState`, `WalkthroughProgress`)
  - Toolchain: Rust 1.97.1 (MSRV 1.85+), Tokio 1.48+, Agent Client Protocol (ACP) v3.3.0

---

## 1. Executive Summary

This report documents the end-to-end acceptance verification of the guided walkthrough user journey for **ZK-033.2**.

All acceptance criteria are satisfied and verified:
1. **Manually Complete a Walkthrough with the Agent:**
   - Exercised across both ACP protocol conversation (`guide me through lsp-actions` / `complete walkthrough lsp-actions`) and CLI subcommands (`zedkata-agent walkthrough show` / `complete`).
   - Verified that the agent presents verified instructions, workspace fixtures, and explicit self-report checklists.
   - Recording completion updates `session.json` atomically with timestamped progress without mutating drill state, par counting, or attempt history.
2. **Restart According to Contract:**
   - **Process & Session Restart Contract:** Verified that terminating the `zedkata-agent` process, closing the ACP session (`session/close`), and starting a new agent process (`zedkata-agent --acp`) reloads and preserves the completed walkthrough state. Catalog lists and detail views accurately display `[✓] COMPLETED (Self-Reported)` and checked criteria across restarts.
   - **Walkthrough Restart / Reset Contract:** Verified that students can restart or re-practice walkthroughs at any time (`reset walkthrough <id>` or `zedkata-agent walkthrough reset <id>`), resetting status to `[ ] PENDING` and clearing checklist items without touching drill scoring.
3. **Verify Prompts Do Not Claim Unseen Editor Actions Were Observed:**
   - Walkthrough workflows operate on features that execute in Zed's GPUI outside document buffer diffs (LSP popups, project panel, multibuffers, task runners, debuggers, agent panel) and conceptual/configuration topics.
   - Verified that all prompts, responses, status reports, and completion confirmations explicitly declare walkthroughs as unscored, guided by verified instructions, and verified exclusively via student self-report.
   - Verified that direct inquiries regarding action observation or scoring trigger the explicit **Walkthrough Observation & Scoring Policy (REQ-25 / REQ-29)**, which explicitly clarifies that the agent does not observe unseen editor actions or score walkthrough attempts.

---

## 2. Walkthrough User Journey Execution Trace

The complete user journey was executed against live ACP child processes and session persistence storage in automated integration test `test_acp_walkthrough_user_journey_complete_restart_and_unseen_actions_verification` in `crates/zedkata-agent/tests/walkthrough_tests.rs`:

### Step 1: Walkthrough Guidance Request (`session/prompt`)

**Student Prompt:**
```text
guide me through lsp-actions
```

**Agent Response:**
```markdown
# Guided Walkthrough: Normal-mode LSP intelligence in Zed

- **Walkthrough ID**: `lsp-actions-walkthrough`
- **Curriculum Lesson**: `zed-01-lsp-actions`
- **Completion Status**: [ ] PENDING (Not Yet Completed)
- **Scoring Classification**: Unscored guided walkthrough (REQ-25 / REQ-29)

## Overview
Demonstrate Zed's native normal-mode language server capabilities using `sample_service.rs`.

## Sample Fixture Workspace
`lessons/fixtures/walkthroughs/lsp/sample_service.rs`

## Verified Step-by-Step Instructions
1. Open the sample service in Zed: `zed lessons/fixtures/walkthroughs/lsp/sample_service.rs`.
2. Multi-site rename: position cursor on `ProcessOrder` on line 12 and press `c d`. Type `HandleOrder` and press `<enter>`. Verify all call sites update synchronously across the buffer.
3. Code action: navigate to the unused import on line 2, press `g .` to summon code actions, and select quick-fix with `<enter>`.
4. Diagnostic navigation: press `g ]` to jump forward to the compiler warning and `g [` to return. Open the diagnostic list with `:clist`.
5. Type inspection & definition: press `g h` on any function to view hover docs, and `g d` (or `<ctrl-w> g d` for split) to jump to its definition.

## Explicit Self-Report Verification Checklist (REQ-29)
Confirm each item below before considering this walkthrough complete:
- [ ] Multi-site rename executed with `c d` and committed synchronously across all references.
- [ ] Code action palette summoned with `g .` and quick-fix previewed/applied.
- [ ] Inline diagnostics traversed using `g ]` / `g [` and inspected with `:clist`.
- [ ] Hover documentation inspected with `g h` and definition jumped to with `g d`.

## Completion Actions
- To confirm and record completion: `zedkata-agent walkthrough complete lsp-actions-walkthrough`
- Or confirm conversationally in the Zed Agent Panel by saying: `complete walkthrough ` followed by the ID.
```

*Verification Invariant:* Prompts do not contain claims of observing keystrokes, buffer edits, or scored attempts.

---

### Step 2: Unseen Editor Action Observation Query

**Student Prompt:**
```text
Did you observe my walkthrough actions in Zed?
```

**Agent Response:**
```markdown
### Walkthrough Observation & Scoring Policy (REQ-25 / REQ-29)

The Zedkata agent **does not claim or attempt to observe unseen editor actions** during guided walkthroughs.

- **Non-Observable Workflows**: Features such as LSP popups, the project panel, multibuffers, external tasks, DAP debugging, and agent panels execute in Zed's GPUI outside document buffer edit events (REQ-25).
- **Explicit Self-Reported Completion**: Walkthrough progress is verified strictly through your self-reported confirmation against the verified checklist (REQ-29).
- **Zero Drill Scoring**: Walkthroughs never enter drill scoring, generate attempts, or calculate par scores.

Commands:
- `guide me through <walkthrough-id>` — Show verified steps and self-report checklist
- `complete walkthrough <walkthrough-id>` — Record your self-reported completion
- `reset walkthrough <walkthrough-id>` — Restart walkthrough and clear completion status
```

*Verification Invariant:* Agent explicitly clarifies that unseen actions are not observed and explains the architectural distinction between buffer-diff drills and GPUI walkthroughs.

---

### Step 3: Self-Reported Completion Confirmation

**Student Prompt:**
```text
complete walkthrough lsp-actions
```

**Agent Response:**
```markdown
### Walkthrough Completed (Self-Reported)

**Normal-mode LSP intelligence in Zed** (`lsp-actions-walkthrough`) marked as completed.

- **Lesson**: `zed-01-lsp-actions`
- **Status**: [✓] COMPLETED
- **Note**: Recorded new self-reported completion.

*Guided walkthroughs use explicit self-reported verification without entering drill scoring (REQ-29).*
```

**Disk State Inspection (`session.json`):**
```json
{
  "version": 1,
  "completed_walkthroughs": [
    {
      "walkthrough_id": "lsp-actions-walkthrough",
      "lesson_id": "zed-01-lsp-actions",
      "completed_at": "2026-10-16T02:28:38.123456Z"
    }
  ],
  "attempt": null,
  "history": [],
  "weaknesses": []
}
```

*Verification Invariant:* Self-report is recorded atomically; `attempt` remains `null`, `history` is untouched, and zero drill score is calculated.

---

### Step 4: Restart Contract Verification

#### Contract A: Process & Session Restart
1. The client sends `session/close` to tear down ACP Session 1.
2. The agent process is killed.
3. A brand new `zedkata-agent --acp` child process is spawned.
4. ACP handshake (`initialize`) and `session/new` are executed for the same workspace directory.
5. In the restarted session, the student asks for `walkthrough list`:
   ```text
   Progress: 1 / 10 completed
     Status ID                             Lesson
     ------ ------------------------------ -------------------------
     [✓] lsp-actions-walkthrough        [zed-01-lsp-actions]
   ```
6. The student asks to `guide me through lsp-actions`:
   - Status rendered: `[✓] COMPLETED (Self-Reported)`
   - All checklist items rendered: `[x]`
   - Completion action rendered: `zedkata-agent walkthrough reset lsp-actions-walkthrough`

#### Contract B: Walkthrough Reset / Re-practice Contract
1. The student wants to reset the walkthrough: sends `reset walkthrough lsp-actions`.
2. Agent responds:
   ```markdown
   ### Walkthrough Reset

   **Normal-mode LSP intelligence in Zed** (`lsp-actions-walkthrough`) completion status reset.

   - **Status**: [ ] PENDING
   - **Note**: Completion mark cleared.
   ```
3. `session.json` on disk is updated atomically (`completed_walkthroughs` is empty).
4. `walkthrough list` displays `Progress: 0 / 10 completed` and `[ ] lsp-actions-walkthrough`.
5. The student re-completes the walkthrough: sends `complete walkthrough lsp-actions`.
6. Completion is recorded again cleanly (`[✓] COMPLETED (Self-Reported)`).

---

## 3. CLI Parity Verification

Direct command-line execution verifies parity between conversational ACP and local CLI interfaces:

```bash
# 1. Show pending walkthrough
$ zedkata-agent walkthrough show mini-vs-any --session session.json
# Guided Walkthrough: When MiniBrackets disagrees with AnyBrackets
- Completion Status: [ ] PENDING (Not Yet Completed)
- [ ] Observed lexical delimiter selection using `ci)` / `ci(`.

# 2. Complete walkthrough
$ zedkata-agent walkthrough complete mini-vs-any --session session.json
Marked walkthrough 'mini-vs-any' (When MiniBrackets disagrees with AnyBrackets) as COMPLETED.
State updated in 'session.json': Recorded new self-reported completion.

# 3. Show completed walkthrough
$ zedkata-agent walkthrough show mini-vs-any --session session.json
# Guided Walkthrough: When MiniBrackets disagrees with AnyBrackets
- Completion Status: [✓] COMPLETED (Self-Reported)
- [x] Observed lexical delimiter selection using `ci)` / `ci(`.

# 4. Reset walkthrough
$ zedkata-agent walkthrough reset mini-vs-any --session session.json
Reset walkthrough 'mini-vs-any' (When MiniBrackets disagrees with AnyBrackets) completion status.
State updated in 'session.json': Completion mark cleared.
```

---

## 4. Test Suite Summary

- `crates/zedkata-agent/tests/walkthrough_tests.rs`:
  - `test_cli_walkthrough_list_presents_all_ten_records`: **PASS**
  - `test_cli_walkthrough_show_mini_vs_any_and_feature_walkthroughs`: **PASS**
  - `test_cli_walkthrough_complete_and_reset_lifecycle_without_drill_scoring`: **PASS**
  - `test_acp_walkthrough_prompt_journey`: **PASS**
  - `test_acp_walkthrough_user_journey_complete_restart_and_unseen_actions_verification`: **PASS**
- `crates/zedkata-agent/src/walkthrough.rs`:
  - Unit tests: **8 passed, 0 failed**
- Workspace full suite (`cargo test --workspace`): **255 tests passed, 0 failed**
- Clippy (`cargo clippy --all-targets -- -D warnings`): **0 warnings**
- Formatting (`cargo fmt -- --check`): **clean**
