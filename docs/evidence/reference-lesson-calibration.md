# ZK-024.3 — Reference Lesson Calibration and Admission Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `motions-03-textobjects`  
**Prerequisites:** `["motions-01-word", "motions-02-find"]`  

---

## 1. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-23):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Targets modal-editing learners moving to Zed. Focuses on delimited text objects (`ci"`, `ci{`, `ci(`), Zed's `MiniQuotes` / `AnyQuotes` behavior, and inner vs. around scoping. | **PASS** |
| **One Concept per Exercise** | Single atomic concept per exercise: quote text objects (`ci"` / `ciq`), block body clearing (`di{` / `ci{`), and nested delimiter selection by cursor position (`ci(`). | **PASS** |
| **`par_edits` is the Lesson** | All exercises calibrated to `par_edits = 1`. Intended modal operators achieve par. Suboptimal alternatives (extra insertions, multi-step deletions, manual line clearing) incur score penalties or exceed max edits. | **PASS** |
| **Goal-Only Task Text** | Task statements state the desired end-state, never keystrokes: "Replace the string...", "Empty the body...", "Replace the arguments inside the inner call...". | **PASS** |
| **Category Hints** | Nudge toward conceptual category without prescribing specific keys ("You do not need to know which kind of quote it is", "Inner versus around is the whole distinction here", "Cursor position decides which bracket pair you get"). | **PASS** |
| **Plausible Code Fixtures** | Start and goal fixtures represent realistic Rust backend code (`load_environment`, `execute_transaction`, `verify_session`). | **PASS** |
| **Prose Word Count** | Lesson prose is 44 words (budget: <80 words). | **PASS** |
| **Guided Walkthrough** | Conceptual / non-drillable topic (`mini-vs-any`) is structured as a guided walkthrough (`[[walkthrough]]`), not an unobservable scored exercise. | **PASS** |
| **Dependency Hygiene** | Module 3 honestly declares `requires = ["motions-01-word", "motions-02-find"]`. Excluded from runnable catalog via `draft = true` until `motions-02-find` is authored and runnable. | **PASS** |

---

## 2. Exercise Calibration Details

### Exercise 1: `swap-string-contents` (Index 0)

- **Task:** `"Replace the string on line 4 with \"production\". Leave the quotes alone."`
- **Hint:** `"You do not need to know which kind of quote it is."`
- **Start Fixture:** `lessons/fixtures/motions-03/swap-string.start.kata`
  ```rust
  pub fn load_environment(config: &Config) -> Result<Environment, ConfigError> {
      let host = config.get_host()?;
      let port = config.get_port()?;
      let stage = "staging";
      Environment::connect(host, port, stage)
  }
  ```
- **Goal Fixture:** `lessons/fixtures/motions-03/swap-string.goal.kata`
  ```rust
  pub fn load_environment(config: &Config) -> Result<Environment, ConfigError> {
      let host = config.get_host()?;
      let port = config.get_port()?;
      let stage = "production";
      Environment::connect(host, port, stage)
  }
  ```
- **Initial Cursor Position:** Line 4, Column 18 (`[4, 18]`).
  - Line 4: `    let stage = "staging";`
  - Column 18 places the cursor on `'s'`, the first character inside `"staging"`.
- **Intended Technique:** `ci"` or `ciq` (Zed `MiniQuotes` default binding `q`):
  1. Student executes `ci"` or `ciq` in Normal mode.
  2. The text `"staging"` (columns 18 to 24) is deleted, leaving `""`, and Insert mode is entered at column 18.
  3. Student types `production` and presses `<Esc>`.
  4. In `count_edit_units`: operator deletion at `(start)` followed immediately by typing at `(start)` coalesces into 1 `EditUnit::Insertion`.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Suboptimal Alternative (Typo & Correction):**
  1. Student types `productionx`, presses `<Esc>` (1 edit unit).
  2. Student deletes trailing `'x'` with Normal mode `x` operator (2nd edit unit).
  3. Edit count: **2** (`max_edits = 3`).
  4. Verdict: **Passed with Penalty (Score: 85)**.
- **Forbidden Alternative (Backspace Mashing):**
  1. Student enters Insert mode and repeatedly presses Backspace to delete `"staging"`.
  2. Emits discrete single-character deletion changes.
  3. Exceeds `forbid.backspace_run = 3`.
  4. Verdict: **Failed** with `"Triggered forbidden pattern: backspace run of 3 consecutive deletions exceeds limit of 3"`.

---

### Exercise 2: `gut-the-block` (Index 1)

