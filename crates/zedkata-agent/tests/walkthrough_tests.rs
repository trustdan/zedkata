use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use zedkata_core::session::SessionState;

fn setup_temp_workspace(prefix: &str) -> (PathBuf, PathBuf) {
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_{}_{}",
        prefix,
        uuid::Uuid::new_v4().simple()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let session_file = temp_dir.join("session.json");
    (temp_dir, session_file)
}

fn cleanup_temp_workspace(temp_dir: &Path) {
    let _ = std::fs::remove_dir_all(temp_dir);
}

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
async fn test_cli_walkthrough_list_presents_all_ten_records() {
    let (temp_dir, session_path) = setup_temp_workspace("cli_wt_list");
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("list")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to execute zedkata-agent walkthrough list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify header and initial count
    assert!(stdout.contains("Zedkata Guided Walkthroughs (Unscored per REQ-25 / REQ-29)"));
    assert!(stdout.contains("Progress: 0 / 10 completed"));

    // Verify all 5 feature walkthroughs
    assert!(stdout.contains("lsp-actions-walkthrough"));
    assert!(stdout.contains("project-panel-walkthrough"));
    assert!(stdout.contains("multibuffer-walkthrough"));
    assert!(stdout.contains("tasks-and-debugger-walkthrough"));
    assert!(stdout.contains("agent-panel-walkthrough"));

    // Verify Mini/Any and conceptual walkthroughs
    assert!(stdout.contains("mini-vs-any"));
    assert!(stdout.contains("keymap-contexts"));
    assert!(stdout.contains("exchange-and-surround"));
    assert!(stdout.contains("single-line-find-scope"));
    assert!(stdout.contains("diagnostic-navigation"));

    cleanup_temp_workspace(&temp_dir);
}

#[tokio::test]
async fn test_cli_walkthrough_show_mini_vs_any_and_feature_walkthroughs() {
    let (temp_dir, session_path) = setup_temp_workspace("cli_wt_show");
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    // 1. Show Mini/Any walkthrough
    let mini_output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("show")
        .arg("mini-vs-any")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to show mini-vs-any");

    assert!(mini_output.status.success());
    let mini_stdout = String::from_utf8_lossy(&mini_output.stdout);
    assert!(
        mini_stdout.contains("Guided Walkthrough: When MiniBrackets disagrees with AnyBrackets")
    );
    assert!(mini_stdout.contains("motions-03-textobjects"));
    assert!(mini_stdout.contains("nested.start.kata"));
    assert!(mini_stdout.contains("Verified Step-by-Step Instructions"));
    assert!(mini_stdout.contains("Explicit Self-Report Verification Checklist (REQ-29)"));
    assert!(mini_stdout.contains("[ ] Observed lexical delimiter selection using `ci)` / `ci(`"));

    // 2. Show LSP actions feature walkthrough
    let lsp_output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("show")
        .arg("lsp-actions-walkthrough")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to show lsp-actions-walkthrough");

    assert!(lsp_output.status.success());
    let lsp_stdout = String::from_utf8_lossy(&lsp_output.stdout);
    assert!(lsp_stdout.contains("Normal-mode LSP intelligence in Zed"));
    assert!(lsp_stdout.contains("sample_service.rs"));
    assert!(lsp_stdout.contains("Multi-site rename"));
    assert!(lsp_stdout.contains("Code action"));
    assert!(lsp_stdout.contains("Diagnostic navigation"));

    cleanup_temp_workspace(&temp_dir);
}

#[tokio::test]
async fn test_cli_walkthrough_complete_and_reset_lifecycle_without_drill_scoring() {
    let (temp_dir, session_path) = setup_temp_workspace("cli_wt_lifecycle");
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    // 1. Mark mini-vs-any completed
    let comp_output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("complete")
        .arg("mini-vs-any")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to complete walkthrough");

    assert!(comp_output.status.success());
    let comp_stdout = String::from_utf8_lossy(&comp_output.stdout);
    assert!(comp_stdout.contains("Marked walkthrough 'mini-vs-any'"));
    assert!(comp_stdout.contains("COMPLETED"));

    // 2. Verify session.json directly
    let session = SessionState::load_from_file(&session_path).expect("session.json must exist");
    assert!(session.is_walkthrough_completed("mini-vs-any"));
    assert_eq!(session.completed_walkthroughs.len(), 1);

    // CRITICAL: Zero drill scoring interference
    assert!(session.attempt.is_none());
    assert!(session.history.is_empty());
    assert!(session.weaknesses.is_empty());

    // 3. Verify walkthrough list shows 1 / 10 and [✓] badge
    let list_output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("list")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to list walkthroughs");

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("Progress: 1 / 10 completed"));
    assert!(list_stdout.contains("[✓] mini-vs-any"));

    // 4. Verify detail view shows COMPLETED (Self-Reported) and [x] in checklist
    let show_output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("show")
        .arg("mini-vs-any")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to show walkthrough");

    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(show_stdout.contains("[✓] COMPLETED (Self-Reported)"));
    assert!(show_stdout.contains("[x] Observed lexical delimiter selection"));

    // 5. Reset walkthrough completion
    let reset_output = Command::new(agent_bin)
        .arg("walkthrough")
        .arg("reset")
        .arg("mini-vs-any")
        .arg("--session")
        .arg(&session_path)
        .output()
        .await
        .expect("Failed to reset walkthrough");

    assert!(reset_output.status.success());
    let reset_stdout = String::from_utf8_lossy(&reset_output.stdout);
    assert!(reset_stdout.contains("Reset walkthrough 'mini-vs-any'"));

    // 6. Verify session reflects reset
    let reset_session = SessionState::load_from_file(&session_path).unwrap();
    assert!(!reset_session.is_walkthrough_completed("mini-vs-any"));
    assert!(reset_session.completed_walkthroughs.is_empty());

    cleanup_temp_workspace(&temp_dir);
}

