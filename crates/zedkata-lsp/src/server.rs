use crate::codec::{self, MessageReader};
use crate::document::DocumentTracker;
use crate::error::LspError;
use crate::feedback::FeedbackEngine;
use crate::protocol::*;
use tokio::io::{AsyncRead, AsyncWrite};

/// Lifecycle state of the Language Server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLifecycleState {
    Uninitialized,
    Initialized,
    ShuttingDown,
    Exited,
}

/// Output produced by processing an incoming message.
#[derive(Debug, Default)]
pub struct MessageOutput {
    pub response: Option<Response>,
    pub notifications: Vec<Notification>,
}

impl MessageOutput {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn response(resp: Response) -> Self {
        Self {
            response: Some(resp),
            notifications: Vec::new(),
        }
    }

    pub fn notification(notif: Notification) -> Self {
        Self {
            response: None,
            notifications: vec![notif],
        }
    }
}

/// The Zedkata LSP Server instance.
pub struct LspServer {
    state: ServerLifecycleState,
    tracker: DocumentTracker,
    received_shutdown: bool,
    feedback: Option<FeedbackEngine>,
}

impl Default for LspServer {
    fn default() -> Self {
        Self::new()
    }
}

impl LspServer {
    pub fn new() -> Self {
        Self {
            state: ServerLifecycleState::Uninitialized,
            tracker: DocumentTracker::new(),
            received_shutdown: false,
            feedback: None,
        }
    }

    pub fn with_feedback(feedback: FeedbackEngine) -> Self {
        Self {
            state: ServerLifecycleState::Uninitialized,
            tracker: DocumentTracker::new(),
            received_shutdown: false,
            feedback: Some(feedback),
        }
    }

    pub fn state(&self) -> ServerLifecycleState {
        self.state
    }

    pub fn tracker(&self) -> &DocumentTracker {
        &self.tracker
    }

    pub fn tracker_mut(&mut self) -> &mut DocumentTracker {
        &mut self.tracker
    }

    pub fn feedback(&self) -> Option<&FeedbackEngine> {
        self.feedback.as_ref()
    }

    pub fn feedback_mut(&mut self) -> Option<&mut FeedbackEngine> {
        self.feedback.as_mut()
    }

    pub fn is_exited(&self) -> bool {
        self.state == ServerLifecycleState::Exited
    }

    pub fn exit_code(&self) -> i32 {
        if self.received_shutdown {
            0
        } else {
            1
        }
    }

    /// Process a single incoming JSON-RPC message, returning responses and/or outgoing notifications.
    pub fn handle_message(&mut self, msg: IncomingMessage) -> Result<MessageOutput, LspError> {
        match msg {
            IncomingMessage::Request(req) => {
                let resp = self.handle_request(req)?;
                Ok(MessageOutput {
                    response: resp,
                    notifications: Vec::new(),
                })
            }
            IncomingMessage::Notification(notif) => {
                let notifications = self.handle_notification(notif)?;
                Ok(MessageOutput {
                    response: None,
                    notifications,
                })
            }
            IncomingMessage::Response(_) => {
                // Client responses to server requests (none sent in current version)
                Ok(MessageOutput::empty())
            }
        }
    }

