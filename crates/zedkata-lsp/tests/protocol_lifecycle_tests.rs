use tokio::io::AsyncReadExt;
use zedkata_lsp::{
    codec::{write_message, MessageReader},
    protocol::*,
    run_server,
};

#[tokio::test]
async fn test_full_protocol_lifecycle_handshake() {
    let (client_io, server_io) = tokio::io::duplex(8192);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    let server_task = tokio::spawn(async move { run_server(server_read, server_write).await });

    let mut reader = MessageReader::new(client_read);

    // 1. Send request BEFORE initialize -> must be rejected with SERVER_NOT_INITIALIZED
    let premature_req = Request::new(RequestId::Int(1), "textDocument/hover", None);
    let payload = serde_json::to_vec(&premature_req).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    let resp_bytes = reader
        .read_message()
        .await
        .unwrap()
        .expect("response expected");
    let resp: Response = serde_json::from_slice(&resp_bytes).unwrap();
    assert_eq!(resp.id, RequestId::Int(1));
    assert!(resp.result.is_none());
    let err = resp.error.expect("error expected for pre-init request");
    assert_eq!(err.code, SERVER_NOT_INITIALIZED);

    // 2. Send initialize request
    let init_params = serde_json::json!({
        "processId": 12345,
        "rootUri": "file:///workspace",
        "capabilities": {
            "general": {
                "positionEncodings": ["utf-16"]
            }
        },
        "clientInfo": {
            "name": "Zed",
            "version": "1.19.2"
        }
    });
    let init_req = Request::new(RequestId::Int(2), "initialize", Some(init_params));
    let payload = serde_json::to_vec(&init_req).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    let resp_bytes = reader
        .read_message()
        .await
        .unwrap()
        .expect("init response expected");
    let resp: Response = serde_json::from_slice(&resp_bytes).unwrap();
    assert_eq!(resp.id, RequestId::Int(2));
    assert!(resp.error.is_none());

    let init_res: InitializeResult = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert_eq!(
        init_res.capabilities.position_encoding,
        Some("utf-16".to_string())
    );
    let sync = init_res
        .capabilities
        .text_document_sync
        .expect("sync capabilities");
    assert_eq!(sync.open_close, Some(true));
    assert_eq!(sync.change, Some(TEXT_DOCUMENT_SYNC_INCREMENTAL));
    assert_eq!(init_res.server_info.unwrap().name, "zedkata-lsp");

    // 3. Send initialized notification
    let initialized_notif = Notification::new("initialized", Some(serde_json::json!({})));
    let payload = serde_json::to_vec(&initialized_notif).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    // 4. Send non-observer editor queries -> all return clean empty/null responses
    let non_observer_methods = [
        "textDocument/hover",
        "textDocument/completion",
        "textDocument/signatureHelp",
        "textDocument/documentSymbol",
        "textDocument/formatting",
        "textDocument/rangeFormatting",
        "textDocument/codeAction",
        "textDocument/inlayHint",
    ];

    for (idx, method) in non_observer_methods.iter().enumerate() {
        let req_id = RequestId::Int(100 + idx as i64);
        let req = Request::new(req_id.clone(), *method, Some(serde_json::json!({})));
        let payload = serde_json::to_vec(&req).unwrap();
        write_message(&mut client_write, &payload).await.unwrap();

        let resp_bytes = reader
            .read_message()
            .await
            .unwrap()
            .expect("empty query response");
        let resp: Response = serde_json::from_slice(&resp_bytes).unwrap();
        assert_eq!(resp.id, req_id);
        assert!(resp.error.is_none());
        assert_eq!(resp.result, Some(serde_json::Value::Null));
    }

    // 5. Send unknown method -> returns METHOD_NOT_FOUND (-32601)
    let unknown_req = Request::new(RequestId::Int(999), "workspace/unknownCommand", None);
    let payload = serde_json::to_vec(&unknown_req).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    let resp_bytes = reader
        .read_message()
        .await
        .unwrap()
        .expect("unknown method response");
    let resp: Response = serde_json::from_slice(&resp_bytes).unwrap();
    assert_eq!(resp.id, RequestId::Int(999));
    let err = resp.error.expect("method not found error expected");
    assert_eq!(err.code, METHOD_NOT_FOUND);

    // 6. Send shutdown request -> returns null result
    let shutdown_req = Request::new(RequestId::Int(1000), "shutdown", None);
    let payload = serde_json::to_vec(&shutdown_req).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    let resp_bytes = reader
        .read_message()
        .await
        .unwrap()
        .expect("shutdown response");
    let resp: Response = serde_json::from_slice(&resp_bytes).unwrap();
    assert_eq!(resp.id, RequestId::Int(1000));
    assert!(resp.error.is_none());
    assert_eq!(resp.result, Some(serde_json::Value::Null));

    // 7. Request after shutdown -> returns INVALID_REQUEST (-32600)
    let post_shutdown_req = Request::new(RequestId::Int(1001), "textDocument/hover", None);
    let payload = serde_json::to_vec(&post_shutdown_req).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    let resp_bytes = reader
        .read_message()
        .await
        .unwrap()
        .expect("post-shutdown response");
    let resp: Response = serde_json::from_slice(&resp_bytes).unwrap();
    assert_eq!(resp.id, RequestId::Int(1001));
    let err = resp.error.expect("invalid request error expected");
    assert_eq!(err.code, INVALID_REQUEST);

    // 8. Send exit notification
    let exit_notif = Notification::new("exit", None);
    let payload = serde_json::to_vec(&exit_notif).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    // 9. Await server exit code -> must be 0 after clean shutdown
    let exit_code = server_task.await.unwrap().unwrap();
    assert_eq!(exit_code, 0);
}

