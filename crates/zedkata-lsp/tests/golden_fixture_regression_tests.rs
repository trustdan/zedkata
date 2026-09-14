use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use zedkata_lsp::{protocol::IncomingMessage, LspServer};

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/lsp")
        .join(filename)
}

#[test]
fn test_replay_golden_capture_connection() {
    // Source: tests/fixtures/lsp/2026-09-13-connection.jsonl
    // Zed Version: 1.19.2+stable.356.df181c6f58d02677b385fa947d6bfde6d3530078
    // Captured: 2026-09-13
    let path = fixture_path("2026-09-13-connection.jsonl");
    let file =
        File::open(&path).unwrap_or_else(|e| panic!("Failed to open fixture {:?}: {}", path, e));
    let reader = BufReader::new(file);

    let mut server = LspServer::new();
    let doc_uri = "file:///C:/Users/Dan/zedkata/spikes/lsp-probe/fixtures/connection.kata";

    for (line_no, line_res) in reader.lines().enumerate() {
        let line = line_res.expect("readable line");
        if line.trim().is_empty() {
            continue;
        }

        let record: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("Line {} invalid JSON: {}", line_no + 1, e));

        let direction = record["direction"].as_str().unwrap_or_default();
        if direction != "in" {
            continue;
        }

        let msg_val = record["message"].clone();
        let incoming = IncomingMessage::from_value(msg_val)
            .unwrap_or_else(|e| panic!("Line {} protocol parse error: {}", line_no + 1, e));

        let seq = record["sequence"].as_i64().unwrap_or(0);
        server
            .handle_message(incoming)
            .unwrap_or_else(|e| panic!("Line {} (seq {}) handler error: {}", line_no + 1, seq, e));

        match seq {
            6 => {
                // Sequence 6: textDocument/didOpen
                let doc = server.tracker().get_document(doc_uri).expect("doc open");
                assert_eq!(doc.version, 0);
                assert!(doc.text.contains("let message = \"hello probe\";"));
            }
            8 => {
                // Sequence 8: deleted "probe" (version 1)
                let text = server.tracker().get_text(doc_uri).expect("doc open");
                assert!(text.contains("let message = \"hello \";"));
            }
            14 => {
                // Sequence 14: typed "zed" (version 4)
                let text = server.tracker().get_text(doc_uri).expect("doc open");
                assert!(text.contains("let message = \"hello zed\";"));
            }
            20 => {
                // Sequence 20: backspaced "zed" away (version 7)
                let text = server.tracker().get_text(doc_uri).expect("doc open");
                assert!(text.contains("let message = \"hello \";"));
            }
            26 => {
                // Sequence 26: typed "Zed" (version 10)
                let text = server.tracker().get_text(doc_uri).expect("doc open");
                assert!(text.contains("let message = \"hello Zed\";"));
            }
            28 => {
                // Sequence 28: textDocument/didClose
                assert!(server.tracker().get_document(doc_uri).is_none());
            }
            _ => {}
        }
    }
}

#[test]
fn test_replay_golden_capture_01_dw() {
    // Source: tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl
    // Zed Version: 1.19.2 (2026-09-13)
    // Scenario: dw deleting "alpha " at (1, 17)..(1, 23), followed by restore and close
    let path = fixture_path("2026-09-13-matrix-01-dw.jsonl");
    let file =
        File::open(&path).unwrap_or_else(|e| panic!("Failed to open fixture {:?}: {}", path, e));
    let reader = BufReader::new(file);

    let mut server = LspServer::new();
    let doc_uri = "file:///C:/Users/Dan/zedkata/spikes/lsp-probe/fixtures/matrix/01-dw.kata";

    for (line_no, line_res) in reader.lines().enumerate() {
        let line = line_res.expect("readable line");
        if line.trim().is_empty() {
            continue;
        }

        let record: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("Line {} invalid JSON: {}", line_no + 1, e));

        if record["direction"].as_str() != Some("in") {
            continue;
        }

        let seq = record["sequence"].as_i64().unwrap_or(0);
        let msg_val = record["message"].clone();
        let incoming = IncomingMessage::from_value(msg_val)
            .unwrap_or_else(|e| panic!("Line {} (seq {}) parse error: {}", line_no + 1, seq, e));

        server
            .handle_message(incoming)
            .unwrap_or_else(|e| panic!("Line {} (seq {}) handle error: {}", line_no + 1, seq, e));

        match seq {
            30 => {
                // didOpen
                let doc = server.tracker().get_document(doc_uri).expect("doc open");
                assert_eq!(doc.version, 0);
                assert!(doc.text.contains("let words = \"alpha bravo charlie\";"));
            }
            32 => {
                // didChange version 1: dw deleted "alpha "
                let doc = server.tracker().get_document(doc_uri).expect("doc open");
                assert_eq!(doc.version, 1);
                assert!(doc.text.contains("let words = \"bravo charlie\";"));
                assert!(!doc.text.contains("alpha"));
                assert_eq!(doc.edits.len(), 1);
                assert_eq!(doc.edits[0].range.start, (1, 17));
                assert_eq!(doc.edits[0].range.end, (1, 23));
                assert_eq!(doc.edits[0].deleted_len, 6);
            }
            34 => {
                // didChange version 2: restore "alpha "
                let text = server.tracker().get_text(doc_uri).expect("doc open");
                assert!(text.contains("let words = \"alpha bravo charlie\";"));
            }
            36 => {
                // didClose
                assert!(server.tracker().get_document(doc_uri).is_none());
            }
            _ => {}
        }
    }
}

