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
