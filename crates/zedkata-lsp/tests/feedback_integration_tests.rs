use std::path::PathBuf;
use zedkata_core::catalog::Catalog;
use zedkata_core::session::{SessionState, Verdict};
use zedkata_lsp::{
    codec::{write_message, MessageReader},
    feedback::{path_to_uri, FeedbackEngine},
    protocol::*,
    run_server_with_engine, LspServer,
};

fn lessons_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
}

fn create_temp_session_path(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    std::env::temp_dir()
        .join(format!("zedkata_test_{}_{}_{}", test_name, pid, now))
        .join("session.json")
}

#[tokio::test]
async fn test_active_exercise_open_starts_attempt_and_clears_diagnostics() {
    let session_path = create_temp_session_path("open_starts_attempt");
    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.current.exercise_index = 0;
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).unwrap();
    let start_fixture_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .unwrap();
    let start_fixture_text = std::fs::read_to_string(&start_fixture_path).unwrap();
    let start_uri = path_to_uri(&start_fixture_path);

    let feedback = FeedbackEngine::new(session_path.clone(), catalog);
    let server = LspServer::with_feedback(feedback);

    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server_with_engine(server_read, server_write, server).await });

    let mut reader = MessageReader::new(client_read);

    // Initialize
    let init_req = Request::new(RequestId::Int(1), "initialize", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_req).unwrap())
        .await
        .unwrap();
    let _init_resp = reader.read_message().await.unwrap().unwrap();

    let init_notif = Notification::new("initialized", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_notif).unwrap())
        .await
        .unwrap();

    // Send didOpen for active exercise buffer
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: start_uri.clone(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: start_fixture_text,
        },
    };
    let open_notif = Notification::new(
        "textDocument/didOpen",
        Some(serde_json::to_value(did_open_params).unwrap()),
    );
    write_message(&mut client_write, &serde_json::to_vec(&open_notif).unwrap())
        .await
        .unwrap();

    // Client receives clear diagnostics notification
    let notif_bytes = reader.read_message().await.unwrap().unwrap();
    let notif_val: serde_json::Value = serde_json::from_slice(&notif_bytes).unwrap();
    assert_eq!(notif_val["method"], "textDocument/publishDiagnostics");
    assert_eq!(notif_val["params"]["uri"], start_uri);
    assert_eq!(
        notif_val["params"]["diagnostics"].as_array().unwrap().len(),
        0
    );

    // Verify session state updated
    let updated_session = SessionState::load_from_file(&session_path).unwrap();
    assert!(updated_session.attempt.is_some());
    let att = updated_session.attempt.unwrap();
    assert_eq!(att.edit_count, 0);
    assert!(matches!(att.verdict, Some(Verdict::InProgress)));
    assert!(updated_session.current.buffer_path.is_some());

    // Clean up
    let _ = std::fs::remove_file(&session_path);
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}

#[tokio::test]
async fn test_unrelated_document_is_ignored() {
    let session_path = create_temp_session_path("unrelated_doc");
    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.current.exercise_index = 0;
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).unwrap();
    let feedback = FeedbackEngine::new(session_path.clone(), catalog);
    let server = LspServer::with_feedback(feedback);

    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server_with_engine(server_read, server_write, server).await });

    let mut reader = MessageReader::new(client_read);

    // Initialize
    let init_req = Request::new(RequestId::Int(1), "initialize", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_req).unwrap())
        .await
        .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap();

    // Send didOpen for an UNRELATED document
    let unrelated_uri = "file:///C:/workspace/scratch.kata";
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: unrelated_uri.to_string(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: "// some unrelated scratch notes\n".to_string(),
        },
    };
    let open_notif = Notification::new(
        "textDocument/didOpen",
        Some(serde_json::to_value(did_open_params).unwrap()),
    );
    write_message(&mut client_write, &serde_json::to_vec(&open_notif).unwrap())
        .await
        .unwrap();

    // Send didChange on the unrelated document
    let did_change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: unrelated_uri.to_string(),
            version: 1,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "// modified scratch\n".to_string(),
        }],
    };
    let change_notif = Notification::new(
        "textDocument/didChange",
        Some(serde_json::to_value(did_change_params).unwrap()),
    );
    write_message(
        &mut client_write,
        &serde_json::to_vec(&change_notif).unwrap(),
    )
    .await
    .unwrap();

    // Verify session state was NOT modified
    let current_session = SessionState::load_from_file(&session_path).unwrap();
    assert!(current_session.attempt.is_none());

    // Clean up
    let _ = std::fs::remove_file(&session_path);
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}

