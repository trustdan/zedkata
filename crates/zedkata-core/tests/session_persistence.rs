use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use zedkata_core::{
    default_session_path, AttemptOutcome, CoreError, CurrentSession, EditRange, HistoryEntry,
    RecordedEdit, SessionState, StudentInfo, Verdict, CURRENT_SCHEMA_VERSION,
};

fn create_temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("zedkata_test_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create temp test dir");
    dir
}

#[test]
fn test_roundtrip_all_session_fields() {
    let mut state = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: Utc::now(),
        },
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 1,
            buffer_path: Some("C:\\Users\\Dan\\zedkata\\test.kata".to_string()),
            opened_at: Some(Utc::now()),
            nav_mode: Some("normal".to_string()),
        },
        attempt: Some(zedkata_core::Attempt {
            id: "att_12345".to_string(),
            edit_count: 1,
            edits: vec![RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (1, 5),
                    end: (1, 10),
                },
                inserted: "".to_string(),
                deleted_len: 5,
            }],
            verdict: Some(Verdict::Passed {
                edits: 1,
                par: Some(1),
                score: 100,
            }),
        }),
        ..Default::default()
    };
    state.history.push(HistoryEntry {
        lesson_id: "motions-01-word".to_string(),
        exercise_id: "delete-word".to_string(),
        passed: true,
        par_edits: Some(1),
        actual_edits: 1,
        score: 100,
        completed_at: Utc::now(),
    });
    state.weaknesses.insert("word-motions".to_string(), 0.15);

    let json = state.to_json_pretty().expect("failed to serialize");
    let parsed = SessionState::from_json(&json).expect("failed to deserialize");

    assert_eq!(state.version, parsed.version);
    assert_eq!(state.current, parsed.current);
    assert_eq!(state.attempt, parsed.attempt);
    assert_eq!(state.history, parsed.history);
    assert_eq!(state.weaknesses, parsed.weaknesses);
}

