use std::path::{Path, PathBuf};
use zedkata_core::{Catalog, CoreError, Lesson};

fn lessons_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons")
}

fn temp_catalog_dir() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("zedkata_catalog_val_{}", nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ===========================================================================
// Shipped Catalog Validation Tests (Positive Suite)
// ===========================================================================

#[test]
fn test_shipped_catalog_integrity() {
    let dir = lessons_dir();
    assert!(dir.is_dir(), "Lessons directory must exist: {:?}", dir);

    let catalog = Catalog::load_from_dir(&dir).expect("Shipped catalog must load without errors");

    // 1. Verify total lesson count across curriculum
    let all = catalog.all_lessons();
    assert_eq!(
        all.len(),
        10,
        "Shipped catalog must contain exactly 10 authored modules"
    );

    // 2. Verify runnable vs draft split
    let runnable = catalog.runnable_lessons();
    assert_eq!(
        runnable.len(),
        5,
        "Catalog must contain exactly 5 runnable drill modules"
    );

    let draft_count = all.iter().filter(|e| e.lesson.draft).count();
    assert_eq!(
        draft_count, 5,
        "Catalog must contain exactly 5 draft walkthrough modules"
    );

    // 3. Verify validate_shipped_catalog passes cleanly
    catalog
        .validate_shipped_catalog()
        .expect("validate_shipped_catalog must pass on shipped catalog");

    // 4. Verify all 5 runnable modules in detail
    let expected_runnable_modules = [
        ("modal-00-baseline", 0, 3),
        ("motions-01-word", 1, 3),
        ("motions-02-find", 2, 3),
        ("motions-03-textobjects", 3, 3),
        ("operators-01-edit", 4, 3),
    ];

    for (id, module_num, ex_count) in expected_runnable_modules {
        let entry = catalog
            .get(id)
            .unwrap_or_else(|| panic!("Runnable lesson '{}' must exist", id));
        assert!(
            !entry.lesson.draft,
            "Lesson '{}' must have draft = false",
            id
        );
        assert_eq!(
            entry.lesson.module,
            Some(module_num),
            "Lesson '{}' module number mismatch",
            id
        );
        assert_eq!(
            entry.lesson.exercises.len(),
            ex_count,
            "Lesson '{}' exercise count mismatch",
            id
        );

        // Verify each exercise in runnable lesson
        for ex in &entry.lesson.exercises {
            // Task and hint
            assert!(
                !ex.task.trim().is_empty(),
                "Exercise '{}' task empty",
                ex.id
            );
            assert!(ex.hint.is_some(), "Exercise '{}' must have hint", ex.id);

            // Fixtures on disk and non-empty
            let start = catalog.resolve_fixture(id, &ex.id, true).unwrap();
            let goal = catalog.resolve_fixture(id, &ex.id, false).unwrap();
            assert!(
                start.is_file(),
                "Start fixture missing for '{}/{}': {:?}",
                id,
                ex.id,
                start
            );
            assert!(
                goal.is_file(),
                "Goal fixture missing for '{}/{}': {:?}",
                id,
                ex.id,
                goal
            );

            let start_content = std::fs::read_to_string(&start).unwrap();
            let goal_content = std::fs::read_to_string(&goal).unwrap();
            assert!(
                !start_content.trim().is_empty(),
                "Start fixture empty for '{}/{}'",
                id,
                ex.id
            );
            assert!(
                !goal_content.trim().is_empty(),
                "Goal fixture empty for '{}/{}'",
                id,
                ex.id
            );

            // Calibrated budget
            assert!(
                ex.par_edits.is_some(),
                "Exercise '{}/{}' must have par_edits",
                id,
                ex.id
            );
            assert!(
                ex.max_edits.is_some(),
                "Exercise '{}/{}' must have max_edits",
                id,
                ex.id
            );
            let par = ex.par_edits.unwrap();
            let max = ex.max_edits.unwrap();
            assert!(
                par <= max,
                "par_edits ({}) > max_edits ({}) for '{}/{}'",
                par,
                max,
                id,
                ex.id
            );

            // Forbid rules
            let forbid = ex
                .forbid
                .as_ref()
                .expect("Runnable drill must have forbid table");
            assert!(
                forbid.backspace_run.is_some(),
                "Exercise '{}/{}' must configure forbid.backspace_run",
                id,
                ex.id
            );

            // 1-based cursor coordinates
            if let Some(cursor) = ex.cursor {
                assert!(
                    cursor[0] >= 1,
                    "Cursor line must be >= 1 for '{}/{}'",
                    id,
                    ex.id
                );
                assert!(
                    cursor[1] >= 1,
                    "Cursor column must be >= 1 for '{}/{}'",
                    id,
                    ex.id
                );
            }
        }
    }

    // 5. Verify all 5 draft walkthrough modules in detail
    let expected_walkthrough_modules = [
        ("zed-01-lsp-actions", 6, "lsp/sample_service.rs"),
        ("zed-02-project-panel", 7, "project_panel"),
        ("zed-03-multibuffer", 8, "multibuffer"),
        ("zed-04-tasks-and-debugger", 9, "tasks_and_debugger"),
        ("zed-05-agent-panel", 10, "agent_panel"),
    ];

    for (id, module_num, fixture_subpath) in expected_walkthrough_modules {
        let entry = catalog
            .get(id)
            .unwrap_or_else(|| panic!("Walkthrough lesson '{}' must exist", id));
        assert!(
            entry.lesson.draft,
            "Walkthrough lesson '{}' must have draft = true",
            id
        );
        assert_eq!(
            entry.lesson.module,
            Some(module_num),
            "Lesson '{}' module number mismatch",
            id
        );
        assert!(
            !entry.lesson.walkthroughs.is_empty(),
            "Walkthrough lesson '{}' must have at least 1 walkthrough",
            id
        );

        let fixture_target = dir
            .join("fixtures")
            .join("walkthroughs")
            .join(fixture_subpath);
        assert!(
            fixture_target.exists(),
            "Walkthrough fixture missing for '{}': {:?}",
            id,
            fixture_target
        );
    }
}

#[test]
fn test_shipped_catalog_prerequisite_dag_acyclicity() {
    let dir = lessons_dir();
    let catalog = Catalog::load_from_dir(&dir).expect("Failed to load catalog");

    // validate_catalog validates DAG acyclicity across ALL 10 lessons
    catalog
        .validate_catalog()
        .expect("Full catalog prerequisite DAG must be acyclic");

    // validate_runnable validates DAG acyclicity and draft exclusion for all 5 runnable lessons
    catalog
        .validate_runnable()
        .expect("Runnable catalog prerequisite DAG must be acyclic with clean draft exclusion");
}

// ===========================================================================
// Negative Validation Tests (Failure Modes & Actionable Diagnostics)
// ===========================================================================

#[test]
fn test_validation_rejects_missing_prerequisite() {
    let dir = temp_catalog_dir();
    let file = dir.join("broken.toml");

    let content = r#"
[lesson]
id = "orphan-lesson"
title = "Orphan"
prose = "Prose"
requires = ["non-existent-prereq"]

[[exercise]]
id = "ex-1"
task = "Do something"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;
    std::fs::write(&file, content).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_catalog().unwrap_err();

    match err {
        CoreError::MissingPrerequisite {
            lesson_id,
            prerequisite,
        } => {
            assert_eq!(lesson_id, "orphan-lesson");
            assert_eq!(prerequisite, "non-existent-prereq");
        }
        other => panic!("Expected MissingPrerequisite, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_validation_rejects_prerequisite_cycle_direct() {
    let dir = temp_catalog_dir();

    let a_content = r#"
[lesson]
id = "lesson-a"
title = "A"
prose = "Prose"
requires = ["lesson-b"]

[[exercise]]
id = "ex-1"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    let b_content = r#"
[lesson]
id = "lesson-b"
title = "B"
prose = "Prose"
requires = ["lesson-a"]

[[exercise]]
id = "ex-1"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    std::fs::write(dir.join("a.toml"), a_content).unwrap();
    std::fs::write(dir.join("b.toml"), b_content).unwrap();

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
fn test_validation_rejects_prerequisite_cycle_transitive() {
    let dir = temp_catalog_dir();

    let a = r#"
[lesson]
id = "node-a"
title = "A"
prose = "Prose"
requires = ["node-b"]

[[exercise]]
id = "ex"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    let b = r#"
[lesson]
id = "node-b"
title = "B"
prose = "Prose"
requires = ["node-c"]

[[exercise]]
id = "ex"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    let c = r#"
[lesson]
id = "node-c"
title = "C"
prose = "Prose"
requires = ["node-a"]

[[exercise]]
id = "ex"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    std::fs::write(dir.join("a.toml"), a).unwrap();
    std::fs::write(dir.join("b.toml"), b).unwrap();
    std::fs::write(dir.join("c.toml"), c).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_catalog().unwrap_err();

    match err {
        CoreError::PrerequisiteCycle { cycle } => {
            assert!(cycle.contains(&"node-a".to_string()));
            assert!(cycle.contains(&"node-b".to_string()));
            assert!(cycle.contains(&"node-c".to_string()));
        }
        other => panic!("Expected PrerequisiteCycle, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_validation_rejects_prerequisite_self_cycle() {
    let dir = temp_catalog_dir();

    let self_loop = r#"
[lesson]
id = "loop"
title = "Loop"
prose = "Prose"
requires = ["loop"]

[[exercise]]
id = "ex"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    std::fs::write(dir.join("loop.toml"), self_loop).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_catalog().unwrap_err();

    match err {
        CoreError::PrerequisiteCycle { cycle } => {
            assert!(cycle.contains(&"loop".to_string()));
        }
        other => panic!("Expected PrerequisiteCycle, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_validation_rejects_runnable_depending_on_draft() {
    let dir = temp_catalog_dir();

    let draft_lesson = r#"
[lesson]
id = "draft-prereq"
title = "Draft Prereq"
prose = "Prose"
draft = true

[[walkthrough]]
id = "wt-1"
title = "Walkthrough"
prompt = "Prompt"
"#;

    let runnable_lesson = r#"
[lesson]
id = "runnable-dependent"
title = "Runnable Dependent"
prose = "Prose"
draft = false
requires = ["draft-prereq"]

[[exercise]]
id = "ex-1"
task = "Task"
start = "ex.start.kata"
goal = "ex.goal.kata"
"#;

    std::fs::write(dir.join("draft.toml"), draft_lesson).unwrap();
    std::fs::write(dir.join("runnable.toml"), runnable_lesson).unwrap();
    std::fs::write(dir.join("ex.start.kata"), "start").unwrap();
    std::fs::write(dir.join("ex.goal.kata"), "goal").unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_runnable().unwrap_err();

    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "lesson.requires");
            assert!(message.contains("cannot depend on draft lesson"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_validation_rejects_missing_start_fixture() {
    let dir = temp_catalog_dir();

    let content = r#"
[lesson]
id = "missing-start"
title = "Missing Start"
prose = "Prose"
draft = false

[[exercise]]
id = "ex-1"
task = "Task"
start = "non_existent.start.kata"
goal = "existing.goal.kata"
"#;
    std::fs::write(dir.join("lesson.toml"), content).unwrap();
    std::fs::write(dir.join("existing.goal.kata"), "goal").unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_runnable().unwrap_err();

    match err {
        CoreError::MissingFixture {
            lesson_id,
            exercise_id,
            path,
        } => {
            assert_eq!(lesson_id, "missing-start");
            assert_eq!(exercise_id, "ex-1");
            assert!(path.to_string_lossy().contains("non_existent.start.kata"));
        }
        other => panic!("Expected MissingFixture, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_validation_rejects_missing_goal_fixture() {
    let dir = temp_catalog_dir();

    let content = r#"
[lesson]
id = "missing-goal"
title = "Missing Goal"
prose = "Prose"
draft = false

[[exercise]]
id = "ex-1"
task = "Task"
start = "existing.start.kata"
goal = "non_existent.goal.kata"
"#;
    std::fs::write(dir.join("lesson.toml"), content).unwrap();
    std::fs::write(dir.join("existing.start.kata"), "start").unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_runnable().unwrap_err();

    match err {
        CoreError::MissingFixture {
            lesson_id,
            exercise_id,
            path,
        } => {
            assert_eq!(lesson_id, "missing-goal");
            assert_eq!(exercise_id, "ex-1");
            assert!(path.to_string_lossy().contains("non_existent.goal.kata"));
        }
        other => panic!("Expected MissingFixture, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_validation_rejects_fixture_path_traversal() {
    let toml = r#"
[lesson]
id = "traversal"
title = "Traversal"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "../outside.kata"
goal = "goal.kata"
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.start");
            assert!(message.contains("parent directory traversal"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_unobservable_forbid_rule() {
    let toml = r#"
[lesson]
id = "unobservable-rule"
title = "Unobservable Rule"
prose = "Prose"
draft = false

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"

[exercise.forbid]
visual_line_select = true
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.forbid.visual_line_select");
            assert!(message.contains("unobservable"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_unknown_forbid_rule() {
    let toml = r#"
[lesson]
id = "unknown-rule"
title = "Unknown Rule"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"

[exercise.forbid]
teleportation = true
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.forbid.teleportation");
            assert!(message.contains("unknown or unsupported"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_invalid_budget_par_greater_than_max() {
    let toml = r#"
[lesson]
id = "bad-budget"
title = "Bad Budget"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"
par_edits = 4
max_edits = 2
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.par_edits");
            assert!(message.contains("cannot exceed max_edits"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_zero_par_edits() {
    let toml = r#"
[lesson]
id = "zero-par"
title = "Zero Par"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"
par_edits = 0
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.par_edits");
            assert!(message.contains("greater than 0"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_zero_max_edits() {
    let toml = r#"
[lesson]
id = "zero-max"
title = "Zero Max"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"
max_edits = 0
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.max_edits");
            assert!(message.contains("greater than 0"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_zero_based_cursor() {
    let toml_line_zero = r#"
[lesson]
id = "cursor-zero"
title = "Cursor Zero"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"
cursor = [0, 5]
"#;
    let err = Lesson::from_toml(toml_line_zero, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.cursor");
            assert!(message.contains("1-based"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }

    let toml_col_zero = r#"
[lesson]
id = "cursor-zero-col"
title = "Cursor Zero Col"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "Task"
start = "start.kata"
goal = "goal.kata"
cursor = [5, 0]
"#;
    let err = Lesson::from_toml(toml_col_zero, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.cursor");
            assert!(message.contains("1-based"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_empty_task() {
    let toml = r#"
[lesson]
id = "empty-task"
title = "Empty Task"
prose = "Prose"

[[exercise]]
id = "ex-1"
task = "   "
start = "start.kata"
goal = "goal.kata"
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "exercise.ex-1.task");
            assert!(message.contains("cannot be empty"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_empty_title() {
    let toml = r#"
[lesson]
id = "empty-title"
title = ""
prose = "Prose"
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "lesson.title");
            assert!(message.contains("cannot be empty"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_empty_prose() {
    let toml = r#"
[lesson]
id = "empty-prose"
title = "Title"
prose = "   "
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "lesson.prose");
            assert!(message.contains("cannot be empty"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_duplicate_exercise_ids() {
    let toml = r#"
[lesson]
id = "dup-ex"
title = "Duplicate Exercises"
prose = "Prose"

[[exercise]]
id = "same-id"
task = "Task 1"
start = "1.start.kata"
goal = "1.goal.kata"

[[exercise]]
id = "same-id"
task = "Task 2"
start = "2.start.kata"
goal = "2.goal.kata"
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
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
fn test_validation_rejects_duplicate_walkthrough_ids() {
    let toml = r#"
[lesson]
id = "dup-wt"
title = "Duplicate Walkthroughs"
prose = "Prose"

[[walkthrough]]
id = "same-wt"
title = "WT 1"
prompt = "Prompt 1"

[[walkthrough]]
id = "same-wt"
title = "WT 2"
prompt = "Prompt 2"
"#;
    let err = Lesson::from_toml(toml, None).unwrap_err();
    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "walkthrough.same-wt.id");
            assert!(message.contains("Duplicate walkthrough ID"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }
}

#[test]
fn test_validation_rejects_runnable_with_no_exercises() {
    let dir = temp_catalog_dir();

    let content = r#"
[lesson]
id = "no-exercises"
title = "No Exercises"
prose = "Prose"
draft = false
"#;
    std::fs::write(dir.join("empty.toml"), content).unwrap();

    let catalog = Catalog::load_from_dir(&dir).unwrap();
    let err = catalog.validate_runnable().unwrap_err();

    match err {
        CoreError::LessonValidationError { field, message, .. } => {
            assert_eq!(field, "lesson.exercise");
            assert!(message.contains("must contain at least one drillable exercise"));
        }
        other => panic!("Expected LessonValidationError, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&dir);
}