#[tokio::test]
async fn test_acp_walkthrough_prompt_journey() {
    let (temp_dir, _session_path) = setup_temp_workspace("acp_wt_journey");
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let mut child = Command::new(agent_bin)
        .arg("--acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn zedkata-agent --acp");

    let mut stdin = child.stdin.take().expect("Child stdin");
    let stdout = child.stdout.take().expect("Child stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": 1,
            "clientInfo": { "name": "zed-editor", "version": "1.19.2" }
        }
    });
    stdin
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let init_resp = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
    assert!(init_resp.contains("zedkata-agent"));

    // 2. New session
    let new_sess_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": temp_dir.to_string_lossy(),
            "mcpServers": []
        }
    });
    stdin
        .write_all(format!("{}\n", new_sess_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let new_sess_resp = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
    let sess_val: serde_json::Value = serde_json::from_str(&new_sess_resp).unwrap();
    let session_id = sess_val["result"]["sessionId"]
        .as_str()
        .expect("sessionId must be string")
        .to_string();

    // 3. Send "walkthrough list" prompt
    let list_prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": "walkthrough list"
                }
            ]
        }
    });
    stdin
        .write_all(format!("{}\n", list_prompt_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    // Read notification chunks and prompt response
    let mut received_list_text = String::new();
    loop {
        let line = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            if v["method"] == "session/update" {
                if let Some(text) = v["params"]["update"]["content"]["text"].as_str() {
                    received_list_text.push_str(text);
                }
            }
            if v["id"] == 3 {
                break;
            }
        }
    }

    assert!(received_list_text.contains("Zedkata Guided Walkthroughs"));
    assert!(received_list_text.contains("mini-vs-any"));
    assert!(received_list_text.contains("lsp-actions-walkthrough"));

    // 4. Send "guide me through mini-vs-any" prompt
    let detail_prompt_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": "guide me through mini-vs-any"
                }
            ]
        }
    });
    stdin
        .write_all(format!("{}\n", detail_prompt_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let mut received_detail_text = String::new();
    loop {
        let line = read_line_with_timeout(&mut reader, Duration::from_secs(5)).await;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            if v["method"] == "session/update" {
                if let Some(text) = v["params"]["update"]["content"]["text"].as_str() {
                    received_detail_text.push_str(text);
                }
            }
            if v["id"] == 4 {
                break;
            }
        }
    }

    assert!(received_detail_text.contains("When MiniBrackets disagrees with AnyBrackets"));
    assert!(received_detail_text.contains("Verified Step-by-Step Instructions"));
    assert!(received_detail_text.contains("Explicit Self-Report Verification Checklist"));

    let _ = child.kill().await;
    cleanup_temp_workspace(&temp_dir);
}

async fn send_prompt_collect_response(
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut BufReader<tokio::process::ChildStdout>,
    req_id: i64,
    session_id: &str,
    prompt: &str,
) -> String {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": prompt
                }
            ]
        }
    });
    stdin
        .write_all(format!("{}\n", req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let mut text_acc = String::new();
    loop {
        let line = read_line_with_timeout(reader, Duration::from_secs(5)).await;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            if v["method"] == "session/update" {
                if let Some(t) = v["params"]["update"]["content"]["text"].as_str() {
                    text_acc.push_str(t);
                }
            }
            if v["id"] == req_id {
                break;
            }
        }
    }
    text_acc
}

