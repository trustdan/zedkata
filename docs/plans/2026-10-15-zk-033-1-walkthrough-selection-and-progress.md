# ZK-033.1 Walkthrough Selection and Progress Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-23, REQ-25, REQ-29  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-23**, **REQ-25**, and **REQ-29**:
- **Acceptance:** Load walkthrough records, present verified steps, and preserve explicit self-reported completion semantics; include Mini/Any and feature walkthroughs without entering drill scoring.
- Non-observable workflows (LSP actions, project panel netrw emulation, multibuffer editing, tasks & debugger, agent panel & ACP, and conceptual text-object differences like `mini-vs-any`) are structured as guided walkthroughs with concrete reproducible steps and explicit self-reported verification checklists.
- Drill scoring (par edits, actual edit counts, goal buffer comparison) is strictly isolated from walkthrough progression.

---

## 2. Architecture & Design

### 2.1 Walkthrough Records & Registry (`crates/zedkata-core` & `crates/zedkata-agent`)
1. Walkthrough definitions exist in the lesson catalog (`Lesson.walkthroughs`).
2. Provide a structured Walkthrough detail model/registry representing:
   - `id`: Unique walkthrough identifier (e.g., `mini-vs-any`, `lsp-actions-walkthrough`).
   - `lesson_id`: Associated lesson identifier (e.g., `motions-03-textobjects`, `zed-01-lsp-actions`).
   - `title`: Human-readable title.
   - `prompt`: High-level purpose and pedagogical overview.
   - `fixture`: Optional sample workspace path (e.g. `lessons/fixtures/walkthroughs/lsp/sample_service.rs`).
   - `steps`: Ordered list of verified recipes and actions to execute in Zed.
   - `checklist`: Concrete self-report verification items.
3. Support all 10 curriculum walkthroughs:
   - Feature walkthroughs:
     - `lsp-actions-walkthrough` (`zed-01-lsp-actions`)
     - `project-panel-walkthrough` (`zed-02-project-panel`)
     - `multibuffer-walkthrough` (`zed-03-multibuffer`)
     - `tasks-and-debugger-walkthrough` (`zed-04-tasks-and-debugger`)
     - `agent-panel-walkthrough` (`zed-05-agent-panel`)
   - Conceptual & configuration walkthroughs:
     - `mini-vs-any` (`motions-03-textobjects`)
     - `keymap-contexts` (`operators-01-edit`)
     - `exchange-and-surround` (`operators-01-edit`)
     - `single-line-find-scope` (`motions-02-find`)
     - `diagnostic-navigation` (`modal-00-baseline`)

### 2.2 Session State & Explicit Completion Semantics (`crates/zedkata-core/src/session.rs`)
1. Extend `SessionState` with `completed_walkthroughs: Vec<WalkthroughProgress>` (default empty).
2. `WalkthroughProgress` records:
   - `walkthrough_id: String`
   - `lesson_id: String`
   - `completed_at: DateTime<Utc>`
3. Add methods:
   - `mark_walkthrough_completed(&mut self, walkthrough_id: &str, lesson_id: &str) -> bool`
   - `is_walkthrough_completed(&self, walkthrough_id: &str) -> bool`
   - `reset_walkthrough(&mut self, walkthrough_id: &str) -> bool`
4. Guarantee: Walkthrough progress never modifies `attempt`, `history`, or `weaknesses`.

### 2.3 Agent CLI & ACP Protocol (`crates/zedkata-agent`)
1. CLI Subcommands:
   - `zedkata-agent walkthrough list`
   - `zedkata-agent walkthrough show <ID>`
   - `zedkata-agent walkthrough complete <ID>`
   - `zedkata-agent walkthrough reset <ID>`
2. ACP Server:
   - Recognize walkthrough intents in conversational prompts ("walkthrough list", "show walkthrough <id>", "complete walkthrough <id>").
   - Fulfill walkthrough inquiries in degraded/offline mode without requiring external LLM provider calls (ADR 0005).

---

## 3. Implementation Steps

1. **Step 1:** Extend `crates/zedkata-core/src/session.rs` with `WalkthroughProgress` and `completed_walkthroughs` in `SessionState`. Add unit tests in `zedkata-core`.
2. **Step 2:** Implement walkthrough registry and rich walkthrough records in `crates/zedkata-agent/src/walkthrough.rs`, populating verified steps and checklists for all 10 walkthroughs (including Mini/Any and the 5 feature walkthroughs).
3. **Step 3:** Implement CLI subcommands and options in `crates/zedkata-agent/src/main.rs`.
4. **Step 4:** Integrate walkthrough handling into ACP conversational server `crates/zedkata-agent/src/server.rs`.
5. **Step 5:** Add integration tests in `crates/zedkata-agent/tests/walkthrough_tests.rs` and `crates/zedkata-core/tests/walkthrough_progress_tests.rs`.
6. **Step 6:** Run formatting, clippy, and full workspace tests.
7. **Step 7:** Update documentation in `docs/BACKLOG.md` and `docs/DEVELOPMENT.md`.
