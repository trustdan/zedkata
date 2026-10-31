use crate::config::{AgentConfig, ProviderKind};
use crate::context::{
    extract_last_attempt_context, AttemptContext, AttemptVerdictContext, ContextConfig,
    ContextError,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use zedkata_core::catalog::Catalog;

/// A structured explanation produced for a drill attempt.
///
/// Adheres to REQ-27 by strictly separating objective observations from technique inferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    /// Objective observations derived strictly from recorded edits and session data.
    pub observations: Vec<String>,
    /// Inferred technique, habits, or motion intent behind the student's actions.
    pub inferences: Vec<String>,
    /// Actionable recommendation for the target motion or approach.
    pub recommendation: String,
    /// Indicates whether context bounds (20 edits / 2 KB) were enforced.
    pub is_bounded: bool,
    /// Truncation note if earlier edits or diffs were omitted.
    pub truncation_note: Option<String>,
}

impl Explanation {
    /// Formats the explanation into student-facing markdown adhering to REQ-27.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("### Observations\n");
        for obs in &self.observations {
            out.push_str(&format!("- {}\n", obs));
        }

        out.push_str("\n### Technique Inference\n");
        for inf in &self.inferences {
            out.push_str(&format!("- {}\n", inf));
        }

        out.push_str("\n### Recommendation\n");
        out.push_str(&self.recommendation);
        out.push('\n');

        if let Some(note) = &self.truncation_note {
            out.push_str(&format!("\n> [!NOTE]\n> {}\n", note));
        }

        out
    }
}

/// Errors returned by model providers during attempt explanation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ProviderError {
    #[error("No active session: {0}")]
    MissingSession(String),

    #[error("Corrupt session: {0}")]
    CorruptSession(String),

    #[error("Stale attempt: {0}")]
    StaleAttempt(String),

    #[error("Underlying context extraction error: {0}")]
    Context(String),

    #[error("Request timed out after {seconds}s: {message}")]
    Timeout { seconds: u64, message: String },

    #[error("Provider rejected request: {message}")]
    Rejection {
        status_code: Option<u16>,
        message: String,
    },

    #[error("Malformed response from model provider: {0}")]
    MalformedResponse(String),

    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    #[error("Provider failure: {0}")]
    Failure(String),
}

impl From<ContextError> for ProviderError {
    fn from(err: ContextError) -> Self {
        ProviderError::Context(err.to_string())
    }
}

impl ProviderError {
    /// Formats the error into student-friendly, actionable markdown with troubleshooting tips,
    /// reassuring the student that their session and progress remain intact.
    pub fn to_user_message(&self) -> String {
        match self {
            ProviderError::MissingSession(_) => {
                "### Zedkata Pedagogical Coach\n\nNo active drill attempt found in `session.json`.\n\nTo receive grounded feedback:\n1. Open an exercise buffer (`*.kata`) or launch a drill from `zedkata-tui`.\n2. Attempt the target motion in Zed.\n3. Request an explanation here in the Agent Panel!".to_string()
            }
            ProviderError::CorruptSession(msg) => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Corrupt Session State**: {}\n\n*Your progress history is intact. Start a fresh drill in Zed to regenerate session state.*",
                    msg
                )
            }
            ProviderError::StaleAttempt(msg) => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Stale Attempt**: {}\n\n*Start a fresh drill attempt in Zed to receive current feedback.*",
                    msg
                )
            }
            ProviderError::Context(msg) => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Context Extraction Error**: {}\n\n*Your session state and progress remain intact.*",
                    msg
                )
            }
            ProviderError::Timeout { seconds, message } => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Request Timed Out**: The model provider did not respond within {} seconds.\n\nDetails: {}\n\n**Troubleshooting**:\n- If running local models (LM Studio / Ollama), ensure the server is running and the model is loaded.\n- If using a cloud provider, check your network connection or try a smaller model.\n- You can configure offline degraded mode (`ZEDKATA_PROVIDER=degraded`) for instant offline feedback.\n\n*Your session state and practice progress are intact.*",
                    seconds, message
                )
            }
            ProviderError::Rejection { status_code, message } => {
                let code_str = status_code.map(|c| format!(" (HTTP {})", c)).unwrap_or_default();
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Provider Rejected Request{}**: {}\n\n**Troubleshooting**:\n- Check that your API key is valid and has sufficient quota.\n- If rate-limited, wait a moment before retrying.\n- You can configure offline degraded mode (`ZEDKATA_PROVIDER=degraded`) to practice without API credentials.\n\n*Your session state and practice progress are intact.*",
                    code_str, message
                )
            }
            ProviderError::MalformedResponse(msg) => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Malformed Model Response**: {}\n\n**Troubleshooting**:\n- The model provider returned an unparseable or truncated response.\n- You can retry your request or switch to a different model in `config.toml`.\n\n*Your session state and practice progress are intact.*",
                    msg
                )
            }
            ProviderError::Cancelled(msg) => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Request Cancelled**: {}\n\n*Your session state and practice progress remain intact.*",
                    msg
                )
            }
            ProviderError::Failure(msg) => {
                format!(
                    "### Zedkata Pedagogical Coach\n\n**Provider Failure**: {}\n\n*Your session state and practice progress are intact.*",
                    msg
                )
            }
        }
    }
}

