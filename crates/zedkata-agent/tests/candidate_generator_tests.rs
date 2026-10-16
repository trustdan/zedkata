use std::collections::HashMap;
use std::sync::Arc;
use zedkata_agent::{
    extract_weakness_context, AgentConfig, CandidateExercise, CandidateStatus, ConfiguredProvider,
    ExerciseGenerator, FakeProvider, ProviderError, ProviderKind, WeaknessContext,
    DEFAULT_FALLBACK_CONCEPT,
};
use zedkata_core::{
    lesson::Lesson,
    session::{HistoryEntry, SessionState, StudentInfo, CURRENT_SCHEMA_VERSION},
};

#[test]
fn test_weakness_context_extraction_with_scored_weaknesses() {
    let mut weaknesses = HashMap::new();
    weaknesses.insert("ci".to_string(), 0.35);
    weaknesses.insert("dw".to_string(), 0.85);
    weaknesses.insert("w".to_string(), 0.10);

    let session = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: chrono::Utc::now(),
        },
        current: zedkata_core::session::CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        attempt: None,
        history: vec![HistoryEntry {
            lesson_id: "motions-01-word".to_string(),
            exercise_id: "delete-word-arg".to_string(),
            passed: false,
            par_edits: Some(1),
            actual_edits: 5,
            score: 0,
            completed_at: chrono::Utc::now(),
        }],
        weaknesses,
        completed_walkthroughs: Vec::new(),
    };

    let ctx = WeaknessContext::from_session(&session, None);

    // Highest severity weakness is selected
    assert_eq!(ctx.target_concept, "dw");
    assert_eq!(ctx.severity, Some(0.85));
    assert!(!ctx.is_unscored_or_clean);

    // All active weaknesses sorted descending
    assert_eq!(ctx.all_weaknesses.len(), 3);
    assert_eq!(ctx.all_weaknesses[0].0, "dw");
    assert_eq!(ctx.all_weaknesses[1].0, "ci");
    assert_eq!(ctx.all_weaknesses[2].0, "w");

    // History failure note is captured
    assert_eq!(ctx.recent_failures.len(), 1);
    assert!(ctx.recent_failures[0].contains("delete-word-arg"));

    // Formatted summary includes details
    let summary = ctx.format_prompt_summary();
    assert!(summary.contains("**Target Concept**: `dw`"));
    assert!(summary.contains("**Severity**: 0.85"));
    assert!(summary.contains("delete-word-arg"));
}

