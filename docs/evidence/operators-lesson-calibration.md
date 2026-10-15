# ZK-026.1 — Native Operator Drills Calibration and Authoring Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `operators-01-edit` (Module 4)  
**Prerequisites:** `["motions-03-textobjects"]`  

---

## 1. Capability Audit & Explicit Deferrals (REQ-22, REQ-24)

Audited against Zed 1.19.2, `assets/keymaps/vim.json`, and `crates/vim/src/`:

| Feature / Motion | Zed Implementation Status | Default Zed Binding | Curriculum Classification | Notes & Decisions |
|:---|:---:|:---|:---:|:---|
| **Single Character Replace** (`r`) | **Native** | `r` -> `vim::Replace` | **Drill** (`replace-char-literal`, Par 1) | Mutates character at cursor directly in normal mode without entering insert mode. Produces 1 atomic edit unit. |
| **Count Prefixes** (`[count]op`) | **Native** | `1`–`9` prefixes | **Drill** (`delete-counted-words`, Par 1) | Numerical prefixes multiply operations into single atomic edit deltas (e.g. `2dw` deletes two words in one edit unit). |
| **Linewise Indent** (`>>` / `<<`) | **Native** | `>>` -> `vim::Indent` | **Drill** (`indent-line-block`, Par 1) | Shifts line by indentation width in a single atomic insertion unit. |
| **Vim Exchange** (`cx`, `cxx`, `X`) | **Unsupported** | None | **Explicitly Deferred** (Walkthrough `exchange-and-surround`) | `vim-exchange` is a third-party Vim plugin (`tommcdo/vim-exchange`), not native Vim and not bundled in Zed. Defer key claims; teach native cut-and-paste (`dd`, `p`) and editor line swaps. |
| **Built-in Surrounds** (`cs`, `ds`, `ys`) | **Native** | `crates/vim/src/surrounds.rs` | **Walkthrough** (`exchange-and-surround`) | Built-in out of the box in Zed, covered conceptually in conjunction with operator substitutions. |
| **Registers & Macros** (`"a`..`"z`, `q`, `@`) | **Native** | Standard Vim | **Module 5** (`operators-02-registers`) | Dedicated to registers, clipboards, and macros in the next curriculum module. |

---

## 2. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-24):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Modal users mastering operator combinations, count prefixes, and linewise formatting in Zed. Teaches single-char replacement (`r`), count multipliers (`2dw`), and linewise indentation (`>>`). | **PASS** |
| **One Concept per Exercise** | Single atomic operator concept per drill: character replacement (`r`), count-prefixed deletion (`2dw`), and linewise indentation (`>>`). | **PASS** |
| **`par_edits` is the Lesson** | All drills calibrated to `par_edits = 1`, `max_edits = 2`. Intended modal actions achieve Par 1 with score 100. Suboptimal multi-step alternatives (e.g. `x` then `i` for replace, or `dw` twice) incur multiple edits and miss Par; backspace runs violate forbid rules. | **PASS** |
| **Goal-Only Task Text** | Task statements state the desired end-state without prescribing keystrokes: "Replace the erroneous 'u' type suffix with 'i' on line 2.", "Delete the two modifier keywords 'pub unsafe' on line 1, leaving the function declaration.", "Indent the match arm on line 3 to align with the rest of the match block." | **PASS** |
| **Category Hints** | Guides toward concept category without dictating key sequences: "Replace the single character at the cursor without entering insert mode.", "Use a numerical count with the word deletion operator to remove both keywords at once.", "Shift the current line right using the linewise indent operator." | **PASS** |
| **Plausible Code Fixtures** | Realistic systems and networking Rust code: buffer allocation capacity (`allocate_buffer`), runtime initialization (`initialize_runtime`), and protocol packet header matching (`parse_header`). | **PASS** |
| **Prose Word Count** | Lesson prose is 49 words (budget: <80 words). | **PASS** |
| **Guided Walkthrough** | Unsupported third-party plugin emulation (`vim-exchange`) and native alternatives (surrounds, cut/paste) are documented in unscored walkthrough `exchange-and-surround`. | **PASS** |
| **Dependency Hygiene** | Module 4 declares honest dependency `requires = ["motions-03-textobjects"]`. Admitted as fully runnable (`draft = false`), expanding the runnable curriculum to 5 modules and 15 calibrated drills. | **PASS** |

---

## 3. Exercise Calibration Details

### Exercise 1: `replace-char-literal` (Index 0)

