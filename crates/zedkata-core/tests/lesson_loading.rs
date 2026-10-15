use std::path::Path;
use zedkata_core::{Catalog, CoreError, Lesson};

#[test]
fn test_valid_lesson_parsing() {
    let toml_content = r#"
[lesson]
id = "test-lesson"
title = "Test Lesson"
module = 1
requires = ["base-lesson"]
concepts = ["dw", "cw"]
prose = "Practice basic word operations."

[[exercise]]
id = "ex-1"
task = "Delete the word"
start = "fixtures/ex1.start.kata"
goal = "fixtures/ex1.goal.kata"
par_edits = 1
max_edits = 2
hint = "Use dw."

[exercise.forbid]
backspace_run = 3

[[walkthrough]]
id = "wt-1"
title = "Word boundaries"
prompt = "Observe the word boundary behavior."
"#;

    let lesson =
        Lesson::from_toml(toml_content, Some(Path::new("dummy.toml"))).expect("parse failed");
    assert_eq!(lesson.id, "test-lesson");
    assert_eq!(lesson.title, "Test Lesson");
    assert_eq!(lesson.module, Some(1));
    assert_eq!(lesson.requires, vec!["base-lesson"]);
    assert_eq!(lesson.concepts, vec!["dw", "cw"]);
    assert_eq!(lesson.prose, "Practice basic word operations.");
    assert!(!lesson.draft);

    assert_eq!(lesson.exercises.len(), 1);
    let ex = &lesson.exercises[0];
    assert_eq!(ex.id, "ex-1");
    assert_eq!(ex.task, "Delete the word");
    assert_eq!(ex.start, "fixtures/ex1.start.kata");
    assert_eq!(ex.goal, "fixtures/ex1.goal.kata");
    assert_eq!(ex.par_edits, Some(1));
    assert_eq!(ex.max_edits, Some(2));
    assert_eq!(ex.hint.as_deref(), Some("Use dw."));
    assert_eq!(ex.forbid.as_ref().and_then(|f| f.backspace_run), Some(3));

    assert_eq!(lesson.walkthroughs.len(), 1);
    let wt = &lesson.walkthroughs[0];
    assert_eq!(wt.id, "wt-1");
    assert_eq!(wt.title, "Word boundaries");
    assert_eq!(wt.prompt, "Observe the word boundary behavior.");
}

