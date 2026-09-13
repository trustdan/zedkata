use crate::error::CoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version: 1,
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

impl SessionState {
    /// Parse session state from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, CoreError> {
        let state = serde_json::from_str(json)?;
        Ok(state)
    }

    /// Serialize session state to a pretty JSON string.
    pub fn to_json_pretty(&self) -> Result<String, CoreError> {
        let s = serde_json::to_string_pretty(self)?;
        Ok(s)
    }
}
