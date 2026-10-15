# ZK-025.1 — Modal Baseline Lesson Calibration and Authoring Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `modal-00-baseline`  
**Prerequisites:** `[]` (Foundational Module 0)  

---

## 1. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-24):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Targets existing modal-editing users moving to Zed. Focuses on core modal foundation (normal vs insert modes, block cursor character placement), entering insert before (`i`) vs append after (`a`), returning to normal mode (`<Esc>` / `ctrl-[`), and Zed's native inline diagnostic rendering and error navigation (`g ]` / `g [`). | **PASS** |
| **One Concept per Exercise** | Single atomic concept per drill: insert mode entry before cursor (`i`), append mode entry after cursor (`a`), and cursor navigation without mutation (`j`/`k`/`h`/`l`) followed by inline insert. | **PASS** |
| **`par_edits` is the Lesson** | All exercises calibrated to `par_edits = 1`, `max_edits = 2`. Intended modal actions achieve Par 1 with score 100. Suboptimal insertions or typo corrections require multiple edit units and miss Par; mashing backspaces triggers forbid rule. | **PASS** |
| **Goal-Only Task Text** | Task statements state the desired end-state, never specific keystrokes: "Make the 'timeout_ms' field public on line 2.", "Terminate the 'endpoint' declaration with a semicolon on line 2.", "Add the 'async' modifier to the 'fetch_status' function signature on line 4." | **PASS** |
| **Category Hints** | Nudge toward conceptual category without prescribing keys: "Enter insert mode before the cursor...", "Append text after the cursor rather than inserting before it.", "Navigate directly to the target keyword before entering insert mode." | **PASS** |
| **Plausible Code Fixtures** | Start and goal fixtures represent realistic Rust backend networking and service configuration code (`ClientConfig`, `build_url`, `fetch_status`). | **PASS** |
| **Prose Word Count** | Lesson prose is 51 words (budget: <80 words). | **PASS** |
| **Guided Walkthrough** | Conceptual and non-drillable diagnostic behavior (`diagnostic-navigation`) is structured as a guided walkthrough (`[[walkthrough]]`), not an unobservable scored drill. | **PASS** |
| **Dependency Hygiene** | Module 0 honestly declares `requires = []`. Admitted as fully runnable (`draft = false`), serving as the root of the curriculum DAG. | **PASS** |

---

## 2. Exercise Calibration Details

### Exercise 1: `insert-visibility` (Index 0)

- **Task:** `"Make the 'timeout_ms' field public on line 2."`
- **Hint:** `"Enter insert mode before the cursor to insert the visibility modifier."`
- **Start Fixture:** `lessons/fixtures/modal-00/insert-visibility.start.kata`
  ```rust
  pub struct ClientConfig {
      timeout_ms: u64,
      max_retries: u32,
  }
  ```
- **Goal Fixture:** `lessons/fixtures/modal-00/insert-visibility.goal.kata`
  ```rust
  pub struct ClientConfig {
      pub timeout_ms: u64,
      max_retries: u32,
  }
  ```
- **Initial Cursor Position:** Line 2, Column 5 (`[2, 5]`).
  - Line 2: `    timeout_ms: u64,`
  - Columns 1–4 are spaces; Column 5 places the normal-mode block cursor directly on `'t'` in `timeout_ms`.
- **Intended Technique:** `i` (Insert before cursor) + `pub ` + `<Esc>` / `Ctrl-[`:
  1. In Normal mode at `[2, 5]`, student presses `i`.
  2. Cursor enters Insert mode at column 5 (before `'t'`).
  3. Student types `pub ` and exits Insert mode with `<Esc>` or `Ctrl-[`.
  4. Contiguous typing in Insert mode coalesces into 1 `EditUnit::Insertion`.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Suboptimal Alternative (Typo & Correction):**
  1. Student enters insert mode and types `pubb `, then `<Esc>` (1 edit).
  2. Student returns to delete the extra `'b'` with normal-mode `x` (2nd edit unit).
  3. Edit count: **2** (`max_edits = 2`).
  4. Verdict: **Passed with Penalty (Score: 85)**.
- **Forbidden Alternative (Backspace Mashing):**
  1. Student enters insert mode, types incorrect text, and mashes backspace 3+ times.
  2. Exceeds `forbid.backspace_run = 3`.
  3. Verdict: **Failed** with `"Triggered forbidden pattern: backspace run of 3 consecutive deletions exceeds limit of 3"`.

---

### Exercise 2: `append-semicolon` (Index 1)

- **Task:** `"Terminate the 'endpoint' declaration with a semicolon on line 2."`
- **Hint:** `"Append text after the cursor rather than inserting before it."`
- **Start Fixture:** `lessons/fixtures/modal-00/append-semicolon.start.kata`
  ```rust
  pub fn build_url() -> String {
      let endpoint = "https://api.zed.dev"
      format!("{}/v1/status", endpoint)
  }
  ```