#[test]
fn test_replay_golden_capture_02_dw_repeat() {
    // Source: tests/fixtures/lsp/2026-09-13-matrix-02-dw-repeat.jsonl
    // Zed Version: 1.19.2 (2026-09-13)
    // Scenario: dw repeat test with intermediate restores
    let path = fixture_path("2026-09-13-matrix-02-dw-repeat.jsonl");
    let file =
        File::open(&path).unwrap_or_else(|e| panic!("Failed to open fixture {:?}: {}", path, e));
    let reader = BufReader::new(file);

    let mut server = LspServer::new();
    let doc_uri = "file:///C:/Users/Dan/zedkata/spikes/lsp-probe/fixtures/matrix/02-dw-repeat.kata";

    for (line_no, line_res) in reader.lines().enumerate() {
        let line = line_res.expect("readable line");
        if line.trim().is_empty() {
            continue;
        }

        let record: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("Line {} invalid JSON: {}", line_no + 1, e));

        if record["direction"].as_str() != Some("in") {
            continue;
        }

        let seq = record["sequence"].as_i64().unwrap_or(0);
        let incoming = IncomingMessage::from_value(record["message"].clone())
            .unwrap_or_else(|e| panic!("Line {} parse error: {}", line_no + 1, e));

        server.handle_message(incoming).unwrap();

        match seq {
            38 => {
                // didOpen
                assert!(server.tracker().get_document(doc_uri).is_some());
            }
            40 => {
                // first dw: deletes "alpha "
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let words = \"bravo charlie\";"));
            }
            42 => {
                // restore "alpha "
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let words = \"alpha bravo charlie\";"));
            }
            44 => {
                // repeated dw: deletes "alpha " again
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let words = \"bravo charlie\";"));
            }
            46 => {
                // restore before close
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let words = \"alpha bravo charlie\";"));
            }
            48 => {
                // didClose
                assert!(server.tracker().get_document(doc_uri).is_none());
            }
            _ => {}
        }
    }
}

#[test]
fn test_replay_golden_capture_03_backspace() {
    // Source: tests/fixtures/lsp/2026-09-13-matrix-03-backspace.jsonl
    // Zed Version: 1.19.2 (2026-09-13)
    // Scenario: 5 discrete backspaces arriving ~180 ms apart deleting "probe"
    let path = fixture_path("2026-09-13-matrix-03-backspace.jsonl");
    let file =
        File::open(&path).unwrap_or_else(|e| panic!("Failed to open fixture {:?}: {}", path, e));
    let reader = BufReader::new(file);

    let mut server = LspServer::new();
    let doc_uri = "file:///C:/Users/Dan/zedkata/spikes/lsp-probe/fixtures/matrix/03-backspace.kata";

    for (line_no, line_res) in reader.lines().enumerate() {
        let line = line_res.expect("readable line");
        if line.trim().is_empty() {
            continue;
        }

        let record: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("Line {} invalid JSON: {}", line_no + 1, e));

        if record["direction"].as_str() != Some("in") {
            continue;
        }

        let seq = record["sequence"].as_i64().unwrap_or(0);
        let incoming = IncomingMessage::from_value(record["message"].clone())
            .unwrap_or_else(|e| panic!("Line {} parse error: {}", line_no + 1, e));

        server.handle_message(incoming).unwrap();

        match seq {
            50 => {
                // didOpen: baseline contains "hello probe"
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \"hello probe\";"));
            }
            52 => {
                // 1st backspace: deletes 'o' from "hello"
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \"hell probe\";"));
            }
            54 => {
                // 2nd backspace: deletes 'l'
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \"hel probe\";"));
            }
            56 => {
                // 3rd backspace: deletes 'l'
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \"he probe\";"));
            }
            58 => {
                // 4th backspace: deletes 'e'
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \"h probe\";"));
            }
            60 => {
                // 5th backspace: deletes 'h' -> leaves " probe"
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \" probe\";"));
                let doc = server.tracker().get_document(doc_uri).unwrap();
                assert_eq!(doc.edits.len(), 5);
                for edit in &doc.edits {
                    assert_eq!(edit.deleted_len, 1);
                    assert!(edit.inserted.is_empty());
                }
            }
            62 => {
                // restored "hello" before close
                let text = server.tracker().get_text(doc_uri).unwrap();
                assert!(text.contains("let message = \"hello probe\";"));
            }
            64 => {
                // didClose
                assert!(server.tracker().get_document(doc_uri).is_none());
            }
            _ => {}
        }
    }
}
