use std::time::Duration;

use agent_client_protocol::ByteStreams;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use zedkata_agent::AgentServer;

/// Helper function to read one JSON-RPC response line with a timeout.
async fn read_jsonrpc_line<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> serde_json::Value {
    let mut line = String::new();
    let res = tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut line)).await;
    match res {
        Ok(Ok(0)) => panic!("EOF reached while waiting for JSON-RPC message"),
        Ok(Ok(_)) => {
            let trimmed = line.trim();
            assert!(
                !trimmed.is_empty(),
                "Emitted line should not be empty whitespace"
            );
            serde_json::from_str(trimmed)
                .unwrap_or_else(|e| panic!("Line is not valid JSON ({e}): {trimmed}"))
        }
        Ok(Err(e)) => panic!("I/O error reading JSON-RPC line: {e}"),
        Err(_) => panic!("Timeout reading JSON-RPC line"),
    }
}

#[tokio::test]
async fn test_initialize_lifecycle() {
    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());
    let server = AgentServer::new();

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    let mut client_reader = BufReader::new(client_reader);

    // Send initialize request
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": 1,
            "clientInfo": {
                "name": "zed-test-client",
                "version": "1.19.2"
            }
        }
    });

    client_writer
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let resp = read_jsonrpc_line(&mut client_reader).await;

    // Verify response
    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["protocolVersion"], 1);
    assert_eq!(resp["result"]["agentInfo"]["name"], "zedkata-agent");

    // Close transport and verify clean server exit
    drop(client_writer);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "Server should terminate cleanly on EOF");
}

