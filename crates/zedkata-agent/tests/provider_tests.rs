use chrono::{Duration, Utc};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use zedkata_agent::{
    build_attempt_user_prompt, system_prompt, AgentConfig, AttemptContext, AttemptExplainer,
    AttemptVerdictContext, ConfiguredProvider, ContextConfig, ForbidRulesContext, ProviderError,
    ProviderKind,
};
use zedkata_core::session::{
    Attempt, CurrentSession, EditRange, RecordedEdit, SessionState, StudentInfo, Verdict,
    CURRENT_SCHEMA_VERSION,
};

fn temp_session_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zedkata_provider_test_{}_{}_{}",
        name,
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn sample_attempt_context(is_truncated: bool) -> AttemptContext {
    AttemptContext {
        lesson_id: "baseline-01-insert-mode".to_string(),
        lesson_title: Some("Insert Mode Baseline".to_string()),
        concepts: vec!["deletion".to_string(), "motion".to_string()],
        exercise_id: Some("insert-01-delete".to_string()),
        exercise_index: 0,
        task_prompt: Some("Delete word 'foo' using dw".to_string()),
        par_edits: Some(1),
        forbid_rules: Some(ForbidRulesContext {
            backspace_run: Some(3),
            unsupported: Vec::new(),
        }),
        attempt_id: "att-sample-123".to_string(),
        total_edits: if is_truncated { 25 } else { 4 },
        bounded_edits: vec![
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 5),
                    end: (0, 6),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 4),
                    end: (0, 5),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 3),
                    end: (0, 4),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
            RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 2),
                    end: (0, 3),
                },
                deleted_len: 1,
                inserted: String::new(),
            },
        ],
        truncated_edits_count: if is_truncated { 21 } else { 0 },
        is_truncated,
        total_diff_bytes: 48,
        diff_summary: "Deleted 4 characters via backspaces".to_string(),
        verdict: AttemptVerdictContext::Failed {
            reason: "Violated forbid rule: backspace_run (consecutive backspaces > 3)".to_string(),
        },
        age_seconds: 10,
    }
}

