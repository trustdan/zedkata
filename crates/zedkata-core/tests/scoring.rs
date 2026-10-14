use chrono::Utc;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use zedkata_core::{
    calculate_score, check_forbid_rules, count_edit_units, create_history_entry, update_weaknesses,
    verify_attempt, Attempt, AttemptOutcome, EditRange, Exercise, ForbidRules, RecordedEdit,
};

fn create_edit(
    start: (u32, u32),
    end: (u32, u32),
    deleted_len: usize,
    inserted: &str,
) -> RecordedEdit {
    RecordedEdit {
        at: Utc::now(),
        range: EditRange { start, end },
        inserted: inserted.to_string(),
        deleted_len,
    }
}

#[test]
fn test_count_edit_units_operator_deletion() {
    // `dw` deleting 6 characters `alpha `
    let edits = vec![create_edit((1, 17), (1, 23), 6, "")];
    assert_eq!(count_edit_units(&edits), 1);
}

#[test]
fn test_count_edit_units_discrete_backspaces() {
    // 5 backspaces deleting characters one-by-one at decrementing columns
    let edits = vec![
        create_edit((2, 23), (2, 24), 1, ""),
        create_edit((2, 22), (2, 23), 1, ""),
        create_edit((2, 21), (2, 22), 1, ""),
        create_edit((2, 20), (2, 21), 1, ""),
        create_edit((2, 19), (2, 20), 1, ""),
    ];
    assert_eq!(count_edit_units(&edits), 5);
}

#[test]
fn test_count_edit_units_contiguous_typing_burst() {
    // Typing "hello" character by character in insert mode
    let edits = vec![
        create_edit((1, 0), (1, 0), 0, "h"),
        create_edit((1, 1), (1, 1), 0, "e"),
        create_edit((1, 2), (1, 2), 0, "l"),
        create_edit((1, 3), (1, 3), 0, "l"),
        create_edit((1, 4), (1, 4), 0, "o"),
    ];
    assert_eq!(count_edit_units(&edits), 1);
}

#[test]
fn test_count_edit_units_non_contiguous_typing() {
    // Typing "he" on line 1, then cursor jumps to line 2 to type "llo"
    let edits = vec![
        create_edit((1, 0), (1, 0), 0, "h"),
        create_edit((1, 1), (1, 1), 0, "e"),
        create_edit((2, 0), (2, 0), 0, "l"),
        create_edit((2, 1), (2, 1), 0, "l"),
        create_edit((2, 2), (2, 2), 0, "o"),
    ];
    assert_eq!(count_edit_units(&edits), 2);
}

#[test]
fn test_count_edit_units_change_and_type() {
    // `ci"`: deletes 10 characters inside quotes, then types "world" contiguously
    let edits = vec![
        create_edit((1, 5), (1, 15), 10, ""),
        create_edit((1, 5), (1, 5), 0, "w"),
        create_edit((1, 6), (1, 6), 0, "o"),
        create_edit((1, 7), (1, 7), 0, "r"),
        create_edit((1, 8), (1, 8), 0, "l"),
        create_edit((1, 9), (1, 9), 0, "d"),
    ];
    // Deletion + contiguous typing in insert mode coalesces to 1 edit unit (par_edits = 1)
    assert_eq!(count_edit_units(&edits), 1);
}

#[test]
fn test_count_edit_units_undo_inversion() {
    // Deletion followed immediately by restoration
    let edits = vec![
        create_edit((1, 17), (1, 23), 6, ""),       // delete `alpha `
        create_edit((1, 17), (1, 17), 0, "alpha "), // restore `alpha ` (undo)
    ];
    assert_eq!(count_edit_units(&edits), 0);

    // Deletion, undo, then correct deletion
    let edits_with_retry = vec![
        create_edit((1, 17), (1, 23), 6, ""),       // mistake delete
        create_edit((1, 17), (1, 17), 0, "alpha "), // undo
        create_edit((1, 17), (1, 23), 6, ""),       // correct delete
    ];
    assert_eq!(count_edit_units(&edits_with_retry), 1);
}

