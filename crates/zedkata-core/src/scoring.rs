use crate::lesson::ForbidRules;
use crate::session::{HistoryEntry, RecordedEdit};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// An internal representation of an active edit unit for counting purposes.
#[derive(Debug, Clone, PartialEq)]
enum EditUnit {
    /// Contiguous insertion starting at a given coordinate and ending at a given coordinate.
    Insertion {
        line: u32,
        start_col: u32,
        end_col: u32,
        len: usize,
    },
    /// A discrete single-character deletion (e.g. one backspace keystroke).
    SingleCharDeletion { line: u32, col: u32 },
    /// An operator deletion spanning one or more characters (e.g. `dw`, `d$`).
    OperatorDeletion {
        start: (u32, u32),
        end: (u32, u32),
        deleted_len: usize,
    },
    /// Replacement (change) operation.
    Replacement {
        start: (u32, u32),
        end: (u32, u32),
        deleted_len: usize,
        inserted_len: usize,
    },
}

impl EditUnit {
    /// Check if a subsequent edit is an exact inversion / undo of this unit.
    fn is_inverted_by(&self, edit: &RecordedEdit) -> bool {
        let inserted_char_count = edit.inserted.chars().count();
        match self {
            EditUnit::OperatorDeletion {
                start, deleted_len, ..
            } => {
                // Deletion was inverted if the incoming edit is an insertion at the deletion start
                // of the exact same character count.
                edit.deleted_len == 0
                    && edit.range.start == *start
                    && inserted_char_count == *deleted_len
            }
            EditUnit::SingleCharDeletion { line, col } => {
                // Single-char deletion inverted by single-char insertion at that position.
                edit.deleted_len == 0
                    && edit.range.start == (*line, *col)
                    && inserted_char_count == 1
            }
            EditUnit::Insertion {
                line,
                start_col,
                len,
                ..
            } => {
                // Insertion inverted by a deletion starting at the insertion start of matching length.
                edit.inserted.is_empty()
                    && edit.range.start == (*line, *start_col)
                    && edit.deleted_len == *len
            }
            EditUnit::Replacement {
                start,
                deleted_len,
                inserted_len,
                ..
            } => {
                edit.range.start == *start
                    && edit.deleted_len == *inserted_len
                    && inserted_char_count == *deleted_len
            }
        }
    }
}

/// Count evidence-supported edit units from a sequence of recorded edits.
///
/// Rules:
/// - Discrete operator deletions (e.g. `dw`) count as 1 edit unit.
/// - Single-character deletions (e.g. backspaces) each count as 1 edit unit.
/// - Sequential character insertions at adjacent text positions (typing in insert mode)
///   coalesce into 1 contiguous edit unit.
/// - Inverting edits (undo operations that restore previous state) cancel the undone unit.
pub fn count_edit_units(edits: &[RecordedEdit]) -> u32 {
    let mut units: Vec<EditUnit> = Vec::new();

    for edit in edits {
        let inserted_char_count = edit.inserted.chars().count();

        // 1. Check if this edit inverts the previous edit unit (undo recognition)
        if let Some(top) = units.last() {
            if top.is_inverted_by(edit) {
                units.pop();
                continue;
            }
        }

        // 2. Classify and process the edit
        let is_pure_deletion = edit.inserted.is_empty() && edit.deleted_len > 0;
        let is_pure_insertion = edit.deleted_len == 0 && inserted_char_count > 0;

        if is_pure_deletion {
            if edit.deleted_len == 1 {
                // Discrete single-character deletion (e.g. Backspace)
                units.push(EditUnit::SingleCharDeletion {
                    line: edit.range.start.0,
                    col: edit.range.start.1,
                });
            } else {
                // Operator deletion (e.g. `dw`)
                units.push(EditUnit::OperatorDeletion {
                    start: edit.range.start,
                    end: edit.range.end,
                    deleted_len: edit.deleted_len,
                });
            }
        } else if is_pure_insertion {
            // Check if contiguous with the top unit
            let mut coalesced = false;
            if let Some(top) = units.last_mut() {
                match top {
                    EditUnit::Insertion {
                        line, end_col, len, ..
                    } => {
                        // Contiguous typing on the same line at the cursor left by prior insertion
                        if edit.range.start.0 == *line && edit.range.start.1 == *end_col {
                            *end_col += inserted_char_count as u32;
                            *len += inserted_char_count;
                            coalesced = true;
                        }
                    }
                    // Typing immediately at the start of a deletion (e.g. `ci"`: delete then type)
                    EditUnit::OperatorDeletion { start, .. } if edit.range.start == *start => {
                        *top = EditUnit::Insertion {
                            line: start.0,
                            start_col: start.1,
                            end_col: start.1 + inserted_char_count as u32,
                            len: inserted_char_count,
                        };
                        coalesced = true;
                    }
                    _ => {}
                }
            }

            if !coalesced {
                units.push(EditUnit::Insertion {
                    line: edit.range.start.0,
                    start_col: edit.range.start.1,
                    end_col: edit.range.start.1 + inserted_char_count as u32,
                    len: inserted_char_count,
                });
            }
        } else {
            // Replacement edit (simultaneous deletion and insertion)
            units.push(EditUnit::Replacement {
                start: edit.range.start,
                end: edit.range.end,
                deleted_len: edit.deleted_len,
                inserted_len: inserted_char_count,
            });
        }
    }

    units.len() as u32
}

