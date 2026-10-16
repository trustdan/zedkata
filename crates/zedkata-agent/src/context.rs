use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zedkata_core::{
    catalog::Catalog,
    session::{RecordedEdit, SessionState, Verdict, CURRENT_SCHEMA_VERSION},
};

/// Maximum edit operations allowed in bounded prompt context (REQ-27, ADR 0005).
pub const MAX_EDIT_OPERATIONS: usize = 20;

/// Maximum diff text payload in bytes allowed in bounded prompt context (2 KB).
pub const MAX_DIFF_BYTES: usize = 2048;

/// Default maximum age of an attempt before it is considered stale (24 hours).
pub const DEFAULT_MAX_STALENESS_SECONDS: u64 = 86400;

/// Configuration options for bounding session attempt context.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum number of recent edit operations to keep.
    pub max_edits: usize,
    /// Maximum byte size of text diff payload.
    pub max_diff_bytes: usize,
    /// Maximum age allowed for the attempt before declaring it stale.
    pub max_staleness: Option<chrono::Duration>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_edits: MAX_EDIT_OPERATIONS,
            max_diff_bytes: MAX_DIFF_BYTES,
            max_staleness: Some(chrono::Duration::seconds(
                DEFAULT_MAX_STALENESS_SECONDS as i64,
            )),
        }
    }
}

/// Errors encountered while extracting attempt context from session state.
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Session file not found at {path:?}. Launch an exercise in Zed to create a session.")]
    MissingSessionFile { path: PathBuf },

    #[error("Session file at {path:?} is corrupt or has an incompatible version: {message}")]
    CorruptSessionFile { path: PathBuf, message: String },

    #[error("No active attempt found in session. Please start or select an exercise in Zed.")]
    NoActiveAttempt,

    #[error("Attempt state is stale or expired (last updated {age_seconds}s ago, max allowed {max_age_seconds}s).")]
    StaleAttempt {
        age_seconds: u64,
        max_age_seconds: u64,
    },

    #[error("IO error while reading session state from {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Captured forbid rules for the active exercise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForbidRulesContext {
    pub backspace_run: Option<u32>,
    pub unsupported: Vec<String>,
}

/// Bounded verdict status of the attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttemptVerdictContext {
    Passed {
        edits: u32,
        par: Option<u32>,
        score: u32,
    },
    Failed {
        reason: String,
    },
    InProgress,
    Unspecified,
}

/// Bounded, ground-truth context of the student's last drill attempt extracted from `session.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttemptContext {
    /// Active lesson ID (e.g. "motions-01-word").
    pub lesson_id: String,
    /// Lesson title (e.g. "Word Motions (w, e, b)").
    pub lesson_title: Option<String>,
    /// Conceptual categories targeted by this lesson.
    pub concepts: Vec<String>,
    /// Active exercise ID (e.g. "word-01-forward-word").
    pub exercise_id: Option<String>,
    /// 0-indexed exercise index within the lesson.
    pub exercise_index: usize,
    /// Task description / goal text.
    pub task_prompt: Option<String>,
    /// Par edit budget for this exercise, if defined.
    pub par_edits: Option<u32>,
    /// Forbid rules active for this exercise (e.g. backspace runs).
    pub forbid_rules: Option<ForbidRulesContext>,
    /// Unique attempt identifier.
    pub attempt_id: String,
    /// Total number of edits recorded in the attempt before truncation.
    pub total_edits: u32,
    /// Strictly bounded list of recent edits (capped at 20 edits and 2 KB diff).
    pub bounded_edits: Vec<RecordedEdit>,
    /// Number of older edits truncated from the head of history.
    pub truncated_edits_count: usize,
    /// True if any edits or diff text were truncated to fit context bounds.
    pub is_truncated: bool,
    /// Total byte length of text diff payload across all bounded edits.
    pub total_diff_bytes: usize,
    /// Human- and model-readable diff summary of the bounded edits.
    pub diff_summary: String,
    /// Verdict / failure trigger status.
    pub verdict: AttemptVerdictContext,
    /// Age in seconds of the attempt relative to context extraction time.
    pub age_seconds: u64,
}

