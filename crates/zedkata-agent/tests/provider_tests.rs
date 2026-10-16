use chrono::{Duration, Utc};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use zedkata_agent::{
    build_attempt_user_prompt, system_prompt, AgentConfig, AttemptContext, AttemptExplainer,
    AttemptVerdictContext, ConfiguredProvider, ContextConfig, ForbidRulesContext, ProviderError,
    ProviderKind,
};
use zedkata_core::session::{
    Attempt, CurrentSession, EditRange, RecordedEdit, SessionState, StudentInfo, Verdict,
    CURRENT_SCHEMA_VERSION,
};

fn temp_session_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zedkata_provider_test_{}_{}_{}",
        name,
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn sample_attempt_context(is_truncated: bool) -> AttemptContext {
    AttemptContext {
        lesson_id: "baseline-01-insert-mode".to_string(),
        lesson_title: Some("Insert Mode Baseline".to_string()),
        concepts: vec!["deletion".to_string(), "motion".to_string()],
        exercise_id: Some("insert-01-delete".to_string()),
        exercise_index: 0,
        task_prompt: Some("Delete word 'foo' using dw".to_string()),
        par_edits: Some(1),
        forbid_rules: Some(ForbidRulesContext {
            backspace_run: Some(3),
            unsupported: Vec::new(),
        }),
        attempt_id: "att-sample-123".to_string(),
        total_edits: if is_truncated { 25 } else { 4 },
        bounded_edits: vec![
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 5),
                    end: (0, 6),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 4),
                    end: (0, 5),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 3),
                    end: (0, 4),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 2),
                    end: (0, 3),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
        ],
        truncated_edits_count: if is_truncated { 21 } else { 0 },
        is_truncated,
        total_diff_bytes: 48,
        diff_summary: "Deleted 4 characters via backspaces".to_string(),
        verdict: AttemptVerdictContext::Failed {
            reason: "Violated forbid rule: backspace_run (consecutive backspaces > 3)".to_string(),
        },
        age_seconds: 10,
    }
}

fn create_temp_session(
    dir: &std::path::Path,
    verdict: Option<Verdict>,
    edits: Vec<RecordedEdit>,
) -> PathBuf {
    let session_path = dir.join("session.json");
    let edit_count = edits.len() as u32;
    let state = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: Utc::now() - Duration::hours(1),
        },
        current: CurrentSession {
            lesson_id: "operators-01-edit".to_string(),
            exercise_index: 0,
            buffer_path: Some("temp.kata".to_string()),
            opened_at: Some(Utc::now() - Duration::minutes(10)),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "attempt-test-123".to_string(),
            edit_count,
            edits,
            verdict,
        }),
        history: Vec::new(),
        weaknesses: std::collections::HashMap::new(),
        completed_walkthroughs: Vec::new(),
    };

    let json = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&session_path, json).unwrap();
    session_path
}

#[test]
fn test_system_prompt_integrity() {
    let prompt = system_prompt();
    assert!(prompt.contains("Zedkata Pedagogical Coach"));
    assert!(prompt.contains("Strict Context Grounding"));
    assert!(prompt.contains("Observations"));
    assert!(prompt.contains("Technique Inference"));
    assert!(prompt.contains("Recommendation"));
}

#[test]
fn test_build_attempt_user_prompt_formatting() {
    let ctx = sample_attempt_context(false);
    let prompt = build_attempt_user_prompt(&ctx);

    assert!(prompt.contains("Insert Mode Baseline"));
    assert!(prompt.contains("baseline-01-insert-mode"));
    assert!(prompt.contains("Delete word 'foo' using dw"));
    assert!(prompt.contains("**Par Budget**: 1 edit(s)"));
    assert!(prompt.contains("`backspace_run` active"));
    assert!(prompt.contains("Violated forbid rule: backspace_run"));
    assert!(prompt.contains("**Total Edits Executed**: 4"));
    assert!(!prompt.contains("omitted due to 20-edit"));
}

#[test]
fn test_build_attempt_user_prompt_with_truncation() {
    let ctx = sample_attempt_context(true);
    let prompt = build_attempt_user_prompt(&ctx);

    assert!(prompt.contains("**Total Edits Executed**: 25"));
    assert!(prompt.contains("Earlier 21 edit(s) omitted due to 20-edit / 2 KB bound limit"));
}

