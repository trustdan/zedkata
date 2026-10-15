use anyhow::Result;
use ratatui::{backend::TestBackend, Terminal};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use zedkata_core::session::{Attempt, CurrentSession, HistoryEntry, SessionState, Verdict};
use zedkata_tui::{app::App, ui};

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

#[test]
fn test_core_derived_prerequisite_unlocking_across_sessions() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("prereq_unlocking");
    let lessons_dir = repo_lessons_dir();
    let now = chrono::Utc::now();

    // 1. Initial fresh session: motions-01-word has no prerequisites, motions-03-textobjects is locked
    let initial_session = SessionState::default();
    initial_session.save_to_file(&session_path)?;

    let app1 = App::load(session_path.clone(), lessons_dir.clone());
    let l0 = &app1
        .lessons()
        .iter()
        .find(|l| l.lesson.id == "motions-01-word")
        .expect("motions-01-word must be present")
        .lesson;
    let (locked0, missing0) = app1.is_lesson_locked(l0);
    assert!(
        !locked0,
        "Lesson motions-01-word should be unlocked initially"
    );
    assert!(missing0.is_empty());

    let l1 = &app1
        .lessons()
        .iter()
        .find(|l| l.lesson.id == "motions-03-textobjects")
        .expect("motions-03-textobjects must be present")
        .lesson;
    let (locked1, missing1) = app1.is_lesson_locked(l1);
    assert!(
        locked1,
        "Lesson motions-03-textobjects should be locked initially"
    );
    assert!(missing1.contains(&"motions-01-word".to_string()));
    assert!(missing1.contains(&"motions-02-find".to_string()));

    // 2. Partial pass: only 1 of 2 exercises completed for motions-01-word
    let mut session = SessionState::load_from_file(&session_path)?;
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-01-word",
            "delete-word-arg",
            true,
            Some(1),
            1,
            100,
            now,
        ));
    session.save_to_file(&session_path)?;

    // Reload App (simulating restart / across sessions)
    let app2 = App::load(session_path.clone(), lessons_dir.clone());
    assert!(
        !app2.is_lesson_completed(l0),
        "motions-01-word is not completed when only 1 of 2 exercises passed"
    );
    let (locked1_step2, missing1_step2) = app2.is_lesson_locked(l1);
    assert!(
        locked1_step2,
        "Lesson 1 must stay locked when lesson 0 is only partially completed"
    );
    assert!(missing1_step2.contains(&"motions-01-word".to_string()));

    // 3. Failed attempt for remaining exercise must NOT satisfy prerequisite
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-01-word",
            "delete-to-word-end",
            false,
            Some(1),
            3,
            0,
            now,
        ));
    // Also failed attempt for motions-02-find
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-02-find",
            "delete-find-delimiter",
            false,
            Some(1),
            3,
            0,
            now,
        ));
    session.save_to_file(&session_path)?;

    let app3 = App::load(session_path.clone(), lessons_dir.clone());
    assert!(!app3.is_lesson_completed(l0));
    let (locked1_step3, _) = app3.is_lesson_locked(l1);
    assert!(
        locked1_step3,
        "Failed attempts must not satisfy prerequisites"
    );

    // 4. Complete all exercises for motions-01-word and motions-02-find
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-01-word",
            "delete-to-word-end",
            true,
            Some(1),
            1,
            100,
            now,
        ));
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-01-word",
            "delete-word-binding",
            true,
            Some(1),
            1,
            100,
            now,
        ));
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-02-find",
            "delete-find-delimiter",
            true,
            Some(1),
            1,
            100,
            now,
        ));
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-02-find",
            "delete-till-char",
            true,
            Some(1),
            1,
            100,
            now,
        ));
    session
        .history
        .push(zedkata_core::scoring::create_history_entry(
            "motions-02-find",
            "change-till-char",
            true,
            Some(1),
            1,
            100,
            now,
        ));
    session.save_to_file(&session_path)?;

    let mut app4 = App::load(session_path.clone(), lessons_dir.clone());
    assert!(
        app4.is_lesson_completed(l0),
        "motions-01-word is now fully completed"
    );
    let (locked1_step4, missing1_step4) = app4.is_lesson_locked(l1);
    assert!(
        !locked1_step4,
        "motions-03-textobjects must unlock when all prerequisites are satisfied"
    );
    assert!(missing1_step4.is_empty());

    // 5. Navigate and activate unlocked lesson
    let l1_idx = app4
        .lessons()
        .iter()
        .position(|l| l.lesson.id == "motions-03-textobjects")
        .expect("motions-03-textobjects must be present");
    app4.selected_lesson_idx = l1_idx;
    app4.selected_exercise_idx = 0;
    app4.activate_selected()?;
    assert_eq!(app4.session.current.lesson_id, "motions-03-textobjects");

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_scored_vs_unscored_mode_and_failure_display() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("mode_display");
    let lessons_dir = repo_lessons_dir();
    let now = chrono::Utc::now();

    // 1. Scored mode: exercise with par_edits
    let scored_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: Some(now),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_scored".to_string(),
            edits: vec![],
            edit_count: 2,
            verdict: Some(Verdict::Passed {
                edits: 2,
                par: Some(1),
                score: 85,
            }),
        }),
        history: vec![HistoryEntry {
            lesson_id: "motions-01-word".to_string(),
            exercise_id: "delete-word-arg".to_string(),
            passed: true,
            par_edits: Some(1),
            actual_edits: 2,
            score: 85,
            completed_at: now,
        }],
        weaknesses: HashMap::new(),
        ..Default::default()
    };
    scored_session.save_to_file(&session_path)?;

    let app_scored = App::load(session_path.clone(), lessons_dir.clone());
    let backend = TestBackend::new(120, 45);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app_scored))?;

    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

    assert!(
        content.contains("Par: 1 edit(s)"),
        "Task pane should show par in scored mode"
    );
    assert!(
        content.contains("score: 85"),
        "Live attempt verdict should show score"
    );
    assert!(
        content.contains("(score 85)"),
        "Recent history should show (score 85)"
    );

    // 2. Unscored mode: exercise with par: None (goal-only)
    let unscored_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: Some(now),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att_unscored".to_string(),
            edits: vec![],
            edit_count: 4,
            verdict: Some(Verdict::Passed {
                edits: 4,
                par: None,
                score: 100,
            }),
        }),
        history: vec![HistoryEntry {
            lesson_id: "motions-01-word".to_string(),
            exercise_id: "unscored-exercise".to_string(),
            passed: true,
            par_edits: None,
            actual_edits: 4,
            score: 100,
            completed_at: now,
        }],
        weaknesses: HashMap::new(),
        ..Default::default()
    };
    unscored_session.save_to_file(&session_path)?;

    let app_unscored = App::load(session_path.clone(), lessons_dir.clone());
    terminal.draw(|f| ui::render(f, &app_unscored))?;

    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

    assert!(
        content.contains("Goal-only / Unscored"),
        "Live attempt verdict should explicitly say Goal-only / Unscored"
    );
    assert!(
        content.contains("(unscored)"),
        "Recent history should explicitly show (unscored)"
    );

    // 3. Failed attempt display
    let failed_session = SessionState {
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: Some(now),
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
        history: vec![HistoryEntry {
            lesson_id: "motions-01-word".to_string(),
            exercise_id: "delete-word-arg".to_string(),
            passed: false,
            par_edits: Some(1),
            actual_edits: 5,
            score: 0,
            completed_at: now,
        }],
        weaknesses: HashMap::new(),
        ..Default::default()
    };
    failed_session.save_to_file(&session_path)?;

    let app_failed = App::load(session_path.clone(), lessons_dir);
    terminal.draw(|f| ui::render(f, &app_failed))?;

    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

    assert!(
        content.contains("FAILED: Exceeded max edit budget (score: 0)"),
        "Failed verdict should show reason and score 0"
    );
    assert!(
        content.contains("(failed)"),
        "Recent history should indicate (failed)"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_weakness_tracking_display_and_sorting() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("weakness_tracking");
    let lessons_dir = repo_lessons_dir();

    // 1. Initial state: clean / no recorded weaknesses
    let mut session = SessionState::default();
    session.save_to_file(&session_path)?;

    let app_clean = App::load(session_path.clone(), lessons_dir.clone());
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app_clean))?;

    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

    assert!(
        content.contains("Weakness: None recorded (clean)"),
        "Empty weaknesses should render clean indicator"
    );

    // 2. Tracked concept weaknesses: check descending severity sorting
    session.weaknesses.insert("dw".to_string(), 0.30);
    session.weaknesses.insert("ci".to_string(), 0.85);
    session.weaknesses.insert("de".to_string(), 0.15);
    session.save_to_file(&session_path)?;

    let app_weaknesses = App::load(session_path.clone(), lessons_dir.clone());
    terminal.draw(|f| ui::render(f, &app_weaknesses))?;

    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

    assert!(content.contains("ci: 85%"));
    assert!(content.contains("dw: 30%"));
    assert!(content.contains("de: 15%"));

    // Verify ordering: ci (85%) should appear before dw (30%)
    let ci_idx = content.find("ci: 85%").expect("ci found");
    let dw_idx = content.find("dw: 30%").expect("dw found");
    let de_idx = content.find("de: 15%").expect("de found");
    assert!(ci_idx < dw_idx, "Highest weakness ci must appear before dw");
    assert!(dw_idx < de_idx, "dw must appear before de");

    // 3. Small terminal 60x15 boundary fit test
    let small_backend = TestBackend::new(60, 15);
    let mut small_terminal = Terminal::new(small_backend)?;
    small_terminal.draw(|f| ui::render(f, &app_weaknesses))?;

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}