- **Task:** `"Replace the erroneous 'u' type suffix with 'i' on line 2."`
- **Hint:** `"Replace the single character at the cursor without entering insert mode."`
- **Start Fixture:** `lessons/fixtures/operators-01/replace-char.start.kata`
  ```rust
  pub fn allocate_buffer() -> i64 {
      let capacity: i64 = 1024u64;
      capacity
  }
  ```
- **Goal Fixture:** `lessons/fixtures/operators-01/replace-char.goal.kata`
  ```rust
  pub fn allocate_buffer() -> i64 {
      let capacity: i64 = 1024i64;
      capacity
  }
  ```
- **Initial Cursor Position:** Line 2, Column 29 (`[2, 29]`).
  - Line 2: `    let capacity: i64 = 1024u64;`
  - Column 29 lands normal-mode block cursor directly on `'u'`.
- **Intended Technique:** `ri` (Replace with character `'i'`):
  1. In Normal mode at `[2, 29]`, student presses `r` followed by `i`.
  2. Character `'u'` is replaced with `'i'` in-place without changing modes.
  3. Edit delta: 1 change replacing `(1, 28)..(1, 29)` with `"i"`.
  4. Coalesces into 1 `EditUnit::Modification` (or deletion + insertion in same range).
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Why Alternatives Miss Par:**
  - `s` + `i` + `Esc`: Subsitute character mode enters Insert mode, producing separate deletion and insertion deltas if interrupted or coalesced into multiple edits.
  - `x` + `i` + `i` + `Esc`: Delete character then insert character produces 2 distinct edit units, missing Par.
- **Forbidden Alternative (Backspace Mashing):**
  - Entering insert mode and pressing backspace 3+ times triggers `forbid.backspace_run = 3`.
  - Verdict: **Failed** with `"Triggered forbidden pattern: backspace run of 3 consecutive deletions exceeds limit of 3"`.

---

### Exercise 2: `delete-counted-words` (Index 1)

- **Task:** `"Delete the two modifier keywords 'pub unsafe' on line 1, leaving the function declaration."`
- **Hint:** `"Use a numerical count with the word deletion operator to remove both keywords at once."`
- **Start Fixture:** `lessons/fixtures/operators-01/delete-counted-words.start.kata`
  ```rust
  pub unsafe fn initialize_runtime() {
      setup_tracing();
  }
  ```
- **Goal Fixture:** `lessons/fixtures/operators-01/delete-counted-words.goal.kata`
  ```rust
  fn initialize_runtime() {
      setup_tracing();
  }
  ```
- **Initial Cursor Position:** Line 1, Column 1 (`[1, 1]`).
  - Line 1: `pub unsafe fn initialize_runtime() {`
  - Column 1 lands cursor on `'p'` of `pub`.
- **Intended Technique:** `2dw` (or `d2w`):
  1. In Normal mode at `[1, 1]`, student executes `2dw`.
  2. Both `pub ` and `unsafe ` are deleted in a single atomic motion.
  3. Resulting buffer has `fn initialize_runtime() {`.
  4. Edit delta: 1 deletion of `"pub unsafe "`.
  5. Edit count: **1** (`par_edits = 1`).
  6. Verdict: **Passed with Par (Score: 100)**.
- **Why Performing `dw` Twice Misses Par:**
  - Executing `dw` deletes `"pub "`, generating edit unit 1.
  - Executing `dw` a second time deletes `"unsafe "`, generating edit unit 2.
  - Edit count is 2 (`max_edits = 2`), achieving a passing score of only 50 and missing Par.
  - Count prefix `2dw` is required to achieve Par 1.

---

### Exercise 3: `indent-line-block` (Index 2)

- **Task:** `"Indent the match arm on line 3 to align with the rest of the match block."`
- **Hint:** `"Shift the current line right using the linewise indent operator."`
- **Start Fixture:** `lessons/fixtures/operators-01/indent-line-block.start.kata`
  ```rust
  pub fn parse_header(val: u8) -> &'static str {
      match val {
      0x01 => "START",
          _ => "UNKNOWN",
      }
  }
  ```
- **Goal Fixture:** `lessons/fixtures/operators-01/indent-line-block.goal.kata`
  ```rust
  pub fn parse_header(val: u8) -> &'static str {
      match val {
          0x01 => "START",
          _ => "UNKNOWN",
      }
  }
  ```
- **Initial Cursor Position:** Line 3, Column 5 (`[3, 5]`).
  - Line 3: `    0x01 => "START",` (4 spaces indentation).
  - Cursor at column 5 lands on `'0'`.
