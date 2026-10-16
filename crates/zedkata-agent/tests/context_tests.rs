use chrono::{Duration, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use zedkata_agent::provider::{AttemptExplainer, ProviderError};
use zedkata_agent::{
    extract_from_session_state, extract_last_attempt_context, AttemptContext,
    AttemptVerdictContext, ContextConfig, ContextError, FakeProvider, MAX_DIFF_BYTES,
    MAX_EDIT_OPERATIONS,
};
use zedkata_core::{
    catalog::Catalog,
    session::{
        Attempt, CurrentSession, EditRange, RecordedEdit, SessionState, StudentInfo, Verdict,
    },
};

fn temp_session_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zedkata_context_test_{}_{}_{}",
        name,
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn lessons_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
}

fn sample_edit(
    line: u32,
    col: u32,
    deleted: usize,
    inserted: &str,
    minutes_ago: i64,
) -> RecordedEdit {
    RecordedEdit {
        at: Utc::now() - Duration::minutes(minutes_ago),
        range: EditRange {
            start: (line, col),
            end: (line, col + deleted as u32),
        },
        inserted: inserted.to_string(),
        deleted_len: deleted,
    }
}

fn make_session(
    lesson_id: &str,
    exercise_index: usize,
    edits: Vec<RecordedEdit>,
    verdict: Option<Verdict>,
) -> SessionState {
    let edit_count = edits.len() as u32;
    SessionState {
        version: 1,
        student: StudentInfo {
            started_at: Utc::now() - Duration::hours(1),
        },
        current: CurrentSession {
            lesson_id: lesson_id.to_string(),
            exercise_index,
            buffer_path: Some("/tmp/test_buffer.kata".to_string()),
            opened_at: Some(Utc::now() - Duration::minutes(30)),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att-12345".to_string(),
            edit_count,
            edits,
            verdict,
        }),
        history: Vec::new(),
        weaknesses: std::collections::HashMap::new(),
        completed_walkthroughs: Vec::new(),
    }
}

