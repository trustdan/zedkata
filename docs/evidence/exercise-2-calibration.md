# ZK-022.2 — Second Starter Exercise Authoring and Calibration Evidence

**Date:** 2026-09-14  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `motions-01-word`  
**Exercise:** `delete-to-word-end` (index 1)  

---

## 1. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-21, REQ-22):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Moves modal-editing learners to Zed without re-teaching vim from zero. Focuses on clean operator navigation. | **PASS** |
| **One Concept per Exercise** | Single verified concept: delete forward to the end of the current word (`de`). | **PASS** |
| **`par_edits` is the Lesson** | `par_edits = 1`, `max_edits = 2`. Intended technique (`de`) achieves par. Mashed backspaces fail forbid rule; lazy `dw` eats trailing whitespace and misses par. | **PASS** |
| **Goal-Only Task Text** | `"Delete the suffix 'Error' from the struct name on line 1."` States the goal, never the keystrokes. | **PASS** |
| **Category Hint** | `"Delete forward to the end of the current word."` Nudges toward word-end category without prescribing keys. | **PASS** |
| **Plausible Code Fixtures** | Start and goal fixtures represent a realistic Rust `Session` struct definition. | **PASS** |
| **Prose Word Count** | Lesson prose is 48 words (budget: <80 words). | **PASS** |
| **Dependency Hygiene** | Module `motions-01-word` specifies `requires = []`; builds directly on normal mode and concepts introduced in the module. | **PASS** |

---

## 2. Fixture Pair and Modal Mechanics

### Start Fixture (`lessons/fixtures/motions-01/delete-to-word-end.start.kata`)
```rust
pub struct SessionError {
    pub retry_count: u32,
    pub is_active: bool,
}
```

### Goal Fixture (`lessons/fixtures/motions-01/delete-to-word-end.goal.kata`)
```rust
pub struct Session {
    pub retry_count: u32,
    pub is_active: bool,
}
```

### Initial Cursor Position
- Line 1, Column 19 (1-based cursor coordinates `[1, 19]`).
- Land position in line 1: character `'E'` of `SessionError` (`pub struct Session` = 18 chars, column 19 = `'E'`).

---

## 3. Technique Calibration

### Intended Technique: `de` (Delete to End of Word)
1. Cursor is positioned at `'E'` in `SessionError`.
2. Student executes `de` in Normal mode.
3. In Vim / Zed modal emulation:
   - The motion `e` advances to the last character of the current word (`'r'`).
   - The `d` operator deletes the span from the cursor through the word end: `"Error"` (5 characters).
   - Trailing whitespace before `{` is preserved.
4. Resulting buffer:
   ```rust
   pub struct Session {
       pub retry_count: u32,
       pub is_active: bool,
   }
   ```
5. Edit count: **1** (`par_edits = 1`).
6. Verdict: **Passed with Par (Score: 100)**.

### Suboptimal Alternative: `dw` (Delete Word and Trailing Space)
1. At `'E'`, `dw` deletes through the word and trailing whitespace up to the next word start (`'{'`).
2. Characters deleted: `"Error "` (6 characters).
3. Resulting buffer:
   ```rust
   pub struct Session{
   ...
   ```
4. Fails goal verification (missing space before `{`). Re-inserting the missing space requires a second edit (`i <Esc>`), which incurs 2 edits and misses par.

### Forbidden Technique: Backspace Mashing
1. Student enters Insert mode and repeatedly presses Backspace to delete the 5 characters.
2. Emits discrete single-character deletion changes.
3. Exceeds `forbid.backspace_run = 3`.
4. Verdict: **Failed** with reason `"forbidden pattern 'backspace_run' exceeded limit of 3"`.

---

## 4. Automated Verification Results

- Verified via end-to-end LSP integration test: `test_exercise_2_delete_to_word_end_par_pass` in `crates/zedkata-lsp/tests/feedback_integration_tests.rs`.
- Catalog validation: `test_workspace_production_lessons_validate` and `test_fixture_resolution_independent_of_cwd` in `crates/zedkata-core/tests/lesson_loading.rs`.
- Total test suite: 132 passing tests across `zedkata-core`, `zedkata-lsp`, and `zedkata-tui`.