fn create_temp_session(
    dir: &std::path::Path,
    verdict: Option<Verdict>,
    edits: Vec<RecordedEdit>,
) -> PathBuf {
    let session_path = dir.join("session.json");
    let edit_count = edits.len() as u32;
    let state = SessionState {
        version: CURRENT_SCHEMA_VERSION,
        student: StudentInfo {
            started_at: Utc::now() - Duration::hours(1),
        },
        current: CurrentSession {
            lesson_id: "operators-01-edit".to_string(),
            exercise_index: 0,
            buffer_path: Some("temp.kata".to_string()),
            opened_at: Some(Utc::now() - Duration::minutes(10)),
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
    fs::write(&session_path, json).unwrap();
    session_path
}

#[test]
fn test_system_prompt_integrity() {
    let prompt = system_prompt();
    assert!(prompt.contains("Zedkata Pedagogical Coach"));
    assert!(prompt.contains("Strict Context Grounding"));
    assert!(prompt.contains("Observations"));
    assert!(prompt.contains("Technique Inference"));
    assert!(prompt.contains("Recommendation"));
}

#[test]
fn test_build_attempt_user_prompt_formatting() {
    let ctx = sample_attempt_context(false);
    let prompt = build_attempt_user_prompt(&ctx);

    assert!(prompt.contains("Insert Mode Baseline"));
    assert!(prompt.contains("baseline-01-insert-mode"));
    assert!(prompt.contains("Delete word 'foo' using dw"));
    assert!(prompt.contains("**Par Budget**: 1 edit(s)"));
    assert!(prompt.contains("`backspace_run` active"));
    assert!(prompt.contains("Violated forbid rule: backspace_run"));
    assert!(prompt.contains("**Total Edits Executed**: 4"));
    assert!(!prompt.contains("omitted due to 20-edit"));
}

#[test]
fn test_build_attempt_user_prompt_with_truncation() {
    let ctx = sample_attempt_context(true);
    let prompt = build_attempt_user_prompt(&ctx);

    assert!(prompt.contains("**Total Edits Executed**: 25"));
    assert!(prompt.contains("Earlier 21 edit(s) omitted due to 20-edit / 2 KB bound limit"));
}

#[test]
fn test_grounded_explanation_separates_observations_from_inferences() {
    let ctx = sample_attempt_context(false);
    let provider = ConfiguredProvider::new(AgentConfig {
        provider: ProviderKind::Fake,
        ..Default::default()
    });

    let explanation = provider.explain_attempt(&ctx).expect("Should explain");

    // Must be bounded
    assert!(explanation.is_bounded);

    // Observations must contain objective metrics
    assert!(!explanation.observations.is_empty());
    let obs_str = explanation.observations.join(" ");
    assert!(obs_str.contains("Attempt executed 4 edits"));
    assert!(obs_str.contains("Verdict: Failed with reason"));
    assert!(obs_str.contains("Edit 1:"));

    // Inferences must contain technique analysis, not raw numbers
    assert!(!explanation.inferences.is_empty());
    let inf_str = explanation.inferences.join(" ");
    assert!(inf_str.contains("Inferred technique"));
    assert!(inf_str.contains("backspace"));

    // Recommendation must provide actionable Vim motion
    assert!(
        explanation.recommendation.contains("dw") || explanation.recommendation.contains("motion")
    );

    // Markdown conversion
    let markdown = explanation.to_markdown();
    assert!(markdown.contains("### Observations"));
    assert!(markdown.contains("### Technique Inference"));
    assert!(markdown.contains("### Recommendation"));
}

#[test]
fn test_configured_provider_offline_modes_work_without_network() {
    let ctx = sample_attempt_context(false);

    // All provider kinds should succeed in offline environments
    let kinds = [
        ProviderKind::Fake,
        ProviderKind::Degraded,
        ProviderKind::LmStudio,
        ProviderKind::Ollama,
        ProviderKind::Anthropic,
        ProviderKind::OpenAI,
        ProviderKind::Gemini,
    ];

    for kind in kinds {
        let config = AgentConfig {
            provider: kind,
            model: "test-model".to_string(),
            ..Default::default()
        };
        let provider = ConfiguredProvider::new(config);
        let explanation = provider
            .explain_attempt(&ctx)
            .unwrap_or_else(|e| panic!("Provider {:?} failed offline: {}", kind, e));

        assert!(
            !explanation.observations.is_empty(),
            "Provider {:?} empty obs",
            kind
        );
        assert!(
            !explanation.inferences.is_empty(),
            "Provider {:?} empty inf",
            kind
        );
        assert!(
            !explanation.recommendation.is_empty(),
            "Provider {:?} empty rec",
            kind
        );
    }
}

#[test]
fn test_configured_provider_explains_real_session_file() {
    let dir = temp_session_dir("real_session");
    let edits = vec![
        RecordedEdit {
            at: Utc::now() - Duration::seconds(10),
            range: EditRange {
                start: (0, 3),
                end: (0, 4),
            },
            inserted: String::new(),
            deleted_len: 1,
        },
        RecordedEdit {
            at: Utc::now() - Duration::seconds(5),
            range: EditRange {
                start: (0, 2),
                end: (0, 3),
            },
            inserted: String::new(),
            deleted_len: 1,
        },
    ];

    let session_path = create_temp_session(
        &dir,
        Some(Verdict::Failed {
            reason: "Over par: 5 edits (par 1)".to_string(),
        }),
        edits,
    );

    let config = AgentConfig {
        provider: ProviderKind::Fake,
        session_path: Some(session_path.clone()),
        ..Default::default()
    };
    let provider = ConfiguredProvider::new(config);

    let explanation = provider
        .explain_session(&session_path, None, &ContextConfig::default())
        .expect("Should explain session file");

    assert!(explanation
        .observations
        .iter()
        .any(|o| o.contains("Over par")));
    assert!(explanation
        .inferences
        .iter()
        .any(|i| i.contains("Edit budget exceeded")));
    assert!(!explanation.recommendation.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_configured_provider_missing_session_file_explicit_error() {
    let non_existent = PathBuf::from("does_not_exist_session_12345.json");
    let provider = ConfiguredProvider::default_configured();

    let res = provider.explain_session(&non_existent, None, &ContextConfig::default());
    assert!(res.is_err());
    match res.unwrap_err() {
        ProviderError::MissingSession(msg) => {
            assert!(msg.contains("No session found"));
        }
        other => panic!("Unexpected error type: {:?}", other),
    }
}

#[test]
fn test_cli_explain_command_with_session_file() {
    let dir = temp_session_dir("cli_explain");
    let edits = vec![RecordedEdit {
        at: Utc::now() - Duration::seconds(5),
        range: EditRange {
            start: (0, 0),
            end: (0, 2),
        },
        inserted: "pub fn".to_string(),
        deleted_len: 2,
    }];

    let session_path = create_temp_session(
        &dir,
        Some(Verdict::Passed {
            edits: 1,
            par: Some(1),
            score: 100,
        }),
        edits,
    );

    let agent_bin = env!("CARGO_BIN_EXE_zedkata-agent");

    let output = Command::new(agent_bin)
        .arg("explain")
        .arg("--session")
        .arg(&session_path)
        .arg("--provider")
        .arg("fake")
        .output()
        .expect("Failed to run zedkata-agent explain");

    assert!(output.status.success(), "CLI explain command must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("### Observations"));
    assert!(stdout.contains("### Technique Inference"));
    assert!(stdout.contains("### Recommendation"));
    assert!(stdout.contains("Passed in 1 edits"));

    let _ = fs::remove_dir_all(&dir);
}

#[derive(Debug, Clone)]
struct CapturedHttpRequest {
    request_line: String,
    headers: String,
    body: String,
}

fn spawn_single_shot_http_server(
    status_line: &'static str,
    response_body: String,
    delay: Option<std::time::Duration>,
) -> (
    String,
    std::sync::mpsc::Receiver<CapturedHttpRequest>,
    std::thread::JoinHandle<()>,
) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://127.0.0.1:{}", addr.port());
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            let mut header_end = None;
            let mut content_len = 0usize;

            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if header_end.is_none() {
                            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                                header_end = Some(pos);
                                let hdr_str = String::from_utf8_lossy(&buf[..pos]);
                                for line in hdr_str.lines() {
                                    if let Some((k, v)) = line.split_once(':') {
                                        if k.trim().eq_ignore_ascii_case("content-length") {
                                            content_len = v.trim().parse::<usize>().unwrap_or(0);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(h_end) = header_end {
                            if buf.len() >= h_end + 4 + content_len {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            let raw_req = String::from_utf8_lossy(&buf).to_string();
            let (hdrs, body) = raw_req
                .split_once("\r\n\r\n")
                .map(|(h, b)| (h.to_string(), b.to_string()))
                .unwrap_or((raw_req.clone(), String::new()));
            let req_line = hdrs.lines().next().unwrap_or("").to_string();
            let _ = tx.send(CapturedHttpRequest {
                request_line: req_line,
                headers: hdrs,
                body,
            });

            if let Some(d) = delay {
                std::thread::sleep(d);
            }

            let http_resp = format!(
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status_line,
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(http_resp.as_bytes());
            let _ = stream.flush();
        }
    });

    (base_url, rx, handle)
}

#[test]
fn test_chatgpt_auth_explicitly_rejected_as_out_of_scope() {
    let err = "chatgpt-auth".parse::<ProviderKind>().unwrap_err();
    assert!(
        err.contains("out of scope"),
        "Expected out-of-scope rejection for chatgpt-auth, got: {}",
        err
    );
    assert!(err.contains("lm-studio"));
    assert!(err.contains("offline"));
}

#[test]
fn test_loopback_http_openai_compatible_explain_and_generate() {
    use zedkata_agent::{ExerciseGenerator, WeaknessContext};

    let ctx = sample_attempt_context(false);
    let weakness_ctx = WeaknessContext {
        target_concept: "dw".to_string(),
        severity: Some(0.8),
        all_weaknesses: vec![("dw".to_string(), 0.8)],
        is_unscored_or_clean: false,
        recent_failures: vec!["Over par".to_string()],
        session_path: None,
    };

    for (kind, api_key) in [
        (ProviderKind::LmStudio, None),
        (ProviderKind::Ollama, None),
        (ProviderKind::OpenAI, Some("sk-openai-loopback".to_string())),
    ] {
        // 1. Test explain_attempt
        let md_explanation = "### 1. Observations\n- Executed 4 backspace edits against par 1.\n\n### 2. Technique Inference\n- Relied on Insert-mode backspaces instead of dw.\n\n### 3. Recommendation\nPress `dw` in Normal mode.";
        let openai_explain_resp = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": md_explanation
                }
            }]
        })
        .to_string();

        let (base_url, rx, handle) =
            spawn_single_shot_http_server("200 OK", openai_explain_resp, None);
        let config = AgentConfig {
            provider: kind,
            model: "test-openai-model".to_string(),
            base_url: Some(format!("{}/v1", base_url)),
            api_key: api_key.clone(),
            timeout_seconds: 5,
            ..Default::default()
        };
        let provider = ConfiguredProvider::new(config);
        let explanation = provider
            .explain_attempt(&ctx)
            .expect("OpenAI-compatible explain_attempt should succeed");
        let captured = rx.recv().expect("Should capture HTTP request");
        let _ = handle.join();

        assert_eq!(captured.request_line, "POST /v1/chat/completions HTTP/1.1");
        if let Some(ref k) = api_key {
            assert!(captured
                .headers
                .contains(&format!("Authorization: Bearer {}", k)));
        }
        let sent_json: serde_json::Value = serde_json::from_str(&captured.body).unwrap();
        assert_eq!(sent_json["model"], "test-openai-model");
        assert!(sent_json["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Zedkata Pedagogical Coach"));
        assert!(sent_json["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains("Delete word 'foo' using dw"));
        assert_eq!(
            explanation.observations,
            vec!["Executed 4 backspace edits against par 1."]
        );
        assert_eq!(
            explanation.inferences,
            vec!["Relied on Insert-mode backspaces instead of dw."]
        );
        assert_eq!(explanation.recommendation, "Press `dw` in Normal mode.");

        // 2. Test generate_candidate
        let candidate_json = serde_json::json!({
            "title": "Loopback Practice: dw",
            "prose": "Use dw to delete words cleanly.",
            "exercise_id": "loopback-dw",
            "task": "Delete 'unused' on line 1.",
            "start_content": "let unused = 1;\n",
            "goal_content": "let = 1;\n",
            "par_edits": 1,
            "max_edits": 2,
            "cursor": [1, 5],
            "hint": "Use dw.",
            "forbid_backspace_run": 3
        })
        .to_string();
        let openai_gen_resp = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": candidate_json
                }
            }]
        })
        .to_string();

        let (base_url_gen, rx_gen, handle_gen) =
            spawn_single_shot_http_server("200 OK", openai_gen_resp, None);
        let config_gen = AgentConfig {
            provider: kind,
            model: "test-openai-model".to_string(),
            base_url: Some(base_url_gen),
            api_key: api_key.clone(),
            timeout_seconds: 5,
            ..Default::default()
        };
        let provider_gen = ConfiguredProvider::new(config_gen);
        let candidate = provider_gen
            .generate_candidate(&weakness_ctx)
            .expect("OpenAI-compatible generate_candidate should succeed");
        let captured_gen = rx_gen.recv().expect("Should capture generate request");
        let _ = handle_gen.join();

        assert_eq!(
            captured_gen.request_line,
            "POST /v1/chat/completions HTTP/1.1"
        );
        assert_eq!(candidate.title, "Loopback Practice: dw");
        assert_eq!(candidate.exercise_id, "loopback-dw");
        assert_eq!(candidate.par_edits, Some(1));
    }
}