/// Trait implemented by attempt explanation providers (LLMs or test doubles).
pub trait AttemptExplainer: Send + Sync + std::fmt::Debug {
    /// Explain the given attempt using bounded context.
    fn explain_attempt(&self, context: &AttemptContext) -> Result<Explanation, ProviderError>;

    /// Optional simulated delay for testing async cancellation and timeout.
    fn simulated_delay(&self) -> Option<std::time::Duration> {
        None
    }

    /// Extract bounded context from `session.json` and generate an explanation,
    /// handling missing, old, corrupt, or oversized states explicitly.
    fn explain_session(
        &self,
        session_path: &Path,
        catalog: Option<&Catalog>,
        config: &ContextConfig,
    ) -> Result<Explanation, ProviderError> {
        let context = match extract_last_attempt_context(session_path, catalog, config) {
            Ok(ctx) => ctx,
            Err(ContextError::MissingSessionFile { path }) => {
                return Err(ProviderError::MissingSession(format!(
                    "No session found at {:?}. Start an exercise in Zed first.",
                    path
                )));
            }
            Err(ContextError::NoActiveAttempt) => {
                return Err(ProviderError::MissingSession(
                    "No active drill attempt found in session state.".to_string(),
                ));
            }
            Err(ContextError::CorruptSessionFile { path, message }) => {
                return Err(ProviderError::CorruptSession(format!(
                    "Session file at {:?} is unparseable or corrupt: {}",
                    path, message
                )));
            }
            Err(ContextError::StaleAttempt {
                age_seconds,
                max_age_seconds,
            }) => {
                return Err(ProviderError::StaleAttempt(format!(
                    "Attempt is stale ({}s old, limit is {}s). Start a fresh drill in Zed.",
                    age_seconds, max_age_seconds
                )));
            }
            Err(ContextError::Io { path, source }) => {
                return Err(ProviderError::Failure(format!(
                    "IO failure reading {:?}: {}",
                    path, source
                )));
            }
        };

        self.explain_attempt(&context)
    }
}

/// Trait implemented by candidate practice exercise generators (LLMs or test doubles).
pub trait ExerciseGenerator: Send + Sync + std::fmt::Debug {
    /// Generate a complete candidate practice exercise for the given weakness context.
    fn generate_candidate(
        &self,
        context: &crate::generator::WeaknessContext,
    ) -> Result<crate::generator::CandidateExercise, ProviderError>;

    /// Optional simulated delay for testing async cancellation and timeout.
    fn simulated_delay(&self) -> Option<std::time::Duration> {
        None
    }
}

/// A fake model provider for testing attempt explanation, context extraction, and practice generation.
///
/// Records all received `AttemptContext` and `WeaknessContext` payloads and synthesizes grounded outputs
/// separating observation from inference without requiring external network access.
#[derive(Debug, Default, Clone)]
pub struct FakeProvider {
    received_contexts: Arc<Mutex<Vec<AttemptContext>>>,
    canned_explanation: Arc<Mutex<Option<Explanation>>>,
    received_weakness_contexts: Arc<Mutex<Vec<crate::generator::WeaknessContext>>>,
    canned_candidate: Arc<Mutex<Option<crate::generator::CandidateExercise>>>,
    injected_error: Arc<Mutex<Option<ProviderError>>>,
    async_delay: Arc<Mutex<Option<std::time::Duration>>>,
}