#[test]
fn test_grounded_explanation_separates_observations_from_inferences() {
    let ctx = sample_attempt_context(false);
    let provider = ConfiguredProvider::new(AgentConfig {
        provider: ProviderKind::Fake,
        ..Default::default()
    });

    let explanation = provider.explain_attempt(&ctx).expect("Should explain");

    // Must be bounded
    assert!(explanation.is_bounded);

    // Observations must contain objective metrics
    assert!(!explanation.observations.is_empty());
    let obs_str = explanation.observations.join(" ");
    assert!(obs_str.contains("Attempt executed 4 edits"));
    assert!(obs_str.contains("Verdict: Failed with reason"));
    assert!(obs_str.contains("Edit 1:"));

    // Inferences must contain technique analysis, not raw numbers
    assert!(!explanation.inferences.is_empty());
    let inf_str = explanation.inferences.join(" ");
    assert!(inf_str.contains("Inferred technique"));
    assert!(inf_str.contains("backspace"));

    // Recommendation must provide actionable Vim motion
    assert!(
        explanation.recommendation.contains("dw") || explanation.recommendation.contains("motion")
    );

    // Markdown conversion
    let markdown = explanation.to_markdown();
    assert!(markdown.contains("### Observations"));
    assert!(markdown.contains("### Technique Inference"));
    assert!(markdown.contains("### Recommendation"));
}

#[test]
fn test_configured_provider_offline_modes_work_without_network() {
    let ctx = sample_attempt_context(false);

    // All provider kinds should succeed in offline environments
    let kinds = [
        ProviderKind::Fake,
        ProviderKind::Degraded,
        ProviderKind::LmStudio,
        ProviderKind::Ollama,
        ProviderKind::Anthropic,
        ProviderKind::OpenAI,
        ProviderKind::Gemini,
    ];

    for kind in kinds {
        let config = AgentConfig {
            provider: kind,
            model: "test-model".to_string(),
            ..Default::default()
        };
        let provider = ConfiguredProvider::new(config);
        let explanation = provider
            .explain_attempt(&ctx)
            .unwrap_or_else(|e| panic!("Provider {:?} failed offline: {}", kind, e));

        assert!(
            !explanation.observations.is_empty(),
            "Provider {:?} empty obs",
            kind
        );
        assert!(
            !explanation.inferences.is_empty(),
            "Provider {:?} empty inf",
            kind
        );
        assert!(
            !explanation.recommendation.is_empty(),
            "Provider {:?} empty rec",
            kind
        );
    }
}

#[test]
fn test_configured_provider_explains_real_session_file() {
    let dir = temp_session_dir("real_session");
    let edits = vec![
        RecordedEdit {
            at: Utc::now() - Duration::seconds(10),
            range: EditRange {
                start: (0, 3),
                end: (0, 4),
            },
            inserted: String::new(),
            deleted_len: 1,
        },
        RecordedEdit {
            at: Utc::now() - Duration::seconds(5),
            range: EditRange {
                start: (0, 2),
                end: (0, 3),
            },
            inserted: String::new(),
            deleted_len: 1,
        },
    ];

    let session_path = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        edits,
    );

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::new(config);

    let explanation = provider
        .explain_session(&session_path, None, &ContextConfig::default())
        .expect("Should explain session file");

    assert!(explanation
        .observations
        .iter()
        .any(|o| o.contains("Over par")));
    assert!(explanation
        .inferences
        .iter()
        .any(|i| i.contains("Edit budget exceeded")));
    assert!(!explanation.recommendation.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_configured_provider_missing_session_file_explicit_error() {
    let non_existent = PathBuf::from("does_not_exist_session_12345.json");
    let provider = ConfiguredProvider::default_configured();

    let res = provider.explain_session(&non_existent, None, &ContextConfig::default());
    assert!(res.is_err());
    match res.unwrap_err() {
        ProviderError::MissingSession(msg) => {
            assert!(msg.contains("No session found"));
        }
        other => panic!("Unexpected error type: {:?}", other),
    }
}

#[test]
fn test_cli_explain_command_with_session_file() {
    let dir = temp_session_dir("cli_explain");
    let edits = vec![RecordedEdit {
        at: Utc::now() - Duration::seconds(5),
        range: EditRange {
            start: (0, 0),
            end: (0, 2),
        },
        inserted: "pub fn".to_string(),
        deleted_len: 2,
    }];

    let session_path = create_temp_session(
        &dir,
        Some(Verdict::Passed {
            edits: 1,
            par: Some(1),
            score: 100,
        }),
        edits,
    );

    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let output = Command::new(agent_bin)
        .arg("explain")
        .arg("--session")
        .arg(&session_path)
        .arg("--provider")
        .arg("fake")
        .output()
        .expect("Failed to run zedkata-agent explain");

    assert!(output.status.success(), "CLI explain command must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("### Observations"));
    assert!(stdout.contains("### Technique Inference"));
    assert!(stdout.contains("### Recommendation"));
    assert!(stdout.contains("Passed in 1 edits"));

    let _ = fs::remove_dir_all(&dir);
}