#[test]
fn test_weakness_context_missing_or_clean_follows_contract() {
    // 1. Session with clean / empty weaknesses
    let session = SessionState::default();
    assert!(session.weaknesses.is_empty());

    let ctx = WeaknessContext::from_session(&session, None);
    assert_eq!(ctx.target_concept, DEFAULT_FALLBACK_CONCEPT);
    assert!(ctx.severity.is_none());
    assert!(ctx.is_unscored_or_clean);
    assert!(ctx.all_weaknesses.is_empty());

    // 2. Custom fallback concept requested
    let ctx_custom = WeaknessContext::from_session(&session, Some("modal-00-baseline"));
    assert_eq!(ctx_custom.target_concept, "modal-00-baseline");
    assert!(ctx_custom.severity.is_none());
    assert!(ctx_custom.is_unscored_or_clean);

    // 3. Non-existent file path gracefully follows contract
    let missing_path = std::env::temp_dir().join(format!(
        "zedkata_missing_session_{}_{}.json",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let ctx_missing = extract_weakness_context(&missing_path, None);
    assert_eq!(ctx_missing.target_concept, DEFAULT_FALLBACK_CONCEPT);
    assert!(ctx_missing.severity.is_none());
    assert!(ctx_missing.is_unscored_or_clean);
}

#[test]
fn test_candidate_exercise_generation_structure_and_fixtures() {
    let mut weaknesses = HashMap::new();
    weaknesses.insert("dw".to_string(), 0.75);

    let session = SessionState {
        weaknesses,
        ..Default::default()
    };

    let ctx = WeaknessContext::from_session(&session, None);
    let fake_provider = FakeProvider::new();

    let candidate = fake_provider.generate_candidate(&ctx).unwrap();

    // Verify candidate lifecycle status: must remain a candidate until validation passes
    assert!(candidate.is_candidate());
    assert_eq!(candidate.status, CandidateStatus::Candidate);

    // Verify lesson metadata
    assert!(
        candidate.lesson_id.starts_with("gen-dw-"),
        "Lesson ID must follow gen-<concept>-<timestamp> pattern: {}",
        candidate.lesson_id
    );
    assert!(candidate.title.contains("Delete word (dw)"));
    assert_eq!(candidate.concepts, vec!["dw".to_string()]);
    assert!(!candidate.draft);
    assert!(!candidate.prose.is_empty());
    assert!(!candidate.task.is_empty());

    // Verify both fixtures
    assert!(
        candidate
            .start_fixture_relative_path
            .starts_with(&format!("fixtures/{}/", candidate.lesson_id)),
        "Start fixture path must be inside lesson subfolder: {}",
        candidate.start_fixture_relative_path
    );
    assert!(candidate
        .start_fixture_relative_path
        .ends_with(".start.kata"));
    assert!(!candidate.start_fixture_content.is_empty());

    assert!(
        candidate
            .goal_fixture_relative_path
            .starts_with(&format!("fixtures/{}/", candidate.lesson_id)),
        "Goal fixture path must be inside lesson subfolder: {}",
        candidate.goal_fixture_relative_path
    );
    assert!(candidate.goal_fixture_relative_path.ends_with(".goal.kata"));
    assert!(!candidate.goal_fixture_content.is_empty());

    // Both fixture buffers must differ
    assert_ne!(
        candidate.start_fixture_content, candidate.goal_fixture_content,
        "Start and goal buffers must differ by the target motion"
    );

    // Check motion ergonomics: Par 1, 1-based cursor
    assert_eq!(candidate.par_edits, Some(1));
    assert_eq!(candidate.max_edits, Some(2));
    let cursor = candidate.cursor.expect("Cursor must be defined");
    assert!(cursor[0] >= 1, "Line must be 1-based");
    assert!(cursor[1] >= 1, "Col must be 1-based");
    assert_eq!(candidate.forbid_backspace_run, Some(3));

    // Recorded context check
    assert_eq!(fake_provider.recorded_weakness_contexts().len(), 1);
    assert_eq!(
        fake_provider
            .last_received_weakness_context()
            .unwrap()
            .target_concept,
        "dw"
    );
}

#[test]
fn test_candidate_exercise_raw_toml_validity_with_core_lesson_parser() {
    let mut weaknesses = HashMap::new();
    weaknesses.insert("ci".to_string(), 0.90);

    let session = SessionState {
        weaknesses,
        ..Default::default()
    };

    let ctx = WeaknessContext::from_session(&session, None);
    let fake_provider = FakeProvider::new();

    let candidate = fake_provider.generate_candidate(&ctx).unwrap();

    // Validate generated TOML parses cleanly using zedkata-core::Lesson
    let parsed_lesson =
        Lesson::from_toml(&candidate.raw_toml, None).expect("Raw TOML must parse cleanly");

    assert_eq!(parsed_lesson.id, candidate.lesson_id);
    assert_eq!(parsed_lesson.title, candidate.title);
    assert_eq!(parsed_lesson.concepts, candidate.concepts);
    assert_eq!(parsed_lesson.exercises.len(), 1);

    let ex = &parsed_lesson.exercises[0];
    assert_eq!(ex.id, candidate.exercise_id);
    assert_eq!(ex.task, candidate.task);
    assert_eq!(ex.start, candidate.start_fixture_relative_path);
    assert_eq!(ex.goal, candidate.goal_fixture_relative_path);
    assert_eq!(ex.par_edits, candidate.par_edits);
    assert_eq!(ex.max_edits, candidate.max_edits);
    assert_eq!(ex.cursor, candidate.cursor);

    let forbid = ex.forbid.as_ref().expect("Forbid rules must be present");
    assert_eq!(forbid.backspace_run, candidate.forbid_backspace_run);
}

#[test]
fn test_candidate_generation_with_missing_unscored_weakness() {
    // When student record is clean / unscored, candidate is still generated successfully for fallback
    let ctx = WeaknessContext {
        target_concept: DEFAULT_FALLBACK_CONCEPT.to_string(),
        severity: None,
        all_weaknesses: Vec::new(),
        is_unscored_or_clean: true,
        recent_failures: Vec::new(),
        session_path: None,
    };

    let fake_provider = FakeProvider::new();
    let candidate = fake_provider.generate_candidate(&ctx).unwrap();

    assert!(candidate.is_candidate());
    assert!(candidate.lesson_id.starts_with("gen-word-motions-"));
    assert_eq!(
        candidate.concepts,
        vec![DEFAULT_FALLBACK_CONCEPT.to_string()]
    );
    assert_ne!(
        candidate.start_fixture_content,
        candidate.goal_fixture_content
    );

    // Verify valid core TOML
    Lesson::from_toml(&candidate.raw_toml, None)
        .expect("Fallback candidate TOML must parse cleanly");
}

#[test]
fn test_parse_candidate_response_markdown_fences_and_slugification() {
    let ctx = WeaknessContext {
        target_concept: "dw".to_string(),
        severity: Some(0.6),
        all_weaknesses: vec![("dw".to_string(), 0.6)],
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    let raw_llm_json = r#"```json
{
  "title": "Practice: Parameter Elimination",
  "prose": "Eliminate redundant parameters cleanly.",
  "exercise_id": "Delete Redundant Parameter #1",
  "task": "Delete the first argument 'param' on line 2.",
  "start_content": "fn test(param: u32, ok: bool) {}\n",
  "goal_content": "fn test(ok: bool) {}\n",
  "par_edits": 1,
  "max_edits": 2,
  "cursor": [2, 9],
  "hint": "Use dw in Normal mode.",
  "forbid_backspace_run": 3
}
```"#;

    let candidate =
        zedkata_agent::parse_candidate_response(raw_llm_json, &ctx, Some(1729000000)).unwrap();

    assert_eq!(candidate.lesson_id, "gen-dw-1729000000");
    // exercise_id slugified properly (lowercase ascii with hyphens)
    assert_eq!(candidate.exercise_id, "delete-redundant-parameter-1");
    assert_eq!(
        candidate.start_fixture_relative_path,
        "fixtures/gen-dw-1729000000/delete-redundant-parameter-1.start.kata"
    );
    assert_eq!(
        candidate.goal_fixture_relative_path,
        "fixtures/gen-dw-1729000000/delete-redundant-parameter-1.goal.kata"
    );
    assert_eq!(candidate.cursor, Some([2, 9]));
    assert!(candidate.is_candidate());

    // Validates against core schema
    Lesson::from_toml(&candidate.raw_toml, None).expect("Parsed candidate must parse into Lesson");
}

#[test]
fn test_parse_candidate_response_malformed_returns_error() {
    let ctx = WeaknessContext {
        target_concept: "dw".to_string(),
        severity: Some(0.5),
        all_weaknesses: Vec::new(),
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    let malformed_raw = "I think you should practice 'dw'. Here is what to do: ... not json ...";
    let res = zedkata_agent::parse_candidate_response(malformed_raw, &ctx, None);

    match res {
        Err(ProviderError::MalformedResponse(msg)) => {
            assert!(msg.contains("Failed to deserialize candidate exercise JSON"));
        }
        other => panic!("Expected MalformedResponse, got {:?}", other),
    }
}

#[tokio::test]
async fn test_configured_provider_async_generation_and_timeout() {
    let config = AgentConfig {
        provider: ProviderKind::Fake,
        timeout_seconds: 1,
        ..Default::default()
    };

    let fake = FakeProvider::new();
    fake.set_async_delay(std::time::Duration::from_millis(1500));

    let provider = ConfiguredProvider::with_generator(config, Arc::new(fake));

    let ctx = WeaknessContext {
        target_concept: "dw".to_string(),
        severity: Some(0.8),
        all_weaknesses: Vec::new(),
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    let res = provider.generate_candidate_async(&ctx, None).await;

    match res {
        Err(ProviderError::Timeout { seconds, message }) => {
            assert_eq!(seconds, 1);
            assert!(message.contains("timed out after 1s"));
        }
        other => panic!("Expected Timeout error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_configured_provider_async_generation_cancellation() {
    let config = AgentConfig {
        provider: ProviderKind::Fake,
        timeout_seconds: 10,
        ..Default::default()
    };

    let fake = FakeProvider::new();
    fake.set_async_delay(std::time::Duration::from_millis(500));

    let provider = ConfiguredProvider::with_generator(config, Arc::new(fake));

    let ctx = WeaknessContext {
        target_concept: "ci".to_string(),
        severity: Some(0.5),
        all_weaknesses: Vec::new(),
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);

    // Cancel after 50ms while task is sleeping 500ms
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = cancel_tx.send(true);
    });

    let res = provider
        .generate_candidate_async(&ctx, Some(cancel_rx))
        .await;

    match res {
        Err(ProviderError::Cancelled(msg)) => {
            assert!(msg.contains("Operation cancelled"));
        }
        other => panic!("Expected Cancelled error, got {:?}", other),
    }
}

#[test]
fn test_fake_provider_with_canned_candidate() {
    let canned = CandidateExercise {
        lesson_id: "gen-custom-12345".to_string(),
        title: "Canned Practice".to_string(),
        module: None,
        requires: Vec::new(),
        concepts: vec!["custom".to_string()],
        draft: false,
        prose: "Canned prose".to_string(),
        exercise_id: "canned-drill".to_string(),
        task: "Canned task".to_string(),
        start_fixture_relative_path: "fixtures/gen-custom-12345/canned.start.kata".to_string(),
        start_fixture_content: "start".to_string(),
        goal_fixture_relative_path: "fixtures/gen-custom-12345/canned.goal.kata".to_string(),
        goal_fixture_content: "goal".to_string(),
        par_edits: Some(1),
        max_edits: Some(2),
        cursor: Some([1, 1]),
        hint: None,
        forbid_backspace_run: Some(3),
        raw_toml: "canned toml".to_string(),
        status: CandidateStatus::Candidate,
    };

    let fake = FakeProvider::with_canned_candidate(canned.clone());

    let ctx = WeaknessContext {
        target_concept: "custom".to_string(),
        severity: Some(0.4),
        all_weaknesses: Vec::new(),
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    let result = fake.generate_candidate(&ctx).unwrap();
    assert_eq!(result, canned);
}
