use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use agent_client_protocol::role::acp::{Agent, Client};
use agent_client_protocol::schema::{
    v1::{
        AgentCapabilities, CancelNotification, CloseSessionRequest, CloseSessionResponse,
        ContentBlock, ContentChunk, Implementation, InitializeRequest, InitializeResponse,
        NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse, SessionId,
        SessionNotification, SessionUpdate, StopReason, TextContent,
    },
    ProtocolVersion,
};
use agent_client_protocol::{ConnectTo, ConnectionTo, Responder};

/// In-memory state tracking an active ACP conversational session.
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub prompt_count: usize,
    pub cancel_sender: Option<tokio::sync::watch::Sender<bool>>,
    pub cwd: Option<std::path::PathBuf>,
}

impl SessionState {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            created_at: chrono::Utc::now(),
            prompt_count: 0,
            cancel_sender: None,
            cwd: None,
        }
    }

    /// Signals cancellation to any in-flight prompt turn.
    pub fn cancel(&mut self) {
        if let Some(tx) = &self.cancel_sender {
            let _ = tx.send(true);
        }
    }
}

/// The Zedkata ACP Agent server.
#[derive(Clone, Default)]
pub struct AgentServer {
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
    provider: Option<crate::provider::ConfiguredProvider>,
}

impl AgentServer {
    /// Creates a new `AgentServer` with an empty session registry and default provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `AgentServer` with an explicit configured provider (e.g. for testing with `FakeProvider`).
    pub fn with_provider(provider: crate::provider::ConfiguredProvider) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            provider: Some(provider),
        }
    }

    /// Returns a list of all currently active session IDs.
    pub fn active_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }

    /// Checks if a session with the given ID is registered.
    pub fn has_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.lock().unwrap();
        sessions.contains_key(session_id)
    }

    /// Runs the ACP server over standard I/O (stdin/stdout).
    ///
    /// Standard output is strictly reserved for JSON-RPC frames.
    pub async fn run_stdio(self) -> Result<(), agent_client_protocol::Error> {
        let transport = agent_client_protocol::Stdio::new();
        self.run(transport).await
    }

    /// Runs the ACP server over an arbitrary transport.
    pub async fn run<T>(self, transport: T) -> Result<(), agent_client_protocol::Error>
    where
        T: ConnectTo<Agent> + 'static,
    {
        let sessions = Arc::clone(&self.sessions);
        let sessions_for_new = Arc::clone(&sessions);
        let sessions_for_prompt = Arc::clone(&sessions);
        let sessions_for_cancel = Arc::clone(&sessions);
        let sessions_for_close = Arc::clone(&sessions);
        let server_provider = self.provider.clone();

        let builder = Agent
            .builder()
            .name("zedkata-agent")
            .on_receive_request(
                async |request: InitializeRequest,
                       responder: Responder<InitializeResponse>,
                       _connection: ConnectionTo<Client>| {
                    tracing::info!(
                        version = ?request.protocol_version,
                        "Received ACP initialize request"
                    );

                    let version = match request.protocol_version {
                        ProtocolVersion::V1 => ProtocolVersion::V1,
                        _ => ProtocolVersion::V1,
                    };

                    let response = InitializeResponse::new(version)
                        .agent_info(
                            Implementation::new("zedkata-agent", env!("CARGO_PKG_VERSION"))
                                .title("Zedkata ACP Agent"),
                        )
                        .agent_capabilities(AgentCapabilities::default());

                    responder.respond(response)
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |request: NewSessionRequest,
                            responder: Responder<NewSessionResponse>,
                            _connection: ConnectionTo<Client>| {
                    let id_str = format!("sess-{}", uuid::Uuid::new_v4().simple());
                    let session_id = SessionId::new(id_str.clone());

                    let cwd_path = request.cwd.clone();
                    {
                        let mut store = sessions_for_new.lock().unwrap();
                        let mut state = SessionState::new(id_str.clone());
                        state.cwd = Some(cwd_path);
                        store.insert(id_str.clone(), state);
                    }

                    tracing::info!(session_id = %id_str, "Created new ACP session");
                    responder.respond(NewSessionResponse::new(session_id))
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |request: PromptRequest,
                            responder: Responder<PromptResponse>,
                            connection: ConnectionTo<Client>| {
                    let id_str = request.session_id.0.to_string();
                    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                    let (session_exists, session_cwd) = {
                        let mut store = sessions_for_prompt.lock().unwrap();
                        if let Some(state) = store.get_mut(&id_str) {
                            state.prompt_count += 1;
                            state.cancel_sender = Some(cancel_tx);
                            (true, state.cwd.clone())
                        } else {
                            (false, None)
                        }
                    };

                    if !session_exists {
                        tracing::warn!(session_id = %id_str, "Prompt received for unknown session");
                    }

                    tracing::info!(session_id = %id_str, "Processing prompt turn");

                    let sessions_store = Arc::clone(&sessions_for_prompt);
                    let provider = server_provider
                        .clone()
                        .unwrap_or_else(crate::provider::ConfiguredProvider::default_configured);

                    let prompt_text = request
                        .prompt
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text(t) => Some(t.text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ");

                    let env_session_path = std::env::var("ZEDKATA_SESSION_PATH")
                        .or_else(|_| std::env::var("ZEDKATA_SESSION"))
                        .ok()
                        .map(std::path::PathBuf::from);
                    let session_path = env_session_path
                        .or_else(|| session_cwd.as_ref().map(|c| c.join("session.json")))
                        .or_else(|| provider.config().session_path.clone())
                        .or_else(|| zedkata_core::session::default_session_path().ok());

                    let walkthrough_response = crate::walkthrough::handle_walkthrough_prompt(
                        &prompt_text,
                        session_path.as_deref(),
                    );

                    let is_generate = {
                        let lower = prompt_text.to_lowercase();
                        lower.contains("generate")
                            || lower.contains("practice")
                            || lower.contains("drill")
                            || lower.contains("exercise")
                    };

                    tokio::spawn(async move {
                        let turn_result = if let Some(resp) = walkthrough_response {
                            Ok(resp)
                        } else if is_generate {
                            if let Some(path) = session_path {
                                provider
                                    .generate_candidate_from_session_async(
                                        &path,
                                        None,
                                        Some(cancel_rx),
                                    )
                                    .await
                                    .map(|candidate| candidate.format_preview())
                            } else {
                                let fallback_ctx = crate::generator::WeaknessContext::from_session_path(
                                    std::path::Path::new("session.json"),
                                    None,
                                );
                                provider
                                    .generate_candidate_async(&fallback_ctx, Some(cancel_rx))
                                    .await
                                    .map(|candidate| candidate.format_preview())
                            }
                        } else {
                            provider
                                .explain_default_session_async(
                                    None,
                                    &crate::context::ContextConfig::default(),
                                    Some(cancel_rx),
                                )
                                .await
                                .map(|explanation| explanation.to_markdown())
                        };

                        // Clear active cancel sender for this session
                        {
                            let mut store = sessions_store.lock().unwrap();
                            if let Some(state) = store.get_mut(&id_str) {
                                state.cancel_sender = None;
                            }
                        }

                        match turn_result {
                            Ok(text) => {
                                let update = SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new(text)),
                                ));
                                let notif =
                                    SessionNotification::new(request.session_id.clone(), update);

                                if let Err(e) = connection.send_notification(notif) {
                                    tracing::error!(
                                        error = %e,
                                        "Failed to send session update notification"
                                    );
                                }

                                responder.respond(PromptResponse::new(StopReason::EndTurn))
                            }
                            Err(crate::provider::ProviderError::Cancelled(msg)) => {
                                tracing::info!(session_id = %id_str, reason = %msg, "Turn cancelled");
                                responder.respond(PromptResponse::new(StopReason::Cancelled))
                            }
                            Err(err) => {
                                tracing::warn!(
                                    session_id = %id_str,
                                    error = %err,
                                    "Turn completed with provider error"
                                );
                                let text = err.to_user_message();
                                let update = SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new(text)),
                                ));
                                let notif =
                                    SessionNotification::new(request.session_id.clone(), update);

                                if let Err(e) = connection.send_notification(notif) {
                                    tracing::error!(
                                        error = %e,
                                        "Failed to send session update notification"
                                    );
                                }

                                responder.respond(PromptResponse::new(StopReason::EndTurn))
                            }
                        }
                    });

                    Ok(())
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_notification(
                async move |notification: CancelNotification, _connection: ConnectionTo<Client>| {
                    let id_str = notification.session_id.0.to_string();
                    tracing::info!(session_id = %id_str, "Received session/cancel notification");
                    let mut store = sessions_for_cancel.lock().unwrap();
                    if let Some(state) = store.get_mut(&id_str) {
                        state.cancel();
                    }
                    Ok(())
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .on_receive_request(
                async move |request: CloseSessionRequest,
                            responder: Responder<CloseSessionResponse>,
                            _connection: ConnectionTo<Client>| {
                    let id_str = request.session_id.0.to_string();
                    {
                        let mut store = sessions_for_close.lock().unwrap();
                        if let Some(mut state) = store.remove(&id_str) {
                            state.cancel();
                        }
                    }

                    tracing::info!(session_id = %id_str, "Closed ACP session");
                    responder.respond(CloseSessionResponse::new())
                },
                agent_client_protocol::on_receive_request!(),
            );

        builder.connect_to(transport).await
    }
}