/// Enforce forbid rules against recorded edits.
///
/// Returns `Ok(())` if no rules were violated, or `Err(reason)` with an actionable explanation.
pub fn check_forbid_rules(
    edits: &[RecordedEdit],
    rules: Option<&ForbidRules>,
) -> Result<(), String> {
    let rules = match rules {
        Some(r) => r,
        None => return Ok(()),
    };

    if !rules.unsupported.is_empty() {
        return Err(format!(
            "Unsupported forbid rule(s): {}",
            rules.unsupported.join(", ")
        ));
    }

    if let Some(limit) = rules.backspace_run {
        let mut current_run: u32 = 0;
        for edit in edits {
            let is_single_char_del = edit.deleted_len == 1 && edit.inserted.is_empty();
            if is_single_char_del {
                current_run += 1;
                if current_run >= limit {
                    return Err(format!(
                        "Triggered forbidden pattern: backspace run of {} consecutive deletions exceeds limit of {}",
                        current_run, limit
                    ));
                }
            } else {
                current_run = 0;
            }
        }
    }

    Ok(())
}

/// Calculate the pass/fail verdict and score for an attempt.
///
/// Returns `(passed, score)`:
/// - If `actual_edits > max_edits`: returns `(false, 0)`.
/// - If `par_edits` is `None` (goal-only exercise): returns `(true, 100)`.
/// - If `actual_edits <= par_edits`: returns `(true, 100)`.
/// - If `actual_edits > par_edits`: deducts 15 points per extra edit over par, clamped to floor of 50.
pub fn calculate_score(
    actual_edits: u32,
    par_edits: Option<u32>,
    max_edits: Option<u32>,
) -> (bool, u32) {
    if let Some(max) = max_edits {
        if actual_edits > max {
            return (false, 0);
        }
    }

    let Some(par) = par_edits else {
        // Goal-only mode
        return (true, 100);
    };

    if actual_edits <= par {
        (true, 100)
    } else {
        let penalty = (actual_edits - par).saturating_mul(15);
        let score = 100u32.saturating_sub(penalty).max(50);
        (true, score)
    }
}

/// Update student concept weaknesses based on exercise outcome.
///
/// Weaknesses are values between 0.0 (mastery) and 1.0 (severe weakness).
/// Updated via exponential moving average (70% prior, 30% new observation), rounded to 2 decimals.
pub fn update_weaknesses(
    current: &HashMap<String, f64>,
    concepts: &[String],
    passed: bool,
    score: u32,
) -> HashMap<String, f64> {
    let mut updated = current.clone();
    if concepts.is_empty() {
        return updated;
    }

    let target_weakness = if !passed {
        1.0
    } else {
        (100u32.saturating_sub(score)) as f64 / 100.0
    };

    for concept in concepts {
        let new_val = match current.get(concept) {
            Some(&curr) => curr * 0.7 + target_weakness * 0.3,
            None => target_weakness,
        };

        let clamped = new_val.clamp(0.0, 1.0);
        let rounded = (clamped * 100.0).round() / 100.0;
        updated.insert(concept.clone(), rounded);
    }

    updated
}

/// Create a standardized history entry for a completed attempt.
pub fn create_history_entry(
    lesson_id: &str,
    exercise_id: &str,
    passed: bool,
    par_edits: Option<u32>,
    actual_edits: u32,
    score: u32,
    completed_at: DateTime<Utc>,
) -> HistoryEntry {
    HistoryEntry {
        lesson_id: lesson_id.to_string(),
        exercise_id: exercise_id.to_string(),
        passed,
        par_edits,
        actual_edits,
        score,
        completed_at,
    }
}