#[tokio::test]
async fn test_exit_without_shutdown_returns_nonzero() {
    let (client_io, server_io) = tokio::io::duplex(4096);
    let (_client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    let server_task = tokio::spawn(async move { run_server(server_read, server_write).await });

    // Send exit directly without prior shutdown
    let exit_notif = Notification::new("exit", None);
    let payload = serde_json::to_vec(&exit_notif).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    let exit_code = server_task.await.unwrap().unwrap();
    assert_eq!(exit_code, 1);
}

#[tokio::test]
async fn test_clean_stdout_framing() {
    let (client_io, server_io) = tokio::io::duplex(4096);
    let (mut client_read, mut client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    tokio::spawn(async move { run_server(server_read, server_write).await });

    let init_req = Request::new(RequestId::Int(42), "initialize", None);
    let payload = serde_json::to_vec(&init_req).unwrap();
    write_message(&mut client_write, &payload).await.unwrap();

    // Read raw bytes from client_read
    let mut raw_bytes = vec![0u8; 1024];
    let n = client_read.read(&mut raw_bytes).await.unwrap();
    let received = &raw_bytes[..n];

    // Must start with Content-Length:
    assert!(
        received.starts_with(b"Content-Length: "),
        "Server output must strictly start with Content-Length header, got: {:?}",
        String::from_utf8_lossy(received)
    );

    // Must contain \r\n\r\n
    let header_sep = b"\r\n\r\n";
    let sep_pos = received
        .windows(header_sep.len())
        .position(|w| w == header_sep)
        .expect("must contain header separator \\r\\n\\r\\n");

    let header_str = std::str::from_utf8(&received[..sep_pos]).unwrap();
    let body_bytes = &received[sep_pos + 4..];

    // Body must be valid JSON matching Content-Length
    let content_len: usize = header_str
        .strip_prefix("Content-Length: ")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(body_bytes.len(), content_len);

    let val: serde_json::Value = serde_json::from_slice(body_bytes).unwrap();
    assert_eq!(val["id"], 42);
    assert!(val.get("result").is_some());
}