impl AttemptContext {
    /// Returns the specific failure trigger or reason, if the attempt failed.
    pub fn failure_reason(&self) -> Option<&str> {
        match &self.verdict {
            AttemptVerdictContext::Failed { reason } => Some(reason.as_str()),
            _ => None,
        }
    }

    /// Formats the bounded context into a structured prompt excerpt adhering to ADR 0005.
    pub fn format_prompt_summary(&self) -> String {
        let mut out = String::new();

        out.push_str("=== Active Attempt Context ===\n");
        out.push_str(&format!("Lesson ID: {}\n", self.lesson_id));
        if let Some(title) = &self.lesson_title {
            out.push_str(&format!("Lesson Title: {}\n", title));
        }
        if !self.concepts.is_empty() {
            out.push_str(&format!("Concepts: {}\n", self.concepts.join(", ")));
        }
        if let Some(ex_id) = &self.exercise_id {
            out.push_str(&format!("Exercise ID: {}\n", ex_id));
        }
        out.push_str(&format!("Exercise Index: {}\n", self.exercise_index));
        if let Some(prompt) = &self.task_prompt {
            out.push_str(&format!("Task Goal: {}\n", prompt));
        }
        if let Some(par) = self.par_edits {
            out.push_str(&format!("Par Edit Budget: {}\n", par));
        }
        if let Some(forbid) = &self.forbid_rules {
            if let Some(run) = forbid.backspace_run {
                out.push_str(&format!("Forbid Rule: Max backspace run = {}\n", run));
            }
            if !forbid.unsupported.is_empty() {
                out.push_str(&format!(
                    "Forbidden Keys/Motions: {}\n",
                    forbid.unsupported.join(", ")
                ));
            }
        }
        out.push_str(&format!("Attempt ID: {}\n", self.attempt_id));
        out.push_str(&format!("Total Edits Captured: {}\n", self.total_edits));

        match &self.verdict {
            AttemptVerdictContext::Passed { edits, par, score } => {
                let par_str = par.map(|p| format!(" (par: {})", p)).unwrap_or_default();
                out.push_str(&format!(
                    "Verdict: Passed in {} edits{}, score: {}\n",
                    edits, par_str, score
                ));
            }
            AttemptVerdictContext::Failed { reason } => {
                out.push_str(&format!("Verdict: Failed - {}\n", reason));
            }
            AttemptVerdictContext::InProgress => {
                out.push_str("Verdict: InProgress\n");
            }
            AttemptVerdictContext::Unspecified => {
                out.push_str("Verdict: Unspecified\n");
            }
        }

        out.push_str(&format!("Attempt Age: {}s\n", self.age_seconds));

        out.push_str("\n=== Bounded Edit Stream ===\n");
        if self.is_truncated {
            out.push_str(&format!(
                "[Bounded Context Notice: {} older edits were omitted to keep context under limit ({} edits retained, {} bytes diff)]\n\n",
                self.truncated_edits_count,
                self.bounded_edits.len(),
                self.total_diff_bytes
            ));
        }

        if self.diff_summary.is_empty() {
            out.push_str("(No edits recorded in attempt)\n");
        } else {
            out.push_str(&self.diff_summary);
        }

        out
    }
}