#[test]
fn test_check_forbid_rules_backspace_run() {
    let rules = ForbidRules {
        backspace_run: Some(3),
        unsupported: vec![],
    };

    // 2 backspaces: under limit of 3
    let under = vec![
        create_edit((2, 23), (2, 24), 1, ""),
        create_edit((2, 22), (2, 23), 1, ""),
    ];
    assert!(check_forbid_rules(&under, Some(&rules)).is_ok());

    // 3 backspaces: hits limit of 3 -> Err
    let at_limit = vec![
        create_edit((2, 23), (2, 24), 1, ""),
        create_edit((2, 22), (2, 23), 1, ""),
        create_edit((2, 21), (2, 22), 1, ""),
    ];
    let err = check_forbid_rules(&at_limit, Some(&rules)).expect_err("should fail");
    assert!(err.contains("Triggered forbidden pattern: backspace run of 3"));

    // Interrupted run: 2 backspaces, 1 typing, 2 backspaces -> passes
    let interrupted = vec![
        create_edit((2, 23), (2, 24), 1, ""),
        create_edit((2, 22), (2, 23), 1, ""),
        create_edit((2, 22), (2, 22), 0, "a"),
        create_edit((2, 22), (2, 23), 1, ""),
        create_edit((2, 21), (2, 22), 1, ""),
    ];
    assert!(check_forbid_rules(&interrupted, Some(&rules)).is_ok());
}

#[test]
fn test_check_forbid_rules_unsupported() {
    let rules = ForbidRules {
        backspace_run: None,
        unsupported: vec!["visual_line_select".to_string()],
    };
    let edits = vec![create_edit((1, 0), (1, 1), 1, "")];
    let err = check_forbid_rules(&edits, Some(&rules)).expect_err("should fail");
    assert!(err.contains("Unsupported forbid rule"));
}

#[test]
fn test_calculate_score_budget_and_goal_only() {
    // Goal-only mode (par_edits is None)
    let (passed, score) = calculate_score(10, None, None);
    assert!(passed);
    assert_eq!(score, 100);

    // Exact par
    let (passed, score) = calculate_score(1, Some(1), Some(3));
    assert!(passed);
    assert_eq!(score, 100);

    // Over par within max: par 1, actual 2 -> 100 - 15 = 85
    let (passed, score) = calculate_score(2, Some(1), Some(3));
    assert!(passed);
    assert_eq!(score, 85);

    // Over par within max: par 1, actual 3 -> 100 - 30 = 70
    let (passed, score) = calculate_score(3, Some(1), Some(3));
    assert!(passed);
    assert_eq!(score, 70);

    // Exceeds max: par 1, max 3, actual 4 -> fails with score 0
    let (passed, score) = calculate_score(4, Some(1), Some(3));
    assert!(!passed);
    assert_eq!(score, 0);

    // Score floor: par 1, max 10, actual 6 -> penalty 75, clamped to floor 50
    let (passed, score) = calculate_score(6, Some(1), Some(10));
    assert!(passed);
    assert_eq!(score, 50);
}

#[test]
fn test_update_weaknesses() {
    let concepts = vec!["word-motions".to_string(), "text-objects".to_string()];
    let current = HashMap::new();

    // 1. Initial pass with 100 score -> weakness 0.0
    let w1 = update_weaknesses(&current, &concepts, true, 100);
    assert_eq!(w1.get("word-motions"), Some(&0.0));
    assert_eq!(w1.get("text-objects"), Some(&0.0));

    // 2. Failure -> weakness increases
    let w2 = update_weaknesses(&w1, &concepts, false, 0);
    // 0.0 * 0.7 + 1.0 * 0.3 = 0.30
    assert_eq!(w2.get("word-motions"), Some(&0.3));

    // 3. Another failure -> weakness increases further
    let w3 = update_weaknesses(&w2, &concepts, false, 0);
    // 0.3 * 0.7 + 1.0 * 0.3 = 0.51
    assert_eq!(w3.get("word-motions"), Some(&0.51));

    // 4. Recovery: pass with 100 -> weakness drops
    let w4 = update_weaknesses(&w3, &concepts, true, 100);
    // 0.51 * 0.7 + 0.0 = 0.36
    assert_eq!(w4.get("word-motions"), Some(&0.36));

    // 5. Empty concepts leaves map intact
    let w5 = update_weaknesses(&w4, &[], false, 0);
    assert_eq!(w5, w4);
}

