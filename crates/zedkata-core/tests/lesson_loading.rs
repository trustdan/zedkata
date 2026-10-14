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

    // All lessons include motions-01-word and motions-03-textobjects
    let all = catalog.all_lessons();
    assert!(all.iter().any(|e| e.lesson.id == "motions-01-word"));
    assert!(all.iter().any(|e| e.lesson.id == "motions-03-textobjects"));

    // Runnable lessons must only include motions-01-word
    let runnable = catalog.runnable_lessons();
    assert_eq!(runnable.len(), 1);
    assert_eq!(runnable[0].lesson.id, "motions-01-word");

    // validate_runnable must succeed with zero errors
    catalog
        .validate_runnable()
        .expect("runnable validation failed");
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
