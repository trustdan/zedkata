use crate::context::{AttemptContext, AttemptVerdictContext};
use crate::provider::Explanation;

/// System prompt used to enforce grounded mistake explanations under REQ-27.
pub const SYSTEM_EXPLANATION_PROMPT: &str = include_str!("../prompts/explanation_system.md");

/// Returns the static system explanation prompt.
pub fn system_prompt() -> &'static str {
    SYSTEM_EXPLANATION_PROMPT
}

/// Builds a structured user prompt containing bounded attempt context.
///
/// Adheres strictly to ADR 0005 and REQ-27 by embedding only the bounded edit delta,
/// task metadata, verdict, and forbid rules without hallucinating unobserved state.
pub fn build_attempt_user_prompt(context: &AttemptContext) -> String {
    let mut out = String::new();

    out.push_str("### Context\n");
    if let Some(title) = &context.lesson_title {
        out.push_str(&format!(
            "- **Lesson**: {} (`{}`)\n",
            title, context.lesson_id
        ));
    } else {
        out.push_str(&format!("- **Lesson ID**: `{}`\n", context.lesson_id));
    }
    out.push_str(&format!(
        "- **Exercise Index**: {}\n",
        context.exercise_index
    ));
    if let Some(task) = &context.task_prompt {
        out.push_str(&format!("- **Task Goal**: {}\n", task));
    }
    if !context.concepts.is_empty() {
        out.push_str(&format!(
            "- **Concepts**: {}\n",
            context.concepts.join(", ")
        ));
    }
    if let Some(par) = context.par_edits {
        out.push_str(&format!("- **Par Budget**: {} edit(s)\n", par));
    }

    if let Some(rules) = &context.forbid_rules {
        if let Some(limit) = rules.backspace_run {
            out.push_str(&format!(
                "- **Forbid Rules**: `backspace_run` active (limit: {} consecutive backspaces)\n",
                limit
            ));
        }
    }

    out.push_str("\n### Attempt Verdict\n");
    match &context.verdict {
        AttemptVerdictContext::Passed { edits, par, score } => {
            let par_str = par.map(|p| format!(" (par: {})", p)).unwrap_or_default();
            out.push_str(&format!(
                "- **Status**: PASSED in {} edit(s){}, score: {}\n",
                edits, par_str, score
            ));
        }
        AttemptVerdictContext::Failed { reason } => {
            out.push_str(&format!(
                "- **Status**: FAILED\n- **Failure Reason**: {}\n",
                reason
            ));
        }
        AttemptVerdictContext::InProgress => {
            out.push_str("- **Status**: IN PROGRESS (mid-attempt guidance requested)\n");
        }
        AttemptVerdictContext::Unspecified => {
            out.push_str("- **Status**: UNSPECIFIED (no verdict finalized)\n");
        }
    }

    out.push_str("\n### Bounded Edit Delta\n");
    out.push_str(&format!(
        "- **Total Edits Executed**: {}\n- **Retained Operations**: {}\n- **Diff Payload**: {} bytes\n",
        context.total_edits,
        context.bounded_edits.len(),
        context.total_diff_bytes
    ));

    if context.is_truncated {
        out.push_str(&format!(
            "- **Note**: Earlier {} edit(s) omitted due to 20-edit / 2 KB bound limit.\n",
            context.truncated_edits_count
        ));
    }

    if context.bounded_edits.is_empty() {
        out.push_str("- *No edit operations recorded for this attempt.*\n");
    } else {
        out.push_str("\n| # | Start (Line, Col) | Deleted Chars | Inserted Text |\n");
        out.push_str("|---|---|---|---|\n");
        for (i, edit) in context.bounded_edits.iter().enumerate() {
            let ins = if edit.inserted.is_empty() {
                "\"\"".to_string()
            } else {
                format!("{:?}", edit.inserted)
            };
            out.push_str(&format!(
                "| {} | ({}, {}) | {} | {} |\n",
                i + 1,
                edit.range.start.0,
                edit.range.start.1,
                edit.deleted_len,
                ins
            ));
        }
    }

    out.push_str("\nPlease analyze this attempt according to your instructions. Strictly separate objective observations from technique inferences, and provide an actionable recommendation.");
    out
}

