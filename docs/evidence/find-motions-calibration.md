# ZK-025.2 — Find-Motion Lesson Calibration and Authoring Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `motions-02-find`  
**Prerequisites:** `["motions-01-word"]` (Module 2)  

---

## 1. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-24):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Modal-editing users learning character-directed intra-line navigation and operator combinations in Zed. Teaches forward inclusive find (`f`), forward exclusive till (`t`), reverse forms (`F`, `T`), repetition (`;`, `,`), and operator combinations (`df`, `dt`, `ct`). | **PASS** |
| **One Concept per Exercise** | Single atomic operator-motion pairing per drill: inclusive delimiter deletion (`df`), exclusive delimiter deletion (`dt`), and atomic change-till (`ct`). | **PASS** |
| **`par_edits` is the Lesson** | All exercises calibrated to `par_edits = 1`, `max_edits = 2` or `3`. Intended modal actions achieve Par 1 with score 100. Suboptimal combinations (e.g., delete then separate insert) miss Par; backspace mashing violates forbid rules. | **PASS** |
| **Goal-Only Task Text** | Task statements state the desired end-state without leaking keystrokes: "Delete the 'legacy_' prefix from the function name on line 2.", "Delete the host IP address on line 2, keeping the colon and port number.", "Replace 'status_flag' on line 2 with 'state', preserving the colon and type annotation." | **PASS** |
| **Category Hints** | Guides toward concept category without prescribing key sequences: "Delete inclusive to a target delimiter character within the identifier.", "Delete up to but not including the colon delimiter.", "Change text up to the colon delimiter in a single atomic motion." | **PASS** |
| **Plausible Code Fixtures** | Start and goal fixtures represent authentic Rust networking and systems code: event dispatchers (`legacy_dispatch_event`), socket address bindings (`127.0.0.1:8080`), and session state structs (`SessionRecord`). | **PASS** |
| **Prose Word Count** | Lesson prose is 63 words (budget: <80 words). | **PASS** |
| **Guided Walkthrough** | Conceptual and non-scored find scoping behavior (`single-line-find-scope`) is structured as an unscored guided walkthrough (`[[walkthrough]]`), covering single-line termination and `;` / `,` repetition. | **PASS** |
| **Dependency Hygiene** | Module 2 declares honest dependency `requires = ["motions-01-word"]`. Admitted as fully runnable (`draft = false`), expanding the runnable curriculum to 3 lessons. | **PASS** |

---

## 2. Exercise Calibration Details

### Exercise 1: `delete-find-delimiter` (Index 0)

- **Task:** `"Delete the 'legacy_' prefix from the function name on line 2."`
- **Hint:** `"Delete inclusive to a target delimiter character within the identifier."`
- **Start Fixture:** `lessons/fixtures/motions-02/delete-find-delimiter.start.kata`
  ```rust
  pub fn handle_inbound_stream() {
      legacy_dispatch_event(&payload);
  }
  ```
- **Goal Fixture:** `lessons/fixtures/motions-02/delete-find-delimiter.goal.kata`
  ```rust
  pub fn handle_inbound_stream() {
      dispatch_event(&payload);
  }
  ```
- **Initial Cursor Position:** Line 2, Column 5 (`[2, 5]`).
  - Line 2: `    legacy_dispatch_event(&payload);`
  - Columns 1–4 are spaces; Column 5 places the normal-mode block cursor directly on `'l'` of `legacy_`.
- **Intended Technique:** `df_` (Delete inclusive to target delimiter `_`):
  1. In Normal mode at `[2, 5]`, student executes `df_`.
  2. Zed deletes from current position up to and including the first `_`.
  3. The `legacy_` prefix is deleted, leaving `dispatch_event(&payload);`.
  4. Coalesces into 1 `EditUnit::Deletion`.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Why Word Motion (`dw`) Fails Here:**
  - In Rust and Vim/Zed syntax rules, `_` is part of `is_keyword`. Consequently, `legacy_dispatch_event` is treated as a single word token.
  - Executing `dw` deletes the entire identifier `legacy_dispatch_event`, leaving `(&payload);`, which breaks the goal.
  - `df_` specifically demonstrates targeted sub-identifier boundary navigation where word motions are ineffective.
- **Forbidden Alternative (Backspace Mashing):**
  - Entering insert mode at column 12 and pressing backspace 3+ times violates `forbid.backspace_run = 3`.
  - Verdict: **Failed** with `"Triggered forbidden pattern: backspace run of 3 consecutive deletions exceeds limit of 3"`.

---

### Exercise 2: `delete-till-char` (Index 1)

- **Task:** `"Delete the host IP address on line 2, keeping the colon and port number."`
- **Hint:** `"Delete up to but not including the colon delimiter."`
- **Start Fixture:** `lessons/fixtures/motions-02/delete-till-char.start.kata`
  ```rust
  pub fn listen_addr() -> &'static str {
      let bind = "127.0.0.1:8080";
      bind
  }
  ```
- **Goal Fixture:** `lessons/fixtures/motions-02/delete-till-char.goal.kata`
  ```rust
  pub fn listen_addr() -> &'static str {
      let bind = ":8080";
      bind
  }
  ```
