use crate::error::LspError;
use serde::{Deserialize, Serialize};

// Standard JSON-RPC 2.0 error codes
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;
pub const SERVER_NOT_INITIALIZED: i64 = -32002;

/// JSON-RPC 2.0 Request ID, supporting integers and strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Int(i64),
    String(String),
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::Int(i) => write!(f, "{}", i),
            RequestId::String(s) => write!(f, "{}", s),
        }
    }
}

/// JSON-RPC 2.0 Request message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Request {
    pub fn new(
        id: RequestId,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 Response error object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ResponseError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

fn deserialize_result<'de, D>(deserializer: D) -> Result<Option<serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(serde_json::Value::deserialize(deserializer)?))
}

/// JSON-RPC 2.0 Response message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_result"
    )]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl Response {
    pub fn ok(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn ok_null(id: RequestId) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::Value::Null),
            error: None,
        }
    }

    pub fn err(id: RequestId, error: ResponseError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 Notification message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Notification {
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Parsed incoming JSON-RPC message.
#[derive(Debug, Clone, PartialEq)]
pub enum IncomingMessage {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

impl IncomingMessage {
    pub fn from_value(value: serde_json::Value) -> Result<Self, LspError> {
        let obj = value
            .as_object()
            .ok_or_else(|| LspError::Protocol("Message must be a JSON object".to_string()))?;

        let has_id = obj.contains_key("id");
        let has_method = obj.contains_key("method");
        let has_result = obj.contains_key("result");
        let has_error = obj.contains_key("error");

        if has_method && has_id {
            let req: Request = serde_json::from_value(value)?;
            Ok(IncomingMessage::Request(req))
        } else if has_method && !has_id {
            let notif: Notification = serde_json::from_value(value)?;
            Ok(IncomingMessage::Notification(notif))
        } else if has_id && (has_result || has_error) {
            let resp: Response = serde_json::from_value(value)?;
            Ok(IncomingMessage::Response(resp))
        } else {
            Err(LspError::Protocol(format!(
                "Cannot determine JSON-RPC message type for: {}",
                serde_json::to_string(&value).unwrap_or_default()
            )))
        }
    }
}

// ---------------------------------------------------------------------------
// LSP 3.17 Data Models
// ---------------------------------------------------------------------------

/// Position in a text document expressed as zero-based line and character offset.
/// Per LSP specification, the character offset is measured in UTF-16 code units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// A range in a text document expressed as (start, end) Positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// Text document sync kinds in LSP.
pub const TEXT_DOCUMENT_SYNC_NONE: u32 = 0;
pub const TEXT_DOCUMENT_SYNC_FULL: u32 = 1;
pub const TEXT_DOCUMENT_SYNC_INCREMENTAL: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentSyncOptions {
    #[serde(rename = "openClose", skip_serializing_if = "Option::is_none")]
    pub open_close: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(rename = "positionEncoding", skip_serializing_if = "Option::is_none")]
    pub position_encoding: Option<String>,
    #[serde(rename = "textDocumentSync", skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<TextDocumentSyncOptions>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo", skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "processId", default)]
    pub process_id: Option<i64>,
    #[serde(rename = "rootUri", default)]
    pub root_uri: Option<String>,
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,
    #[serde(rename = "clientInfo", default)]
    pub client_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    #[serde(rename = "rangeLength", skip_serializing_if = "Option::is_none")]
    pub range_length: Option<u32>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidOpenTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidChangeTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    #[serde(rename = "contentChanges")]
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidCloseTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    pub diagnostics: Vec<Diagnostic>,
}