/// Reads `session.json` strictly from `path` and extracts the bounded last-attempt context.
///
/// Handles missing, old, corrupt, or oversized states explicitly:
/// - Missing file returns [`ContextError::MissingSessionFile`].
/// - Corrupt file or future schema version returns [`ContextError::CorruptSessionFile`].
/// - Missing active attempt returns [`ContextError::NoActiveAttempt`].
/// - Stale attempt older than `config.max_staleness` returns [`ContextError::StaleAttempt`].
/// - Oversized edit history is clamped to `config.max_edits` and `config.max_diff_bytes`.
pub fn extract_last_attempt_context(
    session_path: &Path,
    catalog: Option<&Catalog>,
    config: &ContextConfig,
) -> Result<AttemptContext, ContextError> {
    if !session_path.exists() {
        return Err(ContextError::MissingSessionFile {
            path: session_path.to_path_buf(),
        });
    }

    let content = std::fs::read_to_string(session_path).map_err(|source| ContextError::Io {
        path: session_path.to_path_buf(),
        source,
    })?;

    let session: SessionState = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(err) => {
            return Err(ContextError::CorruptSessionFile {
                path: session_path.to_path_buf(),
                message: err.to_string(),
            });
        }
    };

    if session.version > CURRENT_SCHEMA_VERSION {
        return Err(ContextError::CorruptSessionFile {
            path: session_path.to_path_buf(),
            message: format!(
                "Unsupported session schema version: found {}, supported {}",
                session.version, CURRENT_SCHEMA_VERSION
            ),
        });
    }

    extract_from_session_state(&session, catalog, config)
}

/// Extracts bounded attempt context from an already deserialized [`SessionState`].
pub fn extract_from_session_state(
    session: &SessionState,
    catalog: Option<&Catalog>,
    config: &ContextConfig,
) -> Result<AttemptContext, ContextError> {
    let attempt = match &session.attempt {
        Some(a) => a,
        None => return Err(ContextError::NoActiveAttempt),
    };

    if session.current.lesson_id.trim().is_empty() {
        return Err(ContextError::NoActiveAttempt);
    }

    // Determine attempt age and verify staleness
    let now = Utc::now();
    let last_activity: DateTime<Utc> = attempt
        .edits
        .last()
        .map(|e| e.at)
        .or(session.current.opened_at)
        .unwrap_or(session.student.started_at);

    let age_duration = now.signed_duration_since(last_activity);
    let age_seconds = age_duration.num_seconds().max(0) as u64;

    if let Some(max_staleness) = config.max_staleness {
        let max_seconds = max_staleness.num_seconds().max(0) as u64;
        if age_seconds > max_seconds {
            return Err(ContextError::StaleAttempt {
                age_seconds,
                max_age_seconds: max_seconds,
            });
        }
    }

    // Lookup curriculum catalog metadata if provided
    let (lesson_title, concepts, exercise_id, task_prompt, par_edits, forbid_rules) =
        if let Some(cat) = catalog {
            if let Some(entry) = cat.get(&session.current.lesson_id) {
                let l_title = Some(entry.lesson.title.clone());
                let l_concepts = entry.lesson.concepts.clone();
                if let Some(ex) = entry.lesson.exercises.get(session.current.exercise_index) {
                    let f_rules = ex.forbid.as_ref().map(|f| ForbidRulesContext {
                        backspace_run: f.backspace_run,
                        unsupported: f.unsupported.clone(),
                    });
                    (
                        l_title,
                        l_concepts,
                        Some(ex.id.clone()),
                        Some(ex.task.clone()),
                        ex.par_edits,
                        f_rules,
                    )
                } else {
                    (l_title, l_concepts, None, None, None, None)
                }
            } else {
                (None, Vec::new(), None, None, None, None)
            }
        } else {
            (None, Vec::new(), None, None, None, None)
        };

    // Bound edit history: take at most `config.max_edits`, cap total diff payload at `config.max_diff_bytes`
    let (bounded_edits, truncated_edits_count, is_truncated, total_diff_bytes, diff_summary) =
        bound_edits(&attempt.edits, config.max_edits, config.max_diff_bytes);

    let verdict = match &attempt.verdict {
        Some(Verdict::Passed { edits, par, score }) => AttemptVerdictContext::Passed {
            edits: *edits,
            par: *par,
            score: *score,
        },
        Some(Verdict::Failed { reason }) => AttemptVerdictContext::Failed {
            reason: reason.clone(),
        },
        Some(Verdict::InProgress) => AttemptVerdictContext::InProgress,
        None => AttemptVerdictContext::Unspecified,
    };

    Ok(AttemptContext {
        lesson_id: session.current.lesson_id.clone(),
        lesson_title,
        concepts,
        exercise_id,
        exercise_index: session.current.exercise_index,
        task_prompt,
        par_edits,
        forbid_rules,
        attempt_id: attempt.id.clone(),
        total_edits: attempt.edit_count,
        bounded_edits,
        truncated_edits_count,
        is_truncated,
        total_diff_bytes,
        diff_summary,
        verdict,
        age_seconds,
    })
}

