use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use zedkata_core::session::{Attempt, CurrentSession, SessionState, Verdict};
use zedkata_tui::{
    app::{App, ConnectionStatus},
    event::handle_event,
    ui,
};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn setup_temp_workspace() -> (PathBuf, PathBuf) {
    let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_conn_test_{}_{}_{}",
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
fn test_idle_connection_status() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let app = App::load(session_file, lessons_dir);
    assert_eq!(app.connection_status(), ConnectionStatus::Idle);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_waiting_for_zed_when_unopened() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        ..Default::default()
    };
    session.save_to_file(&session_file)?;

    let app = App::load(session_file, lessons_dir);
    match app.connection_status() {
        ConnectionStatus::WaitingForZed {
            lesson_id,
            exercise_index,
            ..
        } => {
            assert_eq!(lesson_id, "motions-01-word");
            assert_eq!(exercise_index, 0);
        }
        other => panic!("Expected WaitingForZed, got {:?}", other),
    }

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_connected_to_zed_when_buffer_and_attempt_present() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    // Create real buffer file on disk
    let buffer_file = temp_dir.join("ex-01.kata");
    fs::write(&buffer_file, "apple banana cherry")?;

    let session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: Some(buffer_file.to_string_lossy().to_string()),
            opened_at: Some(chrono::Utc::now()),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_conn_1".to_string(),
            edits: vec![],
            edit_count: 0,
            verdict: Some(Verdict::InProgress),
        }),
        ..Default::default()
    };
    session.save_to_file(&session_file)?;

    let app = App::load(session_file, lessons_dir);
    match app.connection_status() {
        ConnectionStatus::Connected {
            lesson_id,
            exercise_index,
            buffer_path,
            attempt_id,
        } => {
            assert_eq!(lesson_id, "motions-01-word");
            assert_eq!(exercise_index, 0);
            assert_eq!(buffer_path, buffer_file);
            assert_eq!(attempt_id, "att_conn_1");
        }
        other => panic!("Expected Connected, got {:?}", other),
    }

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_mismatched_session_and_sync_recovery() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    // Session active on motions-01-word:0
    let session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: None,
            nav_mode: None,
        },
        ..Default::default()
    };
    session.save_to_file(&session_file)?;

    let mut app = App::load(session_file, lessons_dir);
    let active_idx = app
        .lessons()
        .iter()
        .position(|l| l.lesson.id == "motions-01-word")
        .expect("motions-01-word must be in catalog");
    assert_eq!(app.selected_lesson_idx, active_idx);

    // User navigates away to next lesson in TUI
    app.next_lesson();
    assert_ne!(app.selected_lesson_idx, active_idx);

    match app.connection_status() {
        ConnectionStatus::MismatchedSession {
            selected_lesson,
            active_lesson,
            ..
        } => {
            assert_ne!(selected_lesson, active_lesson);
            assert_eq!(active_lesson, "motions-01-word");
        }
        other => panic!("Expected MismatchedSession, got {:?}", other),
    }

    // Render mismatched state
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // Press 's' to trigger sync_to_active
    handle_event(&mut app, make_key_event(KeyCode::Char('s')))?;

    // Now selection is synchronized back to active session drill
    assert_eq!(app.selected_lesson_idx, active_idx);
    assert_eq!(app.selected_exercise_idx, 0);
    assert!(matches!(
        app.connection_status(),
        ConnectionStatus::WaitingForZed { .. }
    ));

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_stale_buffer_missing_on_disk() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let nonexistent_buffer = temp_dir.join("nonexistent_buffer.kata");
    assert!(!nonexistent_buffer.exists());

    let session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: Some(nonexistent_buffer.to_string_lossy().to_string()),
            opened_at: Some(chrono::Utc::now()),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_stale".to_string(),
            edits: vec![],
            edit_count: 1,
            verdict: Some(Verdict::InProgress),
        }),
        ..Default::default()
    };
    session.save_to_file(&session_file)?;

    let app = App::load(session_file, lessons_dir);
    match app.connection_status() {
        ConnectionStatus::StaleBuffer { buffer_path, .. } => {
            assert_eq!(buffer_path, nonexistent_buffer);
        }
        other => panic!("Expected StaleBuffer, got {:?}", other),
    }

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_actionable_verdicts_passed_and_failed_render() -> Result<()> {
    let (temp_dir, session_file) = setup_temp_workspace();
    let lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons");

    let buffer_file = temp_dir.join("ex-01.kata");
    fs::write(&buffer_file, "content")?;

    // 1. Test Passed verdict actionable feedback
    let passed_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: Some(buffer_file.to_string_lossy().to_string()),
            opened_at: Some(chrono::Utc::now()),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_passed".to_string(),
            edits: vec![],
            edit_count: 2,
            verdict: Some(Verdict::Passed {
                edits: 2,
                par: Some(2),
                score: 100,
            }),
        }),
        ..Default::default()
    };
    passed_session.save_to_file(&session_file)?;

    let mut app = App::load(session_file.clone(), lessons_dir.clone());
    app.poll_interval = Duration::from_millis(1);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // 2. Test Failed verdict actionable feedback
    let failed_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: Some(buffer_file.to_string_lossy().to_string()),
            opened_at: Some(chrono::Utc::now()),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_failed".to_string(),
            edits: vec![],
            edit_count: 5,
            verdict: Some(Verdict::Failed {
                reason: "Exceeded max edit budget".to_string(),
            }),
        }),
        ..Default::default()
    };
    failed_session.save_to_file(&session_file)?;

    app.force_poll_session();
    terminal.draw(|f| ui::render(f, &app))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
