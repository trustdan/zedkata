use zedkata_core::{
    compare_buffers, diff_buffers, normalize_buffer, verify_attempt, Attempt, AttemptOutcome,
    Exercise, Verdict,
};

#[test]
fn test_exact_buffer_match() {
    let current = "fn main() {\n    println!(\"hello\");\n}\n";
    let goal = "fn main() {\n    println!(\"hello\");\n}\n";

    assert!(compare_buffers(current, goal));
    assert!(diff_buffers(current, goal).is_none());
}

#[test]
fn test_nonmatching_buffers_return_actionable_diff() {
    let current = "fn main() {\n    println!(\"world\");\n}\n";
    let goal = "fn main() {\n    println!(\"hello\");\n}\n";

    assert!(!compare_buffers(current, goal));

    let diff = diff_buffers(current, goal).expect("expected diff");
    assert_eq!(diff.line, 2);
    assert_eq!(diff.col, 15); // column where 'w' and 'h' differ
    assert_eq!(diff.expected, "    println!(\"hello\");");
    assert_eq!(diff.actual, "    println!(\"world\");");
    assert!(diff.message.contains("Line 2, col 15"));
}

#[test]
fn test_empty_content_handling() {
    // Both empty: match
    assert!(compare_buffers("", ""));
    assert_eq!(normalize_buffer(""), "");
    assert!(diff_buffers("", "").is_none());

    // Empty vs whitespace/newline
    assert!(compare_buffers("", "\n"));
    assert!(compare_buffers("\n\n", ""));

    // Empty current vs non-empty goal: no match
    let non_empty = "let x = 1;\n";
    assert!(!compare_buffers("", non_empty));
    assert!(!compare_buffers(non_empty, ""));

    let diff = diff_buffers("", non_empty).expect("expected diff");
    assert_eq!(diff.line, 1);
    assert!(diff.message.contains("Missing line 1"));
}

#[test]
fn test_significant_whitespace_preserved() {
    // Differing indentation must fail
    let current = "fn test() {\n  let x = 1;\n}\n";
    let goal = "fn test() {\n    let x = 1;\n}\n";

    assert!(!compare_buffers(current, goal));
    let diff = diff_buffers(current, goal).expect("expected diff");
    assert_eq!(diff.line, 2);
    assert_eq!(diff.col, 3); // 2 spaces vs 4 spaces
}

#[test]
fn test_crlf_normalization() {
    let windows_crlf = "fn calculate() -> i32 {\r\n    42\r\n}\r\n";
    let unix_lf = "fn calculate() -> i32 {\n    42\n}\n";
    let mixed = "fn calculate() -> i32 {\r\n    42\n}\n";

    assert!(compare_buffers(windows_crlf, unix_lf));
    assert!(compare_buffers(unix_lf, windows_crlf));
    assert!(compare_buffers(mixed, unix_lf));

    assert_eq!(normalize_buffer(windows_crlf), normalize_buffer(unix_lf));
    assert!(diff_buffers(windows_crlf, unix_lf).is_none());
}

#[test]
fn test_unicode_and_emojis() {
    let current = "let greeting = \"👋 こんにちは, 世界! 🦀\";\n";
    let goal = "let greeting = \"👋 こんにちは, 世界! 🦀\";\n";
    let wrong = "let greeting = \"👋 こんにちは, 世界! 🚀\";\n";

    assert!(compare_buffers(current, goal));
    assert!(diff_buffers(current, goal).is_none());

    assert!(!compare_buffers(current, wrong));
    let diff = diff_buffers(current, wrong).expect("expected diff");
    assert_eq!(diff.line, 1);
    assert!(diff.actual.contains("🦀"));
    assert!(diff.expected.contains("🚀"));
}

#[test]
fn test_attempt_outcome_unfinished() {
    let exercise = test_exercise(1, 2);
    let mut attempt = test_attempt();
    attempt.edit_count = 1;

    let current = "pub fn add(a: i32, b: i32) -> i32 { a }\n";
    let goal = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    assert_eq!(outcome, AttemptOutcome::Unfinished);
}

#[test]
fn test_attempt_outcome_passed_par() {
    let exercise = test_exercise(1, 2);
    let mut attempt = test_attempt();
    attempt.edit_count = 1;

    let current = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let goal = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    assert_eq!(
        outcome,
        AttemptOutcome::Passed {
            edits: 1,
            par: Some(1),
            score: 100,
        }
    );
}

#[test]
fn test_attempt_outcome_passed_within_max() {
    let exercise = test_exercise(1, 3);
    let mut attempt = test_attempt();
    attempt.edit_count = 2; // over par (1), but under max (3)

    let current = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let goal = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    assert_eq!(
        outcome,
        AttemptOutcome::Passed {
            edits: 2,
            par: Some(1),
            score: 85, // 100 - (2 - 1)*15
        }
    );
}

#[test]
fn test_attempt_outcome_failed_over_budget() {
    let exercise = test_exercise(1, 2);
    let mut attempt = test_attempt();
    attempt.edit_count = 5; // exceeds max_edits (2)

    let current = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let goal = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    match outcome {
        AttemptOutcome::Failed { reason } => {
            assert!(reason.contains("Over par budget"));
            assert!(reason.contains("5 edits"));
            assert!(reason.contains("maximum of 2"));
        }
        other => panic!("Expected Failed, got: {:?}", other),
    }
}

#[test]
fn test_attempt_outcome_already_finalized_prevents_duplicate_completions() {
    let exercise = test_exercise(1, 2);
    let mut attempt = test_attempt();
    attempt.edit_count = 1;
    attempt.verdict = Some(Verdict::Passed {
        edits: 1,
        par: Some(1),
        score: 100,
    });

    let current = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let goal = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    // Repeated evaluation of a finalized attempt returns AlreadyFinalized
    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    match outcome {
        AttemptOutcome::AlreadyFinalized { previous_verdict } => {
            assert!(previous_verdict.contains("Passed"));
        }
        other => panic!("Expected AlreadyFinalized, got: {:?}", other),
    }
}

#[test]
fn test_attempt_outcome_already_finalized_failure() {
    let exercise = test_exercise(1, 2);
    let mut attempt = test_attempt();
    attempt.edit_count = 4;
    attempt.verdict = Some(Verdict::Failed {
        reason: "Abandoned".to_string(),
    });

    let current = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let goal = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    let outcome = verify_attempt(current, goal, &attempt, &exercise);
    match outcome {
        AttemptOutcome::AlreadyFinalized { previous_verdict } => {
            assert!(previous_verdict.contains("Abandoned"));
        }
        other => panic!("Expected AlreadyFinalized, got: {:?}", other),
    }
}

fn test_exercise(par: u32, max: u32) -> Exercise {
    Exercise {
        id: "test-exercise".to_string(),
        task: "Complete the function".to_string(),
        start: "fixtures/start.kata".to_string(),
        goal: "fixtures/goal.kata".to_string(),
        par_edits: Some(par),
        max_edits: Some(max),
        hint: None,
        forbid: None,
        cursor: None,
    }
}

fn test_attempt() -> Attempt {
    Attempt {
        id: "attempt-1".to_string(),
        edit_count: 0,
        edits: Vec::new(),
        verdict: None,
    }
}