/// Clamps edit history to `max_edits` and `max_diff_bytes`, prioritizing the most recent operations.
fn bound_edits(
    edits: &[RecordedEdit],
    max_edits: usize,
    max_diff_bytes: usize,
) -> (Vec<RecordedEdit>, usize, bool, usize, String) {
    if edits.is_empty() {
        return (Vec::new(), 0, false, 0, String::new());
    }

    // Step 1: Clamp count from the tail
    let count_clamped = if edits.len() > max_edits {
        &edits[edits.len() - max_edits..]
    } else {
        edits
    };

    let initial_dropped = edits.len() - count_clamped.len();

    // Step 2: Accumulate from newest to oldest up to max_diff_bytes
    let mut selected_rev = Vec::new();
    let mut accumulated_bytes = 0usize;
    let mut diff_truncated = false;

    for edit in count_clamped.iter().rev() {
        let edit_payload_bytes = edit.inserted.len();
        if accumulated_bytes + edit_payload_bytes <= max_diff_bytes {
            accumulated_bytes += edit_payload_bytes;
            selected_rev.push(edit.clone());
        } else if selected_rev.is_empty() {
            // Even a single edit exceeds the limit; clamp its inserted string
            diff_truncated = true;
            let mut clamped_edit = edit.clone();
            let mut truncated_text = clamped_edit.inserted;
            if truncated_text.len() > max_diff_bytes {
                truncated_text.truncate(max_diff_bytes);
            }
            clamped_edit.inserted = truncated_text;
            accumulated_bytes = clamped_edit.inserted.len();
            selected_rev.push(clamped_edit);
            break;
        } else {
            diff_truncated = true;
            break;
        }
    }

    // Restore chronological order (oldest to newest among selected window)
    selected_rev.reverse();
    let bounded_edits = selected_rev;

    let total_dropped = edits.len() - bounded_edits.len();
    let is_truncated = initial_dropped > 0 || total_dropped > 0 || diff_truncated;

    // Generate formatted diff summary
    let mut diff_summary = String::new();
    let start_index = total_dropped + 1;

    for (i, edit) in bounded_edits.iter().enumerate() {
        let op_num = start_index + i;
        let ts = edit.at.format("%H:%M:%S%.3f");
        diff_summary.push_str(&format!(
            "[{}] {} @ (line {}, col {}):\n  - deleted {} chars\n  + inserted {:?}\n",
            op_num, ts, edit.range.start.0, edit.range.start.1, edit.deleted_len, edit.inserted
        ));
    }

    (
        bounded_edits,
        total_dropped,
        is_truncated,
        accumulated_bytes,
        diff_summary,
    )
}

/// Extracts weakness context from `session.json` according to the agreed contract under REQ-28.
///
/// Missing or unscored weakness data follows the contract: falls back cleanly to the
/// agreed default baseline concept without failing.
pub fn extract_weakness_context(
    session_path: &Path,
    fallback_concept: Option<&str>,
) -> crate::generator::WeaknessContext {
    crate::generator::WeaknessContext::from_session_path(session_path, fallback_concept)
}
