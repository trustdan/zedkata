use anyhow::Result;
use chrono::Utc;
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};
use zedkata_core::session::{
    Attempt, EditRange, HistoryEntry, RecordedEdit, SessionState, Verdict,
};
use zedkata_tui::{app::App, ui};

fn setup_temp_workspace() -> (PathBuf, PathBuf) {
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_live_state_test_{}_{}",
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
fn test_controlled_session_updates_and_progress_render() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    // 1. Initialize session with an active lesson and attempt
    let mut initial_session = SessionState::default();
    initial_session.switch_exercise("motions-01-word".to_string(), 0);
    initial_session.start_attempt("attempt_test_1".to_string(), None);
    initial_session.record_edit("attempt_test_1", dummy_edit());
    initial_session.save_to_file(&session_file)?;

    let mut app = App::load(session_file.clone(), lessons_dir);
    app.poll_interval = Duration::from_millis(10);
    assert_eq!(app.session.attempt.as_ref().map(|a| a.edit_count), Some(1));
    assert!(app.session_warning.is_none());

    // Render to TestBackend to verify initial attempt feedback
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // 2. Simulate LSP updating attempt with second edit and InProgress verdict
    sleep(Duration::from_millis(25));
    let mut updated_session = SessionState::load_from_file(&session_file)?;
    updated_session.record_edit("attempt_test_1", dummy_edit());
    if let Some(ref mut att) = updated_session.attempt {
        att.verdict = Some(Verdict::InProgress);
    }
    updated_session.save_to_file(&session_file)?;

    // Poll session in App
    let updated = app.poll_session();
    assert!(updated, "poll_session should detect modification");
    assert_eq!(app.session.attempt.as_ref().map(|a| a.edit_count), Some(2));
    assert_eq!(
        app.session.attempt.as_ref().and_then(|a| a.verdict.clone()),
        Some(Verdict::InProgress)
    );

    // 3. Simulate LSP passing the exercise attempt and adding to history
    sleep(Duration::from_millis(25));
    let mut passed_session = SessionState::load_from_file(&session_file)?;
    if let Some(ref mut att) = passed_session.attempt {
        att.verdict = Some(Verdict::Passed {
            edits: 2,
            par: Some(2),
            score: 100,
        });
    }
    passed_session.history.push(HistoryEntry {
        lesson_id: "motions-01-word".to_string(),
        exercise_id: "ex-01".to_string(),
        passed: true,
        par_edits: Some(2),
        actual_edits: 2,
        score: 100,
        completed_at: chrono::Utc::now(),
    });
    passed_session.save_to_file(&session_file)?;

    let updated2 = app.poll_session();
    assert!(updated2, "poll_session should detect passed state");
    assert_eq!(
        app.session.attempt.as_ref().and_then(|a| a.verdict.clone()),
        Some(Verdict::Passed {
            edits: 2,
            par: Some(2),
            score: 100
        })
    );
    assert_eq!(app.session.history.len(), 1);

    // Render frame to verify successful attempt rendering without crash
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_missing_session_file_does_not_crash_or_erase_state() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    // Do NOT write session_file to disk
    assert!(!session_file.exists());

    let mut app = App::load(session_file.clone(), lessons_dir);
    app.poll_interval = Duration::from_millis(10);
    assert!(app.session_warning.is_none());

    // Polling nonexistent file should safely return false
    let updated = app.poll_session();
    assert!(!updated);
    assert!(app.session_warning.is_none());

    // Rendering should succeed
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_corrupt_session_file_retains_previous_state_and_recovers() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    // 1. Initial valid session
    let session = SessionState {
        student: zedkata_core::session::StudentInfo {
            started_at: Utc::now(),
        },
        attempt: Some(Attempt {
            id: "valid_attempt_01".to_string(),
            edits: vec![],
            edit_count: 5,
            verdict: Some(Verdict::InProgress),
        }),
        ..Default::default()
    };
    session.save_to_file(&session_file)?;

    let mut app = App::load(session_file.clone(), lessons_dir);
    app.poll_interval = Duration::from_millis(10);
    assert_eq!(
        app.session.attempt.as_ref().map(|a| a.id.as_str()),
        Some("valid_attempt_01")
    );
    assert!(app.session_warning.is_none());

    // 2. Overwrite file with corrupt JSON
    sleep(Duration::from_millis(25));
    fs::write(&session_file, b"{\"broken_json\": [incomplete")?;

    let updated = app.poll_session();
    assert!(!updated, "corrupt state should not report success");
    assert!(
        app.session_warning.is_some(),
        "warning should be set on corrupt file"
    );
    // Prior valid in-memory state MUST be preserved!
    assert_eq!(
        app.session.attempt.as_ref().map(|a| a.id.as_str()),
        Some("valid_attempt_01")
    );
    assert_eq!(app.session.attempt.as_ref().map(|a| a.edit_count), Some(5));

    // Render frame to verify warning displays without crash
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // 3. Self-healing: overwrite with valid recovered JSON
    sleep(Duration::from_millis(25));
    let healed_session = SessionState {
        attempt: Some(Attempt {
            id: "recovered_attempt_02".to_string(),
            edits: vec![],
            edit_count: 6,
            verdict: Some(Verdict::Passed {
                edits: 6,
                par: Some(6),
                score: 100,
            }),
        }),
        ..Default::default()
    };
    healed_session.save_to_file(&session_file)?;

    let recovered = app.poll_session();
    assert!(recovered, "recovered state should report success");
    assert!(
        app.session_warning.is_none(),
        "warning should be cleared upon valid recovery"
    );
    assert_eq!(
        app.session.attempt.as_ref().map(|a| a.id.as_str()),
        Some("recovered_attempt_02")
    );

    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_bounded_polling_throttles_frequency() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let session = SessionState::default();
    session.save_to_file(&session_file)?;

    let mut app = App::load(session_file.clone(), lessons_dir);
    // Set a large interval
    app.poll_interval = Duration::from_secs(60);

    // Immediate second poll without modifying or forcing
    let polled = app.poll_session();
    assert!(
        !polled,
        "Second immediate poll should be throttled by interval"
    );

    // Forced poll bypasses interval
    let forced = app.force_poll_session();
    assert!(forced, "Forced poll should reload regardless of interval");

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