#[test]
fn test_create_history_entry() {
    let now = Utc::now();
    let entry = create_history_entry("motions-01-word", "delete-word", true, Some(1), 1, 100, now);
    assert_eq!(entry.lesson_id, "motions-01-word");
    assert_eq!(entry.exercise_id, "delete-word");
    assert!(entry.passed);
    assert_eq!(entry.par_edits, Some(1));
    assert_eq!(entry.actual_edits, 1);
    assert_eq!(entry.score, 100);
    assert_eq!(entry.completed_at, now);
}

#[test]
fn test_verify_attempt_integration_forbid_and_scoring() {
    let exercise = Exercise {
        id: "test-ex".to_string(),
        task: "Delete the word".to_string(),
        start: "fixtures/start.kata".to_string(),
        goal: "fixtures/goal.kata".to_string(),
        par_edits: Some(1),
        max_edits: Some(2),
        hint: None,
        forbid: Some(ForbidRules {
            backspace_run: Some(3),
            unsupported: vec![],
        }),
        cursor: None,
    };

    let current = "alpha bravo\n";
    let goal = "bravo\n";

    // Scenario A: Student did 5 backspaces -> fails immediately on forbid rule
    let mut attempt = Attempt {
        id: "att-1".to_string(),
        edit_count: 5,
        edits: vec![
            create_edit((1, 5), (1, 6), 1, ""),
            create_edit((1, 4), (1, 5), 1, ""),
            create_edit((1, 3), (1, 4), 1, ""),
            create_edit((1, 2), (1, 3), 1, ""),
            create_edit((1, 1), (1, 2), 1, ""),
        ],
        verdict: None,
    };

    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    match outcome {
        AttemptOutcome::Failed { reason } => {
            assert!(reason.contains("backspace run"));
        }
        other => panic!("Expected Failed, got: {:?}", other),
    }

    // Scenario B: Student did 1 `dw` and reached goal -> passes with score 100
    attempt.edit_count = 1;
    attempt.edits = vec![create_edit((1, 0), (1, 6), 6, "")];
    let outcome = verify_attempt(goal, goal, &attempt, &exercise);
    assert_eq!(
        outcome,
        AttemptOutcome::Passed {
            edits: 1,
            par: Some(1),
            score: 100,
        }
    );
}

// ---------------------------------------------------------------------------------
// Replay Phase 0 Golden Matrix LSP Fixtures
// ---------------------------------------------------------------------------------

fn extract_did_change_edits_from_fixture(
    rel_path: &str,
    target_uri_suffix: &str,
) -> Vec<RecordedEdit> {
    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to find repo root");
    let file_path = repo_root.join(rel_path);

    let file = File::open(&file_path)
        .unwrap_or_else(|err| panic!("Failed to open fixture {:?}: {}", file_path, err));
    let reader = BufReader::new(file);

    let mut edits = Vec::new();

    for line_res in reader.lines() {
        let line = line_res.expect("failed to read line");
        let val: serde_json::Value = serde_json::from_str(&line).expect("invalid json line");

        if val["direction"] == "in" && val["message"]["method"] == "textDocument/didChange" {
            let uri = val["message"]["params"]["textDocument"]["uri"]
                .as_str()
                .unwrap_or("");
            if !uri.ends_with(target_uri_suffix) {
                continue;
            }

            let changes = val["message"]["params"]["contentChanges"]
                .as_array()
                .expect("contentChanges array");

            for change in changes {
                let range_val = &change["range"];
                let start_line = range_val["start"]["line"].as_u64().unwrap() as u32;
                let start_char = range_val["start"]["character"].as_u64().unwrap() as u32;
                let end_line = range_val["end"]["line"].as_u64().unwrap() as u32;
                let end_char = range_val["end"]["character"].as_u64().unwrap() as u32;
                let text = change["text"].as_str().unwrap().to_string();

                let deleted_len = if end_line == start_line {
                    (end_char - start_char) as usize
                } else {
                    (end_char) as usize // approximate for single-line tests
                };

                edits.push(RecordedEdit {
                    at: Utc::now(),
                    range: EditRange {
                        start: (start_line, start_char),
                        end: (end_line, end_char),
                    },
                    inserted: text,
                    deleted_len,
                });
            }
        }
    }

    edits
}

