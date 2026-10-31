use crate::error::LspError;
use crate::protocol::{Diagnostic, Notification, Position, PublishDiagnosticsParams, Range};
use chrono::Utc;
use std::path::{Path, PathBuf};
use zedkata_core::catalog::Catalog;
use zedkata_core::session::{RecordedEdit, SessionState, Verdict};
use zedkata_core::verification::{verify_attempt, AttemptOutcome};

/// Engine connecting LSP document events with session state, core verification, and diagnostics.
pub struct FeedbackEngine {
    session_path: PathBuf,
    catalog: Catalog,
    generated_dir: Option<PathBuf>,
    active_attempt_id: Option<String>,
    active_uri: Option<String>,
}

impl FeedbackEngine {
    pub fn new(session_path: PathBuf, catalog: Catalog) -> Self {
        Self {
            session_path,
            catalog,
            generated_dir: None,
            active_attempt_id: None,
            active_uri: None,
        }
    }

    pub fn with_generated_dir(
        session_path: PathBuf,
        catalog: Catalog,
        generated_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            session_path,
            catalog,
            generated_dir,
            active_attempt_id: None,
            active_uri: None,
        }
    }

    pub fn session_path(&self) -> &Path {
        &self.session_path
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn active_attempt_id(&self) -> Option<&str> {
        self.active_attempt_id.as_deref()
    }

    pub fn active_uri(&self) -> Option<&str> {
        self.active_uri.as_deref()
    }

    /// Reloads generated lessons into `self.catalog` if `lesson_id` is not currently present.
    fn ensure_lesson_loaded(&mut self, lesson_id: &str) {
        if self.catalog.get(lesson_id).is_some() {
            return;
        }
        let base_dir = self.catalog.base_dir().to_path_buf();
        if let Ok((reloaded, _)) =
            Catalog::load_with_generated(&base_dir, self.generated_dir.as_deref())
        {
            self.catalog = reloaded;
        }
    }

    /// Check if an opened document URI corresponds to the active exercise buffer in session state.
    pub fn is_active_exercise(&self, uri: &str) -> Result<bool, LspError> {
        let session = SessionState::load_or_init(&self.session_path)?;
        let lesson_id = &session.current.lesson_id;
        if lesson_id.is_empty() {
            return Ok(false);
        }

        let Some(lesson_entry) = self.catalog.get(lesson_id) else {
            return Ok(false);
        };

        let Some(exercise) = lesson_entry
            .lesson
            .exercises
            .get(session.current.exercise_index)
        else {
            return Ok(false);
        };

        let Some(path) = uri_to_path(uri) else {
            return Ok(false);
        };

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.ends_with(".goal.kata") {
            return Ok(false);
        }

        // Also explicitly exclude the resolved goal fixture path
        if let Ok(goal_fixture) = self.catalog.resolve_fixture(lesson_id, &exercise.id, false) {
            if paths_match(&path, &goal_fixture) {
                return Ok(false);
            }
        }

        // 1. If buffer_path is explicitly set in session.current, match against it
        if let Some(ref bp) = session.current.buffer_path {
            if paths_match(&path, Path::new(bp)) {
                return Ok(true);
            }
        }

        // 2. Alternatively, match against the resolved start fixture for the active exercise
        if let Ok(start_fixture) = self.catalog.resolve_fixture(lesson_id, &exercise.id, true) {
            if paths_match(&path, &start_fixture) {
                return Ok(true);
            }
        }

        // 3. Or match exact exercise buffer naming conventions
        let expected_kata = format!("{}.kata", exercise.id);
        let expected_start_kata = format!("{}.start.kata", exercise.id);
        let expected_prefixed_kata = format!("{}-{}.kata", lesson_id, exercise.id);
        if file_name == expected_kata
            || file_name == expected_start_kata
            || file_name == expected_prefixed_kata
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Handle document open event.
    ///
    /// If the opened document matches the active exercise, initializes or synchronizes the
    /// attempt in session state and clears prior diagnostics.
    pub fn on_document_open(
        &mut self,
        uri: &str,
        _text: &str,
    ) -> Result<Option<Notification>, LspError> {
        if let Ok(session) = SessionState::load_or_init(&self.session_path) {
            if !session.current.lesson_id.is_empty() {
                self.ensure_lesson_loaded(&session.current.lesson_id);
            }
        }

        if !self.is_active_exercise(uri)? {
            return Ok(None);
        }

        let mut session = SessionState::load_or_init(&self.session_path)?;
        let path_str = uri_to_path(uri).map(|p| p.to_string_lossy().to_string());

        let needs_fresh_attempt = match session.attempt {
            None => true,
            Some(ref att) => {
                att.verdict.is_none()
                    || matches!(
                        att.verdict,
                        Some(Verdict::Passed { .. }) | Some(Verdict::Failed { .. })
                    )
            }
        };

        if needs_fresh_attempt {
            let attempt_id = format!(
                "attempt_{}_{}",
                Utc::now().timestamp_millis(),
                std::process::id()
            );
            session.start_attempt(attempt_id.clone(), path_str);
            session.save_to_file(&self.session_path)?;
            self.active_attempt_id = Some(attempt_id);
        } else if let Some(ref att) = session.attempt {
            self.active_attempt_id = Some(att.id.clone());
        }

        self.active_uri = Some(uri.to_string());

        // Emit clear diagnostics on open/reset
        Ok(Some(create_publish_diagnostics_notification(uri, vec![])))
    }

    /// Handle incremental document change event.
    ///
    /// Evaluates the current buffer against the exercise goal, enforcing budgets and forbid rules.
    /// If passed or failed, finalizes the attempt, saves session state, and emits diagnostic feedback.
    pub fn on_document_change(
        &mut self,
        uri: &str,
        current_text: &str,
        edits: &[RecordedEdit],
    ) -> Result<Option<Notification>, LspError> {
        if self.active_uri.as_deref() != Some(uri) {
            return Ok(None);
        }

        let mut session = SessionState::load_or_init(&self.session_path)?;
        let Some(active_attempt_id) = self.active_attempt_id.clone() else {
            return Ok(None);
        };

        let Some(ref att) = session.attempt else {
            return Ok(None);
        };

        // Reject stale attempt ID
        if att.id != active_attempt_id {
            return Ok(None);
        }

        // Reject late edits on finalized attempts
        if matches!(
            att.verdict,
            Some(Verdict::Passed { .. }) | Some(Verdict::Failed { .. })
        ) {
            return Ok(None);
        }

        let lesson_id = session.current.lesson_id.clone();
        self.ensure_lesson_loaded(&lesson_id);
        let Some(lesson_entry) = self.catalog.get(&lesson_id) else {
            return Ok(None);
        };

        let Some(exercise) = lesson_entry
            .lesson
            .exercises
            .get(session.current.exercise_index)
        else {
            return Ok(None);
        };

        // Resolve and read goal buffer
        let goal_path = self
            .catalog
            .resolve_fixture(&lesson_id, &exercise.id, false)
            .map_err(|e| LspError::Protocol(format!("Cannot resolve goal fixture: {}", e)))?;
        let goal_text = std::fs::read_to_string(&goal_path).map_err(LspError::Io)?;

        // Synchronize rolling edits and verify attempt against goal
        let outcome = {
            let Some(ref mut attempt) = session.attempt else {
                return Ok(None);
            };
            attempt.edits = edits.to_vec();
            attempt.edit_count = zedkata_core::scoring::count_edit_units(&attempt.edits);
            verify_attempt(current_text, &goal_text, attempt, exercise)
        };

        match outcome {
            AttemptOutcome::Unfinished => {
                session.save_to_file(&self.session_path)?;
                Ok(None)
            }

            AttemptOutcome::Passed { edits, par, score } => {
                session.finalize_attempt(
                    &active_attempt_id,
                    &outcome,
                    &exercise.id,
                    &lesson_entry.lesson.concepts,
                );
                session.save_to_file(&self.session_path)?;

                let is_uncalibrated = lesson_entry.lesson.is_generated();
                let par_display = par
                    .map(|p| {
                        if is_uncalibrated {
                            format!(", par {} (uncalibrated)", p)
                        } else {
                            format!(", par {}", p)
                        }
                    })
                    .unwrap_or_default();
                let message = format!(
                    "Passed — {} edit{}{}, score {}{}",
                    edits,
                    if edits == 1 { "" } else { "s" },
                    par_display,
                    score,
                    if is_uncalibrated {
                        " (generated practice)"
                    } else {
                        ""
                    }
                );

                let diag = Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(3), // Information
                    source: Some("zedkata".to_string()),
                    message,
                };

                Ok(Some(create_publish_diagnostics_notification(
                    uri,
                    vec![diag],
                )))
            }

            AttemptOutcome::Failed { ref reason } => {
                session.finalize_attempt(
                    &active_attempt_id,
                    &outcome,
                    &exercise.id,
                    &lesson_entry.lesson.concepts,
                );
                session.save_to_file(&self.session_path)?;

                let diag = Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(1), // Error
                    source: Some("zedkata".to_string()),
                    message: format!("Failed: {}", reason),
                };

                Ok(Some(create_publish_diagnostics_notification(
                    uri,
                    vec![diag],
                )))
            }

            AttemptOutcome::AlreadyFinalized { .. } => Ok(None),
        }
    }

    /// Handle document close event. Clears published diagnostics for the buffer.
    pub fn on_document_close(&mut self, uri: &str) -> Result<Option<Notification>, LspError> {
        if self.active_uri.as_deref() == Some(uri) {
            self.active_uri = None;
            Ok(Some(create_publish_diagnostics_notification(uri, vec![])))
        } else {
            Ok(None)
        }
    }
}

