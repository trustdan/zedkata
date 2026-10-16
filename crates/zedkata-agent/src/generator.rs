use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zedkata_core::session::SessionState;

/// System prompt used to enforce grounded practice exercise generation under REQ-28 and ADR 0005.
pub const SYSTEM_GENERATOR_PROMPT: &str = include_str!("../prompts/generator_system.md");

/// Returns the static system prompt for candidate exercise generation.
pub fn system_generator_prompt() -> &'static str {
    SYSTEM_GENERATOR_PROMPT
}

/// Fallback concept when no weaknesses are recorded or session state is clean/unscored.
pub const DEFAULT_FALLBACK_CONCEPT: &str = "word-motions";

/// Lifecycle status of a generated practice drill candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateStatus {
    /// Newly generated candidate pending validation and storage under ZK-032.2.
    Candidate,
    /// Candidate has passed structural and fixture validation.
    Validated,
    /// Candidate assets have been committed to disk in the user data path.
    Persisted,
}

/// Captured weakness context used to guide model generation of targeted practice drills.
///
/// Adheres to REQ-28:
/// - Extracts scored concept weaknesses from `session.json`.
/// - If weakness data is missing, clean (0.0), or unscored, follows the agreed contract:
///   falls back to a standard baseline concept (`DEFAULT_FALLBACK_CONCEPT`), sets severity to `None`,
///   and marks `is_unscored_or_clean = true`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaknessContext {
    /// Target concept identified for practice (e.g. "dw", "ci", "word-motions").
    pub target_concept: String,
    /// Severity score for this concept (e.g. Some(0.85)), or None if clean/unscored.
    pub severity: Option<f64>,
    /// All recorded weaknesses sorted descending by severity.
    pub all_weaknesses: Vec<(String, f64)>,
    /// Whether weakness data is missing, clean (0.0), or unscored.
    pub is_unscored_or_clean: bool,
    /// Recent failure reasons on this concept from session history, if any.
    pub recent_failures: Vec<String>,
    /// Optional path to session file from which this context was extracted.
    pub session_path: Option<PathBuf>,
}

impl WeaknessContext {
    /// Creates a new `WeaknessContext` from an in-memory `SessionState`.
    ///
    /// Follows the contract:
    /// - If weaknesses exist with severity > 0.0, selects the highest severity concept.
    /// - If weaknesses are absent or clean (<= 0.0), falls back to `fallback` (or `DEFAULT_FALLBACK_CONCEPT`).
    pub fn from_session(session: &SessionState, fallback: Option<&str>) -> Self {
        let fallback_concept = fallback.unwrap_or(DEFAULT_FALLBACK_CONCEPT).to_string();

        let mut sorted: Vec<(String, f64)> = session
            .weaknesses
            .iter()
            .filter(|(_, &v)| v > 0.0)
            .map(|(k, &v)| (k.clone(), v))
            .collect();

        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (target_concept, severity, is_clean) = if let Some(top) = sorted.first() {
            (top.0.clone(), Some(top.1), false)
        } else {
            (fallback_concept.clone(), None, true)
        };

        // Extract recent failures from history and active attempt
        let mut recent_failures = Vec::new();
        if let Some(att) = &session.attempt {
            if let Some(zedkata_core::session::Verdict::Failed { reason }) = &att.verdict {
                recent_failures.push(reason.clone());
            }
        }

        for entry in session.history.iter().rev().take(10) {
            if !entry.passed {
                let note = format!(
                    "Failed exercise '{}' in {} (score: {})",
                    entry.exercise_id, entry.lesson_id, entry.score
                );
                if !recent_failures.contains(&note) {
                    recent_failures.push(note);
                }
            }
        }

        Self {
            target_concept,
            severity,
            all_weaknesses: sorted,
            is_unscored_or_clean: is_clean,
            recent_failures,
            session_path: None,
        }
    }

