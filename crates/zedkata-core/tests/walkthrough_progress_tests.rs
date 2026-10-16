use zedkata_core::session::SessionState;

#[test]
fn test_session_state_legacy_json_backward_compatibility() {
    let legacy_json = r#"{
        "version": 1,
        "student": { "started_at": "2026-10-15T20:00:00Z" },
        "current": {
            "lesson_id": "motions-03-textobjects",
            "exercise_index": 0,
            "buffer_path": null,
            "opened_at": null,
            "nav_mode": null
        },
        "attempt": null,
        "history": [],
        "weaknesses": {}
    }"#;

    let state =
        SessionState::from_json(legacy_json).expect("Legacy session JSON must parse cleanly");
    assert_eq!(state.version, 1);
    assert!(state.completed_walkthroughs.is_empty());
}

#[test]
fn test_session_state_walkthrough_progress_lifecycle() {
    let mut state = SessionState::default();

    assert!(!state.is_walkthrough_completed("mini-vs-any"));
    assert!(!state.is_walkthrough_completed("lsp-actions-walkthrough"));

    // 1. Mark walkthrough completed
    let newly_added = state.mark_walkthrough_completed("mini-vs-any", "motions-03-textobjects");
    assert!(newly_added);
    assert!(state.is_walkthrough_completed("mini-vs-any"));
    assert_eq!(state.completed_walkthroughs.len(), 1);
    assert_eq!(
        state.completed_walkthroughs[0].walkthrough_id,
        "mini-vs-any"
    );
    assert_eq!(
        state.completed_walkthroughs[0].lesson_id,
        "motions-03-textobjects"
    );

    // 2. Marking again refreshes timestamp and returns false (not newly added)
    let newly_added_again =
        state.mark_walkthrough_completed("mini-vs-any", "motions-03-textobjects");
    assert!(!newly_added_again);
    assert_eq!(state.completed_walkthroughs.len(), 1);

    // 3. Mark a second walkthrough completed
    let added_second =
        state.mark_walkthrough_completed("lsp-actions-walkthrough", "zed-01-lsp-actions");
    assert!(added_second);
    assert_eq!(state.completed_walkthroughs.len(), 2);
    assert!(state.is_walkthrough_completed("lsp-actions-walkthrough"));

    // 4. Verify drill scoring isolation: attempt, history, and weaknesses are unmodified
    assert!(state.attempt.is_none());
    assert!(state.history.is_empty());
    assert!(state.weaknesses.is_empty());

    // 5. Serialize and round-trip deserialize
    let json = state.to_json_pretty().expect("Must serialize to JSON");
    assert!(json.contains("completed_walkthroughs"));
    assert!(json.contains("mini-vs-any"));
    assert!(json.contains("lsp-actions-walkthrough"));

    let restored = SessionState::from_json(&json).expect("Must deserialize from JSON");
    assert_eq!(restored.completed_walkthroughs.len(), 2);
    assert!(restored.is_walkthrough_completed("mini-vs-any"));
    assert!(restored.is_walkthrough_completed("lsp-actions-walkthrough"));

    // 6. Reset a walkthrough
    let removed = state.reset_walkthrough("mini-vs-any");
    assert!(removed);
    assert!(!state.is_walkthrough_completed("mini-vs-any"));
    assert!(state.is_walkthrough_completed("lsp-actions-walkthrough"));
    assert_eq!(state.completed_walkthroughs.len(), 1);

    // 7. Resetting non-existent walkthrough returns false
    let removed_again = state.reset_walkthrough("non-existent");
    assert!(!removed_again);
}