impl FakeProvider {
    /// Creates a new empty `FakeProvider`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `FakeProvider` that always returns the specified canned explanation.
    pub fn with_canned(explanation: Explanation) -> Self {
        Self {
            received_contexts: Arc::new(Mutex::new(Vec::new())),
            canned_explanation: Arc::new(Mutex::new(Some(explanation))),
            received_weakness_contexts: Arc::new(Mutex::new(Vec::new())),
            canned_candidate: Arc::new(Mutex::new(None)),
            injected_error: Arc::new(Mutex::new(None)),
            async_delay: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a `FakeProvider` that always returns the specified canned candidate exercise.
    pub fn with_canned_candidate(candidate: crate::generator::CandidateExercise) -> Self {
        Self {
            received_contexts: Arc::new(Mutex::new(Vec::new())),
            canned_explanation: Arc::new(Mutex::new(None)),
            received_weakness_contexts: Arc::new(Mutex::new(Vec::new())),
            canned_candidate: Arc::new(Mutex::new(Some(candidate))),
            injected_error: Arc::new(Mutex::new(None)),
            async_delay: Arc::new(Mutex::new(None)),
        }
    }

    /// Injects an error to simulate provider rejection or network failure.
    pub fn inject_error(&self, message: impl Into<String>) {
        let mut slot = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = Some(ProviderError::Failure(message.into()));
    }

    /// Injects a timeout error.
    pub fn inject_timeout(&self, seconds: u64, message: impl Into<String>) {
        let mut slot = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = Some(ProviderError::Timeout {
            seconds,
            message: message.into(),
        });
    }

    /// Injects a provider rejection.
    pub fn inject_rejection(&self, status_code: Option<u16>, message: impl Into<String>) {
        let mut slot = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = Some(ProviderError::Rejection {
            status_code,
            message: message.into(),
        });
    }

    /// Injects a malformed model response error.
    pub fn inject_malformed_response(&self, message: impl Into<String>) {
        let mut slot = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = Some(ProviderError::MalformedResponse(message.into()));
    }

    /// Injects a cancellation error.
    pub fn inject_cancellation(&self, message: impl Into<String>) {
        let mut slot = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = Some(ProviderError::Cancelled(message.into()));
    }

    /// Clears any injected error.
    pub fn clear_injected_error(&self) {
        let mut slot = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = None;
    }

    /// Sets an artificial delay to simulate slow model inference (enabling timeout or cancellation testing).
    pub fn set_async_delay(&self, duration: std::time::Duration) {
        let mut slot = self.async_delay.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(duration);
    }

    /// Clears any artificial delay.
    pub fn clear_async_delay(&self) {
        let mut slot = self.async_delay.lock().unwrap_or_else(|e| e.into_inner());
        *slot = None;
    }

    /// Sets or clears the canned explanation.
    pub fn set_canned(&self, explanation: Option<Explanation>) {
        let mut slot = self
            .canned_explanation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = explanation;
    }

    /// Sets or clears the canned candidate exercise.
    pub fn set_canned_candidate(&self, candidate: Option<crate::generator::CandidateExercise>) {
        let mut slot = self
            .canned_candidate
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *slot = candidate;
    }

    /// Returns a copy of all contexts received by this provider.
    pub fn recorded_contexts(&self) -> Vec<AttemptContext> {
        let store = self
            .received_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.clone()
    }

    /// Returns the most recently received attempt context, if any.
    pub fn last_received_context(&self) -> Option<AttemptContext> {
        let store = self
            .received_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.last().cloned()
    }

    /// Returns a copy of all weakness contexts received by this provider.
    pub fn recorded_weakness_contexts(&self) -> Vec<crate::generator::WeaknessContext> {
        let store = self
            .received_weakness_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.clone()
    }

    /// Returns the most recently received weakness context, if any.
    pub fn last_received_weakness_context(&self) -> Option<crate::generator::WeaknessContext> {
        let store = self
            .received_weakness_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.last().cloned()
    }

    /// Returns the total count of attempts received.
    pub fn received_count(&self) -> usize {
        let store = self
            .received_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.len()
    }

    /// Clears the history of received contexts.
    pub fn clear_recorded(&self) {
        let mut store = self
            .received_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        store.clear();
        let mut w_store = self
            .received_weakness_contexts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        w_store.clear();
    }
}

impl ExerciseGenerator for FakeProvider {
    fn simulated_delay(&self) -> Option<std::time::Duration> {
        *self.async_delay.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn generate_candidate(
        &self,
        context: &crate::generator::WeaknessContext,
    ) -> Result<crate::generator::CandidateExercise, ProviderError> {
        {
            let mut store = self
                .received_weakness_contexts
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            store.push(context.clone());
        }

        if let Some(err) = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            return Err(err.clone());
        }

        if let Some(canned) = self
            .canned_candidate
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            return Ok(canned.clone());
        }

        Ok(crate::generator::synthesize_default_candidate(
            context, None,
        ))
    }
}

impl AttemptExplainer for FakeProvider {
    fn simulated_delay(&self) -> Option<std::time::Duration> {
        *self.async_delay.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn explain_attempt(&self, context: &AttemptContext) -> Result<Explanation, ProviderError> {
        // Record the received context for test verification
        {
            let mut store = self
                .received_contexts
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            store.push(context.clone());
        }

        // Check if an error was injected
        if let Some(err) = self
            .injected_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            return Err(err.clone());
        }

        // Return canned explanation if configured
        if let Some(canned) = self
            .canned_explanation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            return Ok(canned.clone());
        }

        // Default: synthesize a grounded explanation separating observations from inference
        let mut observations = Vec::new();
        let mut inferences = Vec::new();

        if context.lesson_title.is_some()
            || context.task_prompt.is_some()
            || context.par_edits.is_some()
            || context.forbid_rules.is_some()
        {
            let mut meta_parts = Vec::new();
            if let Some(title) = &context.lesson_title {
                meta_parts.push(format!("Lesson: {} (`{}`)", title, context.lesson_id));
            } else {
                meta_parts.push(format!("Lesson ID: `{}`", context.lesson_id));
            }
            if !context.concepts.is_empty() {
                meta_parts.push(format!("Concepts: {}", context.concepts.join(", ")));
            }
            if let Some(task) = &context.task_prompt {
                meta_parts.push(format!("Task: '{}'", task));
            }
            if let Some(par) = context.par_edits {
                meta_parts.push(format!("Par budget: {} edit(s)", par));
            }
            if let Some(rules) = &context.forbid_rules {
                if let Some(limit) = rules.backspace_run {
                    meta_parts.push(format!("Forbid rule: backspace_run <= {}", limit));
                }
            }
            if !meta_parts.is_empty() {
                observations.push(meta_parts.join(" | "));
            }
        }

        observations.push(format!(
            "Attempt executed {} edits in total ({} retained in bounded context, {} bytes diff).",
            context.total_edits,
            context.bounded_edits.len(),
            context.total_diff_bytes
        ));