#[tokio::test]
async fn test_active_exercise_reaches_goal_and_passes() {
    let session_path = create_temp_session_path("reach_goal_pass");
    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.current.exercise_index = 0;
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).unwrap();
    let start_fixture_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .unwrap();
    let start_fixture_text = std::fs::read_to_string(&start_fixture_path).unwrap();
    let start_uri = path_to_uri(&start_fixture_path);

    let feedback = FeedbackEngine::new(session_path.clone(), catalog);
    let server = LspServer::with_feedback(feedback);

    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server_with_engine(server_read, server_write, server).await });

    let mut reader = MessageReader::new(client_read);

    // Initialize
    let init_req = Request::new(RequestId::Int(1), "initialize", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_req).unwrap())
        .await
        .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap();

    // didOpen
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: start_uri.clone(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: start_fixture_text,
        },
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didOpen",
            Some(serde_json::to_value(did_open_params).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

    // Read initial clear diagnostics
    let _ = reader.read_message().await.unwrap().unwrap();

    // Perform dw on line 1 deleting "target " at (1, 4) .. (1, 11)
    let did_change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: start_uri.clone(),
            version: 1,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(1, 4), Position::new(1, 11))),
            range_length: Some(7),
            text: "".to_string(),
        }],
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didChange",
            Some(serde_json::to_value(did_change_params).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

    // Client receives congratulatory diagnostic!
    let diag_bytes = reader.read_message().await.unwrap().unwrap();
    let diag_val: serde_json::Value = serde_json::from_slice(&diag_bytes).unwrap();
    assert_eq!(diag_val["method"], "textDocument/publishDiagnostics");
    assert_eq!(diag_val["params"]["uri"], start_uri);
    let diags = diag_val["params"]["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0]["severity"], 3); // Information
    assert!(diags[0]["message"].as_str().unwrap().contains("Passed"));
    assert!(diags[0]["message"].as_str().unwrap().contains("1 edit"));
    assert!(diags[0]["message"].as_str().unwrap().contains("score 100"));

    // Verify persisted session state
    let passed_session = SessionState::load_from_file(&session_path).unwrap();
    let att = passed_session.attempt.unwrap();
    assert_eq!(att.edit_count, 1);
    match att.verdict {
        Some(Verdict::Passed { edits, par, score }) => {
            assert_eq!(edits, 1);
            assert_eq!(par, Some(1));
            assert_eq!(score, 100);
        }
        _ => panic!("Expected Verdict::Passed, got {:?}", att.verdict),
    }

    // Verify history recorded
    assert_eq!(passed_session.history.len(), 1);
    assert_eq!(passed_session.history[0].exercise_id, "delete-word-arg");
    assert!(passed_session.history[0].passed);
    assert_eq!(passed_session.history[0].score, 100);

    // Clean up
    let _ = std::fs::remove_file(&session_path);
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}

#[tokio::test]
async fn test_active_exercise_failure_on_forbid_rule() {
    let session_path = create_temp_session_path("forbid_rule_failure");
    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.current.exercise_index = 0; // backspace_run = 3
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).unwrap();
    let start_fixture_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .unwrap();
    let start_fixture_text = std::fs::read_to_string(&start_fixture_path).unwrap();
    let start_uri = path_to_uri(&start_fixture_path);

    let feedback = FeedbackEngine::new(session_path.clone(), catalog);
    let server = LspServer::with_feedback(feedback);

    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server_with_engine(server_read, server_write, server).await });

    let mut reader = MessageReader::new(client_read);

    // Initialize
    let init_req = Request::new(RequestId::Int(1), "initialize", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_req).unwrap())
        .await
        .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap();

    // didOpen
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: start_uri.clone(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: start_fixture_text,
        },
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didOpen",
            Some(serde_json::to_value(did_open_params).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap(); // consume open diagnostic clear

    // Perform 3 consecutive single-character deletions (backspaces)
    // Line 1: `    target listener: &mut Listener,`
    // Backspaces at columns 10 down to 7 (deleting "get")
    let backspaces = [(9, 10), (8, 9), (7, 8)];
    for (idx, (start_col, end_col)) in backspaces.iter().enumerate() {
        let did_change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: start_uri.clone(),
                version: (idx + 1) as i64,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range::new(
                    Position::new(1, *start_col),
                    Position::new(1, *end_col),
                )),
                range_length: Some(1),
                text: "".to_string(),
            }],
        };
        write_message(
            &mut client_write,
            &serde_json::to_vec(&Notification::new(
                "textDocument/didChange",
                Some(serde_json::to_value(did_change_params).unwrap()),
            ))
            .unwrap(),
        )
        .await
        .unwrap();
    }

    // After 3rd backspace, server emits failure diagnostic!
    let diag_bytes = reader.read_message().await.unwrap().unwrap();
    let diag_val: serde_json::Value = serde_json::from_slice(&diag_bytes).unwrap();
    assert_eq!(diag_val["method"], "textDocument/publishDiagnostics");
    let diags = diag_val["params"]["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0]["severity"], 1); // Error
    assert!(diags[0]["message"].as_str().unwrap().contains("Failed"));
    assert!(diags[0]["message"]
        .as_str()
        .unwrap()
        .contains("backspace run of 3"));

    // Verify session state finalized as Failed and weaknesses updated
    let failed_session = SessionState::load_from_file(&session_path).unwrap();
    let att = failed_session.attempt.unwrap();
    match att.verdict {
        Some(Verdict::Failed { reason }) => {
            assert!(reason.contains("backspace run of 3"));
        }
        _ => panic!("Expected Verdict::Failed, got {:?}", att.verdict),
    }
    assert!(!failed_session.weaknesses.is_empty());

    // Late edit after failure must be ignored
    let late_change = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: start_uri.clone(),
            version: 4,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "late edit".to_string(),
        }],
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didChange",
            Some(serde_json::to_value(late_change).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

    let post_session = SessionState::load_from_file(&session_path).unwrap();
    assert_eq!(post_session.attempt.unwrap().edit_count, att.edit_count);

    // Clean up
    let _ = std::fs::remove_file(&session_path);
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}