#[test]
fn test_loopback_http_anthropic_explain_and_generate() {
    use zedkata_agent::{ExerciseGenerator, WeaknessContext};

    let ctx = sample_attempt_context(false);
    let weakness_ctx = WeaknessContext {
        target_concept: "ci".to_string(),
        severity: Some(0.75),
        all_weaknesses: vec![("ci".to_string(), 0.75)],
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    // 1. Anthropic explain_attempt
    let explain_json_str = serde_json::json!({
        "observations": ["Anthropic observed 4 edits."],
        "inferences": ["Anthropic inferred character-by-character deletion."],
        "recommendation": "Use dw in Normal mode."
    })
    .to_string();
    let anthropic_resp = serde_json::json!({
        "content": [{
            "type": "text",
            "text": explain_json_str
        }]
    })
    .to_string();

    let (base_url, rx, handle) = spawn_single_shot_http_server("200 OK", anthropic_resp, None);
    let config = AgentConfig {
        provider: ProviderKind::Anthropic,
        model: "claude-3-5-sonnet-latest".to_string(),
        base_url: Some(format!("{}/v1", base_url)),
        api_key: Some("sk-ant-loopback-key".to_string()),
        timeout_seconds: 5,
        ..Default::default()
    };
    let provider = ConfiguredProvider::new(config);
    let explanation = provider
        .explain_attempt(&ctx)
        .expect("Anthropic explain_attempt should succeed");
    let captured = rx.recv().unwrap();
    let _ = handle.join();

    assert_eq!(captured.request_line, "POST /v1/messages HTTP/1.1");
    assert!(captured.headers.contains("x-api-key: sk-ant-loopback-key"));
    assert!(captured.headers.contains("anthropic-version: 2023-06-01"));
    let sent_json: serde_json::Value = serde_json::from_str(&captured.body).unwrap();
    assert_eq!(sent_json["model"], "claude-3-5-sonnet-latest");
    assert!(sent_json["system"]
        .as_str()
        .unwrap()
        .contains("Zedkata Pedagogical Coach"));
    assert_eq!(
        explanation.observations,
        vec!["Anthropic observed 4 edits."]
    );

    // 2. Anthropic generate_candidate
    let candidate_json = serde_json::json!({
        "title": "Anthropic Practice: ci",
        "prose": "Use ci\" to change inside quotes.",
        "exercise_id": "anthropic-ci-drill",
        "task": "Replace string contents.",
        "start_content": "let msg = \"old\";\n",
        "goal_content": "let msg = \"new\";\n",
        "par_edits": 1,
        "max_edits": 2,
        "cursor": [1, 12],
        "hint": "Use ci\".",
        "forbid_backspace_run": 3
    })
    .to_string();
    let anthropic_gen_resp = serde_json::json!({
        "content": [{
            "type": "text",
            "text": candidate_json
        }]
    })
    .to_string();

    let (base_url_gen, rx_gen, handle_gen) =
        spawn_single_shot_http_server("200 OK", anthropic_gen_resp, None);
    let config_gen = AgentConfig {
        provider: ProviderKind::Anthropic,
        model: "claude-3-5-sonnet-latest".to_string(),
        base_url: Some(base_url_gen),
        api_key: Some("sk-ant-loopback-key".to_string()),
        timeout_seconds: 5,
        ..Default::default()
    };
    let provider_gen = ConfiguredProvider::new(config_gen);
    let candidate = provider_gen
        .generate_candidate(&weakness_ctx)
        .expect("Anthropic generate_candidate should succeed");
    let captured_gen = rx_gen.recv().unwrap();
    let _ = handle_gen.join();

    assert_eq!(captured_gen.request_line, "POST /v1/messages HTTP/1.1");
    assert_eq!(candidate.exercise_id, "anthropic-ci-drill");
}

#[test]
fn test_loopback_http_gemini_explain_and_generate() {
    use zedkata_agent::{ExerciseGenerator, WeaknessContext};

    let ctx = sample_attempt_context(false);
    let weakness_ctx = WeaknessContext {
        target_concept: "find".to_string(),
        severity: Some(0.65),
        all_weaknesses: vec![("find".to_string(), 0.65)],
        is_unscored_or_clean: false,
        recent_failures: Vec::new(),
        session_path: None,
    };

    // 1. Gemini explain_attempt
    let explain_json_str = serde_json::json!({
        "observations": ["Gemini observed 4 edits."],
        "inferences": ["Gemini inferred manual backspacing."],
        "recommendation": "Use f<char> or dw."
    })
    .to_string();
    let gemini_resp = serde_json::json!({
        "candidates": [{
            "content": {
                "parts": [{ "text": explain_json_str }]
            }
        }]
    })
    .to_string();

    let (base_url, rx, handle) = spawn_single_shot_http_server("200 OK", gemini_resp, None);
    let config = AgentConfig {
        provider: ProviderKind::Gemini,
        model: "gemini-2.0-flash".to_string(),
        base_url: Some(format!("{}/v1beta", base_url)),
        api_key: Some("gemini-loopback-key".to_string()),
        timeout_seconds: 5,
        ..Default::default()
    };
    let provider = ConfiguredProvider::new(config);
    let explanation = provider
        .explain_attempt(&ctx)
        .expect("Gemini explain_attempt should succeed");
    let captured = rx.recv().unwrap();
    let _ = handle.join();

    assert_eq!(
        captured.request_line,
        "POST /v1beta/models/gemini-2.0-flash:generateContent HTTP/1.1"
    );
    assert!(captured
        .headers
        .contains("x-goog-api-key: gemini-loopback-key"));
    let sent_json: serde_json::Value = serde_json::from_str(&captured.body).unwrap();
    assert!(sent_json["systemInstruction"]["parts"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Zedkata Pedagogical Coach"));
    assert_eq!(explanation.observations, vec!["Gemini observed 4 edits."]);

    // 2. Gemini generate_candidate
    let candidate_json = serde_json::json!({
        "title": "Gemini Practice: find",
        "prose": "Use f; to jump to semicolon.",
        "exercise_id": "gemini-find-drill",
        "task": "Delete through semicolon.",
        "start_content": "let a = 1; let b = 2;\n",
        "goal_content": " let b = 2;\n",
        "par_edits": 1,
        "max_edits": 2,
        "cursor": [1, 1],
        "hint": "Use df;.",
        "forbid_backspace_run": 3
    })
    .to_string();
    let gemini_gen_resp = serde_json::json!({
        "candidates": [{
            "content": {
                "parts": [{ "text": candidate_json }]
            }
        }]
    })
    .to_string();

    let (base_url_gen, rx_gen, handle_gen) =
        spawn_single_shot_http_server("200 OK", gemini_gen_resp, None);
    let config_gen = AgentConfig {
        provider: ProviderKind::Gemini,
        model: "gemini-2.0-flash".to_string(),
        base_url: Some(base_url_gen),
        api_key: Some("gemini-loopback-key".to_string()),
        timeout_seconds: 5,
        ..Default::default()
    };
    let provider_gen = ConfiguredProvider::new(config_gen);
    let candidate = provider_gen
        .generate_candidate(&weakness_ctx)
        .expect("Gemini generate_candidate should succeed");
    let captured_gen = rx_gen.recv().unwrap();
    let _ = handle_gen.join();

    assert_eq!(
        captured_gen.request_line,
        "POST /v1beta/models/gemini-2.0-flash:generateContent HTTP/1.1"
    );
    assert_eq!(candidate.exercise_id, "gemini-find-drill");
}

#[test]
fn test_loopback_http_error_status_timeout_and_malformed_response() {
    let ctx = sample_attempt_context(false);

    // 1. HTTP 401 Unauthorized -> ProviderError::Rejection
    let err_body = serde_json::json!({
        "error": { "message": "Invalid authentication credentials" }
    })
    .to_string();
    let (base_url_401, _rx401, handle_401) =
        spawn_single_shot_http_server("401 Unauthorized", err_body, None);
    let provider_401 = ConfiguredProvider::new(AgentConfig {
        provider: ProviderKind::OpenAI,
        model: "gpt-4o-mini".to_string(),
        base_url: Some(base_url_401),
        api_key: Some("bad-key".to_string()),
        timeout_seconds: 5,
        ..Default::default()
    });
    match provider_401.explain_attempt(&ctx) {
        Err(ProviderError::Rejection {
            status_code,
            message,
        }) => {
            assert_eq!(status_code, Some(401));
            assert!(message.contains("Invalid authentication credentials"));
        }
        other => panic!("Expected Rejection(401), got {:?}", other),
    }
    let _ = handle_401.join();

    // 2. Malformed JSON response -> ProviderError::MalformedResponse
    let (base_url_bad, _rx_bad, handle_bad) =
        spawn_single_shot_http_server("200 OK", "{\"choices\": []}".to_string(), None);
    let provider_bad = ConfiguredProvider::new(AgentConfig {
        provider: ProviderKind::LmStudio,
        model: "local-model".to_string(),
        base_url: Some(base_url_bad),
        timeout_seconds: 5,
        ..Default::default()
    });
    match provider_bad.explain_attempt(&ctx) {
        Err(ProviderError::MalformedResponse(msg)) => {
            assert!(msg.contains("Missing expected message content"));
        }
        other => panic!("Expected MalformedResponse, got {:?}", other),
    }
    let _ = handle_bad.join();

    // 3. Slow loopback server -> ProviderError::Timeout
    let (base_url_slow, _rx_slow, handle_slow) = spawn_single_shot_http_server(
        "200 OK",
        "{}".to_string(),
        Some(std::time::Duration::from_millis(1500)),
    );
    let provider_slow = ConfiguredProvider::new(AgentConfig {
        provider: ProviderKind::Ollama,
        model: "mistral".to_string(),
        base_url: Some(base_url_slow),
        timeout_seconds: 1,
        ..Default::default()
    });
    match provider_slow.explain_attempt(&ctx) {
        Err(ProviderError::Timeout { seconds, .. }) => {
            assert_eq!(seconds, 1);
        }
        other => panic!("Expected Timeout, got {:?}", other),
    }
    let _ = handle_slow.join();
}