        match &context.verdict {
            AttemptVerdictContext::Passed { edits, par, score } => {
                let par_label = par
                    .map(|p| format!(" against par of {}", p))
                    .unwrap_or_default();
                observations.push(format!(
                    "Verdict: Passed in {} edits{} with score {}.",
                    edits, par_label, score
                ));
                inferences.push(
                    "Technique achieved goal buffer state within permitted budget.".to_string(),
                );
            }
            AttemptVerdictContext::Failed { reason } => {
                observations.push(format!("Verdict: Failed with reason '{}'.", reason));
                if reason.to_lowercase().contains("backspace") {
                    inferences.push(
                        "Inferred technique: Excessive backspace usage detected. Student likely relied on character-by-character backspacing instead of word/motion operators (e.g. 'dw' or 'd$').".to_string()
                    );
                } else if reason.to_lowercase().contains("par")
                    || reason.to_lowercase().contains("edit")
                {
                    inferences.push(
                        "Inferred technique: Edit budget exceeded. Student likely navigated using individual character motions (h/j/k/l) rather than word jumps (w/e/b).".to_string()
                    );
                } else {
                    inferences.push(format!(
                        "Inferred technique: Attempt diverged from optimal motion due to '{}'.",
                        reason
                    ));
                }
            }
            AttemptVerdictContext::InProgress => {
                observations.push("Verdict: Attempt is currently in progress.".to_string());
                inferences.push("Inferred technique: Student is seeking mid-drill guidance before completing the task.".to_string());
            }
            AttemptVerdictContext::Unspecified => {
                observations
                    .push("Verdict: Attempt has no finalized verdict recorded.".to_string());
            }
        }

        for (i, edit) in context.bounded_edits.iter().take(3).enumerate() {
            observations.push(format!(
                "Edit {}: deleted {} chars, inserted {:?} at line {}, col {}.",
                i + 1,
                edit.deleted_len,
                edit.inserted,
                edit.range.start.0,
                edit.range.start.1
            ));
        }

        let recommendation = if let Some(task) = &context.task_prompt {
            format!("To accomplish '{}', consider using motion operators like 'dw', 'cw', or 'f<char>' to reach Par 1.", task)
        } else {
            "Review target motion in lesson prose and minimize keystrokes to reach Par 1."
                .to_string()
        };

        let truncation_note = if context.is_truncated {
            Some(format!(
                "Bounded context: {} older edits were omitted to keep context under limit ({} edits retained, {} bytes diff).",
                context.truncated_edits_count,
                context.bounded_edits.len(),
                context.total_diff_bytes
            ))
        } else {
            None
        };

        Ok(Explanation {
            observations,
            inferences,
            recommendation,
            is_bounded: true,
            truncation_note,
        })
    }
}

/// A configured model provider for explaining drill attempts under ADR 0005.
///
/// Resolves configuration according to the established precedence:
/// 1. Process environment (`ZEDKATA_PROVIDER`, `ZEDKATA_MODEL`, `ANTHROPIC_API_KEY`, etc.)
/// 2. User configuration file (`config.toml`)
/// 3. Automatic fallback (single key detection or offline default)
///
/// Guarantees that explanation calls work without network access from `core`, `LSP`, or `TUI`.
#[derive(Debug, Clone)]
pub struct ConfiguredProvider {
    config: AgentConfig,
    explainer: Option<Arc<dyn AttemptExplainer>>,
    generator: Option<Arc<dyn ExerciseGenerator>>,
}