    fn handle_request(&mut self, req: Request) -> Result<Option<Response>, LspError> {
        match self.state {
            ServerLifecycleState::Uninitialized => {
                if req.method == "initialize" {
                    let _init_params: InitializeParams = match req.params {
                        Some(p) => serde_json::from_value(p).unwrap_or(InitializeParams {
                            process_id: None,
                            root_uri: None,
                            capabilities: None,
                            client_info: None,
                        }),
                        None => InitializeParams {
                            process_id: None,
                            root_uri: None,
                            capabilities: None,
                            client_info: None,
                        },
                    };

                    let result = InitializeResult {
                        capabilities: ServerCapabilities {
                            position_encoding: Some("utf-16".to_string()),
                            text_document_sync: Some(TextDocumentSyncOptions {
                                open_close: Some(true),
                                change: Some(TEXT_DOCUMENT_SYNC_INCREMENTAL),
                            }),
                        },
                        server_info: Some(ServerInfo {
                            name: "zedkata-lsp".to_string(),
                            version: Some(env!("CARGO_PKG_VERSION").to_string()),
                        }),
                    };

                    self.state = ServerLifecycleState::Initialized;
                    let result_val = serde_json::to_value(result)?;
                    Ok(Some(Response::ok(req.id, result_val)))
                } else {
                    let err = ResponseError::new(SERVER_NOT_INITIALIZED, "Server not initialized");
                    Ok(Some(Response::err(req.id, err)))
                }
            }

            ServerLifecycleState::Initialized => {
                match req.method.as_str() {
                    "shutdown" => {
                        self.state = ServerLifecycleState::ShuttingDown;
                        self.received_shutdown = true;
                        Ok(Some(Response::ok_null(req.id)))
                    }

                    // Explicitly return empty results for non-observer editor queries
                    // per docs/ARCHITECTURE.md: "completions, hover, formatting, symbols. Return empty."
                    "textDocument/hover"
                    | "textDocument/completion"
                    | "textDocument/signatureHelp"
                    | "textDocument/documentSymbol"
                    | "textDocument/formatting"
                    | "textDocument/rangeFormatting"
                    | "textDocument/codeAction"
                    | "textDocument/inlayHint" => Ok(Some(Response::ok_null(req.id))),

                    unknown => {
                        let err = ResponseError::new(
                            METHOD_NOT_FOUND,
                            format!("Method '{}' is not implemented", unknown),
                        );
                        Ok(Some(Response::err(req.id, err)))
                    }
                }
            }

            ServerLifecycleState::ShuttingDown => {
                let err = ResponseError::new(INVALID_REQUEST, "Server is shutting down");
                Ok(Some(Response::err(req.id, err)))
            }

            ServerLifecycleState::Exited => {
                let err = ResponseError::new(INVALID_REQUEST, "Server has exited");
                Ok(Some(Response::err(req.id, err)))
            }
        }
    }

    fn handle_notification(&mut self, notif: Notification) -> Result<Vec<Notification>, LspError> {
        let mut outgoing = Vec::new();

        match notif.method.as_str() {
            "exit" => {
                self.state = ServerLifecycleState::Exited;
            }

            "initialized" => {
                eprintln!("[zedkata-lsp] Client initialized");
            }

            "textDocument/didOpen" => {
                if let Some(params_val) = notif.params {
                    let params: DidOpenTextDocumentParams = serde_json::from_value(params_val)?;
                    self.tracker.open_document(
                        params.text_document.uri.clone(),
                        params.text_document.version,
                        params.text_document.text.clone(),
                    );

                    if let Some(ref mut feedback) = self.feedback {
                        if let Some(out_notif) = feedback.on_document_open(
                            &params.text_document.uri,
                            &params.text_document.text,
                        )? {
                            outgoing.push(out_notif);
                        }
                    }
                }
            }

            "textDocument/didChange" => {
                if let Some(params_val) = notif.params {
                    let params: DidChangeTextDocumentParams = serde_json::from_value(params_val)?;
                    let uri = params.text_document.uri.clone();
                    let version = params.text_document.version;

                    if let Err(e) =
                        self.tracker
                            .apply_changes(&uri, version, &params.content_changes)
                    {
                        eprintln!("[zedkata-lsp] Error applying didChange: {}", e);
                    } else if let Some(ref mut feedback) = self.feedback {
                        if let Some(doc) = self.tracker.get_document(&uri) {
                            if let Some(out_notif) =
                                feedback.on_document_change(&uri, &doc.text, &doc.edits)?
                            {
                                outgoing.push(out_notif);
                            }
                        }
                    }
                }
            }

            "textDocument/didClose" => {
                if let Some(params_val) = notif.params {
                    let params: DidCloseTextDocumentParams = serde_json::from_value(params_val)?;
                    self.tracker.close_document(&params.text_document.uri);

                    if let Some(ref mut feedback) = self.feedback {
                        if let Some(out_notif) =
                            feedback.on_document_close(&params.text_document.uri)?
                        {
                            outgoing.push(out_notif);
                        }
                    }
                }
            }

            _other => {
                // Ignore other notifications (e.g. workspace/didChangeConfiguration, $/cancelRequest)
            }
        }

        Ok(outgoing)
    }
}

