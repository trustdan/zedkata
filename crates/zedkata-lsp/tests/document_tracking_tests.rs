use zedkata_lsp::{
    document::{byte_offset_to_position, position_to_byte_offset, DocumentTracker},
    error::{DocumentError, LspError},
    protocol::{Position, Range, TextDocumentContentChangeEvent},
};

#[test]
fn test_document_open_close_reopen() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///workspace/test.kata";

    assert!(tracker.get_document(uri).is_none());

    // 1. Open
    tracker.open_document(uri.to_string(), 0, "fn main() {}\n".to_string());
    assert_eq!(tracker.get_text(uri), Some("fn main() {}\n"));
    assert_eq!(tracker.get_document(uri).unwrap().version, 0);

    // 2. Close
    let closed = tracker.close_document(uri).expect("doc was open");
    assert_eq!(closed.text, "fn main() {}\n");
    assert!(tracker.get_document(uri).is_none());

    // 3. Reopen with new baseline
    tracker.open_document(uri.to_string(), 1, "fn main() { 42 }\n".to_string());
    assert_eq!(tracker.get_text(uri), Some("fn main() { 42 }\n"));
    assert_eq!(tracker.get_document(uri).unwrap().version, 1);
    assert!(tracker.get_document(uri).unwrap().edits.is_empty());
}

#[test]
fn test_unknown_document_error() {
    let mut tracker = DocumentTracker::new();
    let change = TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: "replacement".to_string(),
    };

    let err = tracker
        .apply_changes("file:///untracked.kata", 1, &[change])
        .unwrap_err();

    match err {
        LspError::Document(DocumentError::Untracked(uri)) => {
            assert_eq!(uri, "file:///untracked.kata");
        }
        _ => panic!("Expected DocumentError::Untracked, got: {:?}", err),
    }
}

#[test]
fn test_stale_and_duplicate_version_rejected() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    tracker.open_document(uri.to_string(), 1, "line 1\n".to_string());

    let change = TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: "line 2\n".to_string(),
    };

    // Advancing to version 2 works
    tracker
        .apply_changes(uri, 2, std::slice::from_ref(&change))
        .unwrap();
    assert_eq!(tracker.get_document(uri).unwrap().version, 2);

    // Duplicate version 2 rejected
    let err_dup = tracker
        .apply_changes(uri, 2, std::slice::from_ref(&change))
        .unwrap_err();
    match err_dup {
        LspError::Document(DocumentError::StaleVersion { current, received }) => {
            assert_eq!(current, 2);
            assert_eq!(received, 2);
        }
        _ => panic!("Expected StaleVersion on duplicate, got {:?}", err_dup),
    }

    // Stale version 1 rejected
    let err_stale = tracker.apply_changes(uri, 1, &[change]).unwrap_err();
    match err_stale {
        LspError::Document(DocumentError::StaleVersion { current, received }) => {
            assert_eq!(current, 2);
            assert_eq!(received, 1);
        }
        _ => panic!(
            "Expected StaleVersion on stale version, got {:?}",
            err_stale
        ),
    }
}

#[test]
fn test_full_document_replacement() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    tracker.open_document(uri.to_string(), 0, "first line\nsecond line\n".to_string());

    let change = TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: "completely new content\n".to_string(),
    };

    tracker.apply_changes(uri, 1, &[change]).unwrap();
    assert_eq!(tracker.get_text(uri).unwrap(), "completely new content\n");

    let doc = tracker.get_document(uri).unwrap();
    assert_eq!(doc.edits.len(), 1);
    assert_eq!(doc.edits[0].range.start, (0, 0));
    assert_eq!(doc.edits[0].inserted, "completely new content\n");
    assert_eq!(
        doc.edits[0].deleted_len,
        "first line\nsecond line\n".chars().count()
    );
}

#[test]
fn test_single_incremental_operator_deletion() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    let initial = "fn main() {\n    let words = \"alpha bravo charlie\";\n}\n";
    tracker.open_document(uri.to_string(), 0, initial.to_string());

    // dw deleting "alpha " at (1, 17)..(1, 23)
    let change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(1, 17), Position::new(1, 23))),
        range_length: Some(6),
        text: "".to_string(),
    };

    tracker.apply_changes(uri, 1, &[change]).unwrap();
    assert_eq!(
        tracker.get_text(uri).unwrap(),
        "fn main() {\n    let words = \"bravo charlie\";\n}\n"
    );

    let doc = tracker.get_document(uri).unwrap();
    assert_eq!(doc.edits.len(), 1);
    assert_eq!(doc.edits[0].range.start, (1, 17));
    assert_eq!(doc.edits[0].range.end, (1, 23));
    assert_eq!(doc.edits[0].deleted_len, 6);
    assert_eq!(doc.edits[0].inserted, "");
}

#[test]
fn test_discrete_backspaces_change_sequence() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    // Target: delete "probe" from line 2: `    let message = "hello probe";\n`
    let initial = "fn main() {\n    let message = \"hello probe\";\n}\n";
    tracker.open_document(uri.to_string(), 0, initial.to_string());

    // 5 backspaces at columns 30 down to 25
    let backspaces = [(30, "e"), (29, "b"), (28, "o"), (27, "r"), (26, "p")];

    for (version, (end_col, _char_deleted)) in backspaces.iter().enumerate() {
        let start_col = end_col - 1;
        let change = TextDocumentContentChangeEvent {
            range: Some(Range::new(
                Position::new(1, start_col),
                Position::new(1, *end_col),
            )),
            range_length: Some(1),
            text: "".to_string(),
        };

        tracker
            .apply_changes(uri, (version + 1) as i64, &[change])
            .unwrap();
    }

    assert_eq!(
        tracker.get_text(uri).unwrap(),
        "fn main() {\n    let message = \"hello \";\n}\n"
    );

    let doc = tracker.get_document(uri).unwrap();
    assert_eq!(doc.edits.len(), 5);
    for edit in &doc.edits {
        assert_eq!(edit.range.start.0, 1);
        assert_eq!(edit.deleted_len, 1);
        assert_eq!(edit.inserted, "");
    }
}