    /// Reads `session.json` from `path` and extracts weakness context.
    ///
    /// If the session file does not exist, is unparseable, or contains no scored weaknesses,
    /// it follows the contract by returning a valid fallback context rather than failing.
    pub fn from_session_path(path: &Path, fallback: Option<&str>) -> Self {
        let fallback_concept = fallback.unwrap_or(DEFAULT_FALLBACK_CONCEPT).to_string();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                return Self {
                    target_concept: fallback_concept,
                    severity: None,
                    all_weaknesses: Vec::new(),
                    is_unscored_or_clean: true,
                    recent_failures: Vec::new(),
                    session_path: Some(path.to_path_buf()),
                };
            }
        };

        let session: SessionState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => {
                return Self {
                    target_concept: fallback_concept,
                    severity: None,
                    all_weaknesses: Vec::new(),
                    is_unscored_or_clean: true,
                    recent_failures: Vec::new(),
                    session_path: Some(path.to_path_buf()),
                };
            }
        };

        let mut ctx = Self::from_session(&session, fallback);
        ctx.session_path = Some(path.to_path_buf());
        ctx
    }

    /// Formats the weakness context into a readable prompt section.
    pub fn format_prompt_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("### Weakness Context\n");
        out.push_str(&format!(
            "- **Target Concept**: `{}`\n",
            self.target_concept
        ));

        if let Some(sev) = self.severity {
            out.push_str(&format!(
                "- **Severity**: {:.2} (active mistake pattern)\n",
                sev
            ));
        } else if self.is_unscored_or_clean {
            out.push_str("- **Status**: Clean / Unscored (no active weaknesses recorded; foundational practice)\n");
        }

        if !self.all_weaknesses.is_empty() {
            out.push_str("- **All Active Weaknesses**:\n");
            for (concept, sev) in &self.all_weaknesses {
                out.push_str(&format!("  * `{}`: {:.2}\n", concept, sev));
            }
        }

        if !self.recent_failures.is_empty() {
            out.push_str("- **Recent Failure Verdicts**:\n");
            for f in &self.recent_failures {
                out.push_str(&format!("  * {}\n", f));
            }
        }

        out
    }
}

/// A complete candidate practice exercise generated by the model.
///
/// Keeps output as a candidate until validation passes under ZK-032.2.
/// Contains all lesson metadata, TOML serialization, and both start/goal fixture buffers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateExercise {
    /// Timestamped lesson identifier (e.g. `gen-dw-1729012345`).
    pub lesson_id: String,
    /// Human-facing lesson title.
    pub title: String,
    /// Optional module grouping.
    pub module: Option<u32>,
    /// Prerequisite lesson IDs (empty for generated practice).
    pub requires: Vec<String>,
    /// Concepts reinforced by this exercise.
    pub concepts: Vec<String>,
    /// Whether this lesson is marked as draft.
    pub draft: bool,
    /// Pedagogical introduction / motion guidance prose.
    pub prose: String,
    /// Exercise identifier within the lesson (e.g. `gen-dw-drill`).
    pub exercise_id: String,
    /// Task instruction explaining the drill goal.
    pub task: String,
    /// Relative fixture path to initial buffer state (e.g. `fixtures/gen-dw-1729012345/drill.start.kata`).
    pub start_fixture_relative_path: String,
    /// Buffer text before the motion.
    pub start_fixture_content: String,
    /// Relative fixture path to target buffer state (e.g. `fixtures/gen-dw-1729012345/drill.goal.kata`).
    pub goal_fixture_relative_path: String,
    /// Buffer text after the motion.
    pub goal_fixture_content: String,
    /// Par edit budget (minimal edits to solve).
    pub par_edits: Option<u32>,
    /// Maximum allowed edits before failure.
    pub max_edits: Option<u32>,
    /// 1-based cursor coordinates [line, col] pointing at target motion start.
    pub cursor: Option<[u32; 2]>,
    /// Optional hint for the student.
    pub hint: Option<String>,
    /// Max consecutive backspaces allowed before failure.
    pub forbid_backspace_run: Option<u32>,
    /// The generated TOML content representing the full lesson.
    pub raw_toml: String,
    /// Current candidate lifecycle status.
    pub status: CandidateStatus,
}

impl CandidateExercise {
    /// Returns true if this exercise is in uncommitted candidate status.
    pub fn is_candidate(&self) -> bool {
        self.status == CandidateStatus::Candidate
    }

    /// Validates the candidate exercise structure, rules, and paths against `root_dir`.
    pub fn validate(&self, root_dir: &Path) -> Result<(), crate::storage::GeneratedAssetError> {
        crate::storage::validate_candidate(self, root_dir)
    }

    /// Validates and atomically stores this candidate's TOML and fixtures in `root_dir`.
    pub fn validate_and_store(
        &mut self,
        root_dir: &Path,
    ) -> Result<crate::storage::StoredAssetPaths, crate::storage::GeneratedAssetError> {
        self.validate_and_store_with_catalog(root_dir, None)
    }

