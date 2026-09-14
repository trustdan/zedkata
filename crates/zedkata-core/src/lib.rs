//! Core domain model, verification rules, and session state for zedkata.
//!
//! Pure synchronous library with zero runtime dependencies.

pub mod catalog;
pub mod error;
pub mod lesson;
pub mod scoring;
pub mod session;
pub mod verification;

pub use catalog::{Catalog, LessonEntry};
pub use error::CoreError;
pub use lesson::{Exercise, ForbidRules, Lesson, Walkthrough};
pub use scoring::{
    calculate_score, check_forbid_rules, count_edit_units, create_history_entry, update_weaknesses,
};
pub use session::{
    default_session_path, Attempt, CurrentSession, EditRange, HistoryEntry, RecordedEdit,
    SessionState, StudentInfo, Verdict, CURRENT_SCHEMA_VERSION,
};
pub use verification::{
    compare_buffers, diff_buffers, normalize_buffer, verify_attempt, AttemptOutcome,
    BufferDifference,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_json_roundtrip() {
        let mut state = SessionState::default();
        state.current.lesson_id = "test-lesson".to_string();
        state.current.exercise_index = 0;

        let json = state.to_json_pretty().expect("failed to serialize");
        let parsed = SessionState::from_json(&json).expect("failed to deserialize");
        assert_eq!(state.current.lesson_id, parsed.current.lesson_id);
        assert_eq!(state.version, 1);
    }
}