#[test]
fn test_malformed_toml_reports_syntax_context() {
    let malformed = r#"
[lesson
id = "unclosed-bracket"
"#;
    let err = Lesson::from_toml(malformed, Some(Path::new("bad.toml"))).unwrap_err();
    match err {
        CoreError::LessonParseError { path, line, .. } => {
            assert_eq!(path.as_deref(), Some(Path::new("bad.toml")));
            assert!(line.is_some());
        }
        other => panic!("Expected LessonParseError, got: {:?}", other),
    }
}

#[test]
fn test_invalid_field_type_reports_context() {
    let bad_type = r#"
[lesson]
id = "bad-type"
title = "Bad Type"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "fixtures/ex.start.kata"
goal = "fixtures/ex.goal.kata"
par_edits = "one"
"#;
    let err = Lesson::from_toml(bad_type, Some(Path::new("bad_type.toml"))).unwrap_err();
    match err {
        CoreError::LessonParseError { path, .. } => {
            assert_eq!(path.as_deref(), Some(Path::new("bad_type.toml")));
        }
        other => panic!("Expected LessonParseError, got: {:?}", other),
    }
}

#[test]
fn test_unsupported_forbid_rule_rejected_with_field_context() {
    let unobservable = r#"
[lesson]
id = "unsupported-forbid"
title = "Unsupported"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "fixtures/ex.start.kata"
goal = "fixtures/ex.goal.kata"

[exercise.forbid]
visual_line_select = true
"#;
    let err = Lesson::from_toml(unobservable, Some(Path::new("unsupported.toml"))).unwrap_err();
    match err {
        CoreError::LessonValidationError {
            path,
            field,
            message,
        } => {
            assert_eq!(path.as_deref(), Some(Path::new("unsupported.toml")));
            assert_eq!(field, "exercise.ex-1.forbid.visual_line_select");
            assert!(message.contains("unobservable"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_unknown_forbid_rule_rejected_with_field_context() {
    let unknown_rule = r#"
[lesson]
id = "unknown-rule"
title = "Unknown Rule"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "fixtures/ex.start.kata"
goal = "fixtures/ex.goal.kata"

[exercise.forbid]
magic_trick = 42
"#;
    let err = Lesson::from_toml(unknown_rule, Some(Path::new("unknown.toml"))).unwrap_err();
    match err {
        CoreError::LessonValidationError {
            path,
            field,
            message,
        } => {
            assert_eq!(path.as_deref(), Some(Path::new("unknown.toml")));
            assert_eq!(field, "exercise.ex-1.forbid.magic_trick");
            assert!(message.contains("unknown or unsupported"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_invalid_budget_par_greater_than_max() {
    let invalid_budget = r#"
[lesson]
id = "bad-budget"
title = "Bad Budget"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "fixtures/ex.start.kata"
goal = "fixtures/ex.goal.kata"
par_edits = 5
max_edits = 2
"#;
    let err = Lesson::from_toml(invalid_budget, Some(Path::new("budget.toml"))).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.par_edits");
            assert!(message.contains("cannot exceed max_edits"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_duplicate_exercise_id_in_lesson() {
    let duplicate_ex = r#"
[lesson]
id = "dup-ex"
title = "Duplicate Exercises"
prose = "Prose"

[[exercise]]
id = "same-id"
task = "Task 1"
start = "fixtures/1.start.kata"
goal = "fixtures/1.goal.kata"

[[exercise]]
id = "same-id"
task = "Task 2"
start = "fixtures/2.start.kata"
goal = "fixtures/2.goal.kata"
"#;
    let err = Lesson::from_toml(duplicate_ex, Some(Path::new("dup.toml"))).unwrap_err();
    match err {
        CoreError::DuplicateExerciseId {
            lesson_id,
            exercise_id,
        } => {
            assert_eq!(lesson_id, "dup-ex");
            assert_eq!(exercise_id, "same-id");
        }
        other => panic!("Expected DuplicateExerciseId, got: {:?}", other),
    }
}

#[test]
fn test_catalog_detects_duplicate_lesson_ids() {
    let dir = tempfile_dir();
    let file1 = dir.join("lesson1.toml");
    let file2 = dir.join("lesson2.toml");

    let content = r#"
[lesson]
id = "duplicate-lesson"
title = "Duplicate Lesson"
prose = "Prose"
"#;
    std::fs::write(&file1, content).unwrap();
    std::fs::write(&file2, content).unwrap();

    let err = Catalog::load_from_dir(&dir).unwrap_err();
    match err {
        CoreError::DuplicateLessonId { id, .. } => {
            assert_eq!(id, "duplicate-lesson");
        }
        other => panic!("Expected DuplicateLessonId, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_catalog_detects_missing_prerequisite() {
    let dir = tempfile_dir();
    let file1 = dir.join("lesson1.toml");

    let content = r#"
[lesson]
id = "lesson-a"
title = "Lesson A"
requires = ["non-existent-lesson"]
prose = "Prose"
"#;
    std::fs::write(&file1, content).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_catalog().unwrap_err();
    match err {
        CoreError::MissingPrerequisite {
            lesson_id,
            prerequisite,
        } => {
            assert_eq!(lesson_id, "lesson-a");
            assert_eq!(prerequisite, "non-existent-lesson");
        }
        other => panic!("Expected MissingPrerequisite, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_catalog_detects_prerequisite_cycle() {
    let dir = tempfile_dir();
    let file1 = dir.join("lesson_a.toml");
    let file2 = dir.join("lesson_b.toml");

    let content_a = r#"
[lesson]
id = "lesson-a"
title = "Lesson A"
requires = ["lesson-b"]
prose = "Prose A"
"#;
    let content_b = r#"
[lesson]
id = "lesson-b"
title = "Lesson B"
requires = ["lesson-a"]
prose = "Prose B"
"#;
    std::fs::write(&file1, content_a).unwrap();
    std::fs::write(&file2, content_b).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_catalog().unwrap_err();
    match err {
        CoreError::PrerequisiteCycle { cycle } => {
            assert!(cycle.contains(&"lesson-a".to_string()));
            assert!(cycle.contains(&"lesson-b".to_string()));
        }
        other => panic!("Expected PrerequisiteCycle, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_catalog_detects_missing_fixture_in_runnable_lesson() {
    let dir = tempfile_dir();
    let lesson_file = dir.join("runnable.toml");

    let content = r#"
[lesson]
id = "runnable-missing-fix"
title = "Runnable"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "fixtures/missing.start.kata"
goal = "fixtures/missing.goal.kata"
"#;
    std::fs::write(&lesson_file, content).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_runnable().unwrap_err();
    match err {
        CoreError::MissingFixture {
            lesson_id,
            exercise_id,
            path,
        } => {
            assert_eq!(lesson_id, "runnable-missing-fix");
            assert_eq!(exercise_id, "ex-1");
            assert!(path.ends_with("missing.start.kata"));
        }
        other => panic!("Expected MissingFixture, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_draft_lessons_excluded_from_runnable_checks() {
    let dir = tempfile_dir();
    let draft_file = dir.join("draft.toml");

    let content = r#"
[lesson]
id = "draft-lesson"
title = "Draft Lesson"
draft = true
requires = ["missing-prereq-is-fine-for-draft"]
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "fixtures/missing.start.kata"
goal = "fixtures/missing.goal.kata"
"#;
    std::fs::write(&draft_file, content).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    assert_eq!(catalog.all_lessons().len(), 1);
    assert_eq!(catalog.runnable_lessons().len(), 0);

    // validate_runnable passes because there are no runnable lessons requiring fixtures
    assert!(catalog.validate_runnable().is_ok());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_fixture_resolution_independent_of_cwd() {
    // Load workspace lessons catalog
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");

    let start_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .expect("resolve start failed");
    let goal_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", false)
        .expect("resolve goal failed");

    assert!(
        start_path.is_file(),
        "start fixture must exist at {:?}",
        start_path
    );
    assert!(
        goal_path.is_file(),
        "goal fixture must exist at {:?}",
        goal_path
    );
    assert!(start_path.is_absolute());
    assert!(goal_path.is_absolute());

    let ex2_start = catalog
        .resolve_fixture("motions-01-word", "delete-to-word-end", true)
        .expect("resolve delete-to-word-end start failed");
    let ex2_goal = catalog
        .resolve_fixture("motions-01-word", "delete-to-word-end", false)
        .expect("resolve delete-to-word-end goal failed");

    assert!(
        ex2_start.is_file(),
        "delete-to-word-end start fixture must exist"
    );
    assert!(
        ex2_goal.is_file(),
        "delete-to-word-end goal fixture must exist"
    );
    assert!(ex2_start.is_absolute());
    assert!(ex2_goal.is_absolute());

    let ex3_start = catalog
        .resolve_fixture("motions-01-word", "delete-word-binding", true)
        .expect("resolve delete-word-binding start failed");
    let ex3_goal = catalog
        .resolve_fixture("motions-01-word", "delete-word-binding", false)
        .expect("resolve delete-word-binding goal failed");

    assert!(
        ex3_start.is_file(),
        "delete-word-binding start fixture must exist"
    );
    assert!(
        ex3_goal.is_file(),
        "delete-word-binding goal fixture must exist"
    );
    assert!(ex3_start.is_absolute());
    assert!(ex3_goal.is_absolute());
}

#[test]
fn test_workspace_production_lessons_validate() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");

    // All lessons include modal-00-baseline, motions-01-word, motions-02-find, and motions-03-textobjects
    let all = catalog.all_lessons();
    assert!(all.iter().any(|e| e.lesson.id == "modal-00-baseline"));
    assert!(all.iter().any(|e| e.lesson.id == "motions-01-word"));
    assert!(all.iter().any(|e| e.lesson.id == "motions-02-find"));
    assert!(all.iter().any(|e| e.lesson.id == "motions-03-textobjects"));

    // Runnable lessons must include modal-00-baseline, motions-01-word, motions-02-find, and motions-03-textobjects
    let runnable = catalog.runnable_lessons();
    assert_eq!(runnable.len(), 4);
    assert!(runnable.iter().any(|e| e.lesson.id == "modal-00-baseline"));
    assert!(runnable.iter().any(|e| e.lesson.id == "motions-01-word"));
    assert!(runnable.iter().any(|e| e.lesson.id == "motions-02-find"));
    assert!(runnable
        .iter()
        .any(|e| e.lesson.id == "motions-03-textobjects"));

    // validate_runnable must succeed with zero errors
    catalog
        .validate_runnable()
        .expect("runnable validation failed");
}

#[test]
fn test_four_module_runnable_curriculum_admission_and_prerequisites() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");

    // 1. All 4 curriculum modules must be admitted and runnable
    let runnable = catalog.runnable_lessons();
    assert_eq!(
        runnable.len(),
        4,
        "Runnable curriculum must contain exactly 4 modules"
    );

    let ids: Vec<&str> = runnable.iter().map(|e| e.lesson.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "modal-00-baseline",
            "motions-01-word",
            "motions-02-find",
            "motions-03-textobjects",
        ]
    );

    // 2. Validate prerequisites graph & draft flags
    let m0 = catalog.get("modal-00-baseline").unwrap();
    assert_eq!(m0.lesson.module, Some(0));
    assert!(m0.lesson.requires.is_empty());
    assert!(!m0.lesson.draft);
    assert_eq!(m0.lesson.exercises.len(), 3);

    let m1 = catalog.get("motions-01-word").unwrap();
    assert_eq!(m1.lesson.module, Some(1));
    assert!(m1.lesson.requires.is_empty());
    assert!(!m1.lesson.draft);
    assert_eq!(m1.lesson.exercises.len(), 3);

    let m2 = catalog.get("motions-02-find").unwrap();
    assert_eq!(m2.lesson.module, Some(2));
    assert_eq!(m2.lesson.requires, vec!["motions-01-word"]);
    assert!(!m2.lesson.draft);
    assert_eq!(m2.lesson.exercises.len(), 3);

    let m3 = catalog.get("motions-03-textobjects").unwrap();
    assert_eq!(m3.lesson.module, Some(3));
    assert_eq!(
        m3.lesson.requires,
        vec!["motions-01-word", "motions-02-find"]
    );
    assert!(!m3.lesson.draft);
    assert_eq!(m3.lesson.exercises.len(), 3);

    // 3. Runnable validation passes with zero errors
    assert!(catalog.validate_runnable().is_ok());

    // 4. Verify all 12 drills have existing, non-empty start and goal fixtures
    let mut total_drills = 0;
    for entry in &runnable {
        for ex in &entry.lesson.exercises {
            total_drills += 1;
            let start = catalog
                .resolve_fixture(&entry.lesson.id, &ex.id, true)
                .unwrap();
            let goal = catalog
                .resolve_fixture(&entry.lesson.id, &ex.id, false)
                .unwrap();
            assert!(start.is_file(), "Start fixture must exist: {:?}", start);
            assert!(goal.is_file(), "Goal fixture must exist: {:?}", goal);
            let start_content = std::fs::read_to_string(&start).unwrap();
            let goal_content = std::fs::read_to_string(&goal).unwrap();
            assert!(!start_content.is_empty());
            assert!(!goal_content.is_empty());
            assert_ne!(
                start_content, goal_content,
                "Start and goal must differ for drill: {}",
                ex.id
            );
        }
    }
    assert_eq!(
        total_drills, 12,
        "Must have exactly 12 runnable drills across the 4 modules"
    );
}

#[test]
fn test_find_motions_lesson_fixtures_and_schema() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");

    let entry = catalog
        .get("motions-02-find")
        .expect("motions-02-find lesson must exist in catalog");
    let lesson = &entry.lesson;

    assert_eq!(lesson.id, "motions-02-find");
    assert_eq!(lesson.title, "Character find motions");
    assert_eq!(lesson.module, Some(2));
    assert!(!lesson.draft, "motions-02-find must be runnable, not draft");
    assert_eq!(lesson.requires, vec!["motions-01-word"]);
    assert_eq!(lesson.exercises.len(), 3);
    assert_eq!(lesson.walkthroughs.len(), 1);

    // Verify all 3 exercise fixtures exist and resolve cleanly
    for ex in &lesson.exercises {
        assert!(ex.par_edits.is_some());
        assert!(ex.max_edits.is_some());
        assert!(ex.cursor.is_some());
        assert!(ex.hint.is_some());

        let start_path = catalog
            .resolve_fixture(&lesson.id, &ex.id, true)
            .expect("start fixture must resolve");
        let goal_path = catalog
            .resolve_fixture(&lesson.id, &ex.id, false)
            .expect("goal fixture must resolve");

        assert!(
            start_path.exists(),
            "start fixture must exist: {:?}",
            start_path
        );
        assert!(
            goal_path.exists(),
            "goal fixture must exist: {:?}",
            goal_path
        );
        assert_ne!(
            std::fs::read_to_string(&start_path).unwrap(),
            std::fs::read_to_string(&goal_path).unwrap(),
            "start and goal fixtures must differ"
        );
    }

    // Verify walkthrough
    assert_eq!(lesson.walkthroughs[0].id, "single-line-find-scope");
}

#[test]
fn test_baseline_lesson_modal_00_baseline_fixtures_and_schema() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");

    let entry = catalog
        .get("modal-00-baseline")
        .expect("modal-00-baseline lesson must exist in catalog");
    let lesson = &entry.lesson;

    assert_eq!(lesson.id, "modal-00-baseline");
    assert_eq!(lesson.title, "Modal baseline");
    assert_eq!(lesson.module, Some(0));
    assert!(
        !lesson.draft,
        "modal-00-baseline must be runnable, not draft"
    );
    assert!(
        lesson.requires.is_empty(),
        "baseline lesson has no prerequisites"
    );
    assert_eq!(lesson.exercises.len(), 3);
    assert_eq!(lesson.walkthroughs.len(), 1);

    // Verify all 3 exercise fixtures exist and resolve cleanly
    for ex in &lesson.exercises {
        assert!(ex.par_edits.is_some());
        assert!(ex.max_edits.is_some());
        assert!(ex.cursor.is_some());
        assert!(ex.hint.is_some());

        let start_path = catalog
            .resolve_fixture(&lesson.id, &ex.id, true)
            .expect("start fixture must resolve");
        let goal_path = catalog
            .resolve_fixture(&lesson.id, &ex.id, false)
            .expect("goal fixture must resolve");

        assert!(
            start_path.exists(),
            "start fixture must exist: {:?}",
            start_path
        );
        assert!(
            goal_path.exists(),
            "goal fixture must exist: {:?}",
            goal_path
        );
        assert_ne!(
            std::fs::read_to_string(&start_path).unwrap(),
            std::fs::read_to_string(&goal_path).unwrap(),
            "start and goal fixtures must differ"
        );
    }

    // Verify walkthrough
    assert_eq!(lesson.walkthroughs[0].id, "diagnostic-navigation");
}

#[test]
fn test_reference_lesson_motions_03_textobjects_fixtures_and_schema() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");
    let entry = catalog
        .get("motions-03-textobjects")
        .expect("motions-03-textobjects must exist in catalog");
    let lesson = &entry.lesson;

    assert_eq!(lesson.module, Some(3));
    assert_eq!(lesson.requires, vec!["motions-01-word", "motions-02-find"]);
    assert!(!lesson.draft);
    assert_eq!(lesson.exercises.len(), 3);
    assert_eq!(lesson.walkthroughs.len(), 1);
    assert_eq!(lesson.walkthroughs[0].id, "mini-vs-any");

    // Expected exercise IDs
    let expected_ids = ["swap-string-contents", "gut-the-block", "nested-brackets"];
    for (i, expected_id) in expected_ids.iter().enumerate() {
        let ex = &lesson.exercises[i];
        assert_eq!(&ex.id, expected_id);
        assert_eq!(ex.par_edits, Some(1));
        assert!(ex.max_edits.is_some());
        assert!(ex.cursor.is_some());

        // Forbid rules must only contain supported rules (e.g. backspace_run), no visual_line_select
        if let Some(ref forbid) = ex.forbid {
            assert_eq!(forbid.backspace_run, Some(3));
            assert!(
                forbid.unsupported.is_empty(),
                "forbid rules must not contain unsupported entries"
            );
        }

        // Fixtures must exist and be non-empty
        let start_path = catalog
            .resolve_fixture(&lesson.id, &ex.id, true)
            .expect("must resolve start fixture");
        let goal_path = catalog
            .resolve_fixture(&lesson.id, &ex.id, false)
            .expect("must resolve goal fixture");

        assert!(
            start_path.is_file(),
            "start fixture must exist: {:?}",
            start_path
        );
        assert!(
            goal_path.is_file(),
            "goal fixture must exist: {:?}",
            goal_path
        );

        let start_content = std::fs::read_to_string(&start_path).expect("read start fixture");
        let goal_content = std::fs::read_to_string(&goal_path).expect("read goal fixture");

        assert!(!start_content.trim().is_empty());
        assert!(!goal_content.trim().is_empty());
        assert_ne!(
            start_content, goal_content,
            "start and goal fixtures must differ"
        );

        // Verify cursor line is valid within the start buffer
        let cursor = ex.cursor.unwrap();
        let start_lines = start_content.lines().count();
        assert!(
            (cursor[0] as usize) <= start_lines,
            "cursor line {} exceeds start buffer line count {}",
            cursor[0],
            start_lines
        );
    }
}

#[test]
fn test_exercise_cursor_coordinates_parsed_and_validated() {
    let toml = r#"
[lesson]
id = "cursor-lesson"
title = "Cursor Lesson"
prose = "Testing cursor coordinates"

[[exercise]]
id = "with-cursor"
task = "Test with cursor"
start = "fixtures/start.kata"
goal = "fixtures/goal.kata"
cursor = [3, 12]

[[exercise]]
id = "without-cursor"
task = "Test without cursor"
start = "fixtures/start.kata"
goal = "fixtures/goal.kata"
"#;

    let lesson = Lesson::from_toml(toml, None).expect("failed to parse valid cursor toml");
    assert_eq!(lesson.exercises[0].cursor, Some([3, 12]));
    assert_eq!(lesson.exercises[1].cursor, None);

    // 0-based line is rejected
    let invalid_line = r#"
[lesson]
id = "invalid-cursor-line"
title = "Invalid"
prose = "Prose"

[[exercise]]
id = "zero-line"
task = "Zero line"
start = "fixtures/start.kata"
goal = "fixtures/goal.kata"
cursor = [0, 5]
"#;
    let err = Lesson::from_toml(invalid_line, None).unwrap_err();
    assert!(err.to_string().contains("1-based"));

    // 0-based col is rejected
    let invalid_col = r#"
[lesson]
id = "invalid-cursor-col"
title = "Invalid"
prose = "Prose"

[[exercise]]
id = "zero-col"
task = "Zero col"
start = "fixtures/start.kata"
goal = "fixtures/goal.kata"
cursor = [1, 0]
"#;
    let err = Lesson::from_toml(invalid_col, None).unwrap_err();
    assert!(err.to_string().contains("1-based"));
}

#[test]
fn test_is_lesson_completed_and_prerequisite_unlocking() {
    let dir = tempfile_dir();
    let l1_file = dir.join("lesson1.toml");
    let l2_file = dir.join("lesson2.toml");

    let l1_content = r#"
[lesson]
id = "lesson-1"
title = "Lesson 1"
requires = []
prose = "Prose 1"

[[exercise]]
id = "ex-1a"
task = "Task 1a"
start = "fixtures/1a.start.kata"
goal = "fixtures/1a.goal.kata"

[[exercise]]
id = "ex-1b"
task = "Task 1b"
start = "fixtures/1b.start.kata"
goal = "fixtures/1b.goal.kata"
"#;

    let l2_content = r#"
[lesson]
id = "lesson-2"
title = "Lesson 2"
requires = ["lesson-1", "external-lesson"]
prose = "Prose 2"

[[exercise]]
id = "ex-2a"
task = "Task 2a"
start = "fixtures/2a.start.kata"
goal = "fixtures/2a.goal.kata"
"#;

    std::fs::write(&l1_file, l1_content).unwrap();
    std::fs::write(&l2_file, l2_content).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let now = chrono::Utc::now();

    // 1. Initially, history is empty
    let mut history = Vec::new();
    assert!(!catalog.is_lesson_completed("lesson-1", &history));
    let (unlocked, missing) = catalog.is_lesson_unlocked("lesson-2", &history);
    assert!(!unlocked);
    assert_eq!(missing, vec!["lesson-1", "external-lesson"]);

    // 2. Only 1 of 2 exercises passed for lesson-1 -> lesson-1 is NOT completed
    history.push(zedkata_core::scoring::create_history_entry(
        "lesson-1",
        "ex-1a",
        true,
        Some(1),
        1,
        100,
        now,
    ));
    assert!(!catalog.is_lesson_completed("lesson-1", &history));
    let (unlocked, missing) = catalog.is_lesson_unlocked("lesson-2", &history);
    assert!(!unlocked);
    assert_eq!(missing, vec!["lesson-1", "external-lesson"]);

    // 3. Second exercise attempted but failed -> still NOT completed
    history.push(zedkata_core::scoring::create_history_entry(
        "lesson-1",
        "ex-1b",
        false,
        Some(1),
        3,
        0,
        now,
    ));
    assert!(!catalog.is_lesson_completed("lesson-1", &history));
    let (unlocked, _) = catalog.is_lesson_unlocked("lesson-2", &history);
    assert!(!unlocked);

    // 4. Second exercise passed -> lesson-1 IS completed
    history.push(zedkata_core::scoring::create_history_entry(
        "lesson-1",
        "ex-1b",
        true,
        Some(1),
        1,
        100,
        now,
    ));
    assert!(catalog.is_lesson_completed("lesson-1", &history));

    // lesson-2 is still locked because external-lesson is missing
    let (unlocked, missing) = catalog.is_lesson_unlocked("lesson-2", &history);
    assert!(!unlocked);
    assert_eq!(missing, vec!["external-lesson"]);

    // 5. External lesson has a failed attempt -> does NOT satisfy prerequisite
    history.push(zedkata_core::scoring::create_history_entry(
        "external-lesson",
        "dummy",
        false,
        None,
        2,
        0,
        now,
    ));
    let (unlocked, missing) = catalog.is_lesson_unlocked("lesson-2", &history);
    assert!(!unlocked);
    assert_eq!(missing, vec!["external-lesson"]);

    // 6. External lesson passes -> lesson-2 is unlocked!
    history.push(zedkata_core::scoring::create_history_entry(
        "external-lesson",
        "dummy",
        true,
        None,
        1,
        100,
        now,
    ));
    let (unlocked, missing) = catalog.is_lesson_unlocked("lesson-2", &history);
    assert!(unlocked);
    assert!(missing.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_reference_lesson_admission_and_prerequisite_validation() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    // 1. Production catalog: draft=false now that motions-01-word and motions-02-find are both runnable
    let catalog = Catalog::load_from_dir(&lessons_dir).expect("failed to load lessons catalog");
    assert!(catalog.validate_runnable().is_ok());

    let entry = catalog.get("motions-03-textobjects").unwrap();
    assert!(!entry.lesson.draft);
    assert_eq!(
        entry.lesson.requires,
        vec!["motions-01-word", "motions-02-find"]
    );

    // 2. Admission readiness test:
    // Create an isolated catalog dir where motions-01-word, motions-02-find, and motions-03-textobjects are present.
    // When motions-03-textobjects has draft = false and prerequisites exist, validate_runnable passes with 0 errors.
    let temp_dir = tempfile_dir();
    let fixtures_dir = temp_dir.join("fixtures").join("motions-03");
    std::fs::create_dir_all(&fixtures_dir).unwrap();

    // Copy motions-03 fixtures to temp
    for file in &[
        "swap-string.start.kata",
        "swap-string.goal.kata",
        "gut-block.start.kata",
        "gut-block.goal.kata",
        "nested.start.kata",
        "nested.goal.kata",
    ] {
        std::fs::copy(
            lessons_dir.join("fixtures").join("motions-03").join(file),
            fixtures_dir.join(file),
        )
        .unwrap();
    }

    // Copy motions-01-word and its fixtures
    let m1_fixtures_dir = temp_dir.join("fixtures").join("motions-01");
    std::fs::create_dir_all(&m1_fixtures_dir).unwrap();
    for file in &[
        "delete-word-arg.start.kata",
        "delete-word-arg.goal.kata",
        "delete-to-word-end.start.kata",
        "delete-to-word-end.goal.kata",
        "delete-word-binding.start.kata",
        "delete-word-binding.goal.kata",
    ] {
        std::fs::copy(
            lessons_dir.join("fixtures").join("motions-01").join(file),
            m1_fixtures_dir.join(file),
        )
        .unwrap();
    }
    std::fs::copy(
        lessons_dir.join("motions-01-word.toml"),
        temp_dir.join("motions-01-word.toml"),
    )
    .unwrap();

    // Write stub motions-02-find (runnable)
    let m2_fixtures_dir = temp_dir.join("fixtures").join("motions-02");
    std::fs::create_dir_all(&m2_fixtures_dir).unwrap();
    std::fs::write(m2_fixtures_dir.join("find.start.kata"), "start").unwrap();
    std::fs::write(m2_fixtures_dir.join("find.goal.kata"), "goal").unwrap();
    let m2_content = r#"
[lesson]
id = "motions-02-find"
title = "Find motions"
module = 2
requires = ["motions-01-word"]
concepts = ["f", "t"]
prose = "Practice character finding."

[[exercise]]
id = "find-char"
task = "Find character"
start = "fixtures/motions-02/find.start.kata"
goal = "fixtures/motions-02/find.goal.kata"
par_edits = 1
max_edits = 2
"#;
    std::fs::write(temp_dir.join("motions-02-find.toml"), m2_content).unwrap();

    // Write motions-03-textobjects with draft = false (admitted)
    let m3_source =
        std::fs::read_to_string(lessons_dir.join("motions-03-textobjects.toml")).unwrap();
    let m3_admitted = m3_source.replace("draft = true", "draft = false");
    std::fs::write(temp_dir.join("motions-03-textobjects.toml"), m3_admitted).unwrap();

    let admitted_catalog = Catalog::load_from_dir(&temp_dir).unwrap();
    let admitted_runnable = admitted_catalog.runnable_lessons();
    assert_eq!(admitted_runnable.len(), 3);
    assert!(admitted_catalog.validate_runnable().is_ok());

    // 3. Negative check: if prerequisite motions-02-find is absent, setting draft = false fails validation
    std::fs::remove_file(temp_dir.join("motions-02-find.toml")).unwrap();
    let missing_prereq_catalog = Catalog::load_from_dir(&temp_dir).unwrap();
    match missing_prereq_catalog.validate_runnable() {
        Err(CoreError::MissingPrerequisite {
            lesson_id,
            prerequisite,
        }) => {
            assert_eq!(lesson_id, "motions-03-textobjects");
            assert_eq!(prerequisite, "motions-02-find");
        }
        other => panic!("expected MissingPrerequisite error, got {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn tempfile_dir() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("zedkata_test_{}", nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