#[tokio::test]
async fn test_acp_walkthrough_user_journey_complete_restart_and_unseen_actions_verification() {
    let (temp_dir, session_path) = setup_temp_workspace("acp_wt_accept");
    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    // =========================================================================
    // STEP 1: Launch ACP agent process 1
    // =========================================================================
    let mut child1 = Command::new(agent_bin)
        .arg("--acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn zedkata-agent --acp");

    let mut stdin1 = child1.stdin.take().expect("Child stdin 1");
    let stdout1 = child1.stdout.take().expect("Child stdout 1");
    let mut reader1 = BufReader::new(stdout1);

    // Initialize ACP handshake
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": 1,
            "clientInfo": { "name": "zed-editor", "version": "1.19.2" }
        }
    });
    stdin1
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    stdin1.flush().await.unwrap();
    let init_resp = read_line_with_timeout(&mut reader1, Duration::from_secs(5)).await;
    assert!(init_resp.contains("zedkata-agent"));

    // Create ACP Session 1
    let new_sess_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": temp_dir.to_string_lossy(),
            "mcpServers": []
        }
    });
    stdin1
        .write_all(format!("{}\n", new_sess_req).as_bytes())
        .await
        .unwrap();
    stdin1.flush().await.unwrap();

    let new_sess_resp = read_line_with_timeout(&mut reader1, Duration::from_secs(5)).await;
    let sess_val: serde_json::Value = serde_json::from_str(&new_sess_resp).unwrap();
    let session_id_1 = sess_val["result"]["sessionId"]
        .as_str()
        .expect("sessionId must be string")
        .to_string();

    // =========================================================================
    // STEP 2: Request walkthrough guidance (e.g. LSP Actions feature walkthrough)
    // =========================================================================
    let guide_resp = send_prompt_collect_response(
        &mut stdin1,
        &mut reader1,
        3,
        &session_id_1,
        "guide me through lsp-actions",
    )
    .await;

    // Verify verified steps and self-report checklist are presented
    assert!(guide_resp.contains("Normal-mode LSP intelligence in Zed"));
    assert!(guide_resp.contains("Verified Step-by-Step Instructions"));
    assert!(guide_resp.contains("Multi-site rename"));
    assert!(guide_resp.contains("Code action"));
    assert!(guide_resp.contains("Diagnostic navigation"));
    assert!(guide_resp.contains("Explicit Self-Report Verification Checklist (REQ-29)"));
    assert!(guide_resp.contains("[ ] PENDING (Not Yet Completed)"));
    assert!(guide_resp.contains("Unscored guided walkthrough (REQ-25 / REQ-29)"));

    // Verify prompt does NOT claim unseen editor actions were observed
    assert!(!guide_resp.contains("I observed your"));
    assert!(!guide_resp.contains("I detected your"));
    assert!(!guide_resp.contains("scored attempt"));
    assert!(!guide_resp.contains("edits verified in buffer"));

    // =========================================================================
    // STEP 3: Verify prompts reject false claims of observing unseen editor actions
    // =========================================================================
    let query_obs = send_prompt_collect_response(
        &mut stdin1,
        &mut reader1,
        4,
        &session_id_1,
        "Did you observe my walkthrough actions in Zed?",
    )
    .await;

    assert!(query_obs.contains("Walkthrough Observation & Scoring Policy"));
    assert!(query_obs.contains("does not claim or attempt to observe unseen editor actions"));
    assert!(query_obs.contains("Non-Observable Workflows"));
    assert!(query_obs.contains("Explicit Self-Reported Completion"));
    assert!(query_obs.contains("Zero Drill Scoring"));

    // =========================================================================
    // STEP 4: Complete the walkthrough manually with the agent
    // =========================================================================
    let complete_resp = send_prompt_collect_response(
        &mut stdin1,
        &mut reader1,
        5,
        &session_id_1,
        "complete walkthrough lsp-actions",
    )
    .await;

    assert!(complete_resp.contains("Walkthrough Completed (Self-Reported)"));
    assert!(complete_resp.contains("[✓] COMPLETED"));
    assert!(complete_resp.contains("Guided walkthroughs use explicit self-reported verification without entering drill scoring (REQ-29)."));

    // Verify disk persistence in session.json
    let session_on_disk =
        SessionState::load_from_file(&session_path).expect("session.json must exist");
    assert!(session_on_disk.is_walkthrough_completed("lsp-actions-walkthrough"));
    assert_eq!(session_on_disk.completed_walkthroughs.len(), 1);

    // Verify absolute zero drill scoring mutation
    assert!(session_on_disk.attempt.is_none());
    assert!(session_on_disk.history.is_empty());
    assert!(session_on_disk.weaknesses.is_empty());

    // =========================================================================
    // STEP 5: Restart according to contract (Phase A: Process & Session Restart)
    // =========================================================================
    // Close Session 1
    let close_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "session/close",
        "params": {
            "sessionId": session_id_1
        }
    });
    stdin1
        .write_all(format!("{}\n", close_req).as_bytes())
        .await
        .unwrap();
    stdin1.flush().await.unwrap();
    let _ = read_line_with_timeout(&mut reader1, Duration::from_secs(5)).await;

    // Terminate child 1 process
    let _ = child1.kill().await;

    // Spawn brand new agent child process 2 (simulating Zed restarting or reopening workspace)
    let mut child2 = Command::new(agent_bin)
        .arg("--acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to restart zedkata-agent --acp");

    let mut stdin2 = child2.stdin.take().expect("Child stdin 2");
    let stdout2 = child2.stdout.take().expect("Child stdout 2");
    let mut reader2 = BufReader::new(stdout2);

    // Initialize handshake on restarted process
    let init_req2 = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": 1,
            "clientInfo": { "name": "zed-editor", "version": "1.19.2" }
        }
    });
    stdin2
        .write_all(format!("{}\n", init_req2).as_bytes())
        .await
        .unwrap();
    stdin2.flush().await.unwrap();
    let _ = read_line_with_timeout(&mut reader2, Duration::from_secs(5)).await;

    // Create new session in restarted process pointing to the same workspace cwd
    let new_sess_req2 = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "cwd": temp_dir.to_string_lossy(),
            "mcpServers": []
        }
    });
    stdin2
        .write_all(format!("{}\n", new_sess_req2).as_bytes())
        .await
        .unwrap();
    stdin2.flush().await.unwrap();
    let new_sess_resp2 = read_line_with_timeout(&mut reader2, Duration::from_secs(5)).await;
    let sess_val2: serde_json::Value = serde_json::from_str(&new_sess_resp2).unwrap();
    let session_id_2 = sess_val2["result"]["sessionId"]
        .as_str()
        .expect("sessionId must be string")
        .to_string();

    // Query list on restarted agent: completion persisted across restart
    let restarted_list = send_prompt_collect_response(
        &mut stdin2,
        &mut reader2,
        3,
        &session_id_2,
        "walkthrough list",
    )
    .await;

    assert!(restarted_list.contains("Progress: 1 / 10 completed"));
    assert!(restarted_list.contains("[✓] lsp-actions-walkthrough"));

    // Query detail on restarted agent: shows completed badges and checkboxes
    let restarted_detail = send_prompt_collect_response(
        &mut stdin2,
        &mut reader2,
        4,
        &session_id_2,
        "guide me through lsp-actions",
    )
    .await;

    assert!(restarted_detail.contains("[✓] COMPLETED (Self-Reported)"));
    assert!(restarted_detail.contains("[x] Multi-site rename executed with `c d`"));
    assert!(restarted_detail.contains("Unscored guided walkthrough (REQ-25 / REQ-29)"));

    // =========================================================================
    // STEP 6: Walkthrough restart / reset contract (Phase B: Re-attempting)
    // =========================================================================
    let reset_resp = send_prompt_collect_response(
        &mut stdin2,
        &mut reader2,
        5,
        &session_id_2,
        "reset walkthrough lsp-actions",
    )
    .await;

    assert!(reset_resp.contains("### Walkthrough Reset"));
    assert!(reset_resp.contains("[ ] PENDING"));

    // Verify session.json reflects reset
    let reset_session = SessionState::load_from_file(&session_path).unwrap();
    assert!(!reset_session.is_walkthrough_completed("lsp-actions-walkthrough"));
    assert!(reset_session.completed_walkthroughs.is_empty());

    // Query list after reset: shows 0 / 10
    let list_after_reset = send_prompt_collect_response(
        &mut stdin2,
        &mut reader2,
        6,
        &session_id_2,
        "walkthrough list",
    )
    .await;
    assert!(list_after_reset.contains("Progress: 0 / 10 completed"));
    assert!(list_after_reset.contains("[ ] lsp-actions-walkthrough"));

    // Re-complete walkthrough again
    let re_complete_resp = send_prompt_collect_response(
        &mut stdin2,
        &mut reader2,
        7,
        &session_id_2,
        "complete walkthrough lsp-actions",
    )
    .await;
    assert!(re_complete_resp.contains("Walkthrough Completed (Self-Reported)"));
    assert!(re_complete_resp.contains("[✓] COMPLETED"));

    let final_session = SessionState::load_from_file(&session_path).unwrap();
    assert!(final_session.is_walkthrough_completed("lsp-actions-walkthrough"));
    assert!(final_session.attempt.is_none());

    let _ = child2.kill().await;
    cleanup_temp_workspace(&temp_dir);
}
