use crate::error::{DocumentError, LspError};
use crate::protocol::{Position, TextDocumentContentChangeEvent};
use chrono::Utc;
use std::collections::HashMap;
use zedkata_core::{EditRange, RecordedEdit};

/// State of a single open document tracked by the language server.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentState {
    pub uri: String,
    pub version: i64,
    pub text: String,
    pub edits: Vec<RecordedEdit>,
}

impl DocumentState {
    pub fn new(uri: String, version: i64, text: String) -> Self {
        Self {
            uri,
            version,
            text,
            edits: Vec::new(),
        }
    }
}

/// In-memory document manager tracking documents opened, changed, and closed by the client.
#[derive(Debug, Default, Clone)]
pub struct DocumentTracker {
    documents: HashMap<String, DocumentState>,
}

impl DocumentTracker {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Record a document opened with its baseline content and initial version.
    pub fn open_document(&mut self, uri: String, version: i64, text: String) {
        let state = DocumentState::new(uri.clone(), version, text);
        self.documents.insert(uri, state);
    }

    /// Get tracked document by URI.
    pub fn get_document(&self, uri: &str) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    /// Get tracked document mutable reference by URI.
    pub fn get_document_mut(&mut self, uri: &str) -> Option<&mut DocumentState> {
        self.documents.get_mut(uri)
    }

    /// Get current text for a document.
    pub fn get_text(&self, uri: &str) -> Option<&str> {
        self.documents.get(uri).map(|d| d.text.as_str())
    }

    /// Close a document and remove it from tracking.
    pub fn close_document(&mut self, uri: &str) -> Option<DocumentState> {
        self.documents.remove(uri)
    }

    /// Apply a sequence of content changes to a tracked document according to LSP specification.
    ///
    /// Checks:
    /// - Document must be tracked (`DocumentError::Untracked`).
    /// - Version must be strictly greater than current version (`DocumentError::StaleVersion`).
    /// - Ranges must be within document bounds and valid (`start <= end`).
    /// - Changes in the array are applied sequentially in order.
    pub fn apply_changes(
        &mut self,
        uri: &str,
        version: i64,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<&DocumentState, LspError> {
        let doc = self
            .documents
            .get_mut(uri)
            .ok_or_else(|| DocumentError::Untracked(uri.to_string()))?;

        if version <= doc.version {
            return Err(DocumentError::StaleVersion {
                current: doc.version,
                received: version,
            }
            .into());
        }

        for change in changes {
            apply_single_change(doc, change)?;
        }

        doc.version = version;
        Ok(doc)
    }
}

/// Apply a single content change event to a document.
fn apply_single_change(
    doc: &mut DocumentState,
    change: &TextDocumentContentChangeEvent,
) -> Result<(), LspError> {
    match change.range {
        None => {
            // Full document replacement
            let prev_text_len = doc.text.chars().count();
            let end_pos = byte_offset_to_position(&doc.text, doc.text.len())?;

            doc.edits.push(RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (0, 0),
                    end: (end_pos.line, end_pos.character),
                },
                inserted: change.text.clone(),
                deleted_len: prev_text_len,
            });

            doc.text = change.text.clone();
            Ok(())
        }
        Some(range) => {
            // Validate range order
            if range.start.line > range.end.line
                || (range.start.line == range.end.line
                    && range.start.character > range.end.character)
            {
                return Err(DocumentError::InvalidRange {
                    start_line: range.start.line,
                    start_col: range.start.character,
                    end_line: range.end.line,
                    end_col: range.end.character,
                }
                .into());
            }

            let start_byte = position_to_byte_offset(&doc.text, range.start)?;
            let end_byte = position_to_byte_offset(&doc.text, range.end)?;

            if start_byte > end_byte {
                return Err(DocumentError::InvalidRange {
                    start_line: range.start.line,
                    start_col: range.start.character,
                    end_line: range.end.line,
                    end_col: range.end.character,
                }
                .into());
            }

            let deleted_slice = &doc.text[start_byte..end_byte];
            let deleted_len = deleted_slice.chars().count();

            doc.edits.push(RecordedEdit {
                at: Utc::now(),
                range: EditRange {
                    start: (range.start.line, range.start.character),
                    end: (range.end.line, range.end.character),
                },
                inserted: change.text.clone(),
                deleted_len,
            });

            doc.text.replace_range(start_byte..end_byte, &change.text);
            Ok(())
        }
    }
}

/// Convert an LSP Position (zero-based line and UTF-16 character offset) to a byte offset
/// in a UTF-8 Rust string.
pub fn position_to_byte_offset(text: &str, pos: Position) -> Result<usize, DocumentError> {
    let bytes = text.as_bytes();
    let total_bytes = bytes.len();

    let mut current_line = 0u32;
    let mut line_start_byte = 0usize;

    // 1. Advance to start of target line
    if pos.line > 0 {
        let mut i = 0;
        while i < total_bytes {
            if bytes[i] == b'\n' {
                current_line += 1;
                line_start_byte = i + 1;
                if current_line == pos.line {
                    break;
                }
            }
            i += 1;
        }

        if current_line < pos.line {
            return Err(DocumentError::PositionOutOfBounds {
                line: pos.line,
                character: pos.character,
            });
        }
    }

    // 2. Find where the line content ends (excluding \r\n or \n)
    let mut line_end_byte = line_start_byte;
    while line_end_byte < total_bytes && bytes[line_end_byte] != b'\n' {
        line_end_byte += 1;
    }
    let mut line_content_end = line_end_byte;
    if line_content_end > line_start_byte && bytes[line_content_end - 1] == b'\r' {
        line_content_end -= 1;
    }

    let line_str = &text[line_start_byte..line_content_end];

    // 3. Count UTF-16 code units within line_str
    if pos.character == 0 {
        return Ok(line_start_byte);
    }

    let mut current_utf16 = 0u32;
    let mut char_byte_offset = 0usize;

    for c in line_str.chars() {
        if current_utf16 == pos.character {
            return Ok(line_start_byte + char_byte_offset);
        }
        let u16_len = c.len_utf16() as u32;
        current_utf16 += u16_len;
        char_byte_offset += c.len_utf8();
    }

    if current_utf16 == pos.character {
        // At the end of the line content
        return Ok(line_start_byte + char_byte_offset);
    }

    Err(DocumentError::PositionOutOfBounds {
        line: pos.line,
        character: pos.character,
    })
}