impl ConfiguredProvider {
    /// Creates a new `ConfiguredProvider` with explicit configuration.
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            explainer: None,
            generator: None,
        }
    }

    /// Resolves configuration automatically using the ADR 0005 hierarchy.
    pub fn default_configured() -> Self {
        Self::new(AgentConfig::resolve())
    }

    /// Creates a `ConfiguredProvider` with an explicit inner explainer (e.g. for testing with `FakeProvider`).
    pub fn with_explainer(config: AgentConfig, explainer: Arc<dyn AttemptExplainer>) -> Self {
        Self {
            config,
            explainer: Some(explainer),
            generator: None,
        }
    }

    /// Creates a `ConfiguredProvider` with an explicit inner generator.
    pub fn with_generator(config: AgentConfig, generator: Arc<dyn ExerciseGenerator>) -> Self {
        Self {
            config,
            explainer: None,
            generator: Some(generator),
        }
    }

    /// Creates a `ConfiguredProvider` with explicit inner explainer and generator.
    pub fn with_explainer_and_generator(
        config: AgentConfig,
        explainer: Arc<dyn AttemptExplainer>,
        generator: Arc<dyn ExerciseGenerator>,
    ) -> Self {
        Self {
            config,
            explainer: Some(explainer),
            generator: Some(generator),
        }
    }

    /// Returns the resolved agent configuration.
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    fn provider_simulated_delay(&self) -> Option<std::time::Duration> {
        if let Some(inner) = &self.explainer {
            inner.simulated_delay()
        } else if let Some(gen) = &self.generator {
            gen.simulated_delay()
        } else {
            None
        }
    }

    /// Explains the attempt from the default session location (`default_session_path`).
    pub fn explain_default_session(
        &self,
        catalog: Option<&Catalog>,
        config: &ContextConfig,
    ) -> Result<Explanation, ProviderError> {
        let session_path = self
            .config
            .session_path
            .clone()
            .or_else(|| zedkata_core::session::default_session_path().ok())
            .ok_or_else(|| {
                ProviderError::MissingSession(
                    "Could not resolve platform-standard session path.".to_string(),
                )
            })?;

        self.explain_session(&session_path, catalog, config)
    }

    /// Explains the attempt asynchronously with timeout and cancellation checks.
    ///
    /// Ensures that cancellation or timeout cleanly terminates in-flight operations
    /// without leaving lingering tasks or mutating session state.
    pub async fn explain_session_async(
        &self,
        session_path: &Path,
        catalog: Option<&Catalog>,
        config: &ContextConfig,
        mut cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
    ) -> Result<Explanation, ProviderError> {
        // Immediate cancellation check
        if let Some(rx) = &cancel_rx {
            if *rx.borrow() {
                return Err(ProviderError::Cancelled(
                    "Operation cancelled before start.".to_string(),
                ));
            }
        }

        let timeout_duration = std::time::Duration::from_secs(self.config.timeout_seconds);
        let sim_delay = self.provider_simulated_delay();

        let this = self.clone();
        let path_buf = session_path.to_path_buf();
        let cat_clone = catalog.cloned();
        let cfg_clone = config.clone();

        let work = async move {
            if let Some(delay) = sim_delay {
                tokio::time::sleep(delay).await;
            }
            tokio::task::spawn_blocking(move || {
                this.explain_session(&path_buf, cat_clone.as_ref(), &cfg_clone)
            })
            .await
            .map_err(|e| ProviderError::Failure(format!("Task execution failed: {}", e)))?
        };

        tokio::select! {
            biased;
            cancelled = async {
                if let Some(rx) = &mut cancel_rx {
                    while rx.changed().await.is_ok() {
                        if *rx.borrow() {
                            return true;
                        }
                    }
                }
                futures::future::pending::<bool>().await
            } => {
                if cancelled {
                    Err(ProviderError::Cancelled(
                        "Operation cancelled by client.".to_string(),
                    ))
                } else {
                    Err(ProviderError::Cancelled("Operation cancelled.".to_string()))
                }
            }
            _ = tokio::time::sleep(timeout_duration) => {
                Err(ProviderError::Timeout {
                    seconds: self.config.timeout_seconds,
                    message: format!(
                        "Request to provider '{}' timed out after {}s.",
                        self.config.provider, self.config.timeout_seconds
                    ),
                })
            }
            res = work => {
                res
            }
        }
    }

    /// Explains the default session asynchronously with timeout and cancellation.
    pub async fn explain_default_session_async(
        &self,
        catalog: Option<&Catalog>,
        config: &ContextConfig,
        cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
    ) -> Result<Explanation, ProviderError> {
        let session_path = self
            .config
            .session_path
            .clone()
            .or_else(|| zedkata_core::session::default_session_path().ok())
            .ok_or_else(|| {
                ProviderError::MissingSession(
                    "Could not resolve platform-standard session path.".to_string(),
                )
            })?;

        self.explain_session_async(&session_path, catalog, config, cancel_rx)
            .await
    }

    /// Generates a candidate practice exercise asynchronously with timeout and cancellation checks.
    ///
    /// Respects `cancel_rx` and request timeout bounds, keeping session state intact.
    pub async fn generate_candidate_async(
        &self,
        context: &crate::generator::WeaknessContext,
        mut cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
    ) -> Result<crate::generator::CandidateExercise, ProviderError> {
        // Immediate cancellation check
        if let Some(rx) = &cancel_rx {
            if *rx.borrow() {
                return Err(ProviderError::Cancelled(
                    "Operation cancelled before start.".to_string(),
                ));
            }
        }

        let timeout_duration = std::time::Duration::from_secs(self.config.timeout_seconds);
        let sim_delay = self.provider_simulated_delay();

        let this = self.clone();
        let ctx_clone = context.clone();

        let work = async move {
            if let Some(delay) = sim_delay {
                tokio::time::sleep(delay).await;
            }
            tokio::task::spawn_blocking(move || this.generate_candidate(&ctx_clone))
                .await
                .map_err(|e| ProviderError::Failure(format!("Task execution failed: {}", e)))?
        };

        tokio::select! {
            biased;
            cancelled = async {
                if let Some(rx) = &mut cancel_rx {
                    while rx.changed().await.is_ok() {
                        if *rx.borrow() {
                            return true;
                        }
                    }
                }
                futures::future::pending::<bool>().await
            } => {
                if cancelled {
                    Err(ProviderError::Cancelled(
                        "Operation cancelled by client.".to_string(),
                    ))
                } else {
                    Err(ProviderError::Cancelled("Operation cancelled.".to_string()))
                }
            }
            _ = tokio::time::sleep(timeout_duration) => {
                Err(ProviderError::Timeout {
                    seconds: self.config.timeout_seconds,
                    message: format!(
                        "Practice generation request to provider '{}' timed out after {}s.",
                        self.config.provider, self.config.timeout_seconds
                    ),
                })
            }
            res = work => {
                res
            }
        }
    }

    /// Extracts weakness context from the given session path and generates a candidate exercise.
    pub fn generate_candidate_from_session(
        &self,
        session_path: &Path,
        fallback_concept: Option<&str>,
    ) -> Result<crate::generator::CandidateExercise, ProviderError> {
        let weakness_ctx =
            crate::generator::WeaknessContext::from_session_path(session_path, fallback_concept);
        self.generate_candidate(&weakness_ctx)
    }

    /// Extracts weakness context from the given session path and generates a candidate exercise asynchronously.
    pub async fn generate_candidate_from_session_async(
        &self,
        session_path: &Path,
        fallback_concept: Option<&str>,
        cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
    ) -> Result<crate::generator::CandidateExercise, ProviderError> {
        let weakness_ctx =
            crate::generator::WeaknessContext::from_session_path(session_path, fallback_concept);
        self.generate_candidate_async(&weakness_ctx, cancel_rx)
            .await
    }

    fn has_configured_http_target(&self) -> bool {
        match self.config.provider {
            ProviderKind::Fake | ProviderKind::Degraded => false,
            ProviderKind::LmStudio | ProviderKind::Ollama => true,
            ProviderKind::Anthropic | ProviderKind::OpenAI | ProviderKind::Gemini => {
                self.config.api_key.is_some()
                    || (self.config.base_url.is_some()
                        && self.config.base_url.as_deref() != Some("http://localhost:1234/v1"))
            }
        }
    }

    fn is_default_local_endpoint(&self) -> bool {
        match self.config.provider {
            ProviderKind::LmStudio => matches!(
                self.config.base_url.as_deref(),
                None | Some("http://localhost:1234/v1")
            ),
            ProviderKind::Ollama => matches!(
                self.config.base_url.as_deref(),
                None | Some("http://localhost:11434/v1") | Some("http://localhost:1234/v1")
            ),
            _ => false,
        }
    }

    /// Dispatches a chat completion prompt over HTTP for the configured provider.
    ///
    /// Returns:
    /// - `Ok(Some(raw_text))` when an HTTP endpoint responds with a valid model message.
    /// - `Ok(None)` when operating without an active local server on default `localhost:1234` / `11434`,
    ///   allowing hybrid fallback to offline synthesis per ADR 0005.
    /// - `Err(ProviderError)` on HTTP status rejections, timeouts, or malformed payloads.
    fn dispatch_chat_prompt(
        &self,
        sys_prompt: &str,
        user_prompt: &str,
    ) -> Result<Option<String>, ProviderError> {
        if !self.has_configured_http_target() {
            return Ok(None);
        }

        let (full_url, extra_headers, body_val) = match self.config.provider {
            ProviderKind::LmStudio | ProviderKind::Ollama | ProviderKind::OpenAI => {
                let default_base = match self.config.provider {
                    ProviderKind::LmStudio => "http://localhost:1234/v1",
                    ProviderKind::Ollama => "http://localhost:11434/v1",
                    _ => "https://api.openai.com/v1",
                };
                let base = match self.config.base_url.as_deref() {
                    Some("http://localhost:1234/v1")
                        if self.config.provider != ProviderKind::LmStudio =>
                    {
                        default_base
                    }
                    Some(b) => b,
                    None => default_base,
                };
                let trimmed = base.trim_end_matches('/');
                let url = if trimmed.ends_with("/chat/completions") {
                    trimmed.to_string()
                } else if trimmed.ends_with("/v1") {
                    format!("{}/chat/completions", trimmed)
                } else {
                    format!("{}/v1/chat/completions", trimmed)
                };

                let mut headers = Vec::new();
                if let Some(ref key) = self.config.api_key {
                    headers.push(("Authorization".to_string(), format!("Bearer {}", key)));
                }

                let payload = serde_json::json!({
                    "model": self.config.model,
                    "messages": [
                        { "role": "system", "content": sys_prompt },
                        { "role": "user", "content": user_prompt }
                    ],
                    "temperature": 0.2
                });
                (url, headers, payload)
            }
            ProviderKind::Anthropic => {
                let base = match self.config.base_url.as_deref() {
                    Some("http://localhost:1234/v1") | None => "https://api.anthropic.com/v1",
                    Some(b) => b,
                };
                let trimmed = base.trim_end_matches('/');
                let url = if trimmed.ends_with("/messages") {
                    trimmed.to_string()
                } else if trimmed.ends_with("/v1") {
                    format!("{}/messages", trimmed)
                } else {
                    format!("{}/v1/messages", trimmed)
                };

                let mut headers = vec![("anthropic-version".to_string(), "2023-06-01".to_string())];
                if let Some(ref key) = self.config.api_key {
                    headers.push(("x-api-key".to_string(), key.clone()));
                }

                let payload = serde_json::json!({
                    "model": self.config.model,
                    "max_tokens": 1024,
                    "system": sys_prompt,
                    "messages": [
                        { "role": "user", "content": user_prompt }
                    ]
                });
                (url, headers, payload)
            }
            ProviderKind::Gemini => {
                let base = match self.config.base_url.as_deref() {
                    Some("http://localhost:1234/v1") | None => {
                        "https://generativelanguage.googleapis.com/v1beta"
                    }
                    Some(b) => b,
                };
                let trimmed = base.trim_end_matches('/');
                let url = if trimmed.contains(":generateContent") {
                    trimmed.to_string()
                } else if trimmed.ends_with("/v1beta") || trimmed.ends_with("/v1") {
                    format!("{}/models/{}:generateContent", trimmed, self.config.model)
                } else {
                    format!(
                        "{}/v1beta/models/{}:generateContent",
                        trimmed, self.config.model
                    )
                };

                let mut headers = Vec::new();
                if let Some(ref key) = self.config.api_key {
                    headers.push(("x-goog-api-key".to_string(), key.clone()));
                }

                let payload = serde_json::json!({
                    "systemInstruction": {
                        "parts": [{ "text": sys_prompt }]
                    },
                    "contents": [
                        {
                            "role": "user",
                            "parts": [{ "text": user_prompt }]
                        }
                    ]
                });
                (url, headers, payload)
            }
            ProviderKind::Fake | ProviderKind::Degraded => return Ok(None),
        };

        let body_str = serde_json::to_string(&body_val)
            .map_err(|e| ProviderError::Failure(format!("Failed to serialize JSON body: {}", e)))?;

        let response_body = match self.send_http_post(&full_url, &extra_headers, &body_str) {
            Ok(Some(body)) => body,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };

        let parsed_json: serde_json::Value = serde_json::from_str(&response_body).map_err(|e| {
            ProviderError::MalformedResponse(format!(
                "Invalid JSON from provider '{}': {}. Body: {}",
                self.config.provider, e, response_body
            ))
        })?;

        let extracted_text = match self.config.provider {
            ProviderKind::LmStudio | ProviderKind::Ollama | ProviderKind::OpenAI => parsed_json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|t| t.as_str())
                .or_else(|| {
                    parsed_json
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|t| t.as_str())
                })
                .or_else(|| parsed_json.get("response").and_then(|t| t.as_str())),
            ProviderKind::Anthropic => parsed_json
                .get("content")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str()),
            ProviderKind::Gemini => parsed_json
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str()),
            ProviderKind::Fake | ProviderKind::Degraded => None,
        };

        match extracted_text {
            Some(text) => Ok(Some(text.to_string())),
            None => Err(ProviderError::MalformedResponse(format!(
                "Missing expected message content field in '{}' response: {}",
                self.config.provider, response_body
            ))),
        }
    }

    fn send_http_post(
        &self,
        full_url: &str,
        extra_headers: &[(String, String)],
        body: &str,
    ) -> Result<Option<String>, ProviderError> {
        use std::io::{Read, Write};
        use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

        let (scheme, rest) = if let Some(r) = full_url.strip_prefix("http://") {
            ("http", r)
        } else if let Some(r) = full_url.strip_prefix("https://") {
            ("https", r)
        } else {
            return Err(ProviderError::Failure(format!(
                "Unsupported URL scheme in '{}'",
                full_url
            )));
        };

        let (host_port, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };

        let default_port: u16 = if scheme == "https" { 443 } else { 80 };
        let (host, port) = match host_port.rfind(':') {
            Some(idx) => {
                let p_str = &host_port[idx + 1..];
                let port_num = p_str.parse::<u16>().map_err(|_| {
                    ProviderError::Failure(format!("Invalid port in URL '{}'", full_url))
                })?;
                (&host_port[..idx], port_num)
            }
            None => (host_port, default_port),
        };

        let sock_addr: SocketAddr = if host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1"
        {
            SocketAddr::from(([127, 0, 0, 1], port))
        } else {
            let mut addrs = (host, port).to_socket_addrs().map_err(|e| {
                ProviderError::Failure(format!("Failed to resolve host '{}': {}", host, e))
            })?;
            addrs.next().ok_or_else(|| {
                ProviderError::Failure(format!("No socket addresses resolved for '{}'", host))
            })?
        };

        let is_default_local = self.is_default_local_endpoint();
        let req_timeout = std::time::Duration::from_secs(self.config.timeout_seconds.max(1));
        let connect_timeout = if is_default_local {
            std::time::Duration::from_millis(150)
        } else {
            req_timeout
        };

        let mut stream = match TcpStream::connect_timeout(&sock_addr, connect_timeout) {
            Ok(s) => s,
            Err(e) => {
                if is_default_local {
                    // Local LM Studio / Ollama server is not running; fall back to offline synthesis
                    return Ok(None);
                }
                if e.kind() == std::io::ErrorKind::TimedOut {
                    return Err(ProviderError::Timeout {
                        seconds: self.config.timeout_seconds,
                        message: format!(
                            "Connection to '{}' timed out after {}s.",
                            full_url, self.config.timeout_seconds
                        ),
                    });
                }
                return Err(ProviderError::Failure(format!(
                    "Failed to connect to '{}': {}",
                    full_url, e
                )));
            }
        };

        let _ = stream.set_read_timeout(Some(req_timeout));
        let _ = stream.set_write_timeout(Some(req_timeout));

        let mut req_str = format!(
            "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nAccept: application/json\r\nConnection: close\r\n",
            path, host, port
        );
        for (k, v) in extra_headers {
            req_str.push_str(&format!("{}: {}\r\n", k, v));
        }
        req_str.push_str(&format!("Content-Length: {}\r\n\r\n{}", body.len(), body));

        stream.write_all(req_str.as_bytes()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut
                || e.kind() == std::io::ErrorKind::WouldBlock
            {
                ProviderError::Timeout {
                    seconds: self.config.timeout_seconds,
                    message: format!("Write to '{}' timed out: {}", full_url, e),
                }
            } else {
                ProviderError::Failure(format!("Failed writing HTTP request: {}", e))
            }
        })?;
        let _ = stream.flush();

        let mut raw_resp = Vec::new();
        let mut chunk = [0u8; 4096];
        let mut header_end_idx: Option<usize> = None;
        let mut expected_content_len: Option<usize> = None;

        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    raw_resp.extend_from_slice(&chunk[..n]);
                    if header_end_idx.is_none() {
                        if let Some(pos) = find_double_crlf(&raw_resp) {
                            header_end_idx = Some(pos);
                            let header_bytes = &raw_resp[..pos];
                            let header_text = String::from_utf8_lossy(header_bytes);
                            for line in header_text.lines() {
                                if let Some((name, val)) = line.split_once(':') {
                                    if name.trim().eq_ignore_ascii_case("content-length") {
                                        expected_content_len = val.trim().parse::<usize>().ok();
                                    }
                                }
                            }
                        }
                    }

                    if let (Some(h_end), Some(c_len)) = (header_end_idx, expected_content_len) {
                        if raw_resp.len() >= h_end + 4 + c_len {
                            break;
                        }
                    }
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::TimedOut
                        || e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    return Err(ProviderError::Timeout {
                        seconds: self.config.timeout_seconds,
                        message: format!(
                            "Request to provider '{}' timed out after {}s.",
                            self.config.provider, self.config.timeout_seconds
                        ),
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(ProviderError::Failure(format!(
                        "Error reading HTTP response from '{}': {}",
                        full_url, e
                    )));
                }
            }
        }

        let resp_str = String::from_utf8_lossy(&raw_resp).to_string();
        let (headers_part, body_part) = match resp_str.split_once("\r\n\r\n") {
            Some((h, b)) => (h, b),
            None => {
                return Err(ProviderError::MalformedResponse(format!(
                    "Malformed HTTP response from '{}': missing header delimiter",
                    full_url
                )));
            }
        };

        let status_line = headers_part.lines().next().unwrap_or("");
        let status_code = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| {
                ProviderError::MalformedResponse(format!(
                    "Malformed HTTP status line from '{}': '{}'",
                    full_url, status_line
                ))
            })?;

        if !(200..300).contains(&status_code) {
            let err_detail = extract_error_detail(body_part, status_line);
            return Err(ProviderError::Rejection {
                status_code: Some(status_code),
                message: err_detail,
            });
        }

        Ok(Some(body_part.to_string()))
    }
}