#[test]
fn test_load_or_init_creates_default_when_absent() {
    let temp_dir = create_temp_dir("absent_file");
    let state_file = temp_dir.join("nonexistent_session.json");

    assert!(!state_file.exists());
    let state = SessionState::load_or_init(&state_file).expect("load_or_init should succeed");
    assert_eq!(state.version, CURRENT_SCHEMA_VERSION);
    assert_eq!(state.current.lesson_id, "");
    assert!(state.history.is_empty());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_load_from_file_missing_returns_io_error() {
    let temp_dir = create_temp_dir("missing_file");
    let state_file = temp_dir.join("missing.json");

    let res = SessionState::load_from_file(&state_file);
    match res {
        Err(CoreError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("Expected IoError(NotFound), got: {:?}", other),
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_corrupt_session_json_returns_corrupt_error() {
    let temp_dir = create_temp_dir("corrupt_json");
    let state_file = temp_dir.join("corrupt_session.json");
    fs::write(&state_file, "{ this is not valid json }").expect("failed to write corrupt file");

    let res = SessionState::load_from_file(&state_file);
    match res {
        Err(CoreError::SessionCorruptError { path, message }) => {
            assert_eq!(path, state_file);
            assert!(!message.is_empty());
        }
        other => panic!("Expected SessionCorruptError, got: {:?}", other),
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_incompatible_schema_version_rejected() {
    let temp_dir = create_temp_dir("incompatible_version");
    let state_file = temp_dir.join("future_session.json");

    let future_state = SessionState {
        version: 999,
        ..Default::default()
    };
    let json = serde_json::to_string(&future_state).expect("serialize future state");
    fs::write(&state_file, json).expect("failed to write future file");

    let res = SessionState::load_from_file(&state_file);
    match res {
        Err(CoreError::IncompatibleSessionVersion { found, supported }) => {
            assert_eq!(found, 999);
            assert_eq!(supported, CURRENT_SCHEMA_VERSION);
        }
        other => panic!("Expected IncompatibleSessionVersion, got: {:?}", other),
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_atomic_file_replacement_and_persistence() {
    let temp_dir = create_temp_dir("atomic_replacement");
    let nested_dir = temp_dir.join("nested").join("folder");
    let state_file = nested_dir.join("session.json");

    let mut state = SessionState::default();
    state.current.lesson_id = "initial-lesson".to_string();

    // 1. Initial write creates parent directories and file
    state
        .save_to_file(&state_file)
        .expect("initial save should succeed");
    assert!(state_file.exists());

    let loaded = SessionState::load_from_file(&state_file).expect("load initial");
    assert_eq!(loaded.current.lesson_id, "initial-lesson");

    // 2. Subsequent write safely replaces target
    state.current.lesson_id = "updated-lesson".to_string();
    state
        .save_to_file(&state_file)
        .expect("update save should succeed");

    let reloaded = SessionState::load_from_file(&state_file).expect("load updated");
    assert_eq!(reloaded.current.lesson_id, "updated-lesson");

    // 3. Verify no lingering temporary files
    let entries: Vec<_> = fs::read_dir(&nested_dir)
        .expect("read nested dir")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_name(), "session.json");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_default_session_path_resolution() {
    // 1. Test override via ZEDKATA_SESSION_PATH
    std::env::set_var("ZEDKATA_SESSION_PATH", "C:\\temp\\override_session.json");
    let resolved = default_session_path().expect("resolved override");
    assert_eq!(resolved, PathBuf::from("C:\\temp\\override_session.json"));
    std::env::remove_var("ZEDKATA_SESSION_PATH");

    // 2. Test standard platform path
    let standard = default_session_path().expect("resolved standard path");
    assert!(standard.ends_with("session.json"));
}

#[test]
fn test_attempt_lifecycle_start_edit_finalize() {
    let mut state = SessionState::default();
    state.current.lesson_id = "motions-01-word".to_string();

    // 1. Start attempt
    state.start_attempt(
        "att_001".to_string(),
        Some("C:\\test\\file.kata".to_string()),
    );
    assert!(state.attempt.is_some());
    let att = state.attempt.as_ref().unwrap();
    assert_eq!(att.id, "att_001");
    assert_eq!(att.edit_count, 0);
    assert_eq!(att.verdict, Some(Verdict::InProgress));

    // 2. Record edit
    let edit = RecordedEdit {
        at: Utc::now(),
        range: EditRange {
            start: (1, 10),
            end: (1, 16),
        },
        inserted: "".to_string(),
        deleted_len: 6,
    };
    assert!(state.record_edit("att_001", edit));
    assert_eq!(state.attempt.as_ref().unwrap().edit_count, 1);

    // 3. Reject stale attempt ID edit
    let stale_edit = RecordedEdit {
        at: Utc::now(),
        range: EditRange {
            start: (1, 0),
            end: (1, 1),
        },
        inserted: "".to_string(),
        deleted_len: 1,
    };
    assert!(!state.record_edit("stale_old_id", stale_edit));
    assert_eq!(state.attempt.as_ref().unwrap().edit_count, 1);

    // 4. Finalize attempt as Passed
    let concepts = vec!["word-motions".to_string()];
    let outcome = AttemptOutcome::Passed {
        edits: 1,
        par: Some(1),
        score: 100,
    };
    assert!(state.finalize_attempt("att_001", &outcome, "delete-word", &concepts));

    // Verdict is Passed
    match state.attempt.as_ref().unwrap().verdict.as_ref() {
        Some(Verdict::Passed { edits, score, .. }) => {
            assert_eq!(*edits, 1);
            assert_eq!(*score, 100);
        }
        other => panic!("Expected Verdict::Passed, got: {:?}", other),
    }

    // History and weaknesses updated
    assert_eq!(state.history.len(), 1);
    assert_eq!(state.history[0].exercise_id, "delete-word");
    assert_eq!(state.history[0].score, 100);
    assert_eq!(state.weaknesses.get("word-motions"), Some(&0.0));

    // 5. Late edit after finalization rejected
    let late_edit = RecordedEdit {
        at: Utc::now(),
        range: EditRange {
            start: (1, 0),
            end: (1, 1),
        },
        inserted: "".to_string(),
        deleted_len: 1,
    };
    assert!(!state.record_edit("att_001", late_edit));

    // 6. Duplicate finalization prevented
    assert!(!state.finalize_attempt("att_001", &outcome, "delete-word", &concepts));
    assert_eq!(state.history.len(), 1); // no duplicate
}

#[test]
fn test_attempt_lifecycle_abandon_and_retry() {
    let mut state = SessionState::default();
    state.start_attempt("att_001".to_string(), None);

    // Abandon active attempt
    assert!(state.abandon_attempt("att_001"));
    match state.attempt.as_ref().unwrap().verdict.as_ref() {
        Some(Verdict::Failed { ref reason }) => {
            assert!(reason.contains("abandoned"));
        }
        other => panic!("Expected Verdict::Failed, got: {:?}", other),
    }

    // Retry with new attempt ID
    state.retry_attempt("att_002".to_string());
    let att = state.attempt.as_ref().unwrap();
    assert_eq!(att.id, "att_002");
    assert_eq!(att.edit_count, 0);
    assert_eq!(att.verdict, Some(Verdict::InProgress));

    // Stale edit from att_001 rejected on active att_002
    let edit = RecordedEdit {
        at: Utc::now(),
        range: EditRange {
            start: (1, 0),
            end: (1, 1),
        },
        inserted: "a".to_string(),
        deleted_len: 0,
    };
    assert!(!state.record_edit("att_001", edit.clone()));
    assert!(state.record_edit("att_002", edit));
    assert_eq!(state.attempt.as_ref().unwrap().edit_count, 1);
}

#[test]
fn test_switch_exercise_clears_attempt() {
    let mut state = SessionState::default();
    state.start_attempt("att_001".to_string(), Some("path.kata".to_string()));
    assert!(state.attempt.is_some());

    state.switch_exercise("motions-02-find".to_string(), 0);
    assert_eq!(state.current.lesson_id, "motions-02-find");
    assert_eq!(state.current.exercise_index, 0);
    assert!(state.attempt.is_none());
    assert!(state.current.buffer_path.is_none());
}
