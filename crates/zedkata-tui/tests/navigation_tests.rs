use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zedkata_core::{
    session::{EditRange, RecordedEdit, SessionState},
    verification::AttemptOutcome,
};
use zedkata_tui::{app::App, event::handle_event, ui};

fn setup_temp_workspace(prefix: &str) -> (PathBuf, PathBuf) {
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_{}_{}_{}",
        prefix,
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

fn repo_lessons_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
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
fn test_advance_within_lesson_updates_task_and_selection() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_advance_within");
    let lessons_dir = repo_lessons_dir();

    let mut app = App::load(session_path.clone(), lessons_dir);
    assert_eq!(app.selected_lesson_idx, 0);
    assert_eq!(app.selected_exercise_idx, 0);

    let ex0 = app.current_exercise().expect("Exercise 0 expected");
    assert_eq!(ex0.id, "delete-word-arg");
    assert!(ex0.task.contains("Delete the unused parameter name"));

    // Advance to next exercise within same lesson
    let advanced = app.advance_exercise();
    assert!(advanced);
    assert_eq!(app.selected_exercise_idx, 1);

    let ex1 = app.current_exercise().expect("Exercise 1 expected");
    assert_eq!(ex1.id, "delete-to-word-end");
    assert!(ex1.task.contains("Delete the suffix 'Error'"));

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_advance_blocked_by_locked_prerequisites() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_advance_locked");
    let lessons_dir = repo_lessons_dir();

    let mut app = App::load(session_path.clone(), lessons_dir);
    // Move to exercise 1 of lesson 0
    app.selected_exercise_idx = 1;

    // Trying to advance past the last exercise of lesson 0 should try lesson 1 ("motions-03-textobjects")
    // which requires ["motions-01-word", "motions-02-find"].
    // Since session history is empty, it must be locked!
    let advanced = app.advance_exercise();
    assert!(!advanced);
    assert_eq!(app.selected_lesson_idx, 0);
    assert_eq!(app.selected_exercise_idx, 1);

    let status = app.status_message.as_deref().unwrap_or("");
    assert!(
        status.contains("locked"),
        "Status should report locked lesson, found: {}",
        status
    );
    assert!(
        status.contains("motions-03-textobjects"),
        "Status should name target lesson, found: {}",
        status
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_advance_across_lessons_when_unlocked() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_advance_unlocked");
    let lessons_dir = repo_lessons_dir();

    let mut initial_session = SessionState::default();
    // Satisfy requirements for motions-03-textobjects: motions-01-word and motions-02-find
    initial_session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-01-word",
            "delete-word-arg",
            true,
            Some(1),
            1,
            100,
            chrono::Utc::now(),
        ));
    initial_session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-02-find",
            "dummy",
            true,
            Some(1),
            1,
            100,
            chrono::Utc::now(),
        ));
    initial_session.save_to_file(&session_path)?;

    let mut app = App::load(session_path.clone(), lessons_dir);
    app.selected_exercise_idx = 1; // at end of lesson 0

    let advanced = app.advance_exercise();
    assert!(advanced, "Should advance to next unlocked lesson");
    assert_eq!(app.selected_lesson_idx, 1);
    assert_eq!(app.selected_exercise_idx, 0);

    let current_lesson = app.current_lesson().expect("Lesson 1 expected");
    assert_eq!(current_lesson.lesson.id, "motions-03-textobjects");

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_advance_at_course_end_reports_completion() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_advance_end");
    let lessons_dir = repo_lessons_dir();

    let mut app = App::load(session_path.clone(), lessons_dir);
    let total_lessons = app.lessons().len();
    assert!(total_lessons > 0);

    // Set selection to the very last lesson and its last exercise
    app.selected_lesson_idx = total_lessons - 1;
    let last_lesson = app.current_lesson().unwrap();
    app.selected_exercise_idx = last_lesson.lesson.exercises.len().saturating_sub(1);

    let advanced = app.advance_exercise();
    assert!(!advanced, "Cannot advance past end of curriculum");
    let status = app.status_message.as_deref().unwrap_or("");
    assert!(
        status.contains("Course completed"),
        "Status should report course completion, got: {}",
        status
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_advance_and_activate_persists_to_session() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_advance_activate");
    let lessons_dir = repo_lessons_dir();

    let mut initial_session = SessionState::default();
    initial_session.switch_exercise("motions-01-word".to_string(), 0);
    initial_session.start_attempt("attempt_prev".to_string(), Some("dummy_path".to_string()));
    initial_session.save_to_file(&session_path)?;

    let mut app = App::load(session_path.clone(), lessons_dir);
    assert_eq!(app.session.current.exercise_index, 0);

    let advanced = app.advance_and_activate()?;
    assert!(advanced);

    let on_disk = SessionState::load_from_file(&session_path)?;
    assert_eq!(on_disk.current.lesson_id, "motions-01-word");
    assert_eq!(on_disk.current.exercise_index, 1);
    // Previous attempt cleared on exercise switch
    assert!(on_disk.attempt.is_none());
    assert!(on_disk.current.buffer_path.is_none());

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_repeated_completion_does_not_corrupt_progress() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_repeated_completion");
    let lessons_dir = repo_lessons_dir();

    let mut session = SessionState::default();
    // Simulate student repeating and passing the SAME exercise multiple times
    for i in 0..5 {
        session
            .history
            .push(zedkata_core::scoring::create_history_entry(
                "motions-01-word",
                "delete-word-arg",
                true,
                Some(1),
                1,
                100,
                chrono::Utc::now() + chrono::Duration::seconds(i),
            ));
    }
    session.save_to_file(&session_path)?;

    let app = App::load(session_path.clone(), lessons_dir);

    // Render TUI feedback pane
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        ui::render(f, &app);
    })?;

    let buffer = terminal.backend().buffer();
    let text = (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer.get(x, y).symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Total unique exercises completed must be 1, NOT 5!
    assert!(
        text.contains("1 of "),
        "Progress must count 1 unique completed exercise despite 5 attempts, output:\n{}",
        text
    );
    assert!(
        !text.contains("5 of "),
        "Progress should not count duplicate passes"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_late_old_buffer_edits_rejected_after_switch() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_late_edits");
    let lessons_dir = repo_lessons_dir();

    let mut session = SessionState::default();
    session.switch_exercise("motions-01-word".to_string(), 0);
    session.start_attempt(
        "attempt_ex0_initial".to_string(),
        Some("ex0.kata".to_string()),
    );
    session.save_to_file(&session_path)?;

    let mut app = App::load(session_path.clone(), lessons_dir);
    // Student advances to exercise 1
    assert!(app.advance_and_activate()?);

    let mut switched_session = SessionState::load_from_file(&session_path)?;
    assert_eq!(switched_session.current.exercise_index, 1);
    assert!(switched_session.attempt.is_none());

    // A late edit arrives from the old buffer (exercise 0) with old attempt ID
    let late_edit = RecordedEdit {
        at: chrono::Utc::now(),
        range: EditRange {
            start: (0, 0),
            end: (0, 6),
        },
        inserted: String::new(),
        deleted_len: 6,
    };

    let accepted = switched_session.record_edit("attempt_ex0_initial", late_edit);
    assert!(!accepted, "Late edit from old attempt must be rejected");

    let finalized = switched_session.finalize_attempt(
        "attempt_ex0_initial",
        &AttemptOutcome::Passed {
            edits: 1,
            par: Some(1),
            score: 100,
        },
        "delete-word-arg",
        &[],
    );
    assert!(
        !finalized,
        "Late finalization from old attempt must be rejected"
    );

    // Exercise 1 state remains pristine
    assert_eq!(switched_session.current.exercise_index, 1);
    assert!(switched_session.attempt.is_none());
    assert_eq!(switched_session.history.len(), 0);

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_tui_key_n_advance_and_activate() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("nav_key_n");
    let lessons_dir = repo_lessons_dir();

    let mut app = App::load(session_path.clone(), lessons_dir);
    assert_eq!(app.selected_exercise_idx, 0);

    // Press 'n' key
    handle_event(&mut app, make_key_event(KeyCode::Char('n')))?;

    assert_eq!(app.selected_exercise_idx, 1);
    let session = SessionState::load_from_file(&session_path)?;
    assert_eq!(session.current.exercise_index, 1);

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