- **Initial Cursor Position:** Line 2, Column 17 (`[2, 17]`).
  - Line 2: `    let bind = "127.0.0.1:8080";`
  - Column 17 places the cursor directly on the first `'1'` of `"127.0.0.1:8080"`.
- **Intended Technique:** `dt:` (Delete till `:`):
  1. In Normal mode at `[2, 17]`, student executes `dt:`.
  2. Zed deletes text from cursor up to, but excluding, `:`.
  3. The IP address `127.0.0.1` is removed while preserving `:8080`.
  4. Edit count: **1** (`par_edits = 1`).
  5. Verdict: **Passed with Par (Score: 100)**.
- **Why `df:` is Suboptimal/Incorrect:**
  - `df:` is inclusive; it deletes `:` as well, producing `"8080"`.
  - Re-inserting the missing `:` requires a second edit unit and misses Par.
- **Why Word Deletion (`dw`) is Clumsy:**
  - `127.0.0.1` contains 4 separate number tokens separated by periods (`.`).
  - Deleting it with word motions requires multiple repetitive motions (`d7w` or `dw` × 7).
  - `dt:` accomplishes the target in exactly 1 atomic motion.

---

### Exercise 3: `change-till-char` (Index 2)

- **Task:** `"Replace 'status_flag' on line 2 with 'state', preserving the colon and type annotation."`
- **Hint:** `"Change text up to the colon delimiter in a single atomic motion."`
- **Start Fixture:** `lessons/fixtures/motions-02/change-till-char.start.kata`
  ```rust
  pub struct SessionRecord {
      status_flag: u32,
      retry_limit: u8,
  }
  ```
- **Goal Fixture:** `lessons/fixtures/motions-02/change-till-char.goal.kata`
  ```rust
  pub struct SessionRecord {
      state: u32,
      retry_limit: u8,
  }
  ```
- **Initial Cursor Position:** Line 2, Column 5 (`[2, 5]`).
  - Line 2: `    status_flag: u32,`
  - Column 5 places the normal-mode block cursor on `'s'` of `status_flag`.
- **Intended Technique:** `ct:` + `state` + `<Esc>`:
  1. In Normal mode at `[2, 5]`, student executes `ct:`.
  2. Zed removes `status_flag` up to `:` and enters Insert mode.
  3. Student types `state` and exits with `<Esc>`.
  4. Emits a single atomic `EditUnit::Replacement` (or contiguous delete+insert sequence coalesced into 1 edit).
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Suboptimal Alternative (Delete then Separate Insert):**
  1. Student executes `dt:` in Normal mode (Edit 1: Deletion).
  2. Student enters Insert mode with `i`, types `state`, and exits with `<Esc>` (Edit 2: Insertion).
  3. Edit count: **2** (`max_edits = 2`).
  4. Verdict: **Passed with Penalty (Score: 85)**.

---

### Guided Walkthrough: `single-line-find-scope`

- **ID:** `single-line-find-scope`
- **Title:** `"Single-line scope of inline find motions"`
- **Prompt:** Instructs student that `f`, `F`, `t`, and `T` search strictly within the current line and do not cross newline boundaries (unlike `/` or `?`). Covers repeating the last find with `;` and reversing with `,`.
- **Routing:** Unscored conceptual walkthrough dispatched to `zedkata-agent` per REQ-25 / REQ-29.

---

## 3. Automated Verification Results

- **Catalog & Fixture Validation:**
  - `crates/zedkata-core/tests/lesson_loading.rs`:
    - `test_workspace_production_lessons_validate`: PASS (validates 3 runnable lessons: `modal-00-baseline`, `motions-01-word`, and `motions-02-find`).
    - `test_find_motions_lesson_fixtures_and_schema`: PASS (validates schema, module 2, non-draft status, prerequisite `motions-01-word`, 3 exercises, 1 walkthrough, and 6 fixture paths).
- **LSP End-to-End Integration:**
  - `crates/zedkata-lsp/tests/feedback_integration_tests.rs`:
    - `test_find_motions_lesson_exercise_1_delete_find_delimiter_par_pass`: PASS (Par 1, score 100).
    - `test_find_motions_lesson_exercise_2_delete_till_char_par_pass`: PASS (Par 1, score 100).
    - `test_find_motions_lesson_exercise_3_change_till_char_par_pass`: PASS (Par 1, score 100).
    - `test_find_motions_lesson_exercise_1_forbid_backspace_run`: PASS (emits Error diagnostic, records failure verdict).
- **TUI & Prerequisite Progression:**
  - `crates/zedkata-tui/tests/navigation_tests.rs`: PASS (unlocked progression across `modal-00-baseline` -> `motions-01-word` -> `motions-02-find`).
  - `crates/zedkata-tui/tests/progress_prerequisites_tests.rs`: PASS (re-verifies multi-lesson prerequisite satisfaction across sessions).
  - `crates/zedkata-tui/tests/starter_journey_tests.rs`: PASS (end-to-end advancement into newly unlocked `motions-02-find`).
  - `crates/zedkata-tui/tests/tui_shell_tests.rs`: PASS (renders 4 lessons, correct lock indicators and navigation).
- **Workspace Test Suite:**
  - Total: **152 tests** passing cleanly with 0 failures across all 3 crates.
  - Linter & Formatting: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt -- --check` pass with 0 warnings.