/// Create a typed `textDocument/publishDiagnostics` notification.
fn create_publish_diagnostics_notification(
    uri: &str,
    diagnostics: Vec<Diagnostic>,
) -> Notification {
    let params = PublishDiagnosticsParams {
        uri: uri.to_string(),
        version: None,
        diagnostics,
    };
    Notification::new(
        "textDocument/publishDiagnostics",
        Some(serde_json::to_value(params).unwrap_or(serde_json::Value::Null)),
    )
}

/// Convert an LSP file URI to a standard filesystem PathBuf.
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    if !uri.starts_with("file://") {
        return None;
    }
    let after_scheme = &uri["file://".len()..];
    let decoded = urlencoding_decode(after_scheme);

    #[cfg(target_os = "windows")]
    {
        if decoded.starts_with('/') && decoded.len() >= 3 && decoded.as_bytes()[2] == b':' {
            return Some(PathBuf::from(&decoded[1..]));
        }
        Some(PathBuf::from(decoded))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Some(PathBuf::from(decoded))
    }
}

/// Convert a filesystem path to an LSP file URI string.
pub fn path_to_uri(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    if s.starts_with('/') {
        format!("file://{}", s)
    } else {
        format!("file:///{}", s)
    }
}

/// Compare two paths for equivalence, taking Windows case-insensitivity into account.
pub fn paths_match(a: &Path, b: &Path) -> bool {
    if let (Ok(ca), Ok(cb)) = (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        return ca == cb;
    }
    let na = a.to_string_lossy().replace('\\', "/");
    let nb = b.to_string_lossy().replace('\\', "/");
    #[cfg(target_os = "windows")]
    {
        na.eq_ignore_ascii_case(&nb)
    }
    #[cfg(not(target_os = "windows"))]
    {
        na == nb
    }
}