#[test]
fn test_replay_phase_0_matrix_01_dw() {
    let edits = extract_did_change_edits_from_fixture(
        "tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl",
        "01-dw.kata",
    );
    // Matrix 01 on 01-dw.kata contains:
    // Seq 32: dw on 01-dw.kata (1,17)..(1,23) deleting `alpha ` (deleted_len = 6)
    // Seq 34: restore `alpha ` before closing (inverts Seq 32)
    let dw_edit = edits
        .iter()
        .find(|e| e.range.start == (1, 17) && e.deleted_len == 6)
        .expect("dw edit found");

    assert_eq!(count_edit_units(std::slice::from_ref(dw_edit)), 1);

    // Replaying both Seq 32 and Seq 34 (the undo/restore) yields 0 net edit units
    assert_eq!(count_edit_units(&edits), 0);
}

#[test]
fn test_replay_phase_0_matrix_02_dw_repeat() {
    let edits = extract_did_change_edits_from_fixture(
        "tests/fixtures/lsp/2026-09-13-matrix-02-dw-repeat.jsonl",
        "02-dw-repeat.kata",
    );
    // Matrix 02 on 02-dw-repeat.kata contains:
    // Seq 40: dw deleting `alpha ` (deleted_len = 6)
    // Seq 42: restore `alpha ` (inverts Seq 40)
    // Seq 44: repeated dw deleting `alpha ` (deleted_len = 6)
    // Seq 46: restore `alpha ` before close (inverts Seq 44)
    assert_eq!(edits.len(), 4);

    // Each individual dw operation is 1 edit unit
    assert_eq!(count_edit_units(std::slice::from_ref(&edits[0])), 1);
    assert_eq!(count_edit_units(std::slice::from_ref(&edits[2])), 1);

    // Replay through Seq 44 (first dw + restore + second dw) yields 1 net edit unit
    assert_eq!(count_edit_units(&edits[0..3]), 1);

    // Full replay through Seq 46 (restored) yields 0 net edit units
    assert_eq!(count_edit_units(&edits), 0);
}

#[test]
fn test_replay_phase_0_matrix_03_backspace() {
    let edits = extract_did_change_edits_from_fixture(
        "tests/fixtures/lsp/2026-09-13-matrix-03-backspace.jsonl",
        "03-backspace.kata",
    );
    // Filter to the backspace sequence on line 2 (Seqs 52, 54, 56, 58, 60)
    let backspaces: Vec<RecordedEdit> = edits
        .into_iter()
        .filter(|e| e.range.start.0 == 2 && e.deleted_len == 1 && e.inserted.is_empty())
        .collect();

    assert_eq!(backspaces.len(), 5);
    assert_eq!(count_edit_units(&backspaces), 5);

    // Rule: backspace_run = 3 should fail
    let rules = ForbidRules {
        backspace_run: Some(3),
        unsupported: vec![],
    };
    let err =
        check_forbid_rules(&backspaces, Some(&rules)).expect_err("backspace run should trigger");
    assert!(err.contains("Triggered forbidden pattern: backspace run of 3"));

    // Rule: backspace_run = 6 should pass
    let lenient = ForbidRules {
        backspace_run: Some(6),
        unsupported: vec![],
    };
    assert!(check_forbid_rules(&backspaces, Some(&lenient)).is_ok());
}