fn find_double_crlf(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|w| w == b"\r\n\r\n")
}

fn extract_error_detail(body: &str, status_line: &str) -> String {
    let trimmed = body.trim();
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(msg) = val
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            return msg.to_string();
        }
        if let Some(msg) = val.get("error").and_then(|e| e.as_str()) {
            return msg.to_string();
        }
        if let Some(msg) = val.get("message").and_then(|m| m.as_str()) {
            return msg.to_string();
        }
    }
    if !trimmed.is_empty() {
        trimmed.to_string()
    } else {
        status_line.to_string()
    }
}

impl AttemptExplainer for ConfiguredProvider {
    fn simulated_delay(&self) -> Option<std::time::Duration> {
        self.provider_simulated_delay()
    }

    fn explain_attempt(&self, context: &AttemptContext) -> Result<Explanation, ProviderError> {
        if let Some(inner) = &self.explainer {
            return inner.explain_attempt(context);
        }

        match self.config.provider {
            ProviderKind::Fake => {
                let fake = FakeProvider::new();
                fake.explain_attempt(context)
            }
            ProviderKind::Degraded => {
                let mut explanation = crate::prompt::synthesize_grounded_explanation(context);
                explanation.inferences.push(
                    "Operating in degraded mode: No LLM provider or credentials configured."
                        .to_string(),
                );
                Ok(explanation)
            }
            _ => {
                let sys = crate::prompt::system_prompt();
                let user = crate::prompt::build_attempt_user_prompt(context);
                if let Some(raw_response) = self.dispatch_chat_prompt(sys, &user)? {
                    return crate::prompt::parse_explanation_response(&raw_response, context);
                }

                let mut explanation = crate::prompt::synthesize_grounded_explanation(context);
                explanation.inferences.push(format!(
                    "Grounded explanation synthesized for provider '{}' (model: '{}').",
                    self.config.provider, self.config.model
                ));
                Ok(explanation)
            }
        }
    }
}