/// Convert a byte offset in a UTF-8 Rust string to an LSP Position (line, UTF-16 character).
pub fn byte_offset_to_position(text: &str, byte_offset: usize) -> Result<Position, DocumentError> {
    if byte_offset > text.len() {
        return Err(DocumentError::PositionOutOfBounds {
            line: 0,
            character: 0,
        });
    }

    let mut line = 0u32;
    let mut line_start_byte = 0usize;
    let bytes = text.as_bytes();

    for (i, &b) in bytes.iter().enumerate().take(byte_offset) {
        if b == b'\n' {
            line += 1;
            line_start_byte = i + 1;
        }
    }

    let line_content = &text[line_start_byte..byte_offset];
    let mut character = 0u32;
    for c in line_content.chars() {
        if c == '\r' || c == '\n' {
            continue;
        }
        character += c.len_utf16() as u32;
    }

    Ok(Position::new(line, character))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Range;

    #[test]
    fn test_position_to_byte_offset_ascii() {
        let text = "fn main() {\n    let a = 1;\n}\n";
        // Line 0: "fn main() {"
        assert_eq!(
            position_to_byte_offset(text, Position::new(0, 0)).unwrap(),
            0
        );
        assert_eq!(
            position_to_byte_offset(text, Position::new(0, 3)).unwrap(),
            3
        );
        assert_eq!(
            position_to_byte_offset(text, Position::new(0, 11)).unwrap(),
            11
        );

        // Line 1: "    let a = 1;" starts at byte 12
        assert_eq!(
            position_to_byte_offset(text, Position::new(1, 0)).unwrap(),
            12
        );
        assert_eq!(
            position_to_byte_offset(text, Position::new(1, 4)).unwrap(),
            16
        );
    }

    #[test]
    fn test_position_to_byte_offset_unicode_and_emojis() {
        let text = "let s = \"café 🌍\";\n";
        // 'café ' -> 'c'(1), 'a'(1), 'f'(1), 'é'(1 code unit, 2 bytes UTF-8), ' '(1)
        // '🌍' -> U+1F30F (2 UTF-16 code units, 4 bytes UTF-8)
        let pos_before_emoji = Position::new(0, 14); // before 🌍
        let offset_before = position_to_byte_offset(text, pos_before_emoji).unwrap();
        assert_eq!(&text[offset_before..offset_before + 4], "🌍");

        let pos_after_emoji = Position::new(0, 16); // after 🌍 (jumped 2 UTF-16 units)
        let offset_after = position_to_byte_offset(text, pos_after_emoji).unwrap();
        assert_eq!(&text[offset_after..offset_after + 1], "\"");
    }

    #[test]
    fn test_position_to_byte_offset_crlf() {
        let text = "first line\r\nsecond line\r\n";
        assert_eq!(
            position_to_byte_offset(text, Position::new(0, 10)).unwrap(),
            10
        );
        assert_eq!(
            position_to_byte_offset(text, Position::new(1, 0)).unwrap(),
            12
        );
        assert_eq!(
            position_to_byte_offset(text, Position::new(1, 11)).unwrap(),
            23
        );
    }

    #[test]
    fn test_incremental_edit_application() {
        let mut tracker = DocumentTracker::new();
        let uri = "file:///test.kata".to_string();
        tracker.open_document(
            uri.clone(),
            0,
            "fn main() {\n    let words = \"alpha bravo\";\n}\n".to_string(),
        );

        // Delete "alpha " at (1, 17) .. (1, 23)
        let change = TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(1, 17), Position::new(1, 23))),
            range_length: Some(6),
            text: "".to_string(),
        };

        tracker.apply_changes(&uri, 1, &[change]).unwrap();
        assert_eq!(
            tracker.get_text(&uri).unwrap(),
            "fn main() {\n    let words = \"bravo\";\n}\n"
        );
        let doc = tracker.get_document(&uri).unwrap();
        assert_eq!(doc.edits.len(), 1);
        assert_eq!(doc.edits[0].range.start, (1, 17));
        assert_eq!(doc.edits[0].range.end, (1, 23));
        assert_eq!(doc.edits[0].deleted_len, 6);
    }

    #[test]
    fn test_stale_version_rejected() {
        let mut tracker = DocumentTracker::new();
        let uri = "file:///test.kata".to_string();
        tracker.open_document(uri.clone(), 5, "hello".to_string());

        let change = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "new".to_string(),
        };

        let err = tracker.apply_changes(&uri, 5, &[change]).unwrap_err();
        match err {
            LspError::Document(DocumentError::StaleVersion { current, received }) => {
                assert_eq!(current, 5);
                assert_eq!(received, 5);
            }
            _ => panic!("Expected StaleVersion error"),
        }
    }
}