- **Intended Technique:** `>>`:
  1. In Normal mode on line 3, student executes `>>`.
  2. Zed shifts line 3 right by 4 spaces, aligning with line 4 (`        _ => "UNKNOWN",`).
  3. Edit delta: 1 insertion of 4 spaces at line 2 col 0.
  4. Edit count: **1** (`par_edits = 1`).
  5. Verdict: **Passed with Par (Score: 100)**.
- **Why Manual Indentation is Suboptimal:**
  - Entering insert mode and typing 4 spaces or pressing Tab creates an insertion, but if cursor positioning is adjusted or backspaced, it easily misses Par or triggers forbid limits.
  - Linewise operator `>>` achieves the exact alignment in 1 atomic action.

---

## 4. Automated Verification Matrix

All 161 automated workspace tests pass across all crates:

- `test_operators_lesson_operators_01_edit_fixtures_and_schema` in `crates/zedkata-core/tests/lesson_loading.rs`:
  - Validates `operators-01-edit` TOML schema, `draft = false`, `module = 4`, requires `["motions-03-textobjects"]`.
  - Verifies start and goal fixtures exist, differ, and resolve cleanly.
- `test_five_module_runnable_curriculum_admission_and_prerequisites`:
  - Validates 5 runnable modules and 15 total drills across the curriculum DAG.
- `test_operators_lesson_exercise_1_replace_char_par_pass` in `crates/zedkata-lsp`: Par 1, score 100.
- `test_operators_lesson_exercise_2_delete_counted_words_par_pass` in `crates/zedkata-lsp`: Par 1, score 100.
- `test_operators_lesson_exercise_3_indent_line_block_par_pass` in `crates/zedkata-lsp`: Par 1, score 100.
- `test_operators_lesson_exercise_1_forbid_backspace_run` in `crates/zedkata-lsp`: Actionable failure diagnostic on forbid limit.
- `test_advance_across_lessons_to_operators_when_unlocked` in `crates/zedkata-tui`: Cross-module advancement from Module 3 to Module 4.
- `test_advance_blocked_from_textobjects_to_operators_when_prerequisites_incomplete` in `crates/zedkata-tui`: Prerequisite lock enforcement.

---

## 5. Manual Acceptance & Curriculum Admission (ZK-026.3)

### Manual Zed Verification
All drills and walkthroughs were manually executed and verified in local Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`) on Windows 11:
1. **`replace-char-literal`**:
   - Cursor placed at line 2 col 29 on `'u'`.
   - Executing `ri` replaces `'u'` with `'i'` in-place without leaving normal mode.
   - Emits exactly 1 edit unit.
   - Meets Par 1 with score 100.
2. **`delete-counted-words`**:
   - Cursor placed at line 1 col 1 on `'p'`.
   - Executing `2dw` cleanly deletes `"pub unsafe "` in a single operation.
   - Emits exactly 1 edit unit.
   - Meets Par 1 with score 100. Doing `dw` twice yields 2 edits and misses Par.
3. **`indent-line-block`**:
   - Cursor placed at line 3 col 5 on `'0'`.
   - Executing `>>` indents the match arm line right by 4 spaces.
   - Emits exactly 1 edit unit.
   - Meets Par 1 with score 100.
4. **`exchange-and-surround` walkthrough**:
   - Verified that `vim-exchange` (`cx`) is not built into Zed.
   - Verified native cut/paste (`dd`, `p`) and native surround operators (`cs"'`, `ds"`, `ysiw"`) function as documented in Zed.
5. **`keymap-contexts` walkthrough**:
   - Verified GPUI context tree hierarchy via `dev: open key context view`.
   - Verified leader save with `"vim_mode == normal && !menu"`, chorded escape `"j k": "vim::NormalBefore"`, and operator delimiter scoping `"vim_operator == a || vim_operator == i"`.

### Checklist Compliance & Admission Sign-Off
- **Prerequisites DAG**: `operators-01-edit` honestly requires `["motions-03-textobjects"]`. All 5 curriculum modules form a continuous, acyclic, fully-runnable learning progression.
- **Conventions**: Goal-only task descriptions, category hints, short prose (49 words < 80 words budget), realistic Rust fixtures.
- **Par Budget Calibration**: Strict Par 1 calibration ensures students execute idiomatic modal actions rather than inefficient fallbacks.
- **Formal Admission**: Content formally admitted to the runnable curriculum catalog. All 161 workspace tests pass cleanly.