impl ExerciseGenerator for ConfiguredProvider {
    fn simulated_delay(&self) -> Option<std::time::Duration> {
        self.provider_simulated_delay()
    }

    fn generate_candidate(
        &self,
        context: &crate::generator::WeaknessContext,
    ) -> Result<crate::generator::CandidateExercise, ProviderError> {
        if let Some(inner) = &self.generator {
            return inner.generate_candidate(context);
        }

        match self.config.provider {
            ProviderKind::Fake => {
                let fake = FakeProvider::new();
                fake.generate_candidate(context)
            }
            ProviderKind::Degraded => {
                let mut candidate = crate::generator::synthesize_default_candidate(context, None);
                candidate.prose.push_str(
                    "\n\n(Operating in degraded mode: Candidate synthesized without external model.)",
                );
                Ok(candidate)
            }
            _ => {
                let sys = crate::generator::system_generator_prompt();
                let user = crate::generator::build_candidate_user_prompt(context);
                if let Some(raw_response) = self.dispatch_chat_prompt(sys, &user)? {
                    return crate::generator::parse_candidate_response(
                        &raw_response,
                        context,
                        None,
                    );
                }

                let mut candidate = crate::generator::synthesize_default_candidate(context, None);
                candidate.prose.push_str(&format!(
                    "\n\n(Generated practice synthesized for provider '{}' model '{}'.)",
                    self.config.provider, self.config.model
                ));
                Ok(candidate)
            }
        }
    }
}