    /// Validates and stores this candidate's TOML and fixtures in `root_dir`, checking for collisions against `catalog`.
    pub fn validate_and_store_with_catalog(
        &mut self,
        root_dir: &Path,
        catalog: Option<&zedkata_core::catalog::Catalog>,
    ) -> Result<crate::storage::StoredAssetPaths, crate::storage::GeneratedAssetError> {
        crate::storage::store_candidate_assets(self, root_dir, catalog)
    }

    /// Formats a student-facing summary preview of the candidate exercise.
    pub fn format_preview(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "### Candidate Practice Drill: {} (`{}`)\n\n",
            self.title, self.lesson_id
        ));
        out.push_str(&format!("**Status**: {:?}\n", self.status));
        out.push_str(&format!("**Concepts**: {}\n", self.concepts.join(", ")));
        out.push_str(&format!("**Task**: {}\n", self.task));
        if let Some(par) = self.par_edits {
            out.push_str(&format!("**Par Budget**: {} edit(s)\n", par));
        }
        if let Some(cursor) = self.cursor {
            out.push_str(&format!(
                "**Starting Cursor**: Line {}, Column {}\n",
                cursor[0], cursor[1]
            ));
        }
        if let Some(hint) = &self.hint {
            out.push_str(&format!("**Hint**: {}\n", hint));
        }
        out.push_str("\n#### Pedagogical Guidance\n");
        out.push_str(&self.prose);
        out.push_str("\n\n#### Buffer Comparison\n");
        out.push_str("```start\n");
        out.push_str(&self.start_fixture_content);
        out.push_str("```\n\n```goal\n");
        out.push_str(&self.goal_fixture_content);
        out.push_str("```\n");
        out
    }

    /// Generates valid lesson TOML matching the `zedkata-core` lesson schema.
    #[allow(clippy::too_many_arguments)]
    pub fn build_toml(
        lesson_id: &str,
        title: &str,
        concepts: &[String],
        prose: &str,
        exercise_id: &str,
        task: &str,
        start_path: &str,
        goal_path: &str,
        par_edits: Option<u32>,
        max_edits: Option<u32>,
        cursor: Option<[u32; 2]>,
        hint: Option<&str>,
        forbid_backspace_run: Option<u32>,
    ) -> String {
        let mut out = String::new();
        out.push_str("[lesson]\n");
        out.push_str(&format!("id = \"{}\"\n", escape_toml_str(lesson_id)));
        out.push_str(&format!("title = \"{}\"\n", escape_toml_str(title)));
        let concept_strs: Vec<String> = concepts
            .iter()
            .map(|c| format!("\"{}\"", escape_toml_str(c)))
            .collect();
        out.push_str(&format!("concepts = [{}]\n", concept_strs.join(", ")));
        out.push_str("draft = false\n");
        out.push_str(&format!("prose = \"\"\"\n{}\"\"\"\n\n", prose.trim()));

        out.push_str("[[exercise]]\n");
        out.push_str(&format!("id = \"{}\"\n", escape_toml_str(exercise_id)));
        out.push_str(&format!("task = \"{}\"\n", escape_toml_str(task)));
        out.push_str(&format!("start = \"{}\"\n", escape_toml_str(start_path)));
        out.push_str(&format!("goal = \"{}\"\n", escape_toml_str(goal_path)));
        if let Some(par) = par_edits {
            out.push_str(&format!("par_edits = {}\n", par));
        }
        if let Some(max) = max_edits {
            out.push_str(&format!("max_edits = {}\n", max));
        }
        if let Some(cur) = cursor {
            out.push_str(&format!("cursor = [{}, {}]\n", cur[0], cur[1]));
        }
        if let Some(h) = hint {
            out.push_str(&format!("hint = \"{}\"\n", escape_toml_str(h)));
        }

        if let Some(run) = forbid_backspace_run {
            out.push_str("\n[exercise.forbid]\n");
            out.push_str(&format!("backspace_run = {}\n", run));
        }

        out
    }
}

fn escape_toml_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Builds the user prompt requesting a candidate practice exercise for the given weakness context.
pub fn build_candidate_user_prompt(context: &WeaknessContext) -> String {
    let mut out = String::new();
    out.push_str("Please generate a targeted modal practice drill based on the following student context:\n\n");
    out.push_str(&context.format_prompt_summary());
    out.push_str("\nRequirements:\n");
    out.push_str(&format!(
        "1. The drill must directly practice the `{}` motion.\n",
        context.target_concept
    ));
    out.push_str("2. Buffer contents must be concise, clean code (3-6 lines).\n");
    out.push_str(
        "3. The optimal solution must require Par 1 (a single discrete modal operation).\n",
    );
    out.push_str("4. Starting cursor coordinates must be 1-based [line, col] pointing at the start of the motion.\n");
    out.push_str("5. Respond strictly with the requested JSON schema.\n");
    out
}