- **Task:** `"Empty the body of the function on line 7, keeping the braces and the signature."`
- **Hint:** `"Inner versus around is the whole distinction here."`
- **Start Fixture:** `lessons/fixtures/motions-03/gut-block.start.kata`
  ```rust
  pub struct Transaction {
      pub id: u64,
      pub amount: i64,
  }

  /// Process a verified transaction.
  pub fn execute_transaction(tx: &Transaction) -> Result<(), TransactionError> {
      validate_balance(tx)?;
      record_audit_log(tx)?;
      apply_ledger_entry(tx)?;
      Ok(())
  }
  ```
- **Goal Fixture:** `lessons/fixtures/motions-03/gut-block.goal.kata`
  ```rust
  pub struct Transaction {
      pub id: u64,
      pub amount: i64,
  }

  /// Process a verified transaction.
  pub fn execute_transaction(tx: &Transaction) -> Result<(), TransactionError> {}
  ```
- **Initial Cursor Position:** Line 8, Column 5 (`[8, 5]`).
  - Column 5 is `'v'` in `validate_balance(tx)?;`, directly inside the function block `{ ... }`.
- **Intended Technique:** `di{` (or `ci{<Esc>` / `diB`):
  1. Student executes `di{` in Normal mode.
  2. In Zed and Vim, inner curly bracket selection selects all text strictly between `{` and `}`: from immediately after `{` on line 7 (offset 78) to immediately before `}` on line 12 (offset 0).
  3. Deletion removes lines 8–11 and intermediate newlines; line 7 and line 12 collapse onto line 7 as `{}`.
  4. Edit count: **1** (`par_edits = 1`).
  5. Verdict: **Passed with Par (Score: 100)**.
- **Suboptimal Alternative (Linewise `dd` Deletion):**
  1. Student deletes each statement with `dd` (4 edits) plus joining braces (`J`).
  2. Edit count: **5+** (exceeds `max_edits = 2`).
  3. Verdict: **Failed** (`OverMaxEdits`).
- **Suboptimal Alternative (`da{` Delete Around):**
  1. `da{` deletes the body and the braces `{}`.
  2. Re-typing `{}` requires an extra edit.
  3. Edit count: **2** (`max_edits = 2`). Misses par.

---

### Exercise 3: `nested-brackets` (Index 2)

- **Task:** `"Replace the arguments inside the inner call on line 11 with \"id, token\". Outer call untouched."`
- **Hint:** `"Cursor position decides which bracket pair you get. Think about where you start."`
- **Start Fixture:** `lessons/fixtures/motions-03/nested.start.kata`
  ```rust
  use std::collections::HashMap;

  pub struct SessionManager {
      sessions: HashMap<u64, String>,
  }

  impl SessionManager {
      pub fn verify_session(&self, user_id: u64, raw_token: &str) -> bool {
          let id = sanitize_id(user_id);
          let token = normalize_token(raw_token);
          validate_credentials(compute_digest(raw_token, 32), self.get_salt(id))
      }
  }
  ```
- **Goal Fixture:** `lessons/fixtures/motions-03/nested.goal.kata`
  ```rust
  use std::collections::HashMap;

  pub struct SessionManager {
      sessions: HashMap<u64, String>,
  }

  impl SessionManager {
      pub fn verify_session(&self, user_id: u64, raw_token: &str) -> bool {
          let id = sanitize_id(user_id);
          let token = normalize_token(raw_token);
          validate_credentials(compute_digest(id, token), self.get_salt(id))
      }
  }
  ```
- **Initial Cursor Position:** Line 11, Column 45 (`[11, 45]`).
  - Line 11: `        validate_credentials(compute_digest(raw_token, 32), self.get_salt(id))`
  - Column 45 is `'r'` of `raw_token`, directly inside the inner `compute_digest(...)` call parentheses.
- **Intended Technique:** `ci(` (or `cib`):
  1. Cursor at column 45 targets the enclosing parentheses pair for `compute_digest(...)` (columns 44 to 58).
  2. `ci(` deletes `"raw_token, 32"` (13 characters) and enters Insert mode at column 45.
  3. Student types `id, token` and presses `<Esc>`.
  4. Deletion and contiguous typing at the deletion offset coalesce into 1 edit unit.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Suboptimal Alternative (Misplaced Cursor / Outer Deletion):**
  1. If cursor is outside column range 44–58 (e.g. at `validate_credentials`), `ci(` targets the outer call parentheses, deleting `compute_digest(...)` and `self.get_salt(id)`.
  2. Requires undo (`u`) or extensive manual typing, consuming edit budget (`max_edits = 4`).

---

## 3. Reference Lesson Calibration Matrix

