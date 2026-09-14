use thiserror::Error;

/// Top-level error type for Language Server Protocol operations.
#[derive(Debug, Error)]
pub enum LspError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("LSP wire framing error: {0}")]
    Framing(String),

    #[error("LSP protocol error: {0}")]
    Protocol(String),

    #[error("Document error: {0}")]
    Document(#[from] DocumentError),

    #[error("Core error: {0}")]
    Core(#[from] zedkata_core::CoreError),
}

/// Errors related to document tracking and incremental change application.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DocumentError {
    #[error("Document is not tracked: {0}")]
    Untracked(String),

    #[error("Stale document version received: current version is {current}, received {received}")]
    StaleVersion { current: i64, received: i64 },

    #[error("Position out of bounds: line {line}, character {character}")]
    PositionOutOfBounds { line: u32, character: u32 },

    #[error("Invalid edit range: start ({start_line}, {start_col}) is after end ({end_line}, {end_col})")]
    InvalidRange {
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    },
}