#[test]
fn test_multiple_changes_in_single_did_change() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    tracker.open_document(uri.to_string(), 0, "one three\n".to_string());

    // LSP allows multiple changes in contentChanges applied in order:
    // Change 1: Insert "two " at (0, 4) -> "one two three\n"
    // Change 2: Insert "four " at (0, 14) -> "one two three four \n"
    let changes = vec![
        TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 4), Position::new(0, 4))),
            range_length: Some(0),
            text: "two ".to_string(),
        },
        TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 13), Position::new(0, 13))),
            range_length: Some(0),
            text: " four\n".to_string(),
        },
    ];

    tracker.apply_changes(uri, 1, &changes).unwrap();
    assert_eq!(tracker.get_text(uri).unwrap(), "one two three four\n\n");

    let doc = tracker.get_document(uri).unwrap();
    assert_eq!(doc.edits.len(), 2);
}

#[test]
fn test_unicode_multibyte_and_emojis() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    // "let unicode = \"café 🌍\";\n"
    // 'café ' -> 5 chars, but 'é' is 2 bytes UTF-8 (1 UTF-16 code unit).
    // '🌍' -> U+1F30F is 4 bytes UTF-8, 2 UTF-16 code units.
    // Line layout:
    // 0..15: `let unicode = "` (15 chars)
    // 15: 'c' (1)
    // 16: 'a' (1)
    // 17: 'f' (1)
    // 18: 'é' (1 UTF-16)
    // 19: ' ' (1)
    // 20..22: '🌍' (2 UTF-16 code units!)
    // 22: '"' (1)
    // 23: ';' (1)
    let initial = "let unicode = \"café 🌍\";\n";
    tracker.open_document(uri.to_string(), 0, initial.to_string());

    // 1. Delete emoji: range is (0, 20) .. (0, 22) (span of 2 in UTF-16)
    let change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(0, 20), Position::new(0, 22))),
        range_length: Some(2),
        text: "".to_string(),
    };

    tracker.apply_changes(uri, 1, &[change]).unwrap();
    assert_eq!(tracker.get_text(uri).unwrap(), "let unicode = \"café \";\n");

    // 2. Insert another emoji and text
    let insert_change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(0, 20), Position::new(0, 20))),
        range_length: Some(0),
        text: "🚀 rocket".to_string(),
    };

    tracker.apply_changes(uri, 2, &[insert_change]).unwrap();
    assert_eq!(
        tracker.get_text(uri).unwrap(),
        "let unicode = \"café 🚀 rocket\";\n"
    );
}

#[test]
fn test_crlf_line_endings_handling() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    let initial = "line one\r\nline two\r\nline three\r\n";
    tracker.open_document(uri.to_string(), 0, initial.to_string());

    // Edit on line 1: replace "two" with "SECOND"
    // "line " is 5 chars. "two" is (1, 5) .. (1, 8)
    let change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(1, 5), Position::new(1, 8))),
        range_length: Some(3),
        text: "SECOND".to_string(),
    };

    tracker.apply_changes(uri, 1, &[change]).unwrap();
    assert_eq!(
        tracker.get_text(uri).unwrap(),
        "line one\r\nline SECOND\r\nline three\r\n"
    );
}

#[test]
fn test_invalid_and_malformed_ranges() {
    let mut tracker = DocumentTracker::new();
    let uri = "file:///test.kata";
    tracker.open_document(uri.to_string(), 0, "alpha bravo\ncharlie\n".to_string());

    // 1. start > end (same line)
    let inv_range = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(0, 8), Position::new(0, 4))),
        range_length: None,
        text: "".to_string(),
    };
    let err = tracker.apply_changes(uri, 1, &[inv_range]).unwrap_err();
    assert!(matches!(
        err,
        LspError::Document(DocumentError::InvalidRange { .. })
    ));

    // 2. line out of bounds (line 5 on 2-line doc)
    let oob_line = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(5, 0), Position::new(5, 2))),
        range_length: None,
        text: "".to_string(),
    };
    let err2 = tracker.apply_changes(uri, 2, &[oob_line]).unwrap_err();
    assert!(matches!(
        err2,
        LspError::Document(DocumentError::PositionOutOfBounds { line: 5, .. })
    ));

    // 3. character out of bounds on line 0 (char 99 on 11-char line)
    let oob_char = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(0, 99), Position::new(0, 100))),
        range_length: None,
        text: "".to_string(),
    };
    let err3 = tracker.apply_changes(uri, 3, &[oob_char]).unwrap_err();
    assert!(matches!(
        err3,
        LspError::Document(DocumentError::PositionOutOfBounds { character: 99, .. })
    ));
}

#[test]
fn test_byte_offset_to_position_roundtrip() {
    let text = "alpha\nbravo café 🌍\ncharlie\n";
    // Check positions at various offsets
    let pos0 = byte_offset_to_position(text, 0).unwrap();
    assert_eq!(pos0, Position::new(0, 0));
    assert_eq!(position_to_byte_offset(text, pos0).unwrap(), 0);

    // Start of line 1
    let pos_l1 = byte_offset_to_position(text, 6).unwrap();
    assert_eq!(pos_l1, Position::new(1, 0));
    assert_eq!(position_to_byte_offset(text, pos_l1).unwrap(), 6);
}