| # | Exercise ID | Concept | Initial Cursor | Intended Technique | Observed Edits | Alternative Technique & Consequence | Forbid Boundaries | Budget |
|:---|:---|:---|:---|:---|:---:|:---|:---|:---:|
| **1** | `swap-string-contents` | `ci"` / `ciq` | `[4, 18]` (`s` in `staging`) | `ci"` / `ciq` then type `"production"` | 1 | Extra edit for typo correction yields 2 edits (score 85); backspace run fails forbid | `backspace_run = 3` | Par: 1<br>Max: 3 |
| **2** | `gut-the-block` | `di{` / `ci{` | `[8, 5]` (`v` in `validate`) | `di{` (clears body, collapses `{}`) | 1 | Linewise `dd` takes 5+ edits (`OverMaxEdits`); `da{` deletes braces and requires re-typing | `backspace_run = 3` | Par: 1<br>Max: 2 |
| **3** | `nested-brackets` | `ci(` (inner) | `[11, 45]` (`r` in `raw_token`) | `ci(` on inner call, type `"id, token"` | 1 | Outer call cursor position deletes entire call graph; `di(` then `i` takes 2 edits | `backspace_run = 3` | Par: 1<br>Max: 4 |

---

## 4. Admission and Prerequisite Dependency Resolution

Per REQ-08, REQ-22, and REQ-23:

1. **Prerequisite Graph:**
   - `motions-03-textobjects` requires `motions-01-word` and `motions-02-find`.
   - `motions-01-word` is fully implemented and runnable (Module 1).
   - `motions-02-find` is fully implemented and runnable (Module 2, ZK-025.2).
2. **Admission Completion (ZK-025.3):**
   - With both `motions-01-word` and `motions-02-find` runnable, `draft = false` is admitted in `lessons/motions-03-textobjects.toml`.
   - The production catalog now exposes 4 runnable modules (`modal-00-baseline`, `motions-01-word`, `motions-02-find`, `motions-03-textobjects`).
   - `catalog.validate_runnable()` succeeds with **0 errors**.
3. **Admission & Prerequisite Validation:**
   - Verified via integration tests `test_reference_lesson_admission_and_prerequisite_validation` and `test_four_module_runnable_curriculum_admission_and_prerequisites`:
     * Catalog validates cleanly across all 4 runnable lessons and all 12 drills.
     * Prerequisite DAG remains strictly acyclic and correctly sequenced.
     * Setting `draft = false` without prerequisites fails validation with `CoreError::MissingPrerequisite`.

---

## 5. Automated Verification Summary

All verification scenarios are validated by automated regression and integration test suites:

- **Admission & Prerequisite Validation:**
  - `test_four_module_runnable_curriculum_admission_and_prerequisites` in `crates/zedkata-core/tests/lesson_loading.rs`.
  - `test_reference_lesson_admission_and_prerequisite_validation` in `crates/zedkata-core/tests/lesson_loading.rs`.
  - `test_reference_lesson_motions_03_textobjects_fixtures_and_schema` in `crates/zedkata-core/tests/lesson_loading.rs`.
- **TUI Navigation & Prerequisite Unlocking:**
  - `test_advance_blocked_from_find_to_textobjects_when_prerequisites_incomplete` in `crates/zedkata-tui/tests/navigation_tests.rs`.
  - `test_advance_across_lessons_to_textobjects_when_unlocked` in `crates/zedkata-tui/tests/navigation_tests.rs`.
  - `test_core_derived_prerequisite_unlocking_across_sessions` in `crates/zedkata-tui/tests/progress_prerequisites_tests.rs`.
- **End-to-End LSP Feedback Tests:**
  - `test_reference_lesson_exercise_1_swap_string_par_pass` in `crates/zedkata-lsp/tests/feedback_integration_tests.rs`.
  - `test_reference_lesson_exercise_1_swap_string_suboptimal_typo_correction_misses_par` in `crates/zedkata-lsp/tests/feedback_integration_tests.rs`.
  - `test_reference_lesson_exercise_1_swap_string_forbid_backspace_run` in `crates/zedkata-lsp/tests/feedback_integration_tests.rs`.
  - `test_reference_lesson_exercise_2_gut_the_block_par_pass` in `crates/zedkata-lsp/tests/feedback_integration_tests.rs`.
  - `test_reference_lesson_exercise_3_nested_brackets_par_pass` in `crates/zedkata-lsp/tests/feedback_integration_tests.rs`.
- **Suite Status:** All 155 tests pass across `zedkata-core`, `zedkata-lsp`, and `zedkata-tui` with 0 clippy warnings and pristine formatting.