#[test]
fn test_extract_bounded_context_golden_normal() {
    let temp_dir = temp_session_dir("golden");
    let session_path = temp_dir.join("session.json");

    let edits = vec![sample_edit(0, 0, 0, "d", 5), sample_edit(0, 1, 4, "", 4)];
    let session = make_session(
        "motions-01-word",
        0,
        edits,
        Some(Verdict::Failed {
            reason: "Over par: 2 edits (par 1)".to_string(),
        }),
    );
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).expect("load catalog");
    let config = ContextConfig::default();

    let context = extract_last_attempt_context(&session_path, Some(&catalog), &config)
        .expect("extract context");

    assert_eq!(context.lesson_id, "motions-01-word");
    assert_eq!(context.exercise_index, 0);
    assert_eq!(context.attempt_id, "att-12345");
    assert_eq!(context.total_edits, 2);
    assert_eq!(context.bounded_edits.len(), 2);
    assert_eq!(context.truncated_edits_count, 0);
    assert!(!context.is_truncated);

    // Catalog enrichment checks
    assert!(context.lesson_title.is_some());
    assert!(context.task_prompt.is_some());
    assert_eq!(context.par_edits, Some(1));
    assert!(context.forbid_rules.is_some());

    // Verdict check
    assert_eq!(
        context.verdict,
        AttemptVerdictContext::Failed {
            reason: "Over par: 2 edits (par 1)".to_string(),
        }
    );
    assert_eq!(context.failure_reason(), Some("Over par: 2 edits (par 1)"));

    // Prompt summary format verification
    let summary = context.format_prompt_summary();
    assert!(summary.contains("Lesson ID: motions-01-word"));
    assert!(summary.contains("Par Edit Budget: 1"));
    assert!(summary.contains("Verdict: Failed - Over par: 2 edits (par 1)"));
    assert!(summary.contains("=== Bounded Edit Stream ==="));
    assert!(summary.contains("- deleted 4 chars"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_extract_oversized_edits_capped_to_20() {
    let temp_dir = temp_session_dir("oversized_count");
    let session_path = temp_dir.join("session.json");

    // Generate 45 edits
    let mut edits = Vec::new();
    for i in 1..=45 {
        edits.push(sample_edit(0, i, 1, &format!("x{}", i), (50 - i) as i64));
    }

    let session = make_session(
        "motions-01-word",
        0,
        edits,
        Some(Verdict::Failed {
            reason: "Violated forbid rule: backspace_run".to_string(),
        }),
    );
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).expect("load catalog");
    let config = ContextConfig::default();

    let context = extract_last_attempt_context(&session_path, Some(&catalog), &config)
        .expect("extract context");

    assert_eq!(context.total_edits, 45);
    assert_eq!(context.bounded_edits.len(), MAX_EDIT_OPERATIONS);
    assert_eq!(context.bounded_edits.len(), 20);
    assert_eq!(context.truncated_edits_count, 25);
    assert!(context.is_truncated);

    // Verify bounded edits are the *last* 20 edits (from i = 26 to i = 45)
    assert_eq!(context.bounded_edits.first().unwrap().inserted, "x26");
    assert_eq!(context.bounded_edits.last().unwrap().inserted, "x45");

    let summary = context.format_prompt_summary();
    assert!(summary.contains("25 older edits were omitted"));
    assert!(summary.contains("[26]"));
    assert!(summary.contains("[45]"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_extract_oversized_diff_capped_to_2kb() {
    let temp_dir = temp_session_dir("oversized_diff");
    let session_path = temp_dir.join("session.json");

    // Generate 5 edits with 600 bytes each -> 3000 bytes total (exceeds 2048)
    let large_payload = "A".repeat(600);
    let mut edits = Vec::new();
    for i in 1..=5 {
        edits.push(sample_edit(0, 0, 0, &large_payload, (10 - i) as i64));
    }

    let session = make_session("motions-01-word", 0, edits, Some(Verdict::InProgress));
    session.save_to_file(&session_path).unwrap();

    let config = ContextConfig::default();
    let context = extract_last_attempt_context(&session_path, None, &config)
        .expect("extract context without catalog");

    assert_eq!(context.total_edits, 5);
    assert!(context.total_diff_bytes <= MAX_DIFF_BYTES);
    assert!(context.bounded_edits.len() < 5);
    assert!(context.is_truncated);
    assert!(context.truncated_edits_count > 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_single_giant_edit_capped_to_2kb() {
    let temp_dir = temp_session_dir("giant_single_edit");
    let session_path = temp_dir.join("session.json");

    // Single edit with 5000 bytes
    let giant_payload = "Z".repeat(5000);
    let edits = vec![sample_edit(0, 0, 0, &giant_payload, 1)];

    let session = make_session("motions-01-word", 0, edits, Some(Verdict::InProgress));
    session.save_to_file(&session_path).unwrap();

    let config = ContextConfig::default();
    let context = extract_last_attempt_context(&session_path, None, &config).unwrap();

    assert_eq!(context.bounded_edits.len(), 1);
    assert_eq!(context.bounded_edits[0].inserted.len(), MAX_DIFF_BYTES);
    assert_eq!(context.total_diff_bytes, MAX_DIFF_BYTES);
    assert!(context.is_truncated);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_missing_session_file() {
    let non_existent = Path::new("non_existent_dir_12345/missing_session.json");
    let config = ContextConfig::default();

    let result = extract_last_attempt_context(non_existent, None, &config);
    assert!(matches!(
        result,
        Err(ContextError::MissingSessionFile { .. })
    ));

    let fake_provider = FakeProvider::new();
    let prov_res = fake_provider.explain_session(non_existent, None, &config);
    assert!(matches!(prov_res, Err(ProviderError::MissingSession(..))));
}

#[test]
fn test_no_active_attempt_in_session() {
    let temp_dir = temp_session_dir("no_attempt");
    let session_path = temp_dir.join("session.json");

    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.attempt = None;
    session.save_to_file(&session_path).unwrap();

    let config = ContextConfig::default();
    let result = extract_last_attempt_context(&session_path, None, &config);
    assert!(matches!(result, Err(ContextError::NoActiveAttempt)));

    let fake_provider = FakeProvider::new();
    let prov_res = fake_provider.explain_session(&session_path, None, &config);
    assert!(matches!(prov_res, Err(ProviderError::MissingSession(..))));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_empty_lesson_id_treated_as_no_active_attempt() {
    let mut session = SessionState::default();
    session.current.lesson_id = "".to_string();
    session.attempt = Some(Attempt {
        id: "att-1".to_string(),
        edit_count: 0,
        edits: Vec::new(),
        verdict: None,
    });

    let config = ContextConfig::default();
    let result = extract_from_session_state(&session, None, &config);
    assert!(matches!(result, Err(ContextError::NoActiveAttempt)));
}

#[test]
fn test_corrupt_session_file_explicit_error() {
    let temp_dir = temp_session_dir("corrupt");
    let session_path = temp_dir.join("session.json");

    // Write malformed JSON
    fs::write(&session_path, "{ broken_json: [").unwrap();

    let config = ContextConfig::default();
    let result = extract_last_attempt_context(&session_path, None, &config);
    assert!(matches!(
        result,
        Err(ContextError::CorruptSessionFile { .. })
    ));

    // Verify the corrupt file was preserved without modification or deletion
    assert_eq!(
        fs::read_to_string(&session_path).unwrap(),
        "{ broken_json: ["
    );

    let fake_provider = FakeProvider::new();
    let prov_res = fake_provider.explain_session(&session_path, None, &config);
    assert!(matches!(prov_res, Err(ProviderError::CorruptSession(..))));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_incompatible_schema_version_explicit_error() {
    let temp_dir = temp_session_dir("incompatible_version");
    let session_path = temp_dir.join("session.json");

    let mut session = make_session("motions-01-word", 0, Vec::new(), None);
    session.version = 999;
    let json = serde_json::to_string_pretty(&session).unwrap();
    fs::write(&session_path, json).unwrap();

    let config = ContextConfig::default();
    let result = extract_last_attempt_context(&session_path, None, &config);
    assert!(matches!(
        result,
        Err(ContextError::CorruptSessionFile { .. })
    ));

    let fake_provider = FakeProvider::new();
    let prov_res = fake_provider.explain_session(&session_path, None, &config);
    assert!(matches!(prov_res, Err(ProviderError::CorruptSession(..))));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_stale_old_attempt_detection() {
    let temp_dir = temp_session_dir("stale_attempt");
    let session_path = temp_dir.join("session.json");

    // Edits from 48 hours ago
    let edits = vec![sample_edit(0, 0, 1, "x", 48 * 60)];
    let session = make_session("motions-01-word", 0, edits, Some(Verdict::InProgress));
    session.save_to_file(&session_path).unwrap();

    // Set max staleness threshold to 1 hour
    let config = ContextConfig {
        max_edits: 20,
        max_diff_bytes: 2048,
        max_staleness: Some(Duration::hours(1)),
    };

    let result = extract_last_attempt_context(&session_path, None, &config);
    assert!(matches!(result, Err(ContextError::StaleAttempt { .. })));

    let fake_provider = FakeProvider::new();
    let prov_res = fake_provider.explain_session(&session_path, None, &config);
    assert!(matches!(prov_res, Err(ProviderError::StaleAttempt(..))));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_fake_provider_receives_context_and_explains_grounded() {
    let temp_dir = temp_session_dir("provider_e2e");
    let session_path = temp_dir.join("session.json");

    let edits = vec![sample_edit(0, 0, 0, "dw", 3), sample_edit(0, 2, 4, "", 2)];
    let session = make_session(
        "motions-01-word",
        0,
        edits,
        Some(Verdict::Failed {
            reason: "Violated forbid rule: backspace_run".to_string(),
        }),
    );
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).expect("load catalog");
    let config = ContextConfig::default();

    let fake_provider = FakeProvider::new();
    assert_eq!(fake_provider.received_count(), 0);

    let explanation = fake_provider
        .explain_session(&session_path, Some(&catalog), &config)
        .expect("provider explanation");

    // Provider recorded the exact context
    assert_eq!(fake_provider.received_count(), 1);
    let recorded = fake_provider.last_received_context().unwrap();
    assert_eq!(recorded.lesson_id, "motions-01-word");
    assert_eq!(recorded.attempt_id, "att-12345");
    assert_eq!(recorded.total_edits, 2);

    // Explanation separates observation from inference
    assert!(!explanation.observations.is_empty());
    assert!(!explanation.inferences.is_empty());
    assert!(explanation.is_bounded);

    // Observations are grounded in actual session events
    let obs_str = explanation.observations.join(" ");
    assert!(obs_str.contains("Attempt executed 2 edits in total"));
    assert!(obs_str.contains("Violated forbid rule: backspace_run"));
    assert!(obs_str.contains("Edit 1"));

    // Inferences assess technique without fabricating data
    let inf_str = explanation.inferences.join(" ");
    assert!(inf_str.contains("Excessive backspace usage detected"));

    // Actionable recommendation given
    assert!(explanation.recommendation.contains("dw"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_fake_provider_error_injection() {
    let fake_provider = FakeProvider::new();
    fake_provider.inject_error("Simulated rate limit / timeout");

    let context = AttemptContext {
        lesson_id: "test".to_string(),
        lesson_title: None,
        concepts: vec![],
        exercise_id: None,
        exercise_index: 0,
        task_prompt: None,
        par_edits: None,
        forbid_rules: None,
        attempt_id: "att-err".to_string(),
        total_edits: 0,
        bounded_edits: vec![],
        truncated_edits_count: 0,
        is_truncated: false,
        total_diff_bytes: 0,
        diff_summary: "".to_string(),
        verdict: AttemptVerdictContext::InProgress,
        age_seconds: 10,
    };

    let res = fake_provider.explain_attempt(&context);
    assert!(matches!(res, Err(ProviderError::Failure(msg)) if msg.contains("rate limit")));

    fake_provider.clear_injected_error();
    assert!(fake_provider.explain_attempt(&context).is_ok());
}
