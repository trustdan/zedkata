use anyhow::Result;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use zedkata_agent::{generator::CandidateExercise, storage::store_candidate_assets};
use zedkata_core::{catalog::Catalog, session::SessionState};
use zedkata_lsp::feedback::FeedbackEngine;
use zedkata_tui::{app::App, launcher::prepare_exercise_buffer, ui};

fn create_temp_env(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zedkata_{}_{}_{}",
        prefix,
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn sample_candidate_exercise(lesson_id: &str, exercise_id: &str) -> CandidateExercise {
    let toml = format!(
        r#"[lesson]
id = "{lesson_id}"
title = "Targeted Deletion Drill"
concepts = ["dw", "word-motions"]
draft = false
prose = "Practice targeted dw deletion on words."

[[exercise]]
id = "{exercise_id}"
task = "Delete the extra word 'broken' using dw."
start = "fixtures/{lesson_id}/{exercise_id}.start.kata"
goal = "fixtures/{lesson_id}/{exercise_id}.goal.kata"
par_edits = 1
max_edits = 2
cursor = [1, 7]
hint = "Place cursor at start of word and press dw."

[exercise.forbid]
backspace_run = 3
"#
    );

    CandidateExercise {
        lesson_id: lesson_id.to_string(),
        title: "Targeted Deletion Drill".to_string(),
        module: None,
        requires: Vec::new(),
        concepts: vec!["dw".to_string(), "word-motions".to_string()],
        draft: false,
        prose: "Practice targeted dw deletion on words.".to_string(),
        exercise_id: exercise_id.to_string(),
        task: "Delete the extra word 'broken' using dw.".to_string(),
        start_fixture_relative_path: format!("fixtures/{}/{}.start.kata", lesson_id, exercise_id),
        start_fixture_content: "const broken value = 42;\n".to_string(),
        goal_fixture_relative_path: format!("fixtures/{}/{}.goal.kata", lesson_id, exercise_id),
        goal_fixture_content: "const value = 42;\n".to_string(),
        par_edits: Some(1),
        max_edits: Some(2),
        cursor: Some([1, 7]),
        hint: Some("Place cursor at start of word and press dw.".to_string()),
        forbid_backspace_run: Some(3),
        raw_toml: toml,
        status: zedkata_agent::generator::CandidateStatus::Candidate,
    }
}

#[test]
fn test_generated_catalog_discovery_outside_repo() -> Result<()> {
    let temp_root = create_temp_env("outside_repo_discovery");
    let shipped_dir = temp_root.join("external_shipped");
    let generated_dir = temp_root.join("external_generated");
    std::fs::create_dir_all(&shipped_dir)?;
    std::fs::create_dir_all(&generated_dir)?;

    // 1. Write one shipped lesson to external_shipped
    let shipped_toml = r#"[lesson]
id = "shipped-01-base"
title = "Base Lesson"
draft = false
prose = "Foundational motions."

[[exercise]]
id = "drill-1"
task = "Test drill"
start = "fixtures/base.start.kata"
goal = "fixtures/base.goal.kata"
par_edits = 1
cursor = [1, 1]
"#;
    std::fs::write(shipped_dir.join("shipped-01-base.toml"), shipped_toml)?;
    let shipped_fixtures = shipped_dir.join("fixtures");
    std::fs::create_dir_all(&shipped_fixtures)?;
    std::fs::write(shipped_fixtures.join("base.start.kata"), "start buffer\n")?;
    std::fs::write(shipped_fixtures.join("base.goal.kata"), "goal buffer\n")?;

    // 2. Store a generated exercise in external_generated
    let mut candidate = sample_candidate_exercise("gen-dw-outside", "gen-dw-drill");
    let stored_paths = store_candidate_assets(&mut candidate, &generated_dir, None)?;
    assert!(stored_paths.toml_path.exists());
    assert!(stored_paths.start_fixture_path.exists());
    assert!(stored_paths.goal_fixture_path.exists());

    // 3. Load catalog with generated dir completely outside repo
    let (catalog, merge_summary) =
        Catalog::load_with_generated(&shipped_dir, Some(&generated_dir))?;

    assert_eq!(catalog.all_lessons().len(), 2);
    assert!(catalog.get("shipped-01-base").is_some());
    let gen_entry = catalog
        .get("gen-dw-outside")
        .expect("generated lesson must be present in catalog");
    assert_eq!(gen_entry.lesson.title, "Targeted Deletion Drill");
    assert!(gen_entry.lesson.is_generated());

    let summary = merge_summary.expect("merge summary should be present");
    assert_eq!(summary.added, vec!["gen-dw-outside".to_string()]);
    assert!(summary.skipped_collisions.is_empty());

    // 4. Verify fixture path resolution outside repo
    let start_res = catalog.resolve_fixture("gen-dw-outside", "gen-dw-drill", true)?;
    assert_eq!(start_res, stored_paths.start_fixture_path);
    assert!(start_res.is_file());

    let goal_res = catalog.resolve_fixture("gen-dw-outside", "gen-dw-drill", false)?;
    assert_eq!(goal_res, stored_paths.goal_fixture_path);
    assert!(goal_res.is_file());

    let _ = std::fs::remove_dir_all(&temp_root);
    Ok(())
}

#[test]
fn test_select_and_launch_generated_exercise_buffer() -> Result<()> {
    let temp_root = create_temp_env("select_launch_gen");
    let shipped_dir = temp_root.join("shipped");
    let generated_dir = temp_root.join("generated");
    std::fs::create_dir_all(&shipped_dir)?;
    std::fs::create_dir_all(&generated_dir)?;

    let mut candidate = sample_candidate_exercise("gen-dw-launch", "gen-dw-drill");
    store_candidate_assets(&mut candidate, &generated_dir, None)?;

    let session_file = temp_root.join("session.json");
    let exercises_dir = temp_root.join("exercises");
    std::env::set_var(
        "ZEDKATA_EXERCISES_DIR",
        exercises_dir.to_string_lossy().to_string(),
    );

    let (catalog, _) = Catalog::load_with_generated(&shipped_dir, Some(&generated_dir))?;

    // Select the generated exercise in session
    let mut session = SessionState::default();
    session.switch_exercise("gen-dw-launch".to_string(), 0);
    session.save_to_file(&session_file)?;

    // Prepare buffer
    let buffer_path =
        prepare_exercise_buffer(&catalog, "gen-dw-launch", "gen-dw-drill", &exercises_dir)?;

    assert!(buffer_path.exists());

    // Buffer content must match start fixture
    let buffer_content = std::fs::read_to_string(&buffer_path)?;
    assert_eq!(buffer_content, "const broken value = 42;\n");

    let attempt_id = "attempt_1001";
    session.start_attempt(
        attempt_id.to_string(),
        Some(buffer_path.to_string_lossy().to_string()),
    );
    session.save_to_file(&session_file)?;

    // Session state reflects active attempt and buffer path
    let loaded_session = SessionState::load_from_file(&session_file)?;
    assert_eq!(loaded_session.current.lesson_id, "gen-dw-launch");
    assert_eq!(
        loaded_session.current.buffer_path,
        Some(buffer_path.to_string_lossy().to_string())
    );
    assert!(loaded_session.attempt.is_some());

    std::env::remove_var("ZEDKATA_EXERCISES_DIR");
    let _ = std::fs::remove_dir_all(&temp_root);
    Ok(())
}

#[test]
fn test_lsp_feedback_and_scoring_uncalibrated_indication() -> Result<()> {
    let temp_root = create_temp_env("lsp_feedback_uncalibrated");
    let shipped_dir = temp_root.join("shipped");
    let generated_dir = temp_root.join("generated");
    std::fs::create_dir_all(&shipped_dir)?;
    std::fs::create_dir_all(&generated_dir)?;

    let mut candidate = sample_candidate_exercise("gen-dw-score", "gen-dw-drill");
    store_candidate_assets(&mut candidate, &generated_dir, None)?;

    let (catalog, _) = Catalog::load_with_generated(&shipped_dir, Some(&generated_dir))?;

    let session_file = temp_root.join("session.json");
    let mut session = SessionState::default();
    session.switch_exercise("gen-dw-score".to_string(), 0);
    session.save_to_file(&session_file)?;

    let buffer_path = temp_root.join("gen-dw-score-gen-dw-drill.kata");
    std::fs::write(&buffer_path, "const broken value = 42;\n")?;
    let uri = format!(
        "file:///{}",
        buffer_path.to_string_lossy().replace('\\', "/")
    );
    let buffer_path_str = buffer_path.to_string_lossy();

    let mut feedback = FeedbackEngine::new(session_file.clone(), catalog);

    // 1. Open document: initiates attempt
    let open_notif = feedback.on_document_open(&uri, &buffer_path_str)?;
    assert!(open_notif.is_some());

    // 2. Perform passing edit (dw removes 'broken ')
    let edited_text = "const value = 42;\n";
    let recorded_edits = vec![zedkata_core::session::RecordedEdit {
        at: chrono::Utc::now(),
        range: zedkata_core::session::EditRange {
            start: (0, 6),
            end: (0, 13),
        },
        inserted: String::new(),
        deleted_len: 7,
    }];

    let change_notif = feedback
        .on_document_change(&uri, edited_text, &recorded_edits)?
        .expect("feedback notification expected on passing attempt");

    // 3. Verify diagnostic indicates uncalibrated budget and generated practice
    let json_val = serde_json::to_value(&change_notif)?;
    let diag_msg = json_val["params"]["diagnostics"][0]["message"]
        .as_str()
        .expect("diagnostic message must be string");

    assert!(
        diag_msg.contains("par 1 (uncalibrated)"),
        "Diagnostic message must explicitly mark uncalibrated par budget: '{}'",
        diag_msg
    );
    assert!(
        diag_msg.contains("(generated practice)"),
        "Diagnostic message must indicate generated practice: '{}'",
        diag_msg
    );

    // 4. Verify session state history records completion
    let session_after = SessionState::load_from_file(&session_file)?;
    assert_eq!(session_after.history.len(), 1);
    let hist = &session_after.history[0];
    assert_eq!(hist.lesson_id, "gen-dw-score");
    assert_eq!(hist.exercise_id, "gen-dw-drill");
    assert!(hist.passed);
    assert_eq!(hist.par_edits, Some(1));
    assert_eq!(hist.actual_edits, 1);
    assert_eq!(hist.score, 100);

    let _ = std::fs::remove_dir_all(&temp_root);
    Ok(())
}

#[test]
fn test_tui_uncalibrated_budget_rendering_and_completion_badge() -> Result<()> {
    let temp_root = create_temp_env("tui_uncalibrated_render");
    let shipped_dir = temp_root.join("shipped");
    let generated_dir = temp_root.join("generated");
    std::fs::create_dir_all(&shipped_dir)?;
    std::fs::create_dir_all(&generated_dir)?;

    let mut candidate = sample_candidate_exercise("gen-dw-tui", "gen-dw-drill");
    store_candidate_assets(&mut candidate, &generated_dir, None)?;

    let session_file = temp_root.join("session.json");
    let mut session = SessionState::default();
    session.switch_exercise("gen-dw-tui".to_string(), 0);
    session.save_to_file(&session_file)?;

    let app = App::load_with_generated(
        session_file.clone(),
        shipped_dir.clone(),
        Some(generated_dir.clone()),
    );

    // 1. Initial uncompleted state: should have [G] badge and uncalibrated notices
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| ui::render(f, &app))?;
    let buffer = terminal.backend().buffer().clone();
    let content: String = buffer.content().iter().map(|c| c.symbol()).collect();

    // Must show [G] and [gen] for generated lesson in catalog pane
    assert!(
        content.contains("[G]") || content.contains("[gen]"),
        "Catalog pane must indicate generated lesson: '{}'",
        content
    );

    // Must show explicit uncalibrated budget notice in Task pane
    assert!(
        content.contains("uncalibrated"),
        "Task pane must display uncalibrated notice: '{}'",
        content
    );
    assert!(
        content.contains("Par: 1 edit(s) (uncalibrated)"),
        "Budget line must qualify uncalibrated Par: '{}'",
        content
    );

    // 2. Complete the exercise by appending a pass to session history
    let mut completed_session = SessionState::load_from_file(&session_file)?;
    completed_session
        .history
        .push(zedkata_core::session::HistoryEntry {
            lesson_id: "gen-dw-tui".to_string(),
            exercise_id: "gen-dw-drill".to_string(),
            passed: true,
            par_edits: Some(1),
            actual_edits: 1,
            score: 100,
            completed_at: chrono::Utc::now(),
        });
    completed_session.save_to_file(&session_file)?;

    // Reload app to verify completion
    let completed_app =
        App::load_with_generated(session_file.clone(), shipped_dir, Some(generated_dir));

    assert!(completed_app.is_lesson_completed(&candidate_to_lesson(&candidate)));

    terminal.draw(|f| ui::render(f, &completed_app))?;
    let buffer2 = terminal.backend().buffer().clone();
    let content2: String = buffer2.content().iter().map(|c| c.symbol()).collect();

    // After completion, catalog pane shows [✓]
    assert!(
        content2.contains("[✓]"),
        "Completed generated lesson must show [✓] badge: '{}'",
        content2
    );
    // Recent history line shows uncalibrated score
    assert!(
        content2.contains("score 100, uncalibrated"),
        "Progress pane must qualify score as uncalibrated: '{}'",
        content2
    );

    let _ = std::fs::remove_dir_all(&temp_root);
    Ok(())
}