#[tokio::test]
async fn test_document_close_clears_diagnostics() {
    let session_path = create_temp_session_path("close_clears_diag");
    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.current.exercise_index = 0;
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).unwrap();
    let start_fixture_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .unwrap();
    let start_fixture_text = std::fs::read_to_string(&start_fixture_path).unwrap();
    let start_uri = path_to_uri(&start_fixture_path);

    let feedback = FeedbackEngine::new(session_path.clone(), catalog);
    let server = LspServer::with_feedback(feedback);

    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server_with_engine(server_read, server_write, server).await });

    let mut reader = MessageReader::new(client_read);

    // Initialize & didOpen
    let init_req = Request::new(RequestId::Int(1), "initialize", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_req).unwrap())
        .await
        .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap();

    let did_open = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: start_uri.clone(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: start_fixture_text,
        },
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didOpen",
            Some(serde_json::to_value(did_open).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap(); // open clear

    // Send didClose
    let did_close = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier {
            uri: start_uri.clone(),
        },
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didClose",
            Some(serde_json::to_value(did_close).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

    // Client receives empty diagnostics notification clearing feedback
    let close_bytes = reader.read_message().await.unwrap().unwrap();
    let close_val: serde_json::Value = serde_json::from_slice(&close_bytes).unwrap();
    assert_eq!(close_val["method"], "textDocument/publishDiagnostics");
    assert_eq!(close_val["params"]["uri"], start_uri);
    assert_eq!(
        close_val["params"]["diagnostics"].as_array().unwrap().len(),
        0
    );

    // Clean up
    let _ = std::fs::remove_file(&session_path);
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}

#[tokio::test]
async fn test_exercise_retry_and_switch_ignores_old_exercise() {
    let session_path = create_temp_session_path("retry_and_switch");
    let mut session = SessionState::default();
    session.current.lesson_id = "motions-01-word".to_string();
    session.current.exercise_index = 0;
    session.save_to_file(&session_path).unwrap();

    let catalog = Catalog::load_from_dir(&lessons_dir()).unwrap();
    let start_fixture_path = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .unwrap();
    let start_fixture_text = std::fs::read_to_string(&start_fixture_path).unwrap();
    let start_uri = path_to_uri(&start_fixture_path);

    let feedback = FeedbackEngine::new(session_path.clone(), catalog);
    let server = LspServer::with_feedback(feedback);

    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server_with_engine(server_read, server_write, server).await });

    let mut reader = MessageReader::new(client_read);

    // Initialize
    let init_req = Request::new(RequestId::Int(1), "initialize", None);
    write_message(&mut client_write, &serde_json::to_vec(&init_req).unwrap())
        .await
        .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap();

    // didOpen
    let did_open = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: start_uri.clone(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: start_fixture_text.clone(),
        },
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didOpen",
            Some(serde_json::to_value(did_open).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    let _ = reader.read_message().await.unwrap().unwrap(); // open clear

    // 1. Pass exercise 0
    let did_change = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: start_uri.clone(),
            version: 1,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(1, 4), Position::new(1, 11))),
            range_length: Some(7),
            text: "".to_string(),
        }],
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didChange",
            Some(serde_json::to_value(did_change).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    let pass_diag = reader.read_message().await.unwrap().unwrap();
    let pass_val: serde_json::Value = serde_json::from_slice(&pass_diag).unwrap();
    assert!(pass_val["params"]["diagnostics"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Passed"));

    // 2. Student or TUI calls retry_attempt() on session state
    let mut session_to_retry = SessionState::load_from_file(&session_path).unwrap();
    session_to_retry.retry_attempt("att-retry-01".to_string());
    session_to_retry.save_to_file(&session_path).unwrap();

    // Reopen buffer (re-dispatching didOpen)
    let reopen = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: start_uri.clone(),
            language_id: "zedkata".to_string(),
            version: 0,
            text: start_fixture_text,
        },
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didOpen",
            Some(serde_json::to_value(reopen).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

    // Clears pass diagnostic
    let clear_diag = reader.read_message().await.unwrap().unwrap();
    let clear_val: serde_json::Value = serde_json::from_slice(&clear_diag).unwrap();
    assert_eq!(
        clear_val["params"]["diagnostics"].as_array().unwrap().len(),
        0
    );

    let retried_session = SessionState::load_from_file(&session_path).unwrap();
    assert_eq!(retried_session.attempt.unwrap().edit_count, 0);

    // 3. Switch exercise to index 1 in session state
    let mut session_to_switch = SessionState::load_from_file(&session_path).unwrap();
    session_to_switch.switch_exercise("motions-01-word".to_string(), 1);
    session_to_switch.save_to_file(&session_path).unwrap();

    // Now send edit to old exercise 0 buffer -> must be ignored!
    let edit_old_buffer = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: start_uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "edit old buffer".to_string(),
        }],
    };
    write_message(
        &mut client_write,
        &serde_json::to_vec(&Notification::new(
            "textDocument/didChange",
            Some(serde_json::to_value(edit_old_buffer).unwrap()),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

    let post_switch_session = SessionState::load_from_file(&session_path).unwrap();
    assert_eq!(post_switch_session.current.exercise_index, 1);
    assert!(post_switch_session.attempt.is_none());

    // Clean up
    let _ = std::fs::remove_file(&session_path);
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}