/// Helper struct for deserializing raw model JSON output.
#[derive(Debug, Deserialize)]
pub struct RawCandidatePayload {
    pub title: String,
    pub prose: String,
    pub exercise_id: String,
    pub task: String,
    pub start_content: String,
    pub goal_content: String,
    pub par_edits: Option<u32>,
    pub max_edits: Option<u32>,
    pub cursor: Option<[u32; 2]>,
    pub hint: Option<String>,
    pub forbid_backspace_run: Option<u32>,
}

/// Parses the model's raw text response into a structured `CandidateExercise`.
///
/// Handles markdown fences, raw JSON, and slugification of IDs adhering to ADR 0005.
pub fn parse_candidate_response(
    raw_text: &str,
    context: &WeaknessContext,
    timestamp_override: Option<u64>,
) -> Result<CandidateExercise, crate::provider::ProviderError> {
    let trimmed = raw_text.trim();

    // Extract JSON from markdown fence if present
    let json_text = if let Some(start) = trimmed.find("```json") {
        let rest = &trimmed[start + 7..];
        if let Some(end) = rest.find("```") {
            rest[..end].trim()
        } else {
            rest.trim()
        }
    } else if let Some(start) = trimmed.find("```") {
        let rest = &trimmed[start + 3..];
        if let Some(end) = rest.find("```") {
            rest[..end].trim()
        } else {
            rest.trim()
        }
    } else {
        trimmed
    };

    let payload: RawCandidatePayload = serde_json::from_str(json_text).map_err(|e| {
        crate::provider::ProviderError::MalformedResponse(format!(
            "Failed to deserialize candidate exercise JSON: {}. Response: {}",
            e, json_text
        ))
    })?;

    let timestamp = timestamp_override.unwrap_or_else(|| Utc::now().timestamp() as u64);
    let concept_slug = slugify(&context.target_concept);
    let lesson_id = format!("gen-{}-{}", concept_slug, timestamp);

    let mut exercise_slug = slugify(&payload.exercise_id);
    if exercise_slug.is_empty() {
        exercise_slug = format!("drill-{}", concept_slug);
    }

    let start_path = format!("fixtures/{}/{}.start.kata", lesson_id, exercise_slug);
    let goal_path = format!("fixtures/{}/{}.goal.kata", lesson_id, exercise_slug);

    let raw_toml = CandidateExercise::build_toml(
        &lesson_id,
        &payload.title,
        std::slice::from_ref(&context.target_concept),
        &payload.prose,
        &exercise_slug,
        &payload.task,
        &start_path,
        &goal_path,
        payload.par_edits,
        payload.max_edits,
        payload.cursor,
        payload.hint.as_deref(),
        payload.forbid_backspace_run,
    );

    Ok(CandidateExercise {
        lesson_id,
        title: payload.title,
        module: None,
        requires: Vec::new(),
        concepts: vec![context.target_concept.clone()],
        draft: false,
        prose: payload.prose,
        exercise_id: exercise_slug,
        task: payload.task,
        start_fixture_relative_path: start_path,
        start_fixture_content: payload.start_content,
        goal_fixture_relative_path: goal_path,
        goal_fixture_content: payload.goal_content,
        par_edits: payload.par_edits,
        max_edits: payload.max_edits,
        cursor: payload.cursor,
        hint: payload.hint,
        forbid_backspace_run: payload.forbid_backspace_run,
        raw_toml,
        status: CandidateStatus::Candidate,
    })
}