/// Deterministically synthesizes a grounded explanation directly from bounded context.
///
/// Guaranteed to:
/// 1. Separate objective observations (counts, coords, diffs, verdicts) from technique inferences.
/// 2. Operate completely offline with zero network access.
/// 3. Provide actionable Vim recommendations tailored to the observed exercise and failure mode.
pub fn synthesize_grounded_explanation(context: &AttemptContext) -> Explanation {
    let mut observations = Vec::new();
    let mut inferences = Vec::new();

    // 1. Observations: objective facts
    if context.lesson_title.is_some()
        || context.task_prompt.is_some()
        || context.par_edits.is_some()
        || context.forbid_rules.is_some()
    {
        let mut meta_parts = Vec::new();
        if let Some(title) = &context.lesson_title {
            meta_parts.push(format!("Lesson: {} (`{}`)", title, context.lesson_id));
        } else {
            meta_parts.push(format!("Lesson ID: `{}`", context.lesson_id));
        }
        if !context.concepts.is_empty() {
            meta_parts.push(format!("Concepts: {}", context.concepts.join(", ")));
        }
        if let Some(task) = &context.task_prompt {
            meta_parts.push(format!("Task: '{}'", task));
        }
        if let Some(par) = context.par_edits {
            meta_parts.push(format!("Par budget: {} edit(s)", par));
        }
        if let Some(rules) = &context.forbid_rules {
            if let Some(limit) = rules.backspace_run {
                meta_parts.push(format!("Forbid rule: backspace_run <= {}", limit));
            }
        }
        if !meta_parts.is_empty() {
            observations.push(meta_parts.join(" | "));
        }
    }

    let edit_scope_desc = if context.is_truncated {
        format!(
            "Attempt executed {} edits in total ({} retained in bounded window, {} older edits omitted, {} bytes diff).",
            context.total_edits,
            context.bounded_edits.len(),
            context.truncated_edits_count,
            context.total_diff_bytes
        )
    } else {
        format!(
            "Attempt executed {} edits in total ({} retained in bounded context, {} bytes diff).",
            context.total_edits,
            context.bounded_edits.len(),
            context.total_diff_bytes
        )
    };
    observations.push(edit_scope_desc);

    match &context.verdict {
        AttemptVerdictContext::Passed { edits, par, score } => {
            let par_label = par
                .map(|p| format!(" against par of {}", p))
                .unwrap_or_default();
            observations.push(format!(
                "Verdict: Passed in {} edits{} with score {}.",
                edits, par_label, score
            ));
            inferences
                .push("Technique achieved goal buffer state within permitted budget.".to_string());
        }
        AttemptVerdictContext::Failed { reason } => {
            observations.push(format!("Verdict: Failed with reason '{}'.", reason));
            let reason_lower = reason.to_lowercase();
            if reason_lower.contains("backspace") {
                inferences.push(
                    "Inferred technique: Excessive backspace usage detected. Student relied on character-by-character backspacing instead of word/motion operators (e.g. 'dw' or 'd$').".to_string(),
                );
            } else if reason_lower.contains("par") || reason_lower.contains("edit") {
                inferences.push(
                    "Inferred technique: Edit budget exceeded. Student navigated or edited using individual character motions (h/j/k/l or Delete) rather than semantic word motions (w/e/b or dw).".to_string(),
                );
            } else {
                inferences.push(format!(
                    "Inferred technique: Attempt diverged from optimal motion due to '{}'.",
                    reason
                ));
            }
        }
        AttemptVerdictContext::InProgress => {
            observations.push("Verdict: Attempt is currently in progress.".to_string());
            inferences.push(
                "Inferred technique: Student is requesting mid-drill guidance before completing the exercise."
                    .to_string(),
            );
        }
        AttemptVerdictContext::Unspecified => {
            observations.push("Verdict: Attempt has no finalized verdict recorded.".to_string());
        }
    }

    // Concrete edit observations (first 5 retained edits)
    for (i, edit) in context.bounded_edits.iter().take(5).enumerate() {
        observations.push(format!(
            "Edit {}: deleted {} chars, inserted {:?} at line {}, col {}.",
            i + 1,
            edit.deleted_len,
            edit.inserted,
            edit.range.start.0,
            edit.range.start.1
        ));
    }

    // Formulate actionable recommendation based on task prompt or failure mode
    let recommendation = if let Some(task) = &context.task_prompt {
        let task_lower = task.to_lowercase();
        if task_lower.contains("delete") || task_lower.contains("remove") {
            format!("To accomplish '{}', use motion deletion operators like 'dw', 'de', or 'd$' in Normal mode to achieve Par 1 without manual backspacing.", task)
        } else if task_lower.contains("replace") || task_lower.contains("swap") {
            format!("To accomplish '{}', use 'r<char>' for single character replacement, or 'cw' / 'ci\"' to replace words in a single edit.", task)
        } else if task_lower.contains("find") || task_lower.contains("seek") {
            format!("To accomplish '{}', jump directly to the target character using 'f<char>' or 't<char>' rather than repeatedly tapping 'l' or 'w'.", task)
        } else {
            format!("To accomplish '{}', review the target motion in the lesson prose and execute the minimal operator sequence in Normal mode to reach Par.", task)
        }
    } else {
        "Review the target motion in the lesson instructions and minimize keystrokes to reach Par."
            .to_string()
    };

    let truncation_note = if context.is_truncated {
        Some(format!(
            "Bounded context: {} older edits were omitted to keep context under limit ({} edits retained, {} bytes diff).",
            context.truncated_edits_count,
            context.bounded_edits.len(),
            context.total_diff_bytes
        ))
    } else {
        None
    };

    Explanation {
        observations,
        inferences,
        recommendation,
        is_bounded: true,
        truncation_note,
    }
}

