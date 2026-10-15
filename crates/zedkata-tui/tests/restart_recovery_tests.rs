use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zedkata_core::session::{
    Attempt, CurrentSession, HistoryEntry, SessionState, Verdict, CURRENT_SCHEMA_VERSION,
};
use zedkata_tui::{
    app::{App, ConnectionStatus, FixtureStatus},
    event::handle_event,
    ui,
};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn setup_temp_workspace() -> (PathBuf, PathBuf) {
    let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_restart_test_{}_{}_{}",
        std::process::id(),
        nonce,
        count
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

#[test]
fn test_manual_restart_restores_completion_and_unlocks() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = temp_dir.join("lessons");
    let fixtures_dir = lessons_dir.join("fixtures");
    fs::create_dir_all(&fixtures_dir)?;

    let f1_start = fixtures_dir.join("ex1.start.kata");
    let f1_goal = fixtures_dir.join("ex1.goal.kata");
    let f2_start = fixtures_dir.join("ex2.start.kata");
    let f2_goal = fixtures_dir.join("ex2.goal.kata");
    fs::write(&f1_start, "a")?;
    fs::write(&f1_goal, "b")?;
    fs::write(&f2_start, "c")?;
    fs::write(&f2_goal, "d")?;

    let l1_toml = r#"
[lesson]
id = "lesson-01"
title = "Lesson 1"
requires = []
concepts = ["test"]
prose = "First lesson"

[[exercise]]
id = "ex-01"
task = "Do 1"
start = "fixtures/ex1.start.kata"
goal = "fixtures/ex1.goal.kata"
par_edits = 1
max_edits = 2

[[exercise]]
id = "ex-02"
task = "Do 2"
start = "fixtures/ex2.start.kata"
goal = "fixtures/ex2.goal.kata"
par_edits = 1
max_edits = 2
"#;
    fs::write(lessons_dir.join("lesson-01.toml"), l1_toml)?;

    let l2_toml = r#"
[lesson]
id = "lesson-02"
title = "Lesson 2"
requires = ["lesson-01"]
concepts = ["advanced"]
prose = "Second lesson"

[[exercise]]
id = "ex-01"
task = "Do 2.1"
start = "fixtures/ex1.start.kata"
goal = "fixtures/ex1.goal.kata"
par_edits = 1
max_edits = 2
"#;
    fs::write(lessons_dir.join("lesson-02.toml"), l2_toml)?;

    // 1. Setup session with lesson-01 fully completed in history
    let session = SessionState {
        current: CurrentSession {
            lesson_id: "lesson-02".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        history: vec![
            HistoryEntry {
                lesson_id: "lesson-01".to_string(),
                exercise_id: "ex-01".to_string(),
                passed: true,
                par_edits: Some(1),
                actual_edits: 1,
                score: 100,
                completed_at: chrono::Utc::now(),
            },
            HistoryEntry {
                lesson_id: "lesson-01".to_string(),
                exercise_id: "ex-02".to_string(),
                passed: true,
                par_edits: Some(1),
                actual_edits: 1,
                score: 100,
                completed_at: chrono::Utc::now(),
            },
        ],
        weaknesses: [("test".to_string(), 0.15)].into_iter().collect(),
        ..Default::default()
    };
    session.save_to_file(&session_file)?;

    // 2. Simulate application restart
    let app = App::load(session_file.clone(), lessons_dir.clone());

    // Lesson lesson-01 should be completed
    let lessons = app.lessons();
    let m1 = lessons
        .iter()
        .find(|l| l.lesson.id == "lesson-01")
        .expect("lesson-01");
    assert!(app.is_lesson_completed(&m1.lesson));

    // lesson-02 should be unlocked because its requirement (lesson-01) is completed
    let m2 = lessons
        .iter()
        .find(|l| l.lesson.id == "lesson-02")
        .expect("lesson-02");
    let (is_locked, missing) = app.is_lesson_locked(&m2.lesson);
    assert!(!is_locked, "Expected lesson-02 to be unlocked");
    assert!(missing.is_empty());

    // Selection restored to session's active lesson (lesson-02, ex 0)
    let current_lesson = app.current_lesson().expect("current lesson");
    assert_eq!(current_lesson.lesson.id, "lesson-02");
    assert_eq!(app.selected_exercise_idx, 0);

    // Render to terminal without panic and verify completion badges
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_unfinished_attempt_handling_and_stale_buffer_recovery() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let buffer_file = temp_dir.join("ex-01.kata");
    fs::write(&buffer_file, "in progress content")?;

    // 1. Session was closed with an unfinished attempt in progress
    let unfinished_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: Some(buffer_file.to_string_lossy().to_string()),
            opened_at: Some(chrono::Utc::now()),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_in_flight".to_string(),
            edits: vec![],
            edit_count: 3,
            verdict: Some(Verdict::InProgress),
        }),
        history: vec![],
        ..Default::default()
    };
    unfinished_session.save_to_file(&session_file)?;

    // 2. Restart app: unfinished attempt is retained, NOT counted as failure in history
    let mut app = App::load(session_file.clone(), lessons_dir.clone());
    assert!(app.session.history.is_empty());
    assert!(app.session.attempt.is_some());
    let att = app.session.attempt.as_ref().unwrap();
    assert_eq!(att.id, "att_in_flight");
    assert_eq!(att.edit_count, 3);
    assert_eq!(att.verdict, Some(Verdict::InProgress));

    // Buffer exists -> connected
    match app.connection_status() {
        ConnectionStatus::Connected { buffer_path, .. } => {
            assert_eq!(buffer_path, buffer_file);
        }
        other => panic!("Expected Connected status, got {:?}", other),
    }

    // 3. Simulate buffer deleted from disk while app was closed or during work
    fs::remove_file(&buffer_file)?;

    // Restart/reload: status transitions to StaleBuffer
    let stale_app = App::load(session_file.clone(), lessons_dir.clone());
    match stale_app.connection_status() {
        ConnectionStatus::StaleBuffer { buffer_path, .. } => {
            assert_eq!(buffer_path, buffer_file);
        }
        other => panic!("Expected StaleBuffer, got {:?}", other),
    }

    // 4. Pressing 'r' resets attempt and restores clean buffer
    handle_event(&mut app, make_key_event(KeyCode::Char('r')))?;
    let reloaded_session = SessionState::load_from_file(&session_file)?;
    assert!(reloaded_session.attempt.is_some());
    assert_ne!(
        reloaded_session.attempt.unwrap().id,
        "att_in_flight",
        "Reset should issue fresh attempt ID"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_corrupt_session_preserves_backup_without_silent_overwrite() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    // 1. Write corrupted JSON
    let corrupt_bytes = b"{{{ invalid json broken session data }}}";
    fs::write(&session_file, corrupt_bytes)?;

    // 2. Start TUI app with corrupt file
    let mut app = App::load(session_file.clone(), lessons_dir.clone());

    // Corrupt file was preserved, not silently erased
    assert!(app.corrupt_backup_path.is_some());
    let backup_path = app.corrupt_backup_path.clone().unwrap();
    assert!(backup_path.exists());
    assert_eq!(fs::read(&backup_path)?, corrupt_bytes);

    // session_warning informs user
    assert!(app.session_warning.is_some());
    let warn = app.session_warning.as_ref().unwrap();
    assert!(warn.contains("Damaged state preserved"));

    // Render should show recovery notice
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // 3. User selects or switches exercise: writes fresh session without overwriting backup
    app.activate_selected()?;
    assert!(session_file.exists());
    assert!(backup_path.exists());
    assert_eq!(fs::read(&backup_path)?, corrupt_bytes);

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_incompatible_session_version_preserves_backup() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let future_session = SessionState {
        version: 999,
        history: vec![HistoryEntry {
            lesson_id: "motions-01-word".to_string(),
            exercise_id: "delete-to-word-end".to_string(),
            passed: true,
            par_edits: Some(1),
            actual_edits: 1,
            score: 100,
            completed_at: chrono::Utc::now(),
        }],
        ..Default::default()
    };
    let json = future_session.to_json_pretty().expect("json");
    fs::write(&session_file, &json)?;

    // Load with future incompatible schema version
    let app = App::load(session_file.clone(), lessons_dir.clone());
    assert!(app.corrupt_backup_path.is_some());
    let backup_path = app.corrupt_backup_path.as_ref().unwrap();
    assert!(backup_path.exists());
    assert_eq!(fs::read_to_string(backup_path)?, json);
    assert_eq!(app.session.version, CURRENT_SCHEMA_VERSION);

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_missing_fixture_detection_and_safeguards() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = temp_dir.join("lessons");
    fs::create_dir_all(&lessons_dir)?;

    // Create a lesson TOML pointing to nonexistent start/goal fixtures
    let lesson_toml = r#"
[lesson]
id = "missing-fixture-lesson"
title = "Missing Fixture Lesson"
requires = []
concepts = ["test"]
prose = "Test lesson with missing fixtures"

[[exercise]]
id = "ex-01"
task = "Do something"
start = "fixtures/nonexistent.start.kata"
goal = "fixtures/nonexistent.goal.kata"
par_edits = 1
max_edits = 2
"#;
    fs::write(lessons_dir.join("lesson.toml"), lesson_toml)?;

    let mut app = App::load(session_file.clone(), lessons_dir);
    assert_eq!(app.lessons().len(), 1);

    // 1. Fixture status detects missing start fixture
    match app.current_exercise_fixture_status() {
        FixtureStatus::MissingStart(path) => {
            assert!(path.ends_with("nonexistent.start.kata"));
        }
        other => panic!("Expected MissingStart, got {:?}", other),
    }

    // 2. Launch selected detects missing fixture and aborts safely
    app.launch_selected()?;
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap();
    assert!(msg.contains("Cannot launch: start fixture missing"));

    // 3. Reset active detects missing fixture and aborts safely
    app.session.current.lesson_id = "missing-fixture-lesson".to_string();
    app.session.current.exercise_index = 0;
    app.reset_active()?;
    let reset_msg = app.status_message.as_ref().unwrap();
    assert!(reset_msg.contains("Cannot reset: start fixture missing"));

    // 4. Render UI with MissingFixture status
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
