use anyhow::Result;
use chrono::Utc;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zedkata_core::{
    catalog::Catalog,
    session::{Attempt, CurrentSession, EditRange, RecordedEdit, SessionState, Verdict},
};
use zedkata_tui::{app::App, event::handle_event, launcher::reset_exercise_attempt, ui};

fn setup_temp_workspace() -> (PathBuf, PathBuf) {
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_retry_test_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_file = temp_dir.join("session.json");
    (temp_dir, session_file)
}

fn cleanup_temp_workspace(temp_dir: &Path) {
    let _ = fs::remove_dir_all(temp_dir);
}

fn make_key_event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn dummy_edit() -> RecordedEdit {
    RecordedEdit {
        at: Utc::now(),
        range: EditRange {
            start: (0, 0),
            end: (0, 0),
        },
        inserted: "x".to_string(),
        deleted_len: 0,
    }
}

#[test]
fn test_reset_issues_fresh_attempt_and_clears_verdict() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");
    let catalog = Catalog::load_from_dir(&lessons_dir)?;

    let initial_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_old_failed".to_string(),
            edits: vec![dummy_edit()],
            edit_count: 5,
            verdict: Some(Verdict::Failed {
                reason: "Budget exceeded".to_string(),
            }),
        }),
        ..Default::default()
    };
    initial_session.save_to_file(&session_file)?;

    let result = reset_exercise_attempt(&session_file, &catalog, None)?;
    assert_ne!(result.attempt_id, "att_old_failed");
    assert_eq!(result.exercise_id, "delete-word-arg");

    let post_session = SessionState::load_from_file(&session_file)?;
    let att = post_session.attempt.expect("attempt should exist");
    assert_eq!(att.id, result.attempt_id);
    assert_eq!(att.edit_count, 0);
    assert!(att.edits.is_empty());
    assert_eq!(
        att.verdict,
        Some(Verdict::InProgress),
        "verdict must be reset to InProgress upon retry"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_reset_restores_dirty_buffer_to_baseline() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");
    let catalog = Catalog::load_from_dir(&lessons_dir)?;

    let initial_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        ..Default::default()
    };
    initial_session.save_to_file(&session_file)?;

    // Initial reset creates the buffer from the start fixture
    let result = reset_exercise_attempt(&session_file, &catalog, None)?;
    let start_fixture_path = catalog.resolve_fixture("motions-01-word", "delete-word-arg", true)?;
    let baseline_content = fs::read_to_string(&start_fixture_path)?;

    // Mutate the buffer file on disk with dirty/corrupt text
    fs::write(&result.buffer_path, "DIRTY MUTATED TEXT GARBAGE")?;
    assert_ne!(fs::read_to_string(&result.buffer_path)?, baseline_content);

    // Call reset again
    let result2 = reset_exercise_attempt(&session_file, &catalog, None)?;
    assert_eq!(result.buffer_path, result2.buffer_path);

    // Buffer file on disk MUST match baseline content exactly!
    let restored_content = fs::read_to_string(&result2.buffer_path)?;
    assert_eq!(
        restored_content, baseline_content,
        "Buffer file on disk must be restored to baseline fixture"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_late_updates_from_pre_retry_attempt_are_rejected() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");
    let catalog = Catalog::load_from_dir(&lessons_dir)?;

    let initial_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        ..Default::default()
    };
    initial_session.save_to_file(&session_file)?;

    let result1 = reset_exercise_attempt(&session_file, &catalog, None)?;
    let old_attempt_id = result1.attempt_id;

    // Trigger retry, getting new attempt ID
    let result2 = reset_exercise_attempt(&session_file, &catalog, None)?;
    let new_attempt_id = result2.attempt_id;
    assert_ne!(old_attempt_id, new_attempt_id);

    // Load session and attempt to record a late update targeting old_attempt_id
    let mut session = SessionState::load_from_file(&session_file)?;
    let accepted = session.record_edit(&old_attempt_id, dummy_edit());
    assert!(
        !accepted,
        "Late edit with old attempt ID must be rejected by session state"
    );

    let att = session.attempt.unwrap();
    assert_eq!(att.id, new_attempt_id);
    assert_eq!(att.edit_count, 0);

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_tui_retry_key_r_and_reload_key_r_upper() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");
    let catalog = Catalog::load_from_dir(&lessons_dir)?;

    let initial_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_tui_retry_test".to_string(),
            edits: vec![dummy_edit()],
            edit_count: 3,
            verdict: Some(Verdict::Failed {
                reason: "Budget exceeded".to_string(),
            }),
        }),
        ..Default::default()
    };
    initial_session.save_to_file(&session_file)?;

    let mut app = App::load(session_file.clone(), lessons_dir);

    // Initial buffer creation
    let result = reset_exercise_attempt(&session_file, &catalog, None)?;
    fs::write(&result.buffer_path, "DIRTY EDITS IN BUFFER")?;

    // Press 'r' in TUI
    handle_event(&mut app, make_key_event(KeyCode::Char('r')))?;

    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .contains("Reset attempt"));
    assert_eq!(app.session.attempt.as_ref().unwrap().edit_count, 0);
    assert_eq!(
        app.session.attempt.as_ref().unwrap().verdict,
        Some(Verdict::InProgress)
    );

    // Buffer file on disk should be clean
    let start_fixture_path = catalog.resolve_fixture("motions-01-word", "delete-word-arg", true)?;
    let baseline_content = fs::read_to_string(&start_fixture_path)?;
    assert_eq!(fs::read_to_string(&result.buffer_path)?, baseline_content);

    // Render frame to verify UI renders clean attempt state without panic
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // Press 'R' in TUI (Reload)
    handle_event(&mut app, make_key_event(KeyCode::Char('R')))?;
    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .contains("Refreshed catalog"));

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