#[derive(serde::Deserialize)]
struct RawExplanationJson {
    observations: Vec<String>,
    inferences: Vec<String>,
    recommendation: String,
    truncation_note: Option<String>,
}

/// Parses a model response into a structured [`Explanation`].
///
/// Supports either:
/// 1. Structured JSON (`{"observations": [...], "inferences": [...], "recommendation": "..."}`), raw or inside markdown fences.
/// 2. Structured Markdown with `### Observations` (`### 1. Observations`), `### Technique Inference` (`### 2. Technique Inference`), and `### Recommendation` (`### 3. Recommendation`) sections.
pub fn parse_explanation_response(
    raw_text: &str,
    context: &AttemptContext,
) -> Result<Explanation, crate::provider::ProviderError> {
    let trimmed = raw_text.trim();
    if trimmed.is_empty() {
        return Err(crate::provider::ProviderError::MalformedResponse(
            "Empty explanation response from model provider.".to_string(),
        ));
    }

    let default_truncation_note = if context.is_truncated {
        Some(format!(
            "Bounded context: {} older edits were omitted to keep context under limit ({} edits retained, {} bytes diff).",
            context.truncated_edits_count,
            context.bounded_edits.len(),
            context.total_diff_bytes
        ))
    } else {
        None
    };

    // 1. Check if JSON (raw or fenced)
    let json_candidate = if let Some(start) = trimmed.find("```json") {
        let rest = &trimmed[start + 7..];
        rest.find("```").map(|end| rest[..end].trim())
    } else if trimmed.starts_with('{') {
        Some(trimmed)
    } else {
        None
    };

    if let Some(json_str) = json_candidate {
        let parsed: RawExplanationJson = serde_json::from_str(json_str).map_err(|e| {
            crate::provider::ProviderError::MalformedResponse(format!(
                "Failed to deserialize explanation JSON: {}. Response: {}",
                e, json_str
            ))
        })?;
        if parsed.observations.is_empty()
            || parsed.inferences.is_empty()
            || parsed.recommendation.trim().is_empty()
        {
            return Err(crate::provider::ProviderError::MalformedResponse(
                "Explanation JSON must include non-empty observations, inferences, and recommendation."
                    .to_string(),
            ));
        }
        return Ok(Explanation {
            observations: parsed.observations,
            inferences: parsed.inferences,
            recommendation: parsed.recommendation.trim().to_string(),
            is_bounded: true,
            truncation_note: parsed.truncation_note.or(default_truncation_note),
        });
    }

    // 2. Parse structured Markdown sections
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Section {
        None,
        Observations,
        Inferences,
        Recommendation,
    }

    let mut current_section = Section::None;
    let mut observations = Vec::new();
    let mut inferences = Vec::new();
    let mut rec_lines = Vec::new();

    for line in trimmed.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('#') {
            let lower = line_trimmed.to_lowercase();
            if lower.contains("observation") {
                current_section = Section::Observations;
                continue;
            } else if lower.contains("inference") || lower.contains("technique") {
                current_section = Section::Inferences;
                continue;
            } else if lower.contains("recommendation") {
                current_section = Section::Recommendation;
                continue;
            }
        }

        if line_trimmed.is_empty() || line_trimmed.starts_with('>') {
            continue;
        }

        let item_text = line_trimmed
            .strip_prefix("- ")
            .or_else(|| line_trimmed.strip_prefix("* "))
            .unwrap_or(line_trimmed)
            .trim()
            .to_string();

        if item_text.is_empty() {
            continue;
        }

        match current_section {
            Section::Observations => observations.push(item_text),
            Section::Inferences => inferences.push(item_text),
            Section::Recommendation => rec_lines.push(item_text),
            Section::None => {}
        }
    }

    if observations.is_empty() || inferences.is_empty() || rec_lines.is_empty() {
        return Err(crate::provider::ProviderError::MalformedResponse(format!(
            "Model response missing required Observations, Technique Inference, or Recommendation sections: {}",
            trimmed
        )));
    }

    Ok(Explanation {
        observations,
        inferences,
        recommendation: rec_lines.join(" "),
        is_bounded: true,
        truncation_note: default_truncation_note,
    })
}