/// Synthesizes a deterministic, grounded `CandidateExercise` without network access.
///
/// Tailors the exercise to the target concept (e.g. `dw`, `de`, `ci`, `find`, or fallback `word-motions`).
pub fn synthesize_default_candidate(
    context: &WeaknessContext,
    timestamp_override: Option<u64>,
) -> CandidateExercise {
    let timestamp = timestamp_override.unwrap_or_else(|| Utc::now().timestamp() as u64);
    let concept_slug = slugify(&context.target_concept);
    let lesson_id = format!("gen-{}-{}", concept_slug, timestamp);

    let (title, prose, ex_id, task, start_content, goal_content, cursor, hint) =
        match context.target_concept.as_str() {
            "dw" => (
                "Generated Practice: Delete word (dw)".to_string(),
                "The 'dw' operator deletes from the cursor through the next word boundary in Normal mode.\nPractice cleanly eliminating redundant identifiers without backspacing."
                    .to_string(),
                "delete-redundant-param".to_string(),
                "Delete the unused parameter name 'temp' on line 2.".to_string(),
                "fn calculate(temp: i32, value: i32) -> i32 {\n    value * 2\n}\n".to_string(),
                "fn calculate(value: i32) -> i32 {\n    value * 2\n}\n".to_string(),
                Some([1, 14]),
                Some("Position cursor on 'temp' and press 'dw'.".to_string()),
            ),
            "de" => (
                "Generated Practice: Delete to word end (de)".to_string(),
                "The 'de' operator deletes from the cursor to the end of the current word.\nUse it when you want to trim suffixes or identifiers while keeping trailing whitespace."
                    .to_string(),
                "delete-word-suffix".to_string(),
                "Delete the suffix 'Error' from the struct name on line 1.".to_string(),
                "struct ProcessError {\n    code: u32,\n}\n".to_string(),
                "struct Process {\n    code: u32,\n}\n".to_string(),
                Some([1, 15]),
                Some("Position cursor on 'Error' and press 'de'.".to_string()),
            ),
            "ci\"" | "textobjects" | "ci" => (
                "Generated Practice: Change inside text object (ci\")".to_string(),
                "The 'ci\"' operator deletes the text inside surrounding quotes and switches to Insert mode.\nIt avoids manual cursor positioning to the start of the string."
                    .to_string(),
                "change-inside-quotes".to_string(),
                "Replace the message inside the quotes on line 2 with 'ready'.".to_string(),
                "fn status() -> &'static str {\n    \"initializing\"\n}\n".to_string(),
                "fn status() -> &'static str {\n    \"ready\"\n}\n".to_string(),
                Some([2, 8]),
                Some("Inside the quotes, press 'ci\"', type 'ready', then Escape.".to_string()),
            ),
            "f" | "find" => (
                "Generated Practice: Find character motion (f)".to_string(),
                "The 'f<char>' motion jumps forward inline to the next occurrence of <char>.\nUse it to quickly navigate across statements without multiple 'w' or 'l' keypresses."
                    .to_string(),
                "find-semicolon".to_string(),
                "Delete through the semicolon on line 1 using 'df;'.".to_string(),
                "let result = compute(); let next = 1;\n".to_string(),
                " let next = 1;\n".to_string(),
                Some([1, 1]),
                Some("Press 'df;' to delete from start of line through the semicolon.".to_string()),
            ),
            _ => (
                format!("Generated Practice: {} motions", context.target_concept),
                format!(
                    "Targeted practice drill reinforcing the '{}' motion.\nExecute the target operation cleanly to reach Par 1.",
                    context.target_concept
                ),
                format!("{}-drill", concept_slug),
                format!("Execute the target motion for '{}' to update the buffer.", context.target_concept),
                "// Starter buffer for practice\nlet mut data = vec![1, 2, 3];\n".to_string(),
                "// Starter buffer for practice\nlet data = vec![1, 2, 3];\n".to_string(),
                Some([2, 5]),
                Some(format!("Use modal motion for '{}' to reach Par 1.", context.target_concept)),
            ),
        };

    let start_path = format!("fixtures/{}/{}.start.kata", lesson_id, ex_id);
    let goal_path = format!("fixtures/{}/{}.goal.kata", lesson_id, ex_id);

    let raw_toml = CandidateExercise::build_toml(
        &lesson_id,
        &title,
        std::slice::from_ref(&context.target_concept),
        &prose,
        &ex_id,
        &task,
        &start_path,
        &goal_path,
        Some(1),
        Some(2),
        cursor,
        hint.as_deref(),
        Some(3),
    );

    CandidateExercise {
        lesson_id,
        title,
        module: None,
        requires: Vec::new(),
        concepts: vec![context.target_concept.clone()],
        draft: false,
        prose,
        exercise_id: ex_id,
        task,
        start_fixture_relative_path: start_path,
        start_fixture_content: start_content,
        goal_fixture_relative_path: goal_path,
        goal_fixture_content: goal_content,
        par_edits: Some(1),
        max_edits: Some(2),
        cursor,
        hint,
        forbid_backspace_run: Some(3),
        raw_toml,
        status: CandidateStatus::Candidate,
    }
}

/// Helper function to convert any string into a lowercase ASCII slug with hyphens.
pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}