fn candidate_to_lesson(candidate: &CandidateExercise) -> zedkata_core::lesson::Lesson {
    zedkata_core::lesson::Lesson::from_toml(&candidate.raw_toml, None)
        .expect("candidate toml is valid")
}

#[test]
fn test_select_and_complete_generated_practice_full_journey() -> Result<()> {
    let temp_root = create_temp_env("full_journey_gen");
    let shipped_dir = temp_root.join("shipped");
    let generated_dir = temp_root.join("generated");
    let exercises_dir = temp_root.join("exercises");
    std::fs::create_dir_all(&shipped_dir)?;
    std::fs::create_dir_all(&generated_dir)?;
    std::fs::create_dir_all(&exercises_dir)?;

    // 1. Author and store a targeted generated practice candidate
    let mut candidate = sample_candidate_exercise("gen-dw-journey", "gen-dw-drill");
    let _stored_paths = store_candidate_assets(&mut candidate, &generated_dir, None)?;

    // 2. Discover catalog outside repo
    let (catalog, summary) = Catalog::load_with_generated(&shipped_dir, Some(&generated_dir))?;
    assert!(catalog.get("gen-dw-journey").is_some());
    assert_eq!(summary.map(|s| s.added.len()), Some(1));

    // 3. Select generated exercise in session state
    let session_file = temp_root.join("session.json");
    let mut session = SessionState::default();
    session.switch_exercise("gen-dw-journey".to_string(), 0);
    session.save_to_file(&session_file)?;

    // 4. Prepare writable exercise buffer from generated fixture
    let buffer_path =
        prepare_exercise_buffer(&catalog, "gen-dw-journey", "gen-dw-drill", &exercises_dir)?;
    assert!(buffer_path.exists());
    let initial_content = std::fs::read_to_string(&buffer_path)?;
    assert_eq!(initial_content, "const broken value = 42;\n");

    // 5. Connect LSP FeedbackEngine and observe buffer didOpen
    let uri = format!(
        "file:///{}",
        buffer_path.to_string_lossy().replace('\\', "/")
    );
    let buffer_path_str = buffer_path.to_string_lossy();
    let mut feedback = FeedbackEngine::new(session_file.clone(), catalog.clone());

    let open_notif = feedback.on_document_open(&uri, &buffer_path_str)?;
    assert!(open_notif.is_some());

    let session_during = SessionState::load_from_file(&session_file)?;
    assert_eq!(session_during.current.lesson_id, "gen-dw-journey");
    assert!(session_during.attempt.is_some());

    // 6. Perform the intended modal motion (dw) to match goal
    let edited_text = "const value = 42;\n";
    let edits = vec![zedkata_core::session::RecordedEdit {
        at: chrono::Utc::now(),
        range: zedkata_core::session::EditRange {
            start: (0, 6),
            end: (0, 13),
        },
        inserted: String::new(),
        deleted_len: 7,
    }];

    let change_notif = feedback
        .on_document_change(&uri, edited_text, &edits)?
        .expect("change notification on pass");

    // Diagnostic message must explicitly distinguish uncalibrated budget
    let json_val = serde_json::to_value(&change_notif)?;
    let msg = json_val["params"]["diagnostics"][0]["message"]
        .as_str()
        .unwrap();
    assert!(msg.contains("par 1 (uncalibrated)"));
    assert!(msg.contains("(generated practice)"));

    // 7. Verify session history records completion
    let session_completed = SessionState::load_from_file(&session_file)?;
    assert_eq!(session_completed.history.len(), 1);
    assert!(session_completed.history[0].passed);
    assert_eq!(session_completed.history[0].score, 100);

    // 8. TUI renders completed [✓] badge and uncalibrated progress notes
    let app = App::load_with_generated(session_file.clone(), shipped_dir, Some(generated_dir));
    assert!(app.is_lesson_completed(&candidate_to_lesson(&candidate)));

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::render(f, &app))?;
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();

    assert!(content.contains("[✓]"));
    assert!(content.contains("score 100, uncalibrated"));

    let _ = std::fs::remove_dir_all(&temp_root);
    Ok(())
}
