use crate::error::CoreError;
use crate::verification::AttemptOutcome;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The current supported session state schema version.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Single session state persisted to `session.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub version: u32,
    pub student: StudentInfo,
    pub current: CurrentSession,
    pub attempt: Option<Attempt>,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub weaknesses: HashMap<String, f64>,
    #[serde(default)]
    pub completed_walkthroughs: Vec<WalkthroughProgress>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            student: StudentInfo {
                started_at: Utc::now(),
            },
            current: CurrentSession {
                lesson_id: String::new(),
                exercise_index: 0,
                buffer_path: None,
                opened_at: None,
                nav_mode: None,
            },
            attempt: None,
            history: Vec::new(),
            weaknesses: HashMap::new(),
            completed_walkthroughs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentInfo {
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentSession {
    pub lesson_id: String,
    pub exercise_index: usize,
    pub buffer_path: Option<String>,
    pub opened_at: Option<DateTime<Utc>>,
    pub nav_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub edit_count: u32,
    #[serde(default)]
    pub edits: Vec<RecordedEdit>,
    pub verdict: Option<Verdict>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedEdit {
    pub at: DateTime<Utc>,
    pub range: EditRange,
    pub inserted: String,
    pub deleted_len: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditRange {
    pub start: (u32, u32),
    pub end: (u32, u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "details")]
pub enum Verdict {
    Passed {
        edits: u32,
        par: Option<u32>,
        score: u32,
    },
    Failed {
        reason: String,
    },
    InProgress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub lesson_id: String,
    pub exercise_id: String,
    pub passed: bool,
    pub par_edits: Option<u32>,
    pub actual_edits: u32,
    pub score: u32,
    pub completed_at: DateTime<Utc>,
}

/// Record of explicit self-reported completion of an unscored guided walkthrough.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkthroughProgress {
    pub walkthrough_id: String,
    pub lesson_id: String,
    pub completed_at: DateTime<Utc>,
}

/// Resolve the standard platform path for `session.json`.
///
/// Precedence:
/// 1. `ZEDKATA_SESSION_PATH` or `ZEDKATA_SESSION` environment variable if set and non-empty.
/// 2. Windows: `%LOCALAPPDATA%\zedkata\session.json` (fallback `%USERPROFILE%\AppData\Local\zedkata\session.json`).
/// 3. Unix / macOS: `$XDG_STATE_HOME/zedkata/session.json` (fallback `~/.local/state/zedkata/session.json`).
pub fn default_session_path() -> Result<PathBuf, CoreError> {
    if let Ok(override_path) =
        std::env::var("ZEDKATA_SESSION_PATH").or_else(|_| std::env::var("ZEDKATA_SESSION"))
    {
        if !override_path.trim().is_empty() {
            return Ok(PathBuf::from(override_path));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data)
                .join("zedkata")
                .join("session.json"));
        }
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            return Ok(PathBuf::from(user_profile)
                .join("AppData")
                .join("Local")
                .join("zedkata")
                .join("session.json"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg_state) = std::env::var("XDG_STATE_HOME") {
            return Ok(PathBuf::from(xdg_state)
                .join("zedkata")
                .join("session.json"));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local")
                .join("state")
                .join("zedkata")
                .join("session.json"));
        }
    }

    Err(CoreError::SessionPathError(
        "Could not resolve platform standard state directory for zedkata".to_string(),
    ))
}

impl SessionState {
    /// Parse session state from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, CoreError> {
        let state: Self = serde_json::from_str(json)?;
        if state.version > CURRENT_SCHEMA_VERSION {
            return Err(CoreError::IncompatibleSessionVersion {
                found: state.version,
                supported: CURRENT_SCHEMA_VERSION,
            });
        }
        Ok(state)
    }

    /// Serialize session state to a pretty JSON string.
    pub fn to_json_pretty(&self) -> Result<String, CoreError> {
        let s = serde_json::to_string_pretty(self)?;
        Ok(s)
    }

    /// Load session state strictly from a file path.
    pub fn load_from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path)?;

        match Self::from_json(&content) {
            Ok(state) => Ok(state),
            Err(CoreError::IncompatibleSessionVersion { found, supported }) => {
                Err(CoreError::IncompatibleSessionVersion { found, supported })
            }
            Err(err) => Err(CoreError::SessionCorruptError {
                path: path.to_path_buf(),
                message: err.to_string(),
            }),
        }
    }

    /// Safely preserves a corrupt or incompatible session file by renaming it to a timestamped backup
    /// (`<filename>.corrupt.<timestamp>.bak`) so that subsequent writes do not silently destroy recoverable data.
    ///
    /// Returns the backup PathBuf on success.
    pub fn backup_corrupt_file(path: &Path) -> Result<PathBuf, CoreError> {
        if !path.exists() {
            return Err(CoreError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cannot backup nonexistent file: {:?}", path),
            )));
        }

        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let orig_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("session.json");
        let backup_name = format!("{}.corrupt.{}.bak", orig_name, nanos);
        let backup_path = match path.parent() {
            Some(parent) => parent.join(backup_name),
            None => PathBuf::from(backup_name),
        };

        std::fs::rename(path, &backup_path)?;
        Ok(backup_path)
    }

    /// Load session state from file if valid; if the file is absent, initializes a default session.
    /// If the file is present but corrupt or incompatible, preserves the corrupt file to a timestamped backup
    /// (`<filename>.corrupt.<timestamp>.bak`) and returns a fresh default session along with the backup path.
    pub fn load_or_recover(path: &Path) -> Result<(Self, Option<PathBuf>), CoreError> {
        if !path.exists() {
            return Ok((Self::default(), None));
        }

        match Self::load_from_file(path) {
            Ok(session) => Ok((session, None)),
            Err(CoreError::IncompatibleSessionVersion { .. })
            | Err(CoreError::SessionCorruptError { .. }) => {
                let backup_path = Self::backup_corrupt_file(path)?;
                Ok((Self::default(), Some(backup_path)))
            }
            Err(err) => Err(err),
        }
    }

    /// Load session state from file if it exists, or initialize a fresh default session if absent.
    pub fn load_or_init(path: &Path) -> Result<Self, CoreError> {
        if !path.exists() {
            Ok(Self::default())
        } else {
            Self::load_from_file(path)
        }
    }

    /// Persist session state to disk using safe atomic temporary-file replacement.
    ///
    /// Writes state to `session.json.<pid>_<timestamp>.tmp` in the same directory, flushes and
    /// syncs to disk, then atomically renames over destination. On failure, the temporary
    /// file is removed and the original file is preserved.
    pub fn save_to_file(&self, path: &Path) -> Result<(), CoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pid = std::process::id();
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let tmp_filename = format!("session.json.{}_{}.tmp", pid, now);
        let tmp_path = match path.parent() {
            Some(p) => p.join(tmp_filename),
            None => PathBuf::from(tmp_filename),
        };

        let json = self.to_json_pretty()?;

        // Perform temporary write and flush
        let write_res = (|| -> Result<(), std::io::Error> {
            let mut file = std::fs::File::create(&tmp_path)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(err) = write_res {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(CoreError::IoError(err));
        }

        // Atomically replace destination
        if let Err(err) = std::fs::rename(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(CoreError::IoError(err));
        }

        Ok(())
    }

    /// Begin a new attempt for the active exercise.
    pub fn start_attempt(&mut self, attempt_id: String, buffer_path: Option<String>) {
        self.current.buffer_path = buffer_path;
        self.current.opened_at = Some(Utc::now());
        self.attempt = Some(Attempt {
            id: attempt_id,
            edit_count: 0,
            edits: Vec::new(),
            verdict: Some(Verdict::InProgress),
        });
    }

    /// Record an incremental edit for the active attempt.
    ///
    /// Returns `true` if accepted and recorded, or `false` if rejected
    /// (e.g. attempt ID is stale or attempt is already finalized).
    pub fn record_edit(&mut self, attempt_id: &str, edit: RecordedEdit) -> bool {
        let Some(ref mut att) = self.attempt else {
            return false;
        };

        if att.id != attempt_id {
            return false;
        }

        if matches!(
            att.verdict,
            Some(Verdict::Passed { .. }) | Some(Verdict::Failed { .. })
        ) {
            return false;
        }

        att.edits.push(edit);
        att.edit_count = crate::scoring::count_edit_units(&att.edits);
        true
    }

    /// Finalize an attempt with an evaluated outcome.
    ///
    /// Updates verdict, appends to `history` idempotently, and updates `weaknesses`.
    /// Returns `true` if successfully transitioned, `false` if stale or already finalized.
    pub fn finalize_attempt(
        &mut self,
        attempt_id: &str,
        outcome: &AttemptOutcome,
        exercise_id: &str,
        concepts: &[String],
    ) -> bool {
        let Some(ref mut att) = self.attempt else {
            return false;
        };

        if att.id != attempt_id {
            return false;
        }

        if matches!(
            att.verdict,
            Some(Verdict::Passed { .. }) | Some(Verdict::Failed { .. })
        ) {
            return false;
        }

        match outcome {
            AttemptOutcome::Passed { edits, par, score } => {
                att.verdict = Some(Verdict::Passed {
                    edits: *edits,
                    par: *par,
                    score: *score,
                });

                // Prevent duplicate history entries for the exact same attempt
                let already_recorded = self.history.iter().any(|h| {
                    h.lesson_id == self.current.lesson_id
                        && h.exercise_id == exercise_id
                        && h.actual_edits == *edits
                        && h.score == *score
                });

                if !already_recorded {
                    self.history.push(crate::scoring::create_history_entry(
                        &self.current.lesson_id,
                        exercise_id,
                        true,
                        *par,
                        *edits,
                        *score,
                        Utc::now(),
                    ));
                }

                self.weaknesses =
                    crate::scoring::update_weaknesses(&self.weaknesses, concepts, true, *score);
                true
            }
            AttemptOutcome::Failed { reason } => {
                att.verdict = Some(Verdict::Failed {
                    reason: reason.clone(),
                });

                // Prevent duplicate history entries for the exact same failed attempt
                let already_recorded = self.history.iter().any(|h| {
                    h.lesson_id == self.current.lesson_id
                        && h.exercise_id == exercise_id
                        && !h.passed
                        && h.actual_edits == att.edit_count
                });

                if !already_recorded {
                    self.history.push(crate::scoring::create_history_entry(
                        &self.current.lesson_id,
                        exercise_id,
                        false,
                        None,
                        att.edit_count,
                        0,
                        Utc::now(),
                    ));
                }

                self.weaknesses =
                    crate::scoring::update_weaknesses(&self.weaknesses, concepts, false, 0);
                true
            }
            AttemptOutcome::AlreadyFinalized { .. } | AttemptOutcome::Unfinished => false,
        }
    }

    /// Abandon an active attempt manually (e.g. student resets exercise or navigates away).
    pub fn abandon_attempt(&mut self, attempt_id: &str) -> bool {
        let Some(ref mut att) = self.attempt else {
            return false;
        };

        if att.id != attempt_id {
            return false;
        }

        if matches!(
            att.verdict,
            Some(Verdict::Passed { .. }) | Some(Verdict::Failed { .. })
        ) {
            return false;
        }

        att.verdict = Some(Verdict::Failed {
            reason: "Attempt abandoned by student".to_string(),
        });
        true
    }

    /// Reset and restart the active exercise under a fresh attempt ID.
    pub fn retry_attempt(&mut self, new_attempt_id: String) {
        self.current.opened_at = Some(Utc::now());
        self.attempt = Some(Attempt {
            id: new_attempt_id,
            edit_count: 0,
            edits: Vec::new(),
            verdict: Some(Verdict::InProgress),
        });
    }

    /// Switch active exercise and lesson.
    pub fn switch_exercise(&mut self, lesson_id: String, exercise_index: usize) {
        self.current.lesson_id = lesson_id;
        self.current.exercise_index = exercise_index;
        self.current.buffer_path = None;
        self.current.opened_at = None;
        self.attempt = None;
    }

    /// Mark an unscored guided walkthrough as completed via self-report.
    ///
    /// Preserves explicit self-reported completion semantics (REQ-29) without entering
    /// drill scoring or modifying active attempt state. Returns true if newly added, or
    /// false if already completed (with timestamp refreshed).
    pub fn mark_walkthrough_completed(&mut self, walkthrough_id: &str, lesson_id: &str) -> bool {
        if let Some(existing) = self
            .completed_walkthroughs
            .iter_mut()
            .find(|w| w.walkthrough_id == walkthrough_id)
        {
            existing.completed_at = Utc::now();
            false
        } else {
            self.completed_walkthroughs.push(WalkthroughProgress {
                walkthrough_id: walkthrough_id.to_string(),
                lesson_id: lesson_id.to_string(),
                completed_at: Utc::now(),
            });
            true
        }
    }

    /// Check if an unscored guided walkthrough has been self-reported as completed.
    pub fn is_walkthrough_completed(&self, walkthrough_id: &str) -> bool {
        self.completed_walkthroughs
            .iter()
            .any(|w| w.walkthrough_id == walkthrough_id)
    }

    /// Reset self-reported completion of an unscored guided walkthrough.
    ///
    /// Returns true if an entry was removed.
    pub fn reset_walkthrough(&mut self, walkthrough_id: &str) -> bool {
        let initial_len = self.completed_walkthroughs.len();
        self.completed_walkthroughs
            .retain(|w| w.walkthrough_id != walkthrough_id);
        self.completed_walkthroughs.len() < initial_len
    }
}