- **Goal Fixture:** `lessons/fixtures/modal-00/append-semicolon.goal.kata`
  ```rust
  pub fn build_url() -> String {
      let endpoint = "https://api.zed.dev";
      format!("{}/v1/status", endpoint)
  }
  ```
- **Initial Cursor Position:** Line 2, Column 40 (`[2, 40]`).
  - Line 2 has 40 characters; Column 40 places the normal-mode block cursor on the closing double quote `"` of `"https://api.zed.dev"`.
- **Intended Technique:** `a` (Append after cursor) or `A` (Append at line end) + `;` + `<Esc>`:
  1. In Normal mode at `[2, 40]`, student presses `a`.
  2. Cursor enters Insert mode at column 41 (after `"`).
  3. Student types `;` and presses `<Esc>`.
  4. Line 2 becomes `    let endpoint = "https://api.zed.dev";`.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Common Stumbling Block (Pressing `i` instead of `a`):**
  1. Student mistakenly presses `i` (insert before cursor) at `[2, 40]`.
  2. Typing `;` inserts the semicolon inside the string: `"https://api.zed.dev;"`.
  3. Fails goal verification. Correcting requires navigating outside the string and re-inserting, incurring additional edits and missing Par.

---

### Exercise 3: `navigate-and-insert` (Index 2)

- **Task:** `"Add the 'async' modifier to the 'fetch_status' function signature on line 4."`
- **Hint:** `"Navigate directly to the target keyword before entering insert mode."`
- **Start Fixture:** `lessons/fixtures/modal-00/navigate-and-insert.start.kata`
  ```rust
  use std::time::Duration;

  /// Fetch remote service health status.
  pub fn fetch_status(timeout: Duration) -> Result<HttpStatus, NetworkError> {
      let client = HttpClient::new(timeout);
      client.get("/health")
  }
  ```
- **Goal Fixture:** `lessons/fixtures/modal-00/navigate-and-insert.goal.kata`
  ```rust
  use std::time::Duration;

  /// Fetch remote service health status.
  pub async fn fetch_status(timeout: Duration) -> Result<HttpStatus, NetworkError> {
      let client = HttpClient::new(timeout);
      client.get("/health")
  }
  ```
- **Initial Cursor Position:** Line 1, Column 1 (`[1, 1]`).
  - Starts at file origin on `use`.
- **Intended Technique:** Normal-mode directional navigation (`3j` or `jjj`, then `w` or `l` to `[4, 5]` on `'f'` of `fn`) + `i` + `async ` + `<Esc>`:
  1. Cursor navigates through lines without buffer modification.
  2. Because navigation emits 0 LSP `didChange` notifications, movement does not count against the edit budget.
  3. At `[4, 5]`, student presses `i`, types `async `, and exits with `<Esc>`.
  4. Resulting buffer matches the goal fixture.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.

---

### Guided Walkthrough: `diagnostic-navigation`

- **ID:** `diagnostic-navigation`
- **Title:** `"Inline diagnostics and error jumping in Zed"`
- **Prompt:** Directs student to create a intentional syntax error, observe Zed's native inline diagnostic banner rendering directly beneath the line, navigate across diagnostics with `g ]` and `g [`, and restore buffer health with `u`.
- **Routing:** Unscored conceptual walkthrough dispatched to `zedkata-agent` per REQ-25 / REQ-29.

---

## 3. Automated Verification Results

- **Catalog & Fixture Validation:**
  - `crates/zedkata-core/tests/lesson_loading.rs`:
    - `test_workspace_production_lessons_validate`: PASS (verifies 2 runnable lessons: `modal-00-baseline` and `motions-01-word`).
    - `test_baseline_lesson_modal_00_baseline_fixtures_and_schema`: PASS (validates schema, module 0, non-draft status, empty prerequisites, 3 exercises, 1 walkthrough, and fixture resolution).
- **LSP End-to-End Integration:**
  - `crates/zedkata-lsp/tests/feedback_integration_tests.rs`:
    - `test_baseline_lesson_exercise_1_insert_visibility_par_pass`: PASS (Par 1, score 100).
    - `test_baseline_lesson_exercise_2_append_semicolon_par_pass`: PASS (Par 1, score 100).
    - `test_baseline_lesson_exercise_3_navigate_and_insert_par_pass`: PASS (Par 1, score 100).
    - `test_baseline_lesson_exercise_1_forbid_backspace_run`: PASS (emits Error diagnostic, records failure verdict).
- **Workspace Test Suite:**
  - Total: **147 tests** passing cleanly with 0 failures.
  - Linter & Formatting: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt -- --check` pass with 0 warnings.
