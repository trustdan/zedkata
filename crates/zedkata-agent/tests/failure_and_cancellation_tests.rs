use chrono::{Duration as ChronoDuration, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::ByteStreams;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use zedkata_agent::{
    AgentConfig, AgentServer, ConfiguredProvider, ContextConfig, FakeProvider, ProviderError,
    ProviderKind,
};
use zedkata_core::session::{
    Attempt, CurrentSession, EditRange, RecordedEdit, SessionState, StudentInfo, Verdict,
    CURRENT_SCHEMA_VERSION,
};

fn temp_session_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zedkata_failure_test_{}_{}_{}",
        name,
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn create_temp_session(
    dir: &Path,
    verdict: Option<Verdict>,
    edits: Vec<RecordedEdit>,
) -> (PathBuf, String) {
    let session_path = dir.join("session.json");
    let edit_count = edits.len() as u32;
    let state = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: Utc::now() - ChronoDuration::hours(1),
        },
        current: CurrentSession {
            lesson_id: "operators-01-edit".to_string(),
            exercise_index: 0,
            buffer_path: Some("temp.kata".to_string()),
            opened_at: Some(Utc::now() - ChronoDuration::minutes(10)),
            nav_mode: None,
        },
        attempt: Some(Attempt {
            id: "attempt-test-123".to_string(),
            edit_count,
            edits,
            verdict,
        }),
        history: Vec::new(),
        weaknesses: std::collections::HashMap::new(),
        completed_walkthroughs: Vec::new(),
    };

    let json = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&session_path, &json).unwrap();
    (session_path, json)
}

fn sample_edits() -> Vec<RecordedEdit> {
    vec![
        RecordedEdit {
            at: Utc::now() - ChronoDuration::seconds(10),
            range: EditRange {
                start: (0, 3),
                end: (0, 4),
            },
            inserted: String::new(),
            deleted_len: 1,
        },
        RecordedEdit {
            at: Utc::now() - ChronoDuration::seconds(5),
            range: EditRange {
                start: (0, 2),
                end: (0, 3),
            },
            inserted: String::new(),
            deleted_len: 1,
        },
    ]
}

