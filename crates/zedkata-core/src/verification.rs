use crate::lesson::Exercise;
use crate::session::{Attempt, Verdict};
use serde::{Deserialize, Serialize};

/// Detailed explanation of where two buffers diverge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferDifference {
    pub line: usize,
    pub col: usize,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

/// Explicit outcome of evaluating an attempt against an exercise goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "details")]
pub enum AttemptOutcome {
    /// The buffer has not yet reached the goal, and the attempt is still in progress.
    Unfinished,

    /// The buffer matches the goal within all constraints.
    Passed {
        edits: u32,
        par: Option<u32>,
        score: u32,
    },

    /// The attempt reached a finalized failure state with an actionable explanation.
    Failed { reason: String },

    /// The attempt was already finalized previously; repeated evaluations are idempotent
    /// and will not generate duplicate completion events or history entries.
    AlreadyFinalized { previous_verdict: String },
}

/// Normalize buffer content according to the agreed newline policy:
/// - Replaces Windows CRLF (`\r\n`) and classic Mac CR (`\r`) with standard LF (`\n`).
/// - Standardizes trailing newlines so missing or extra file-end newlines do not cause false mismatches.
/// - Preserves intra-line whitespace, indentation, blank lines, and UTF-8 Unicode characters.
pub fn normalize_buffer(content: &str) -> String {
    let crlf_normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = crlf_normalized.trim_end_matches('\n');
    if trimmed.is_empty() {
        String::new()
    } else {
        let mut result = trimmed.to_string();
        result.push('\n');
        result
    }
}

/// Compare a current buffer against a goal buffer under the normalized newline policy.
pub fn compare_buffers(current: &str, goal: &str) -> bool {
    normalize_buffer(current) == normalize_buffer(goal)
}

/// Identify the first line and column where two buffers differ, returning actionable diff context.
pub fn diff_buffers(current: &str, goal: &str) -> Option<BufferDifference> {
    let norm_current = normalize_buffer(current);
    let norm_goal = normalize_buffer(goal);

    if norm_current == norm_goal {
        return None;
    }

    let current_lines: Vec<&str> = norm_current.lines().collect();
    let goal_lines: Vec<&str> = norm_goal.lines().collect();

    let max_lines = current_lines.len().max(goal_lines.len());
    for i in 0..max_lines {
        let line_num = i + 1;
        match (current_lines.get(i), goal_lines.get(i)) {
            (Some(c_line), Some(g_line)) => {
                if c_line != g_line {
                    // Find first differing character column
                    let mut col = 1;
                    for (c_char, g_char) in c_line.chars().zip(g_line.chars()) {
                        if c_char != g_char {
                            break;
                        }
                        col += 1;
                    }

                    return Some(BufferDifference {
                        line: line_num,
                        col,
                        expected: (*g_line).to_string(),
                        actual: (*c_line).to_string(),
                        message: format!(
                            "Line {}, col {}: expected '{}', found '{}'",
                            line_num, col, g_line, c_line
                        ),
                    });
                }
            }
            (None, Some(g_line)) => {
                return Some(BufferDifference {
                    line: line_num,
                    col: 1,
                    expected: (*g_line).to_string(),
                    actual: String::new(),
                    message: format!(
                        "Missing line {}: expected '{}', but buffer ended prematurely",
                        line_num, g_line
                    ),
                });
            }
            (Some(c_line), None) => {
                return Some(BufferDifference {
                    line: line_num,
                    col: 1,
                    expected: String::new(),
                    actual: (*c_line).to_string(),
                    message: format!(
                        "Unexpected extra line {}: found '{}', but goal has no more lines",
                        line_num, c_line
                    ),
                });
            }
            (None, None) => break,
        }
    }

    None
}

/// Evaluate an attempt against the exercise goal, enforcing idempotence and budgets.
pub fn verify_attempt(
    current_buffer: &str,
    goal_buffer: &str,
    attempt: &Attempt,
    exercise: &Exercise,
) -> AttemptOutcome {
    // 1. If already finalized, do not produce duplicate completion events
    if let Some(ref verdict) = attempt.verdict {
        let status_desc = match verdict {
            Verdict::Passed { edits, score, .. } => {
                format!("Passed ({} edits, score {})", edits, score)
            }
            Verdict::Failed { reason } => format!("Failed ({})", reason),
            Verdict::InProgress => String::new(),
        };

        if !status_desc.is_empty() {
            return AttemptOutcome::AlreadyFinalized {
                previous_verdict: status_desc,
            };
        }
    }

    // 2. Determine evidence-supported edit count
    let actual_edits = if !attempt.edits.is_empty() {
        crate::scoring::count_edit_units(&attempt.edits)
    } else {
        attempt.edit_count
    };

    // 3. Enforce forbid rules against recorded edits
    if let Err(forbid_reason) =
        crate::scoring::check_forbid_rules(&attempt.edits, exercise.forbid.as_ref())
    {
        return AttemptOutcome::Failed {
            reason: forbid_reason,
        };
    }

    // 4. Check buffer match against goal
    if compare_buffers(current_buffer, goal_buffer) {
        let (passed, score) =
            crate::scoring::calculate_score(actual_edits, exercise.par_edits, exercise.max_edits);

        if !passed {
            return AttemptOutcome::Failed {
                reason: format!(
                    "Over par budget: reached goal in {} edits, exceeding maximum of {}",
                    actual_edits,
                    exercise.max_edits.unwrap_or(0)
                ),
            };
        }

        AttemptOutcome::Passed {
            edits: actual_edits,
            par: exercise.par_edits,
            score,
        }
    } else {
        // Buffer has not yet reached the goal
        AttemptOutcome::Unfinished
    }
}
