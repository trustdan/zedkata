# ZK-033.1 — Walkthrough Selection and Progress Evidence

- **Task:** [ZK-033.1](../BACKLOG.md#task-zk-033-1) (Implement walkthrough selection and progress)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-23](../REQUIREMENTS.md#L61), [REQ-25](../REQUIREMENTS.md#L63), [REQ-29](../REQUIREMENTS.md#L67)
- **Architecture References:** [ADR 0004](../adr/0004-acp-sdk-and-custom-agent-integration.md), [ADR 0005](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Components:**
  - `crates/zedkata-core/src/session.rs` (`WalkthroughProgress`, `completed_walkthroughs`, `mark_walkthrough_completed`, `is_walkthrough_completed`, `reset_walkthrough`)
  - `crates/zedkata-agent/src/walkthrough.rs` (`WalkthroughRecord`, registry of all 10 walkthroughs, step presentation, self-report checklist, atomic session updates, ACP conversational handling)
  - `crates/zedkata-agent/src/main.rs` (`walkthrough` CLI subcommand and flags: `list`, `show <id>`, `complete <id>`, `reset <id>`)
  - `crates/zedkata-agent/src/server.rs` (ACP server prompt integration routing walkthrough inquiries and self-reported completions)
  - `crates/zedkata-core/tests/walkthrough_progress_tests.rs` (Session state backward compatibility and walkthrough lifecycle unit tests)
  - `crates/zedkata-agent/tests/walkthrough_tests.rs` (Automated integration tests for CLI list/show/complete/reset and ACP prompt turns)

---

## 1. Executive Summary

Task **ZK-033.1** implements walkthrough selection, step presentation, and explicit self-reported progress tracking in `zedkata-agent` and `zedkata-core`.

All acceptance criteria are satisfied:
1. **Load Walkthrough Records:**
   - 10 curated walkthrough records covering all curriculum non-observable editor features (REQ-25) and conceptual text-object comparisons (REQ-23):
     - **Feature Walkthroughs:** `lsp-actions-walkthrough` (Module 6), `project-panel-walkthrough` (Module 7), `multibuffer-walkthrough` (Module 8), `tasks-and-debugger-walkthrough` (Module 9), and `agent-panel-walkthrough` (Module 10).
     - **Conceptual & Config Walkthroughs:** `mini-vs-any` (Module 3 text objects: MiniBrackets vs AnyBrackets), `keymap-contexts` (Module 4), `exchange-and-surround` (Module 4), `single-line-find-scope` (Module 2), and `diagnostic-navigation` (Module 0).
2. **Present Verified Steps & Self-Report Verification Criteria:**
   - Each walkthrough presents concrete, verified step-by-step instructions, associated sample workspace/fixture paths, and an explicit self-report checklist adhering strictly to REQ-29.
   - Surfaces verified steps via both CLI (`zedkata-agent walkthrough show <id>`) and conversational ACP protocol (`guide me through <id>` in the Zed Agent Panel).
   - Functions fully in offline/degraded mode without requiring external LLM provider calls (ADR 0005).
3. **Preserve Explicit Self-Reported Completion Semantics Without Drill Scoring:**
   - Walkthrough progress is stored in `session.json` under `completed_walkthroughs: Vec<WalkthroughProgress>`.
   - Never touches or generates an `Attempt`, never counts edits, and never assigns par or score.
   - Explicit confirmation commands (`zedkata-agent walkthrough complete <id>` or conversational "complete walkthrough <id>") record the timestamped self-report and update `session.json` atomically.
   - Progress is displayed in catalog listings (`zedkata-agent walkthrough list` showing `Progress: X / 10 completed` with `[✓]` badges).

---

## 2. Walkthrough Catalog

| Walkthrough ID | Module | Title | Fixture Path | Verification Mode |
|---|---|---|---|---|
| `lsp-actions-walkthrough` | `zed-01-lsp-actions` | Normal-mode LSP intelligence in Zed | `lessons/fixtures/walkthroughs/lsp/sample_service.rs` | Self-report checklist |
| `project-panel-walkthrough` | `zed-02-project-panel` | Netrw-style project panel navigation and file management | `lessons/fixtures/walkthroughs/project_panel/` | Self-report checklist |
| `multibuffer-walkthrough` | `zed-03-multibuffer` | Multibuffer project search and cross-file editing | `lessons/fixtures/walkthroughs/multibuffer/` | Self-report checklist |
| `tasks-and-debugger-walkthrough` | `zed-04-tasks-and-debugger` | Workspace tasks and DAP debugging in Zed | `lessons/fixtures/walkthroughs/tasks_and_debugger/` | Self-report checklist |
| `agent-panel-walkthrough` | `zed-05-agent-panel` | Zed Agent Panel and Agent Client Protocol (ACP) | `lessons/fixtures/walkthroughs/agent_panel/` | Self-report checklist |
| `mini-vs-any` | `motions-03-textobjects` | When MiniBrackets disagrees with AnyBrackets | `lessons/fixtures/motions-03/nested.start.kata` | Self-report checklist |
| `keymap-contexts` | `operators-01-edit` | Keymap contexts and modal binding customization | (User keymap configuration) | Self-report checklist |
| `exchange-and-surround` | `operators-01-edit` | Text exchange vs surround operators | `lessons/fixtures/operators-01/indent-line-block.start.kata` | Self-report checklist |
| `single-line-find-scope` | `motions-02-find` | Single-line find scope and repeat motions in Zed | `lessons/fixtures/motions-02/change-till-char.start.kata` | Self-report checklist |
| `diagnostic-navigation` | `modal-00-baseline` | Inline diagnostics and error jumping in Zed | `lessons/fixtures/modal-00/swap-words.start.kata` | Self-report checklist |

---

## 3. Automated Verification

- `crates/zedkata-core/tests/walkthrough_progress_tests.rs`:
  - `test_session_state_legacy_json_backward_compatibility`: PASS
  - `test_session_state_walkthrough_progress_lifecycle`: PASS
- `crates/zedkata-agent/src/walkthrough.rs`:
  - `test_walkthrough_registry_contains_all_curriculum_walkthroughs`: PASS
  - `test_walkthrough_detail_formatting_and_checklist`: PASS
  - `test_mark_and_reset_in_session_persists_without_drill_scoring`: PASS
  - `test_handle_walkthrough_prompt_routing`: PASS
- `crates/zedkata-agent/tests/walkthrough_tests.rs`:
  - `test_cli_walkthrough_list_presents_all_ten_records`: PASS
  - `test_cli_walkthrough_show_mini_vs_any_and_feature_walkthroughs`: PASS
  - `test_cli_walkthrough_complete_and_reset_lifecycle_without_drill_scoring`: PASS
  - `test_acp_walkthrough_prompt_journey`: PASS

Full workspace test suite: **254 passed, 0 failed**. Clippy clean with 0 warnings. Rustfmt clean.
