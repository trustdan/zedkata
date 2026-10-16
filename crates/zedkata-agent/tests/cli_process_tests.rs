use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Read a line from reader with timeout
async fn read_line_with_timeout<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    timeout_duration: Duration,
) -> String {
    let mut line = String::new();
    let res = tokio::time::timeout(timeout_duration, reader.read_line(&mut line)).await;
    match res {
        Ok(Ok(0)) => panic!("EOF reached while waiting for child process output"),
        Ok(Ok(_)) => line,
        Ok(Err(e)) => panic!("I/O error reading from child process: {e}"),
        Err(_) => panic!(
            "Timeout reading from child process after {:?}",
            timeout_duration
        ),
    }
}

#[tokio::test]
async fn test_cli_help_and_version() {
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let help_output = Command::new(agent_bin)
        .arg("--help")
        .output()
        .await
        .expect("Failed to execute zedkata-agent --help");

    assert!(help_output.status.success());
    let stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(stdout.contains("Zedkata ACP Agent"));
    assert!(stdout.contains("--acp"));

    let version_output = Command::new(agent_bin)
        .arg("--version")
        .output()
        .await
        .expect("Failed to execute zedkata-agent --version");

    assert!(version_output.status.success());
    let version_str = String::from_utf8_lossy(&version_output.stdout);
    assert!(version_str.contains("zedkata-agent"));
}

#[tokio::test]
async fn test_cli_missing_acp_flag_gives_usage_and_exits() {
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let output = Command::new(agent_bin)
        .output()
        .await
        .expect("Failed to execute zedkata-agent without flags");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Use --acp to launch the Agent Client Protocol server over stdio."));
}

#[tokio::test]
async fn test_cli_invalid_binary_name_gives_actionable_os_error() {
    let bad_bin = "non_existent_zedkata_agent_binary_xyz_12345.exe";

    let res = Command::new(bad_bin).spawn();
    assert!(res.is_err(), "Spawning non-existent binary must fail");
    let err = res.unwrap_err();
    assert_eq!(
        err.kind(),
        std::io::ErrorKind::NotFound,
        "Error kind must be NotFound"
    );
}

#[tokio::test]
async fn test_cli_acp_stdio_child_process_lifecycle() {
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let mut child = Command::new(agent_bin)
        .arg("--acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn zedkata-agent --acp");

    let mut stdin = child.stdin.take().expect("Child stdin must be piped");
    let stdout = child.stdout.take().expect("Child stdout must be piped");
    let mut reader = BufReader::new(stdout);

    // 1. Send ACP initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": 1,
            "clientInfo": {
                "name": "zed-editor",
                "version": "1.19.2"
            }
        }
    });

    stdin
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let line = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
    let resp: serde_json::Value =
        serde_json::from_str(line.trim()).expect("Stdout line must be valid JSON-RPC");

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["protocolVersion"], 1);
    assert_eq!(resp["result"]["agentInfo"]["name"], "zedkata-agent");

    // 2. Send session/new
    let new_session_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": "C:\\Users\\Dan\\zedkata",
            "mcpServers": []
        }
    });

    stdin
        .write_all(format!("{}\n", new_session_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let line = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
    let session_resp: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(session_resp["id"], 2);
    let session_id = session_resp["result"]["sessionId"]
        .as_str()
        .expect("sessionId should be string");
    assert!(session_id.starts_with("sess-"));

    // 3. Send session/prompt
    let prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": "Hello Zedkata agent!"
                }
            ]
        }
    });

    stdin
        .write_all(format!("{}\n", prompt_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    // Read update notification
    let line1 = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
    let notif_or_resp: serde_json::Value = serde_json::from_str(line1.trim()).unwrap();

    // In ACP, the agent streams session/update notifications then responds to prompt
    if notif_or_resp.get("method").is_some() {
        assert_eq!(notif_or_resp["method"], "session/update");
        // Read subsequent prompt response
        let line2 = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
        let prompt_resp: serde_json::Value = serde_json::from_str(line2.trim()).unwrap();
        assert_eq!(prompt_resp["id"], 3);
        assert_eq!(prompt_resp["result"]["stopReason"], "end_turn");
    } else {
        assert_eq!(notif_or_resp["id"], 3);
        assert_eq!(notif_or_resp["result"]["stopReason"], "end_turn");
    }

    // 4. Send session/close
    let close_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "session/close",
        "params": {
            "sessionId": session_id
        }
    });

    stdin
        .write_all(format!("{}\n", close_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let close_line = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
    let close_resp: serde_json::Value = serde_json::from_str(close_line.trim()).unwrap();
    assert_eq!(close_resp["id"], 4);

    // 5. Close stdin and verify child exits cleanly
    drop(stdin);
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("Child process wait timed out")
        .expect("Child process failed to exit");

    assert!(status.success(), "Child process should exit cleanly on EOF");
}