/// Decode percent-encoded characters (e.g. %20 -> ' ', %C3%A9 -> 'é') as UTF-8.
fn urlencoding_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut clone = chars.clone();
            if let (Some(h1), Some(h2)) = (clone.next(), clone.next()) {
                if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                    let _ = chars.next();
                    let _ = chars.next();
                    bytes.push(((d1 << 4) | d2) as u8);
                    continue;
                }
            }
        }
        let mut buf = [0u8; 4];
        bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_to_path_and_back() {
        let uri = "file:///C:/workspace/test%20dir/file.kata";
        let path = uri_to_path(uri).unwrap();
        #[cfg(target_os = "windows")]
        assert_eq!(path, PathBuf::from("C:\\workspace\\test dir\\file.kata"));

        let reconstructed = path_to_uri(&path);
        assert!(reconstructed.starts_with("file:///"));
        assert!(reconstructed.contains("workspace/test dir/file.kata"));
    }

    #[test]
    fn test_uri_to_path_decodes_multibyte_utf8_percent_sequences() {
        // %C3%A9 = 'é' (2-byte UTF-8), %F0%9F%8C%8D = '🌍' (4-byte UTF-8)
        let uri = "file:///C:/workspace/caf%C3%A9%20%F0%9F%8C%8D/drill.kata";
        let path = uri_to_path(uri).unwrap();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("café 🌍"),
            "Expected UTF-8 decoded path containing 'café 🌍', got: {}",
            path_str
        );
    }

    #[test]
    fn test_paths_match_case_and_slashes() {
        let p1 = Path::new("C:/Users/Dan/file.kata");
        let p2 = Path::new("C:\\Users\\Dan\\file.kata");
        assert!(paths_match(p1, p2));

        #[cfg(target_os = "windows")]
        {
            let p3 = Path::new("c:/users/dan/file.kata");
            assert!(paths_match(p1, p3));
        }
    }
}
