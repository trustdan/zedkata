# ZK-022.3 — Third Starter Exercise Authoring, Three-Exercise Calibration, and Goal-Only Evidence

**Date:** 2026-09-14  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `motions-01-word`  
**Exercise:** `delete-word-binding` (index 2)  

---

## 1. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-21, REQ-22):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Moves modal-editing learners to Zed without re-teaching vim from zero. Focuses on combining word navigation (`w`) with operator deletion (`dw`). | **PASS** |
| **One Concept per Exercise** | Single verified concept: word-forward navigation (`w`) targeting an identifier followed by clean word deletion (`dw`). | **PASS** |
| **`par_edits` is the Lesson** | `par_edits = 1`, `max_edits = 2`. Intended technique (`w` + `dw`) achieves par. Mashed backspaces fail forbid rule; lazy `de` leaves double spaces and misses par; premature `dw` destroys `let`. | **PASS** |
| **Goal-Only Task Text** | `"Remove the unused 'mut' binding modifier on line 2."` States the goal, never the keystrokes. | **PASS** |
| **Category Hint** | `"Advance to the target word before deleting."` Nudges toward word-navigation category without prescribing keys. | **PASS** |
| **Plausible Code Fixtures** | Start and goal fixtures represent a realistic Rust event dispatch pipeline using `Session`. | **PASS** |
| **Prose Word Count** | Lesson prose is 48 words (budget: <80 words). | **PASS** |
| **Dependency Hygiene** | Module `motions-01-word` specifies `requires = []`; builds directly on normal mode and concepts introduced in the module. | **PASS** |

---

## 2. Fixture Pair and Modal Mechanics

### Start Fixture (`lessons/fixtures/motions-01/delete-word-binding.start.kata`)
```rust
pub fn process_payload(payload: &[u8]) -> Result<(), AppError> {
    let mut session = Session::new(payload)?;
    session.dispatch()
}
```

### Goal Fixture (`lessons/fixtures/motions-01/delete-word-binding.goal.kata`)
```rust
pub fn process_payload(payload: &[u8]) -> Result<(), AppError> {
    let session = Session::new(payload)?;
    session.dispatch()
}
```

### Initial Cursor Position
- Line 2, Column 5 (1-based cursor coordinates `[2, 5]`).
- Land position in line 2: character `'l'` of `let` (columns 1-4 are indentation whitespace `    `, column 5 is `'l'`).

---

## 3. Exercise 3 Technique Calibration

### Intended Technique: `w` + `dw` (Navigate to Word and Delete)
1. Cursor is positioned at `'l'` in `let` on line 2.
2. In Normal mode, student executes `w`:
   - Zed advances the cursor forward across `"let "` to the first character of the next word: column 9 (`'m'` in `mut`).
   - Normal mode cursor movement emits 0 LSP `textDocument/didChange` notifications.
3. Student executes `dw`:
   - Deletes from the cursor through the word and trailing whitespace: `"mut "` (4 characters).
   - Emits a single LSP `textDocument/didChange` delta.
4. Resulting line 2:
   ```rust
       let session = Session::new(payload)?;
   ```
5. Matches goal fixture cleanly with exactly 1 space between `let` and `session`.
6. Edit count: **1** (`par_edits = 1`).
7. Verdict: **Passed with Par (Score: 100)**.

### Suboptimal Alternative 1: `w` + `de` (Word End Deletion Leaving Whitespace)
1. Student navigates to `'m'` in `mut` with `w`.
2. Student executes `de` instead of `dw`.
3. Motion `e` deletes to the end of the current word (`"mut"`), preserving the trailing space.
4. Resulting line 2:
   ```rust
       let  session = Session::new(payload)?;
   ```
5. Fails goal verification (extra whitespace between `let` and `session`).
6. Deleting the extra whitespace requires a second edit (`x` or `dw`), taking 2 edits (`max_edits = 2`), resulting in a score penalty (score: 50, misses par).

### Suboptimal Alternative 2: Premature `dw` Without Navigation
1. From `'l'` in `let`, student immediately executes `dw`.
2. Deletes `"let "`.
3. Resulting line 2:
   ```rust
       mut session = Session::new(payload)?;
   ```
4. Produces invalid syntax and fails goal verification.

### Forbidden Technique: Backspace Mashing
1. Student enters Insert mode after `mut` and presses Backspace 4 times.
2. Zed emits discrete single-character deletion changes.
3. Exceeds `forbid.backspace_run = 3`.
4. Verdict: **Failed** with reason `"forbidden pattern 'backspace_run' exceeded limit of 3"`.

---

## 4. Starter Course Three-Exercise Calibration Matrix

All three starter exercises for `motions-01-word` are recorded, calibrated, and verified against Zed modal behavior:

| # | Exercise ID | Concept | Initial Cursor | Intended Technique | Observed Edits | Alternative Technique & Consequence | Forbid Boundaries | Budget |
|:---|:---|:---|:---|:---|:---:|:---|:---|:---:|
| **1** | `delete-word-arg` | `dw` | `[2, 5]` (`t` in `target`) | `dw` (deletes `"target "`) | 1 | `de` leaves trailing space before `listener` (fails goal or requires second edit) | `backspace_run = 3` | Par: 1<br>Max: 2 |
| **2** | `delete-to-word-end` | `de` | `[1, 19]` (`E` in `SessionError`) | `de` (deletes `"Error"`, keeps ` {`) | 1 | `dw` deletes trailing space before `{` (fails goal or requires second edit) | `backspace_run = 3` | Par: 1<br>Max: 2 |
| **3** | `delete-word-binding` | `w` + `dw` | `[2, 5]` (`l` in `let`) | `w` to `mut`, then `dw` (deletes `"mut "`) | 1 | `de` leaves double space; premature `dw` deletes `let`; arrow/insert mashing fails forbid | `backspace_run = 3` | Par: 1<br>Max: 2 |

---

## 5. Goal-Only Mode Verification (Unscored Technique)

In goal-only mode (`par_edits: None`), exercises verify buffer correctness against the goal fixture without computing or fabricating a synthetic technique score:

1. **Contract**:
   - `calculate_score(edits, None, max)` returns `(true, 100)`.
   - `AttemptOutcome::Passed { edits, par: None, score: 100 }`.
2. **Diagnostic Feedback**:
   - Emits `"Passed — <N> edits, score 100"` without any synthetic `par` text.
3. **TUI and CLI Display**:
   - Task Pane renders: `Budget: Goal-only (unscored) | Max: <max>`.
   - Live Attempt verdict renders: `PASSED! <N> edit(s) (Goal-only / Unscored)`.
   - Headless `status` CLI renders: `Verdict: PASSED (edits: <N>, Goal-only / Unscored)`.
4. **Validation**:
   - Verified by integration test `test_goal_only_mode_labeled_unscored` in `feedback_integration_tests.rs`.

---

## 6. Automated Verification Results

- **LSP End-to-End Tests**:
  - `test_exercise_3_delete_word_binding_par_pass` validates full LSP `didOpen` -> `didChange` (`w` + `dw`) -> diagnostic feedback -> session persistence.
  - `test_goal_only_mode_labeled_unscored` validates goal-only mode feedback and session representation.
- **Fixture Resolution Tests**:
  - `test_fixture_resolution_independent_of_cwd` validates start and goal fixture existence for all three starter exercises.
- **Suite Status**: All 135 workspace tests pass cleanly across `zedkata-core`, `zedkata-lsp`, and `zedkata-tui`.