#[tokio::test]
async fn test_session_lifecycle_and_prompt_turn() {
    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());
    let server = AgentServer::new();
    let server_clone = server.clone();

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    let mut client_reader = BufReader::new(client_reader);

    // Step 1: Initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { "protocolVersion": 1 }
    });
    client_writer
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();
    let _init_resp = read_jsonrpc_line(&mut client_reader).await;

    // Step 2: New Session
    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": "/workspace",
            "mcpServers": []
        }
    });
    client_writer
        .write_all(format!("{}\n", session_new_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let session_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(session_resp["id"], 2);
    let session_id = session_resp["result"]["sessionId"]
        .as_str()
        .expect("sessionId should be string")
        .to_string();
    assert!(
        session_id.starts_with("sess-"),
        "Session ID format should start with sess-"
    );

    // Server should record the session
    assert!(server_clone.has_session(&session_id));
    assert_eq!(server_clone.active_sessions().len(), 1);

    // Step 3: Prompt
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": "Explain why my last attempt failed"
                }
            ]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // The server emits a session/update notification before the response
    let notif = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(notif["jsonrpc"], "2.0");
    assert_eq!(notif["method"], "session/update");
    assert_eq!(notif["params"]["sessionId"], session_id);
    assert_eq!(
        notif["params"]["update"]["sessionUpdate"],
        "agent_message_chunk"
    );

    // Then the server responds to the prompt request
    let prompt_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(prompt_resp["result"]["stopReason"], "end_turn");

    // Step 4: Close Session
    let close_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "session/close",
        "params": {
            "sessionId": session_id
        }
    });
    client_writer
        .write_all(format!("{}\n", close_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let close_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(close_resp["id"], 4);

    // Session should be removed
    assert!(!server_clone.has_session(&session_id));
    assert_eq!(server_clone.active_sessions().len(), 0);

    // Clean shutdown
    drop(client_writer);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unsupported_request_returns_method_not_found() {
    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());
    let server = AgentServer::new();

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    let mut client_reader = BufReader::new(client_reader);

    // Send unsupported request method
    let bad_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "unsupported/nonexistent_method",
        "params": { "foo": "bar" }
    });
    client_writer
        .write_all(format!("{}\n", bad_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let err_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(err_resp["jsonrpc"], "2.0");
    assert_eq!(err_resp["id"], 42);
    assert!(
        err_resp["error"].is_object(),
        "Expected error object in response"
    );
    assert_eq!(
        err_resp["error"]["code"], -32601,
        "Code should be -32601 (Method not found)"
    );

    // Verify connection remains healthy after rejected request
    let ping_init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 43,
        "method": "initialize",
        "params": { "protocolVersion": 1 }
    });
    client_writer
        .write_all(format!("{}\n", ping_init).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let success_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(success_resp["id"], 43);
    assert_eq!(success_resp["result"]["protocolVersion"], 1);

    drop(client_writer);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stdout_framing_hygiene() {
    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());
    let server = AgentServer::new();

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    let mut client_reader = BufReader::new(client_reader);

    // Send multiple varied requests
    let requests = vec![
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": 1 }
        }),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": { "cwd": "/test", "mcpServers": [] }
        }),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "invalid/query",
            "params": {}
        }),
    ];

    for req in requests {
        client_writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
    }
    client_writer.flush().await.unwrap();

    // Read responses for all 3 requests
    for _ in 0..3 {
        let msg = read_jsonrpc_line(&mut client_reader).await;
        // Verify every single emitted line is framed JSON-RPC 2.0
        assert_eq!(
            msg["jsonrpc"], "2.0",
            "Every message must specify jsonrpc 2.0"
        );
        assert!(
            msg["id"].is_number(),
            "Every response message must correlate to an ID"
        );
    }

    drop(client_writer);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_acp_generate_practice_persists_drill_to_generated_dir_with_admitted_status() {
    use chrono::Utc;
    use std::collections::HashMap;
    use zedkata_agent::{AgentConfig, ConfiguredProvider, ProviderKind};
    use zedkata_core::catalog::Catalog;
    use zedkata_core::session::{
        CurrentSession, SessionState, StudentInfo, CURRENT_SCHEMA_VERSION,
    };

    let temp_root = std::env::temp_dir().join(format!(
        "zedkata_acp_gen_persist_{}_{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let workspace_dir = temp_root.join("workspace");
    let generated_dir = temp_root.join("generated");
    std::fs::create_dir_all(&workspace_dir).unwrap();
    std::fs::create_dir_all(&generated_dir).unwrap();

    // Write session.json in workspace CWD with a weakness on 'dw'
    let mut weaknesses = HashMap::new();
    weaknesses.insert("dw".to_string(), 0.85);
    let session = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: Utc::now(),
        },
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: None,
            opened_at: Some(Utc::now()),
            nav_mode: None,
        },
        attempt: None,
        history: Vec::new(),
        weaknesses,
        completed_walkthroughs: Vec::new(),
    };
    let session_file = workspace_dir.join("session.json");
    std::fs::write(
        &session_file,
        serde_json::to_string_pretty(&session).unwrap(),
    )
    .unwrap();

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        generated_dir: Some(generated_dir.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::new(config);
    let server = AgentServer::with_provider(provider);

    let (mut client_writer, server_reader) = tokio::io::duplex(8192);
    let (server_writer, client_reader) = tokio::io::duplex(8192);
    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());
    let server_handle = tokio::spawn(async move { server.run(server_transport).await });
    let mut client_reader = BufReader::new(client_reader);

    // 1. Initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { "protocolVersion": 1 }
    });
    client_writer
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();
    let _ = read_jsonrpc_line(&mut client_reader).await;

    // 2. New Session with CWD pointing to workspace_dir
    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": workspace_dir.to_string_lossy().to_string(),
            "mcpServers": []
        }
    });
    client_writer
        .write_all(format!("{}\n", session_new_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();
    let session_resp = read_jsonrpc_line(&mut client_reader).await;
    let session_id = session_resp["result"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Prompt: "generate practice"
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "generate practice" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let notif = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(notif["method"], "session/update");
    let chunk_text = notif["params"]["update"]["content"]["text"]
        .as_str()
        .expect("chunk text should be string");

    assert!(
        chunk_text.contains("Status: Admitted"),
        "ACP generate response must report Status: Admitted, got:\n{}",
        chunk_text
    );
    assert!(
        chunk_text.contains("**Status**: Persisted"),
        "ACP generate response must show candidate status Persisted, got:\n{}",
        chunk_text
    );
    assert!(
        chunk_text.contains("gen-dw-"),
        "Generated drill should target 'dw' weakness from session CWD, got:\n{}",
        chunk_text
    );

    let prompt_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(prompt_resp["result"]["stopReason"], "end_turn");

    // Verify files were persisted to generated_dir and load cleanly into Catalog
    let cat = Catalog::load_from_dir(&generated_dir).expect("Generated dir should load as Catalog");
    let lessons = cat.all_lessons();
    assert_eq!(
        lessons.len(),
        1,
        "Exactly one generated lesson should be stored on disk"
    );
    let stored_lesson = &lessons[0].lesson;
    assert!(stored_lesson.id.starts_with("gen-dw-"));
    let start_fix = cat
        .resolve_fixture(&stored_lesson.id, &stored_lesson.exercises[0].id, true)
        .unwrap();
    let goal_fix = cat
        .resolve_fixture(&stored_lesson.id, &stored_lesson.exercises[0].id, false)
        .unwrap();
    assert!(start_fix.is_file());
    assert!(goal_fix.is_file());

    drop(client_writer);
    let _ = server_handle.await.unwrap();
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[tokio::test]
async fn test_acp_explain_attempt_uses_session_cwd_and_includes_catalog_metadata() {
    use chrono::Utc;
    use std::sync::Arc;
    use zedkata_agent::{AgentConfig, ConfiguredProvider, FakeProvider, ProviderKind};
    use zedkata_core::session::{
        Attempt, CurrentSession, EditRange, RecordedEdit, SessionState, StudentInfo, Verdict,
        CURRENT_SCHEMA_VERSION,
    };

    let temp_root = std::env::temp_dir().join(format!(
        "zedkata_acp_explain_cwd_{}_{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    std::fs::create_dir_all(&temp_root).unwrap();

    // Write session.json in temp_root referencing shipped lesson 'motions-01-word', exercise 0 ('delete-word-arg')
    let session = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: Utc::now(),
        },
        current: CurrentSession {
            lesson_id: "motions-01-word".to_string(),
            exercise_index: 0,
            buffer_path: Some("delete-word-arg.kata".to_string()),
            opened_at: Some(Utc::now()),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "att-cwd-test-1".to_string(),
            edit_count: 4,
            edits: vec![RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (1, 17),
                    end: (1, 18),
                },
                inserted: String::new(),
                deleted_len: 1,
            }],
            verdict: Some(Verdict::Failed {
                reason: "Violated forbid rule: backspace_run".to_string(),
            }),
        }),
        history: Vec::new(),
        weaknesses: std::collections::HashMap::new(),
        completed_walkthroughs: Vec::new(),
    };
    let session_file = temp_root.join("session.json");
    std::fs::write(
        &session_file,
        serde_json::to_string_pretty(&session).unwrap(),
    )
    .unwrap();

    let fake = FakeProvider::new();
    let config = AgentConfig {
        provider: ProviderKind::Fake,
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake.clone()));
    let server = AgentServer::with_provider(provider);

    let (mut client_writer, server_reader) = tokio::io::duplex(8192);
    let (server_writer, client_reader) = tokio::io::duplex(8192);
    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());
    let server_handle = tokio::spawn(async move { server.run(server_transport).await });
    let mut client_reader = BufReader::new(client_reader);

    // 1. Initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { "protocolVersion": 1 }
    });
    client_writer
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();
    let _ = read_jsonrpc_line(&mut client_reader).await;

    // 2. New Session with CWD = temp_root
    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": temp_root.to_string_lossy().to_string(),
            "mcpServers": []
        }
    });
    client_writer
        .write_all(format!("{}\n", session_new_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();
    let session_resp = read_jsonrpc_line(&mut client_reader).await;
    let session_id = session_resp["result"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Prompt: "explain attempt"
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "explain attempt" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    let notif = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(notif["method"], "session/update");
    let chunk_text = notif["params"]["update"]["content"]["text"]
        .as_str()
        .expect("chunk text should be string");

    let prompt_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(prompt_resp["result"]["stopReason"], "end_turn");

    // Verify FakeProvider received AttemptContext populated from session CWD + loaded Catalog
    let received = fake
        .last_received_context()
        .expect("FakeProvider should have received AttemptContext");
    assert_eq!(received.attempt_id, "att-cwd-test-1");
    assert_eq!(received.lesson_id, "motions-01-word");
    assert!(
        received.lesson_title.is_some(),
        "Catalog lesson_title must be populated"
    );
    assert!(
        received.task_prompt.is_some(),
        "Catalog task_prompt must be populated"
    );
    assert_eq!(received.par_edits, Some(1), "Catalog par_edits must be 1");
    assert_eq!(
        received.forbid_rules.as_ref().and_then(|f| f.backspace_run),
        Some(3),
        "Catalog forbid_rules.backspace_run must be 3"
    );

    // Verify markdown explanation output includes catalog metadata
    assert!(chunk_text.contains("Par budget: 1 edit(s)"));
    assert!(chunk_text.contains("Forbid rule: backspace_run <= 3"));
    assert!(chunk_text.contains(received.task_prompt.as_deref().unwrap()));

    drop(client_writer);
    let _ = server_handle.await.unwrap();
    let _ = std::fs::remove_dir_all(&temp_root);
}
