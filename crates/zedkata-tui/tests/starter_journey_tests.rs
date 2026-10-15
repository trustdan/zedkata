use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::{Path, PathBuf},
};
use zedkata_core::{
    session::{Attempt, Verdict},
    verification::AttemptOutcome,
};
use zedkata_tui::{app::App, event::handle_event, ui};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn setup_temp_workspace(prefix: &str) -> (PathBuf, PathBuf) {
    let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_{}_{}_{}_{}",
        prefix,
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
fn test_starter_course_user_journey_end_to_end() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("starter_course_journey");
    let lessons_dir = repo_lessons_dir();

    // -------------------------------------------------------------------------
    // Phase 1: Setup & Initial Session Initialization
    // -------------------------------------------------------------------------
    assert!(
        !session_path.exists(),
        "Session file must not exist before setup"
    );

    let mut app = App::load(session_path.clone(), lessons_dir.clone());
    assert!(app.session_warning.is_none());
    assert_eq!(app.session.history.len(), 0);

    let starter_idx = app
        .lessons()
        .iter()
        .position(|l| l.lesson.id == "motions-01-word")
        .expect("Starter course motions-01-word must be present in catalog");
    app.selected_lesson_idx = starter_idx;
    app.selected_exercise_idx = 0;

    {
        let initial_lesson = app.current_lesson().expect("Must have motions-01-word");
        assert_eq!(initial_lesson.lesson.id, "motions-01-word");
        assert_eq!(initial_lesson.lesson.exercises.len(), 3);
        assert!(!app.is_lesson_completed(&initial_lesson.lesson));
    }

    // Initial render test: UI renders cleanly at 80x24
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;

    // -------------------------------------------------------------------------
    // Phase 2: Select Exercise 1 (delete-word-arg) & Reveal Hint
    // -------------------------------------------------------------------------
    let (ex1_id, ex1_task, ex1_hint) = {
        let ex = app.current_exercise().expect("Exercise 1 must exist");
        (
            ex.id.clone(),
            ex.task.clone(),
            ex.hint.clone().unwrap_or_default(),
        )
    };
    assert_eq!(ex1_id, "delete-word-arg");
    assert!(ex1_task.contains("Delete the unused parameter name 'target'"));

    // Activate selected exercise in session
    assert!(app.activate_selected().is_ok());
    assert_eq!(app.session.current.lesson_id, "motions-01-word");
    assert_eq!(app.session.current.exercise_index, 0);

    // Initially hint is hidden
    assert!(!app.show_hint);

    // Toggle hint via '?' key
    handle_event(&mut app, make_key_event(KeyCode::Char('?')))?;
    assert!(app.show_hint, "Hint should be visible after pressing '?'");

    // Verify hint content matches category guidance
    assert_eq!(
        ex1_hint,
        "A single word deletion will remove the identifier and trailing space."
    );

    // Render frame with hint revealed
    terminal.draw(|f| ui::render(f, &app))?;
    let buffer_str = format!("{:?}", terminal.backend().buffer());
    assert!(
        buffer_str.contains("trailing space") || buffer_str.contains("single word"),
        "Rendered frame must contain revealed hint"
    );

    // Toggle hint off via '?' key
    handle_event(&mut app, make_key_event(KeyCode::Char('?')))?;
    assert!(!app.show_hint, "Hint should be hidden after toggling");

    // -------------------------------------------------------------------------
    // Phase 3: Make a Mistake (Rule Violation / Backspace Run)
    // -------------------------------------------------------------------------
    // Simulate LSP observing 7 consecutive backspaces, exceeding forbid.backspace_run = 3
    let mistake_attempt_id = "attempt_ex1_mistake";
    app.session.attempt = Some(Attempt {
        id: mistake_attempt_id.to_string(),
        edit_count: 7,
        edits: Vec::new(),
        verdict: Some(Verdict::InProgress),
    });

    let fail_outcome = AttemptOutcome::Failed {
        reason: "forbidden pattern 'backspace_run' exceeded limit of 3 (observed 7 consecutive single-character deletions)".to_string(),
    };
    let finalized = app.session.finalize_attempt(
        mistake_attempt_id,
        &fail_outcome,
        "delete-word-arg",
        &["dw".to_string()],
    );
    assert!(finalized, "Mistake attempt should finalize successfully");
    app.session.save_to_file(&session_path)?;

    // Verify mistake persisted
    assert_eq!(app.session.history.len(), 1);
    assert!(!app.session.history[0].passed);
    assert_eq!(app.session.history[0].score, 0);
    assert!(app.session.weaknesses.get("dw").copied().unwrap_or(0.0) > 0.0);

    // Verify active attempt state reflects FAILED
    if let Some(ref att) = app.session.attempt {
        match &att.verdict {
            Some(Verdict::Failed { reason }) => {
                assert!(reason.contains("backspace_run"));
            }
            other => panic!("Expected Verdict::Failed, got {:?}", other),
        }
    }

    // Render frame displaying failed verdict and weakness
    terminal.draw(|f| ui::render(f, &app))?;

    // -------------------------------------------------------------------------
    // Phase 4: Retry / Reset Active Attempt
    // -------------------------------------------------------------------------
    // Student presses 'r' to retry
    handle_event(&mut app, make_key_event(KeyCode::Char('r')))?;

    // Verdict must be reset to InProgress
    if let Some(ref att) = app.session.attempt {
        assert_eq!(att.edit_count, 0);
        assert_eq!(att.verdict, Some(Verdict::InProgress));
    }

    // -------------------------------------------------------------------------
    // Phase 5: Complete Exercise 1 (delete-word-arg) with Par (dw)
    // -------------------------------------------------------------------------
    let pass1_attempt_id = app
        .session
        .attempt
        .as_ref()
        .map(|a| a.id.clone())
        .unwrap_or_else(|| "attempt_ex1_pass".to_string());
    if let Some(ref mut att) = app.session.attempt {
        att.edit_count = 1;
    }

    let pass1_outcome = AttemptOutcome::Passed {
        edits: 1,
        par: Some(1),
        score: 100,
    };
    let finalized_pass1 = app.session.finalize_attempt(
        &pass1_attempt_id,
        &pass1_outcome,
        "delete-word-arg",
        &["dw".to_string()],
    );
    assert!(finalized_pass1);
    app.session.save_to_file(&session_path)?;

    assert_eq!(app.session.history.len(), 2);
    assert!(app.session.history[1].passed);
    assert_eq!(app.session.history[1].score, 100);

    // -------------------------------------------------------------------------
    // Phase 6: Advance to Exercise 2 (delete-to-word-end) & Complete with Par (de)
    // -------------------------------------------------------------------------
    // Press 'n' to advance
    handle_event(&mut app, make_key_event(KeyCode::Char('n')))?;
    assert_eq!(app.selected_exercise_idx, 1);
    assert_eq!(app.session.current.exercise_index, 1);

    let (ex2_id, ex2_task, ex2_hint) = {
        let ex = app.current_exercise().expect("Exercise 2 must exist");
        (
            ex.id.clone(),
            ex.task.clone(),
            ex.hint.clone().unwrap_or_default(),
        )
    };
    assert_eq!(ex2_id, "delete-to-word-end");
    assert!(ex2_task.contains("Delete the suffix 'Error'"));

    // Reveal hint for exercise 2
    handle_event(&mut app, make_key_event(KeyCode::Char('?')))?;
    assert!(app.show_hint);
    assert_eq!(ex2_hint, "Delete forward to the end of the current word.");
    handle_event(&mut app, make_key_event(KeyCode::Char('?')))?;
    assert!(!app.show_hint);

    // Student executes intended technique 'de': 1 edit, par 1, score 100
    let pass2_attempt_id = "attempt_ex2_pass";
    app.session.attempt = Some(Attempt {
        id: pass2_attempt_id.to_string(),
        edit_count: 1,
        edits: Vec::new(),
        verdict: Some(Verdict::InProgress),
    });

    let pass2_outcome = AttemptOutcome::Passed {
        edits: 1,
        par: Some(1),
        score: 100,
    };
    let finalized_pass2 = app.session.finalize_attempt(
        pass2_attempt_id,
        &pass2_outcome,
        "delete-to-word-end",
        &["de".to_string()],
    );
    assert!(finalized_pass2);
    app.session.save_to_file(&session_path)?;

    assert_eq!(app.session.history.len(), 3);
    assert!(app.session.history[2].passed);
    assert_eq!(app.session.history[2].score, 100);

    // -------------------------------------------------------------------------
    // Phase 7: Advance to Exercise 3 (delete-word-binding) & Complete with Par (w + dw)
    // -------------------------------------------------------------------------
    // Press 'n' to advance
    handle_event(&mut app, make_key_event(KeyCode::Char('n')))?;
    assert_eq!(app.selected_exercise_idx, 2);
    assert_eq!(app.session.current.exercise_index, 2);

    let (ex3_id, ex3_task, ex3_hint) = {
        let ex = app.current_exercise().expect("Exercise 3 must exist");
        (
            ex.id.clone(),
            ex.task.clone(),
            ex.hint.clone().unwrap_or_default(),
        )
    };
    assert_eq!(ex3_id, "delete-word-binding");
    assert!(ex3_task.contains("Remove the unused 'mut' binding modifier"));

    // Reveal hint for exercise 3
    handle_event(&mut app, make_key_event(KeyCode::Char('?')))?;
    assert!(app.show_hint);
    assert_eq!(ex3_hint, "Advance to the target word before deleting.");
    handle_event(&mut app, make_key_event(KeyCode::Char('?')))?;
    assert!(!app.show_hint);

    // Student executes intended technique 'w' + 'dw': 1 edit, par 1, score 100
    let pass3_attempt_id = "attempt_ex3_pass";
    app.session.attempt = Some(Attempt {
        id: pass3_attempt_id.to_string(),
        edit_count: 1,
        edits: Vec::new(),
        verdict: Some(Verdict::InProgress),
    });

    let pass3_outcome = AttemptOutcome::Passed {
        edits: 1,
        par: Some(1),
        score: 100,
    };
    let finalized_pass3 = app.session.finalize_attempt(
        pass3_attempt_id,
        &pass3_outcome,
        "delete-word-binding",
        &["w".to_string(), "dw".to_string()],
    );
    assert!(finalized_pass3);
    app.session.save_to_file(&session_path)?;

    assert_eq!(app.session.history.len(), 4);
    assert!(app.session.history[3].passed);
    assert_eq!(app.session.history[3].score, 100);

    // Verify all 3 exercises in motions-01-word are completed!
    {
        let lesson_entry = app.current_lesson().expect("Lesson entry");
        assert!(
            app.is_lesson_completed(&lesson_entry.lesson),
            "Lesson motions-01-word must be 100% completed"
        );
    }

    // Advance into unlocked downstream lesson (motions-02-find)
    let advanced_to_next = app.advance_and_activate()?;
    assert!(
        advanced_to_next,
        "Advances to unlocked downstream lesson motions-02-find"
    );
    assert_eq!(app.session.current.lesson_id, "motions-02-find");
    assert_eq!(app.session.current.exercise_index, 0);

    // -------------------------------------------------------------------------
    // Phase 8: Quit TUI
    // -------------------------------------------------------------------------
    handle_event(&mut app, make_key_event(KeyCode::Char('q')))?;
    assert!(app.should_quit, "Pressing 'q' sets should_quit flag");

    // Explicitly drop app instance to simulate process termination
    drop(app);

    // -------------------------------------------------------------------------
    // Phase 9: Resume with NO Manual Session-File Editing
    // -------------------------------------------------------------------------
    // Verify file content is intact and valid JSON
    let session_raw = fs::read_to_string(&session_path)?;
    assert!(session_raw.contains("\"version\": 1") || session_raw.contains("\"version\":1"));
    assert!(session_raw.contains("motions-01-word"));
    assert!(session_raw.contains("delete-word-binding"));

    // Relaunch fresh App instance pointing to the untouched session file
    let resumed_app = App::load(session_path.clone(), lessons_dir.clone());

    // Zero corruption warning
    assert!(
        resumed_app.session_warning.is_none(),
        "Resumed session must load without any corruption warning"
    );

    // Current position preserved in advanced lesson
    assert_eq!(resumed_app.session.current.lesson_id, "motions-02-find");
    assert_eq!(resumed_app.session.current.exercise_index, 0);

    // Full history intact: 4 total attempts (1 failed mistake, 3 par passes)
    assert_eq!(resumed_app.session.history.len(), 4);

    let lessons = resumed_app.lessons();
    let word_lesson = lessons
        .iter()
        .find(|l| l.lesson.id == "motions-01-word")
        .expect("motions-01-word entry");
    assert!(
        resumed_app.is_lesson_completed(&word_lesson.lesson),
        "Lesson completion must be preserved after resume"
    );

    // Weaknesses tracked across restart
    assert!(resumed_app.session.weaknesses.contains_key("dw"));

    // UI render of resumed state
    terminal.draw(|f| ui::render(f, &resumed_app))?;
    let resumed_buffer = format!("{:?}", terminal.backend().buffer());
    // Should show 3 passed exercises and 100% completion in curriculum pane
    assert!(
        resumed_buffer.contains("motions-01-word"),
        "Resumed UI must render curriculum with motions-01-word"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