/// Run the LSP observer over asynchronous reader and writer streams.
pub async fn run_server<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    reader: R,
    writer: W,
) -> Result<i32, LspError> {
    let server = LspServer::new();
    run_server_with_engine(reader, writer, server).await
}

/// Run the LSP observer with a customized server instance (e.g. configured with FeedbackEngine).
pub async fn run_server_with_engine<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    reader: R,
    mut writer: W,
    mut server: LspServer,
) -> Result<i32, LspError> {
    let mut msg_reader = MessageReader::new(reader);

    while let Some(payload) = msg_reader.read_message().await? {
        let json_val: serde_json::Value = match serde_json::from_slice(&payload) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[zedkata-lsp] JSON parse error: {}", e);
                continue;
            }
        };

        let incoming = match IncomingMessage::from_value(json_val) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("[zedkata-lsp] Protocol error: {}", e);
                continue;
            }
        };

        let output = server.handle_message(incoming)?;

        if let Some(resp) = output.response {
            let resp_bytes = serde_json::to_vec(&resp)?;
            codec::write_message(&mut writer, &resp_bytes).await?;
        }

        for notif in output.notifications {
            let notif_bytes = serde_json::to_vec(&notif)?;
            codec::write_message(&mut writer, &notif_bytes).await?;
        }

        if server.is_exited() {
            break;
        }
    }

    Ok(server.exit_code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_handshake() {
        let mut server = LspServer::new();
        assert_eq!(server.state(), ServerLifecycleState::Uninitialized);

        // Pre-init request rejected
        let hover_req = Request::new(RequestId::Int(1), "textDocument/hover", None);
        let out = server
            .handle_message(IncomingMessage::Request(hover_req))
            .unwrap();
        let resp = out.response.unwrap();
        assert_eq!(resp.error.unwrap().code, SERVER_NOT_INITIALIZED);

        // Initialize request
        let init_req = Request::new(RequestId::Int(2), "initialize", None);
        let out = server
            .handle_message(IncomingMessage::Request(init_req))
            .unwrap();
        let resp = out.response.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(server.state(), ServerLifecycleState::Initialized);

        let init_res: InitializeResult = serde_json::from_value(resp.result.unwrap()).unwrap();
        assert_eq!(
            init_res.capabilities.position_encoding,
            Some("utf-16".to_string())
        );
        assert_eq!(
            init_res.capabilities.text_document_sync.unwrap().change,
            Some(TEXT_DOCUMENT_SYNC_INCREMENTAL)
        );

        // Initialized notification
        let init_notif = Notification::new("initialized", None);
        let out = server
            .handle_message(IncomingMessage::Notification(init_notif))
            .unwrap();
        assert!(out.notifications.is_empty());

        // Unsupported hover returns null
        let hover_req2 = Request::new(RequestId::Int(3), "textDocument/hover", None);
        let out = server
            .handle_message(IncomingMessage::Request(hover_req2))
            .unwrap();
        let resp2 = out.response.unwrap();
        assert_eq!(resp2.result, Some(serde_json::Value::Null));

        // Shutdown
        let shutdown_req = Request::new(RequestId::Int(4), "shutdown", None);
        let out = server
            .handle_message(IncomingMessage::Request(shutdown_req))
            .unwrap();
        let resp3 = out.response.unwrap();
        assert_eq!(resp3.result, Some(serde_json::Value::Null));
        assert_eq!(server.state(), ServerLifecycleState::ShuttingDown);

        // Exit notification
        let exit_notif = Notification::new("exit", None);
        let out = server
            .handle_message(IncomingMessage::Notification(exit_notif))
            .unwrap();
        assert!(out.notifications.is_empty());
        assert_eq!(server.state(), ServerLifecycleState::Exited);
        assert_eq!(server.exit_code(), 0);
    }
}