/// Helper to read one JSON-RPC line from the ACP duplex stream.
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
async fn test_provider_timeout_produces_clear_message_and_intact_session() {
    let dir = temp_session_dir("timeout");
    let (session_path, original_content) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    // Setup fake provider with 2-second simulated delay
    let fake = FakeProvider::new();
    fake.set_async_delay(Duration::from_secs(2));

    // Configure provider with 1-second timeout (forcing a timeout)
    let config = AgentConfig {
        provider: ProviderKind::Fake,
        model: "test-model".to_string(),
        session_path: Some(session_path.clone()),
        timeout_seconds: 1,
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake));

    // Execute async explanation
    let result = provider
        .explain_session_async(&session_path, None, &ContextConfig::default(), None)
        .await;

    // Must return ProviderError::Timeout
    assert!(result.is_err(), "Expected timeout error");
    let err = result.unwrap_err();
    match &err {
        ProviderError::Timeout { seconds, message } => {
            assert_eq!(*seconds, 1);
            assert!(message.contains("timed out"));
        }
        other => panic!("Expected Timeout, got {:?}", other),
    }

    // Verify user-facing message has clear troubleshooting and confirms session/progress intact
    let user_msg = err.to_user_message();
    assert!(user_msg.contains("Request Timed Out"));
    assert!(user_msg.contains("Troubleshooting"));
    assert!(user_msg.contains("Your session state and practice progress are intact."));

    // Verify session.json on disk is 100% untouched
    let current_content = fs::read_to_string(&session_path).unwrap();
    assert_eq!(
        current_content, original_content,
        "Session file must remain completely unmodified after timeout"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_provider_rejection_produces_clear_message_and_intact_session() {
    let dir = temp_session_dir("rejection");
    let (session_path, original_content) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    // Inject 429 Rate Limit rejection into fake provider
    let fake = FakeProvider::new();
    fake.inject_rejection(Some(429), "Too Many Requests: Rate limit exceeded");

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake));

    let result = provider
        .explain_session_async(&session_path, None, &ContextConfig::default(), None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match &err {
        ProviderError::Rejection {
            status_code,
            message,
        } => {
            assert_eq!(*status_code, Some(429));
            assert!(message.contains("Rate limit exceeded"));
        }
        other => panic!("Expected Rejection, got {:?}", other),
    }

    let user_msg = err.to_user_message();
    assert!(user_msg.contains("Provider Rejected Request (HTTP 429)"));
    assert!(user_msg.contains("rate-limited, wait a moment"));
    assert!(user_msg.contains("offline degraded mode"));
    assert!(user_msg.contains("Your session state and practice progress are intact."));

    let current_content = fs::read_to_string(&session_path).unwrap();
    assert_eq!(current_content, original_content);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_malformed_model_response_produces_clear_message_and_intact_session() {
    let dir = temp_session_dir("malformed");
    let (session_path, original_content) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    // Inject malformed model response
    let fake = FakeProvider::new();
    fake.inject_malformed_response("Response truncated before completing JSON markdown block.");

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake));

    let result = provider
        .explain_session_async(&session_path, None, &ContextConfig::default(), None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match &err {
        ProviderError::MalformedResponse(msg) => {
            assert!(msg.contains("Response truncated"));
        }
        other => panic!("Expected MalformedResponse, got {:?}", other),
    }

    let user_msg = err.to_user_message();
    assert!(user_msg.contains("Malformed Model Response"));
    assert!(user_msg.contains("unparseable or truncated"));
    assert!(user_msg.contains("Your session state and practice progress are intact."));

    let current_content = fs::read_to_string(&session_path).unwrap();
    assert_eq!(current_content, original_content);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_acp_prompt_turn_timeout_returns_end_turn_with_clean_user_chunk() {
    let dir = temp_session_dir("acp_timeout");
    let (session_path, original_content) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());

    // Fake provider with 2s delay, server configured with 1s timeout
    let fake = FakeProvider::new();
    fake.set_async_delay(Duration::from_secs(2));

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        timeout_seconds: 1,
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake));
    let server = AgentServer::with_provider(provider);

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });
    let mut client_reader = BufReader::new(client_reader);

    // Initialize
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

    // New Session
    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": { "cwd": "/workspace", "mcpServers": [] }
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

    // Prompt request
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "Explain attempt" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // Expect session/update chunk with clean timeout message
    let update_msg = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(update_msg["method"], "session/update");
    let text = update_msg["params"]["update"]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("Request Timed Out"));
    assert!(text.contains("Your session state and practice progress are intact."));

    // Expect prompt response with stopReason: "end_turn"
    let prompt_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(prompt_resp["result"]["stopReason"], "end_turn");

    // Verify session file remains untouched
    let current_content = fs::read_to_string(&session_path).unwrap();
    assert_eq!(current_content, original_content);

    drop(client_writer);
    let _ = server_handle.await.unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_acp_prompt_turn_provider_rejection_returns_clean_user_chunk() {
    let dir = temp_session_dir("acp_rejection");
    let (session_path, _) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());

    let fake = FakeProvider::new();
    fake.inject_rejection(Some(401), "Unauthorized: Invalid API key");

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake));
    let server = AgentServer::with_provider(provider);

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });
    let mut client_reader = BufReader::new(client_reader);

    // Initialize
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

    // New Session
    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": { "cwd": "/workspace", "mcpServers": [] }
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

    // Prompt request
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "Explain attempt" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // Update chunk containing rejection guidance
    let update_msg = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(update_msg["method"], "session/update");
    let text = update_msg["params"]["update"]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("Provider Rejected Request (HTTP 401)"));
    assert!(text.contains("Check that your API key is valid"));

    // End turn cleanly
    let prompt_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(prompt_resp["result"]["stopReason"], "end_turn");

    drop(client_writer);
    let _ = server_handle.await.unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_acp_cancellation_terminates_turn_with_stop_reason_cancelled() {
    let dir = temp_session_dir("acp_cancel");
    let (session_path, original_content) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());

    // Fake provider with 3s delay
    let fake = FakeProvider::new();
    fake.set_async_delay(Duration::from_secs(3));

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        timeout_seconds: 10,
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake.clone()));
    let server = AgentServer::with_provider(provider);

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });
    let mut client_reader = BufReader::new(client_reader);

    // Initialize
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

    // New Session
    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": { "cwd": "/workspace", "mcpServers": [] }
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

    // Send prompt request (starts in-flight work)
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "Explain attempt" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // Allow prompt to begin processing, then send CancelNotification
    tokio::time::sleep(Duration::from_millis(50)).await;

    let cancel_notif = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "session/cancel",
        "params": {
            "sessionId": session_id
        }
    });
    client_writer
        .write_all(format!("{}\n", cancel_notif).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // The ACP server should immediately abort and respond with StopReason::Cancelled
    let prompt_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(
        prompt_resp["result"]["stopReason"], "cancelled",
        "Server MUST respond with StopReason::Cancelled per ACP specification"
    );

    // Verify session remains intact and healthy on disk
    let current_content = fs::read_to_string(&session_path).unwrap();
    assert_eq!(current_content, original_content);

    // Verify that subsequent prompts on the SAME session work normally
    fake.clear_async_delay();
    let second_prompt = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "Try again" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", second_prompt).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // Now should receive update chunk and end_turn
    let update_msg = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(update_msg["method"], "session/update");
    let second_resp = read_jsonrpc_line(&mut client_reader).await;
    assert_eq!(second_resp["id"], 4);
    assert_eq!(second_resp["result"]["stopReason"], "end_turn");

    drop(client_writer);
    let _ = server_handle.await.unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_acp_session_close_during_in_flight_prompt_cancels_cleanly() {
    let dir = temp_session_dir("acp_close");
    let (session_path, _) = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        sample_edits(),
    );

    let (mut client_writer, server_reader) = tokio::io::duplex(4096);
    let (server_writer, client_reader) = tokio::io::duplex(4096);

    let server_transport = ByteStreams::new(server_writer.compat_write(), server_reader.compat());

    let fake = FakeProvider::new();
    fake.set_async_delay(Duration::from_secs(3));

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::with_explainer(config, Arc::new(fake));
    let server = AgentServer::with_provider(provider);

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });
    let mut client_reader = BufReader::new(client_reader);

    // Initialize & Session New
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

    let session_new_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": { "cwd": "/workspace", "mcpServers": [] }
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

    // Start prompt
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [{ "type": "text", "text": "In flight request" }]
        }
    });
    client_writer
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // While prompt is in flight, send session/close
    tokio::time::sleep(Duration::from_millis(50)).await;

    let close_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "session/close",
        "params": { "sessionId": session_id }
    });
    client_writer
        .write_all(format!("{}\n", close_req).as_bytes())
        .await
        .unwrap();
    client_writer.flush().await.unwrap();

    // Both prompt (aborted) and close respond cleanly
    let resp1 = read_jsonrpc_line(&mut client_reader).await;
    let resp2 = read_jsonrpc_line(&mut client_reader).await;
    let (prompt_resp, close_resp) = if resp1["id"] == 3 {
        (resp1, resp2)
    } else {
        (resp2, resp1)
    };
    assert_eq!(prompt_resp["id"], 3);
    assert_eq!(prompt_resp["result"]["stopReason"], "cancelled");
    assert_eq!(close_resp["id"], 4);

    drop(client_writer);
    let result = server_handle.await.unwrap();
    assert!(
        result.is_ok(),
        "Server must shutdown cleanly with no lingering tasks"
    );
    let _ = fs::remove_dir_all(&dir);
}
