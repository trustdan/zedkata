use std::path::PathBuf;
use zedkata_agent::{
    synthesize_default_candidate, validate_candidate, verify_path_within_root, CandidateExercise,
    CandidateStatus, GeneratedAssetError, WeaknessContext,
};
use zedkata_core::catalog::{default_data_dir, default_generated_dir, Catalog};

fn temp_test_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zedkata_test_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup_test_dir(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

fn create_valid_test_candidate(id: &str) -> CandidateExercise {
    let ctx = WeaknessContext {
        target_concept: "dw".to_string(),
        severity: Some(0.85),
        all_weaknesses: vec![("dw".to_string(), 0.85)],
        is_unscored_or_clean: false,
        recent_failures: vec![],
        session_path: None,
    };

    let mut candidate = synthesize_default_candidate(&ctx, Some(1729000000));
    candidate.lesson_id = id.to_string();
    candidate.exercise_id = format!("{}-drill", id);
    let start = format!(
        "fixtures/{}/{}.start.kata",
        candidate.lesson_id, candidate.exercise_id
    );
    let goal = format!(
        "fixtures/{}/{}.goal.kata",
        candidate.lesson_id, candidate.exercise_id
    );
    candidate.start_fixture_relative_path = start.clone();
    candidate.goal_fixture_relative_path = goal.clone();
    candidate.raw_toml = CandidateExercise::build_toml(
        &candidate.lesson_id,
        &candidate.title,
        &candidate.concepts,
        &candidate.prose,
        &candidate.exercise_id,
        &candidate.task,
        &start,
        &goal,
        candidate.par_edits,
        candidate.max_edits,
        candidate.cursor,
        candidate.hint.as_deref(),
        candidate.forbid_backspace_run,
    );
    candidate
}

#[test]
fn test_reject_malformed_toml() {
    let temp = temp_test_dir("malformed_toml");
    let mut candidate = create_valid_test_candidate("gen-malformed-toml");
    candidate.raw_toml = "[[broken toml unclosed string\nid = \"foo\"".to_string();

    let err = validate_candidate(&candidate, &temp).unwrap_err();
    match err {
        GeneratedAssetError::MalformedToml(msg) => {
            assert!(!msg.is_empty());
        }
        other => panic!("Expected MalformedToml error, got {:?}", other),
    }
    cleanup_test_dir(&temp);
}

#[test]
fn test_reject_invalid_rules() {
    let temp = temp_test_dir("invalid_rules");

    // 1. par_edits is 0
    let mut candidate_zero_par = create_valid_test_candidate("gen-zero-par");
    candidate_zero_par.par_edits = Some(0);
    candidate_zero_par.raw_toml = CandidateExercise::build_toml(
        &candidate_zero_par.lesson_id,
        &candidate_zero_par.title,
        &candidate_zero_par.concepts,
        &candidate_zero_par.prose,
        &candidate_zero_par.exercise_id,
        &candidate_zero_par.task,
        &candidate_zero_par.start_fixture_relative_path,
        &candidate_zero_par.goal_fixture_relative_path,
        Some(0),
        Some(2),
        candidate_zero_par.cursor,
        candidate_zero_par.hint.as_deref(),
        None,
    );
    let err = validate_candidate(&candidate_zero_par, &temp).unwrap_err();
    assert!(matches!(err, GeneratedAssetError::InvalidRules(_)));

    // 2. par_edits > max_edits
    let mut candidate_par_gt_max = create_valid_test_candidate("gen-par-gt-max");
    candidate_par_gt_max.par_edits = Some(5);
    candidate_par_gt_max.max_edits = Some(2);
    candidate_par_gt_max.raw_toml = CandidateExercise::build_toml(
        &candidate_par_gt_max.lesson_id,
        &candidate_par_gt_max.title,
        &candidate_par_gt_max.concepts,
        &candidate_par_gt_max.prose,
        &candidate_par_gt_max.exercise_id,
        &candidate_par_gt_max.task,
        &candidate_par_gt_max.start_fixture_relative_path,
        &candidate_par_gt_max.goal_fixture_relative_path,
        Some(5),
        Some(2),
        candidate_par_gt_max.cursor,
        candidate_par_gt_max.hint.as_deref(),
        None,
    );
    let err = validate_candidate(&candidate_par_gt_max, &temp).unwrap_err();
    assert!(matches!(err, GeneratedAssetError::InvalidRules(_)));

    // 3. 0-based cursor
    let mut candidate_zero_cursor = create_valid_test_candidate("gen-zero-cursor");
    candidate_zero_cursor.cursor = Some([0, 1]);
    candidate_zero_cursor.raw_toml = CandidateExercise::build_toml(
        &candidate_zero_cursor.lesson_id,
        &candidate_zero_cursor.title,
        &candidate_zero_cursor.concepts,
        &candidate_zero_cursor.prose,
        &candidate_zero_cursor.exercise_id,
        &candidate_zero_cursor.task,
        &candidate_zero_cursor.start_fixture_relative_path,
        &candidate_zero_cursor.goal_fixture_relative_path,
        Some(1),
        Some(2),
        Some([0, 1]),
        candidate_zero_cursor.hint.as_deref(),
        None,
    );
    let err = validate_candidate(&candidate_zero_cursor, &temp).unwrap_err();
    assert!(matches!(err, GeneratedAssetError::InvalidRules(_)));

    // 4. Non-draft lesson with unobservable forbid rule (e.g. visual_line_select)
    let mut candidate_unobservable = create_valid_test_candidate("gen-unobservable");
    candidate_unobservable
        .raw_toml
        .push_str("visual_line_select = true\n");
    let err = validate_candidate(&candidate_unobservable, &temp).unwrap_err();
    assert!(matches!(err, GeneratedAssetError::InvalidRules(_)));

    cleanup_test_dir(&temp);
}

#[test]
fn test_reject_missing_or_identical_fixtures() {
    let temp = temp_test_dir("missing_fixtures");

    // 1. Empty start buffer
    let mut candidate_empty_start = create_valid_test_candidate("gen-empty-start");
    candidate_empty_start.start_fixture_content = "   \n".to_string();
    let err = validate_candidate(&candidate_empty_start, &temp).unwrap_err();
    match err {
        GeneratedAssetError::MissingFixture(msg) => {
            assert!(msg.contains("Start fixture buffer content cannot be empty"));
        }
        other => panic!("Expected MissingFixture, got {:?}", other),
    }

    // 2. Empty goal buffer
    let mut candidate_empty_goal = create_valid_test_candidate("gen-empty-goal");
    candidate_empty_goal.goal_fixture_content = String::new();
    let err = validate_candidate(&candidate_empty_goal, &temp).unwrap_err();
    match err {
        GeneratedAssetError::MissingFixture(msg) => {
            assert!(msg.contains("Goal fixture buffer content cannot be empty"));
        }
        other => panic!("Expected MissingFixture, got {:?}", other),
    }

    // 3. Start and goal buffers identical (degenerate drill)
    let mut candidate_identical = create_valid_test_candidate("gen-identical");
    candidate_identical.start_fixture_content = "fn main() {}\n".to_string();
    candidate_identical.goal_fixture_content = "fn main() {}\n".to_string();
    let err = validate_candidate(&candidate_identical, &temp).unwrap_err();
    match err {
        GeneratedAssetError::MissingFixture(msg) => {
            assert!(msg.contains("identical"));
        }
        other => panic!(
            "Expected MissingFixture for identical buffers, got {:?}",
            other
        ),
    }

    cleanup_test_dir(&temp);
}

#[test]
fn test_reject_path_traversal_escaping_content_root() {
    let temp = temp_test_dir("traversal");

    // Direct helper check
    let err = verify_path_within_root(&temp, "../../etc/passwd").unwrap_err();
    assert!(matches!(err, GeneratedAssetError::PathTraversal { .. }));

    let err = verify_path_within_root(&temp, "fixtures/../../../secret.kata").unwrap_err();
    assert!(matches!(err, GeneratedAssetError::PathTraversal { .. }));

    // Windows / Unix absolute paths
    let err = verify_path_within_root(&temp, "/etc/passwd").unwrap_err();
    assert!(matches!(err, GeneratedAssetError::PathTraversal { .. }));

    // Candidate with traversing start fixture path
    let mut candidate_traversal = create_valid_test_candidate("gen-traversal");
    candidate_traversal.start_fixture_relative_path = "../../etc/shadow.kata".to_string();
    let err = validate_candidate(&candidate_traversal, &temp).unwrap_err();
    assert!(matches!(err, GeneratedAssetError::PathTraversal { .. }));

    // Candidate with malicious lesson ID
    let mut candidate_bad_id = create_valid_test_candidate("gen-bad-id");
    candidate_bad_id.lesson_id = "../evil".to_string();
    let err = validate_candidate(&candidate_bad_id, &temp).unwrap_err();
    assert!(matches!(err, GeneratedAssetError::PathTraversal { .. }));

    cleanup_test_dir(&temp);
}

#[test]
fn test_id_collision_preserves_existing_lesson_on_disk() {
    let temp = temp_test_dir("collision_disk");

    // Create an existing lesson on disk
    let existing_id = "gen-existing-123";
    let existing_toml_path = temp.join(format!("{}.toml", existing_id));
    std::fs::write(
        &existing_toml_path,
        "# PREEXISTING LESSON - MUST NEVER BE OVERWRITTEN",
    )
    .unwrap();

    let mut candidate = create_valid_test_candidate(existing_id);
    assert_eq!(candidate.status, CandidateStatus::Candidate);

    let err = candidate.validate_and_store(&temp).unwrap_err();
    match err {
        GeneratedAssetError::IdCollision {
            lesson_id,
            existing_path,
        } => {
            assert_eq!(lesson_id, existing_id);
            assert_eq!(existing_path, existing_toml_path);
        }
        other => panic!("Expected IdCollision error, got {:?}", other),
    }

    // Verify existing file is untouched
    let content = std::fs::read_to_string(&existing_toml_path).unwrap();
    assert_eq!(content, "# PREEXISTING LESSON - MUST NEVER BE OVERWRITTEN");

    cleanup_test_dir(&temp);
}

#[test]
fn test_id_collision_with_existing_catalog_preserves_existing_lesson() {
    let temp = temp_test_dir("collision_catalog");

    // Load actual shipped catalog from workspace lessons directory
    let shipped_lessons_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");
    let catalog = Catalog::load_from_dir(&shipped_lessons_dir).expect("Shipped catalog loads");

    // Attempt to store a candidate with the same ID as shipped lesson 'operators-01-edit'
    let mut candidate = create_valid_test_candidate("operators-01-edit");
    let err = candidate
        .validate_and_store_with_catalog(&temp, Some(&catalog))
        .unwrap_err();

    match err {
        GeneratedAssetError::IdCollision { lesson_id, .. } => {
            assert_eq!(lesson_id, "operators-01-edit");
        }
        other => panic!("Expected IdCollision error, got {:?}", other),
    }

    // Ensure no files were written to root
    assert!(!temp.join("operators-01-edit.toml").exists());

    cleanup_test_dir(&temp);
}

#[test]
fn test_valid_assets_write_together_atomically() {
    let temp = temp_test_dir("atomic_write");

    let mut candidate = create_valid_test_candidate("gen-dw-99999");
    assert_eq!(candidate.status, CandidateStatus::Candidate);

    let stored = candidate
        .validate_and_store(&temp)
        .expect("Storage should succeed");
    assert_eq!(candidate.status, CandidateStatus::Persisted);

    // Verify written paths
    assert!(stored.toml_path.is_file());
    assert!(stored.start_fixture_path.is_file());
    assert!(stored.goal_fixture_path.is_file());

    // Verify file contents match candidate
    let read_toml = std::fs::read_to_string(&stored.toml_path).unwrap();
    assert_eq!(read_toml, candidate.raw_toml);

    let read_start = std::fs::read_to_string(&stored.start_fixture_path).unwrap();
    assert_eq!(read_start, candidate.start_fixture_content);

    let read_goal = std::fs::read_to_string(&stored.goal_fixture_path).unwrap();
    assert_eq!(read_goal, candidate.goal_fixture_content);

    // Verify catalog can load this directory
    let catalog = Catalog::load_from_dir(&temp).expect("Catalog loads generated dir");
    assert!(catalog.get("gen-dw-99999").is_some());

    // Verify fixture resolution works
    let resolved_start = catalog
        .resolve_fixture("gen-dw-99999", &candidate.exercise_id, true)
        .expect("Resolves start fixture");
    assert_eq!(resolved_start, stored.start_fixture_path);

    let resolved_goal = catalog
        .resolve_fixture("gen-dw-99999", &candidate.exercise_id, false)
        .expect("Resolves goal fixture");
    assert_eq!(resolved_goal, stored.goal_fixture_path);

    // Verify runnable validation passes completely
    catalog
        .validate_runnable()
        .expect("Runnable validation passes");

    cleanup_test_dir(&temp);
}

#[test]
fn test_catalog_merge_preserves_existing_on_collision() {
    let temp_shipped = temp_test_dir("merge_shipped");
    let temp_generated = temp_test_dir("merge_generated");

    // 1. Shipped dir with lesson-a
    let mut candidate_a = create_valid_test_candidate("lesson-a");
    candidate_a.validate_and_store(&temp_shipped).unwrap();

    let mut catalog = Catalog::load_from_dir(&temp_shipped).unwrap();
    assert_eq!(catalog.all_lessons().len(), 1);

    // 2. Generated dir with duplicate lesson-a AND new lesson-b
    let mut candidate_a_dup = create_valid_test_candidate("lesson-a");
    candidate_a_dup.title = "Duplicate lesson-a title".to_string();
    candidate_a_dup.raw_toml = CandidateExercise::build_toml(
        &candidate_a_dup.lesson_id,
        "Duplicate lesson-a title",
        &candidate_a_dup.concepts,
        &candidate_a_dup.prose,
        &candidate_a_dup.exercise_id,
        &candidate_a_dup.task,
        &candidate_a_dup.start_fixture_relative_path,
        &candidate_a_dup.goal_fixture_relative_path,
        candidate_a_dup.par_edits,
        candidate_a_dup.max_edits,
        candidate_a_dup.cursor,
        candidate_a_dup.hint.as_deref(),
        candidate_a_dup.forbid_backspace_run,
    );
    candidate_a_dup.validate_and_store(&temp_generated).unwrap();

    let mut candidate_b = create_valid_test_candidate("lesson-b");
    candidate_b.validate_and_store(&temp_generated).unwrap();

    // 3. Merge generated dir into catalog
    let summary = catalog.merge_from_dir(&temp_generated).unwrap();

    assert_eq!(summary.added, vec!["lesson-b"]);
    assert_eq!(summary.skipped_collisions, vec!["lesson-a"]);

    // Verify lesson-a in catalog is preserved from shipped dir
    let entry_a = catalog.get("lesson-a").unwrap();
    assert_eq!(entry_a.path, temp_shipped.join("lesson-a.toml"));
    assert_ne!(entry_a.lesson.title, "Duplicate lesson-a title");

    // Verify lesson-b is in catalog and points to generated dir
    let entry_b = catalog.get("lesson-b").unwrap();
    assert_eq!(entry_b.path, temp_generated.join("lesson-b.toml"));

    cleanup_test_dir(&temp_shipped);
    cleanup_test_dir(&temp_generated);
}

#[test]
fn test_default_generated_dir_resolution() {
    let temp = temp_test_dir("default_dir_res");

    // Test ZEDKATA_GENERATED_DIR override
    std::env::set_var("ZEDKATA_GENERATED_DIR", temp.to_string_lossy().to_string());
    let resolved = default_generated_dir().expect("Resolves override");
    assert_eq!(resolved, temp);

    // Test ZEDKATA_DATA_DIR override
    std::env::remove_var("ZEDKATA_GENERATED_DIR");
    std::env::set_var("ZEDKATA_DATA_DIR", temp.to_string_lossy().to_string());
    let resolved_data = default_data_dir().expect("Resolves data dir");
    assert_eq!(resolved_data, temp);

    let resolved_gen = default_generated_dir().expect("Resolves generated dir under data dir");
    assert_eq!(resolved_gen, temp.join("lessons").join("generated"));

    // Cleanup env
    std::env::remove_var("ZEDKATA_DATA_DIR");
    cleanup_test_dir(&temp);
}

#[test]
fn test_cli_generate_store_flag() {
    let temp = temp_test_dir("cli_store");

    // Run zedkata-agent generate --concept dw --store --out-dir <temp>
    let binary = env!("CARGO_BIN_EXE_zedkata-agent");
    let status = std::process::Command::new(binary)
        .arg("generate")
        .arg("--concept")
        .arg("dw")
        .arg("--store")
        .arg("--out-dir")
        .arg(&temp)
        .status()
        .expect("Executes zedkata-agent generate");

    assert!(status.success(), "CLI generate --store should succeed");

    // Check that at least one .toml was written to temp
    let entries: Vec<_> = std::fs::read_dir(&temp)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let toml_count = entries
        .iter()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .count();
    assert_eq!(toml_count, 1, "Expected exactly 1 TOML file stored");

    // Check fixtures directory
    assert!(
        temp.join("fixtures").is_dir(),
        "Expected fixtures/ directory created"
    );

    cleanup_test_dir(&temp);
}
